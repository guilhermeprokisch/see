use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{execute, queue};
use std::env;
use std::io::{self, Write};
use std::process::{Command, Stdio};
use std::time::Duration;
use unicode_width::UnicodeWidthChar;

const STATUS_HELP: &str =
    "q quit  j/k scroll  / search  n/N next-prev  PgUp/PgDn page  g/G top/bottom";
const ESC: char = '\x1b';

pub fn run_page_mode() -> io::Result<()> {
    let rendered = capture_rendered_output()?;
    let mut pager = InternalPager::new(rendered);
    pager.run()
}

fn capture_rendered_output() -> io::Result<String> {
    let current_exe = env::current_exe()?;
    let filtered_args: Vec<_> = env::args_os()
        .skip(1)
        .filter(|arg| {
            let value = arg.to_string_lossy();
            !matches!(
                value.as_ref(),
                "--page" | "--page=true" | "--pager" | "--pager=true"
            )
        })
        .collect();

    let output = Command::new(current_exe)
        .args(filtered_args)
        .arg("--render-images=false")
        .env("SEE_FORCE_COLORS", "1")
        .env("SEE_FORCE_INTERACTIVE", "1")
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .output()?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        Err(io::Error::other(format!(
            "page renderer exited with status {}",
            output.status
        )))
    }
}

#[derive(Clone)]
struct DisplayLine {
    text: String,
    source_line: usize,
}

enum Mode {
    View,
    SearchInput,
}

struct InternalPager {
    source_lines: Vec<String>,
    searchable_lines: Vec<String>,
    wrapped_lines: Vec<DisplayLine>,
    first_wrap_for_source: Vec<usize>,
    wrapped_cols: u16,
    top_row: usize,
    mode: Mode,
    search_input: String,
    last_search: Option<String>,
    status_message: Option<String>,
}

impl InternalPager {
    fn new(rendered: String) -> Self {
        let mut source_lines: Vec<String> = rendered.lines().map(|line| line.to_string()).collect();
        if rendered.ends_with('\n') {
            source_lines.push(String::new());
        }
        if source_lines.is_empty() {
            source_lines.push(String::new());
        }

        let searchable_lines = source_lines
            .iter()
            .map(|line| strip_control_sequences(line))
            .collect();

        Self {
            source_lines,
            searchable_lines,
            wrapped_lines: Vec::new(),
            first_wrap_for_source: Vec::new(),
            wrapped_cols: 0,
            top_row: 0,
            mode: Mode::View,
            search_input: String::new(),
            last_search: None,
            status_message: None,
        }
    }

    fn run(&mut self) -> io::Result<()> {
        let mut stdout = io::stdout();
        terminal::enable_raw_mode()?;
        execute!(stdout, EnterAlternateScreen, Hide)?;

        let result = self.event_loop(&mut stdout);

        execute!(stdout, Show, LeaveAlternateScreen)?;
        terminal::disable_raw_mode()?;
        result
    }

    fn event_loop(&mut self, stdout: &mut io::Stdout) -> io::Result<()> {
        loop {
            self.draw(stdout)?;

            if !event::poll(Duration::from_millis(250))? {
                continue;
            }

            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    let (_, rows) = terminal::size()?;
                    let page_height = rows.saturating_sub(1) as usize;

                    match self.mode {
                        Mode::View => {
                            if self.handle_view_key(key.code, page_height)? {
                                return Ok(());
                            }
                        }
                        Mode::SearchInput => self.handle_search_key(key.code)?,
                    }
                }
                Event::Resize(_, _) => self.ensure_wrapped_lines()?,
                _ => {}
            }
        }
    }

    fn handle_view_key(&mut self, code: KeyCode, page_height: usize) -> io::Result<bool> {
        match code {
            KeyCode::Char('q') => return Ok(true),
            KeyCode::Char('j') | KeyCode::Down => self.scroll_down(1),
            KeyCode::Char('k') | KeyCode::Up => self.scroll_up(1),
            KeyCode::PageDown | KeyCode::Char(' ') => self.scroll_down(page_height.max(1)),
            KeyCode::PageUp => self.scroll_up(page_height.max(1)),
            KeyCode::Char('g') => self.top_row = 0,
            KeyCode::Char('G') => self.top_row = self.max_top_row(page_height),
            KeyCode::Char('/') => {
                self.mode = Mode::SearchInput;
                self.search_input = self.last_search.clone().unwrap_or_default();
                self.status_message = None;
            }
            KeyCode::Char('n') => self.jump_to_match(true),
            KeyCode::Char('N') => self.jump_to_match(false),
            _ => {}
        }

        Ok(false)
    }

    fn handle_search_key(&mut self, code: KeyCode) -> io::Result<()> {
        match code {
            KeyCode::Esc => {
                self.mode = Mode::View;
                self.search_input.clear();
                self.status_message = None;
            }
            KeyCode::Enter => {
                let query = self.search_input.trim().to_string();
                self.mode = Mode::View;
                self.search_input.clear();

                if query.is_empty() {
                    self.status_message = Some("empty search".to_string());
                } else {
                    self.last_search = Some(query);
                    self.jump_to_match(true);
                }
            }
            KeyCode::Backspace => {
                self.search_input.pop();
            }
            KeyCode::Char(ch) => {
                self.search_input.push(ch);
            }
            _ => {}
        }

        Ok(())
    }

    fn draw(&mut self, stdout: &mut io::Stdout) -> io::Result<()> {
        self.ensure_wrapped_lines()?;

        let (cols, rows) = terminal::size()?;
        let content_rows = rows.saturating_sub(1) as usize;
        let last_row = (self.top_row + content_rows).min(self.wrapped_lines.len());

        queue!(stdout, MoveTo(0, 0), Clear(ClearType::All))?;

        for (screen_row, line) in self.wrapped_lines[self.top_row..last_row]
            .iter()
            .enumerate()
        {
            queue!(stdout, MoveTo(0, screen_row as u16))?;
            write!(stdout, "{}", line.text)?;
        }

        queue!(
            stdout,
            MoveTo(0, rows.saturating_sub(1)),
            Clear(ClearType::CurrentLine)
        )?;
        write!(
            stdout,
            "\x1b[7m{:width$}\x1b[0m",
            self.status_line(content_rows),
            width = cols as usize
        )?;
        stdout.flush()?;
        Ok(())
    }

    fn ensure_wrapped_lines(&mut self) -> io::Result<()> {
        let (cols, rows) = terminal::size()?;
        let width = cols.max(1);

        if self.wrapped_cols == width && !self.wrapped_lines.is_empty() {
            return Ok(());
        }

        let previous_source_line = self.current_source_line();
        let page_height = rows.saturating_sub(1) as usize;

        self.wrapped_cols = width;
        self.wrapped_lines.clear();
        self.first_wrap_for_source = vec![0; self.source_lines.len()];

        for (source_line, line) in self.source_lines.iter().enumerate() {
            self.first_wrap_for_source[source_line] = self.wrapped_lines.len();

            let wrapped = wrap_ansi_line(line, width as usize);
            if wrapped.is_empty() {
                self.wrapped_lines.push(DisplayLine {
                    text: String::new(),
                    source_line,
                });
            } else {
                self.wrapped_lines.extend(
                    wrapped
                        .into_iter()
                        .map(|text| DisplayLine { text, source_line }),
                );
            }
        }

        if self.wrapped_lines.is_empty() {
            self.wrapped_lines.push(DisplayLine {
                text: String::new(),
                source_line: 0,
            });
        }

        self.top_row = self
            .first_wrap_for_source
            .get(previous_source_line)
            .copied()
            .unwrap_or(0)
            .min(self.max_top_row(page_height));

        Ok(())
    }

    fn status_line(&self, page_height: usize) -> String {
        match self.mode {
            Mode::SearchInput => format!("/{}", self.search_input),
            Mode::View => {
                let total_rows = self.wrapped_lines.len();
                let end_row = (self.top_row + page_height).min(total_rows);
                let mut status = format!(
                    " see page  rows {}-{} / {}  {}",
                    self.top_row.saturating_add(1).min(total_rows.max(1)),
                    end_row,
                    total_rows,
                    STATUS_HELP
                );

                if let Some(message) = &self.status_message {
                    status.push_str("  ");
                    status.push_str(message);
                } else if let Some(query) = &self.last_search {
                    status.push_str("  /");
                    status.push_str(query);
                }

                status
            }
        }
    }

    fn scroll_down(&mut self, count: usize) {
        let (_, rows) = terminal::size().unwrap_or((0, 1));
        let page_height = rows.saturating_sub(1) as usize;
        self.top_row = (self.top_row + count).min(self.max_top_row(page_height));
        self.status_message = None;
    }

    fn scroll_up(&mut self, count: usize) {
        self.top_row = self.top_row.saturating_sub(count);
        self.status_message = None;
    }

    fn max_top_row(&self, page_height: usize) -> usize {
        self.wrapped_lines.len().saturating_sub(page_height)
    }

    fn current_source_line(&self) -> usize {
        self.wrapped_lines
            .get(self.top_row)
            .map(|line| line.source_line)
            .unwrap_or(0)
    }

    fn jump_to_match(&mut self, forward: bool) {
        let Some(query) = self.last_search.as_ref().map(|query| query.to_lowercase()) else {
            self.status_message = Some("no active search".to_string());
            return;
        };

        let current_source = self.current_source_line();
        let next_match = if forward {
            self.find_match_forward(&query, current_source.saturating_add(1))
                .or_else(|| self.find_match_forward(&query, 0))
        } else {
            current_source
                .checked_sub(1)
                .and_then(|start| self.find_match_backward(&query, start))
                .or_else(|| {
                    self.find_match_backward(&query, self.searchable_lines.len().saturating_sub(1))
                })
        };

        if let Some(source_line) = next_match {
            self.top_row = self
                .first_wrap_for_source
                .get(source_line)
                .copied()
                .unwrap_or(0);
            self.status_message = Some(format!("match line {}", source_line + 1));
        } else {
            self.status_message = Some(format!("no matches for /{}", query));
        }
    }

    fn find_match_forward(&self, query: &str, start: usize) -> Option<usize> {
        self.searchable_lines
            .iter()
            .enumerate()
            .skip(start)
            .find(|(_, line)| line.to_lowercase().contains(query))
            .map(|(index, _)| index)
    }

    fn find_match_backward(&self, query: &str, start: usize) -> Option<usize> {
        self.searchable_lines
            .iter()
            .enumerate()
            .take(start.saturating_add(1))
            .rev()
            .find(|(_, line)| line.to_lowercase().contains(query))
            .map(|(index, _)| index)
    }
}

fn wrap_ansi_line(line: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![String::new()];
    }

    let mut wrapped = Vec::new();
    let mut current = String::new();
    let mut current_width = 0;
    let mut index = 0;
    let mut active_sgr = String::new();
    let mut active_link = String::new();

    while index < line.len() {
        if let Some((sequence, next_index)) = consume_control_sequence(line, index) {
            current.push_str(&sequence);
            update_active_sequences(&sequence, &mut active_sgr, &mut active_link);
            index = next_index;
            continue;
        }

        let ch = line[index..].chars().next().unwrap_or('\0');
        let ch_len = ch.len_utf8();
        let ch_width = UnicodeWidthChar::width(ch).unwrap_or(0).max(1);

        if current_width > 0 && current_width + ch_width > width {
            wrapped.push(finalize_segment(
                &current,
                !active_sgr.is_empty(),
                !active_link.is_empty(),
            ));
            current.clear();
            current.push_str(&active_link);
            current.push_str(&active_sgr);
            current_width = 0;
        }

        current.push(ch);
        current_width += ch_width;
        index += ch_len;
    }

    wrapped.push(finalize_segment(
        &current,
        !active_sgr.is_empty(),
        !active_link.is_empty(),
    ));

    wrapped
}

fn finalize_segment(segment: &str, has_active_sgr: bool, has_active_link: bool) -> String {
    let mut rendered = segment.to_string();

    if has_active_link {
        rendered.push_str("\x1b]8;;\x1b\\");
    }
    if has_active_sgr {
        rendered.push_str("\x1b[0m");
    }

    rendered
}

fn consume_control_sequence(line: &str, index: usize) -> Option<(String, usize)> {
    let bytes = line.as_bytes();
    if bytes.get(index).copied()? != ESC as u8 {
        return None;
    }

    match bytes.get(index + 1).copied() {
        Some(b'[') => {
            let mut end = index + 2;
            while end < bytes.len() {
                if (0x40..=0x7e).contains(&bytes[end]) {
                    return Some((line[index..=end].to_string(), end + 1));
                }
                end += 1;
            }
            Some((line[index..].to_string(), line.len()))
        }
        Some(b']') => {
            let mut end = index + 2;
            while end < bytes.len() {
                if bytes[end] == 0x07 {
                    return Some((line[index..=end].to_string(), end + 1));
                }
                if bytes[end] == ESC as u8 && bytes.get(end + 1).copied() == Some(b'\\') {
                    return Some((line[index..=end + 1].to_string(), end + 2));
                }
                end += 1;
            }
            Some((line[index..].to_string(), line.len()))
        }
        Some(_) => {
            let next_index = index + 2;
            Some((line[index..next_index].to_string(), next_index))
        }
        None => Some((line[index..].to_string(), line.len())),
    }
}

fn update_active_sequences(sequence: &str, active_sgr: &mut String, active_link: &mut String) {
    if sequence.starts_with("\x1b[") && sequence.ends_with('m') {
        let sgr = &sequence[2..sequence.len() - 1];
        if sgr.is_empty() || sgr.split(';').any(|part| part == "0") {
            active_sgr.clear();
        } else {
            active_sgr.push_str(sequence);
        }
        return;
    }

    if sequence.starts_with("\x1b]8;;") {
        if sequence == "\x1b]8;;\x1b\\" || sequence == "\x1b]8;;\u{7}" {
            active_link.clear();
        } else {
            *active_link = sequence.to_string();
        }
    }
}

fn strip_control_sequences(line: &str) -> String {
    let mut stripped = String::new();
    let mut index = 0;

    while index < line.len() {
        if let Some((_, next_index)) = consume_control_sequence(line, index) {
            index = next_index;
            continue;
        }

        let ch = line[index..].chars().next().unwrap_or('\0');
        stripped.push(ch);
        index += ch.len_utf8();
    }

    stripped
}
