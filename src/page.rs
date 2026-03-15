use crate::config::get_config;
use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{execute, queue};
use std::env;
use std::ffi::OsString;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant, SystemTime};
use unicode_width::UnicodeWidthChar;

const STATUS_HELP: &str = "q quit  r reload  j/k scroll  PgUp/PgDn page  g/G top/bottom";
const ESC: char = '\x1b';
pub fn run_page_mode(file_paths: Option<Vec<PathBuf>>) -> io::Result<()> {
    let rendered = capture_rendered_output(None)?;
    let watch_state = WatchState::from_sources(file_paths);
    run_rendered_page_mode(rendered, watch_state)
}

fn run_rendered_page_mode(rendered: String, watch_state: Option<WatchState>) -> io::Result<()> {
    let mut pager = InternalPager::new(rendered, watch_state);
    pager.run()
}

fn capture_rendered_output(input: Option<&str>) -> io::Result<String> {
    let current_exe = env::current_exe()?;
    let filtered_args = render_capture_args(env::args_os().skip(1));

    let mut child = Command::new(current_exe)
        .args(filtered_args)
        .env("SEE_FORCE_COLORS", "1")
        .env("SEE_FORCE_INTERACTIVE", "1")
        .stdin(if input.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;

    if let Some(input) = input {
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(input.as_bytes())?;
        }
    }

    let output = child.wait_with_output()?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        Err(io::Error::other(format!(
            "page renderer exited with status {}",
            output.status
        )))
    }
}

fn render_capture_args(args: impl IntoIterator<Item = OsString>) -> Vec<OsString> {
    let mut filtered_args: Vec<_> = args
        .into_iter()
        .filter(|arg| {
            let value = arg.to_string_lossy();
            !is_page_capture_flag(&value)
        })
        .collect();

    // The pager re-runs `see` to capture rendered output. Force the child down
    // the plain render path so config-driven paging or watching cannot recurse.
    filtered_args.push(OsString::from("--page=false"));
    filtered_args.push(OsString::from("--watch=false"));
    filtered_args.push(OsString::from("--render-images=false"));
    filtered_args
}

fn is_page_capture_flag(value: &str) -> bool {
    matches!(value, "--page" | "--pager" | "--watch")
        || value.starts_with("--page=")
        || value.starts_with("--pager=")
        || value.starts_with("--watch=")
        || value.starts_with("--watch-interval-ms=")
}

#[derive(Clone)]
struct DisplayLine {
    text: String,
    source_line: usize,
}

#[derive(Clone, PartialEq, Eq)]
struct FileSignature {
    exists: bool,
    len: u64,
    modified: Option<SystemTime>,
}

struct FileWatchState {
    paths: Vec<PathBuf>,
    signatures: Vec<FileSignature>,
    interval: Duration,
    last_check: Instant,
}

impl FileWatchState {
    fn from_paths(file_paths: Option<Vec<PathBuf>>) -> Option<Self> {
        let config = get_config();
        if !config.watch {
            return None;
        }

        let paths: Vec<PathBuf> = file_paths
            .unwrap_or_default()
            .into_iter()
            .filter(|path| path.is_file())
            .collect();

        if paths.is_empty() {
            return None;
        }

        let signatures = paths.iter().map(file_signature).collect();
        Some(Self {
            paths,
            signatures,
            interval: Duration::from_millis(config.watch_interval_ms.max(50)),
            last_check: Instant::now(),
        })
    }

    fn should_check(&self) -> bool {
        self.last_check.elapsed() >= self.interval
    }

    fn has_changed(&mut self) -> bool {
        self.last_check = Instant::now();
        let mut changed = false;

        for (index, path) in self.paths.iter().enumerate() {
            let signature = file_signature(path);
            if self.signatures.get(index) != Some(&signature) {
                changed = true;
                if let Some(existing) = self.signatures.get_mut(index) {
                    *existing = signature;
                }
            }
        }

        changed
    }
}

enum WatchState {
    Files(FileWatchState),
}

impl WatchState {
    fn from_sources(file_paths: Option<Vec<PathBuf>>) -> Option<Self> {
        FileWatchState::from_paths(file_paths).map(Self::Files)
    }

    fn should_check(&self) -> bool {
        match self {
            Self::Files(state) => state.should_check(),
        }
    }

    fn has_changed(&mut self) -> bool {
        match self {
            Self::Files(state) => state.has_changed(),
        }
    }
}

struct InternalPager {
    source_lines: Vec<String>,
    wrapped_lines: Vec<DisplayLine>,
    first_wrap_for_source: Vec<usize>,
    wrapped_cols: u16,
    top_row: usize,
    status_message: Option<String>,
    watch_state: Option<WatchState>,
    anchor_source_line: usize,
    needs_redraw: bool,
    follow_bottom: bool,
}

impl InternalPager {
    fn new(rendered: String, watch_state: Option<WatchState>) -> Self {
        let mut source_lines: Vec<String> = rendered.lines().map(|line| line.to_string()).collect();
        if rendered.ends_with('\n') {
            source_lines.push(String::new());
        }
        if source_lines.is_empty() {
            source_lines.push(String::new());
        }

        Self {
            source_lines,
            wrapped_lines: Vec::new(),
            first_wrap_for_source: Vec::new(),
            wrapped_cols: 0,
            top_row: 0,
            status_message: None,
            follow_bottom: watch_state.is_some(),
            watch_state,
            anchor_source_line: 0,
            needs_redraw: true,
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
            self.maybe_reload()?;
            self.draw_if_needed(stdout)?;

            if !event::poll(Duration::from_millis(50))? {
                continue;
            }

            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    let (_, rows) = terminal::size()?;
                    let page_height = rows.saturating_sub(1) as usize;

                    if self.handle_view_key(key.code, page_height)? {
                        return Ok(());
                    }
                }
                Event::Resize(_, _) => {
                    self.wrapped_cols = 0;
                    self.needs_redraw = true;
                    self.ensure_wrapped_lines()?;
                }
                _ => {}
            }
        }
    }

    fn handle_view_key(&mut self, code: KeyCode, page_height: usize) -> io::Result<bool> {
        match code {
            KeyCode::Char('q')
            | KeyCode::Char('Q')
            | KeyCode::Char('\u{3}')
            | KeyCode::Char('\u{4}')
            | KeyCode::Esc => return Ok(true),
            KeyCode::Char('r') => self.reload_from_source()?,
            KeyCode::Char('j') | KeyCode::Down => self.scroll_down(1),
            KeyCode::Char('k') | KeyCode::Up => self.scroll_up(1),
            KeyCode::PageDown | KeyCode::Char(' ') => self.scroll_down(page_height.max(1)),
            KeyCode::PageUp => self.scroll_up(page_height.max(1)),
            KeyCode::Char('g') => {
                self.top_row = 0;
                self.follow_bottom = false;
                self.needs_redraw = true;
            }
            KeyCode::Char('G') => {
                self.top_row = self.max_top_row(page_height);
                self.follow_bottom = true;
                self.needs_redraw = true;
            }
            _ => {}
        }

        Ok(false)
    }

    fn draw_if_needed(&mut self, stdout: &mut io::Stdout) -> io::Result<()> {
        if !self.needs_redraw {
            return Ok(());
        }

        self.draw(stdout)?;
        self.needs_redraw = false;
        Ok(())
    }

    fn draw(&mut self, stdout: &mut io::Stdout) -> io::Result<()> {
        self.ensure_wrapped_lines()?;

        let (cols, rows) = terminal::size()?;
        let content_rows = rows.saturating_sub(1) as usize;
        let last_row = (self.top_row + content_rows).min(self.wrapped_lines.len());

        for (screen_row, line) in self.wrapped_lines[self.top_row..last_row]
            .iter()
            .enumerate()
        {
            queue!(
                stdout,
                MoveTo(0, screen_row as u16),
                Clear(ClearType::CurrentLine)
            )?;
            write!(stdout, "{}", line.text)?;
        }

        for screen_row in (last_row - self.top_row)..content_rows {
            queue!(
                stdout,
                MoveTo(0, screen_row as u16),
                Clear(ClearType::CurrentLine)
            )?;
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

        let previous_source_line = self.anchor_source_line;
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

        self.top_row = if self.follow_bottom {
            self.max_top_row(page_height)
        } else {
            self.first_wrap_for_source
                .get(previous_source_line)
                .copied()
                .unwrap_or(0)
                .min(self.max_top_row(page_height))
        };
        self.needs_redraw = true;

        Ok(())
    }

    fn maybe_reload(&mut self) -> io::Result<()> {
        let should_reload = self
            .watch_state
            .as_ref()
            .map(|state| state.should_check())
            .unwrap_or(false);

        if !should_reload {
            return Ok(());
        }

        if self
            .watch_state
            .as_mut()
            .map(|state| state.has_changed())
            .unwrap_or(false)
        {
            self.reload_from_source()?;
        }

        Ok(())
    }

    fn reload_from_source(&mut self) -> io::Result<()> {
        let input = self.watch_state.as_ref().map(|_| String::new());

        match capture_rendered_output(input.as_deref()) {
            Ok(rendered) => {
                self.set_rendered(rendered);
                if matches!(self.watch_state, Some(WatchState::Files(_))) && self.follow_bottom {
                    let (_, rows) = terminal::size().unwrap_or((0, 1));
                    let page_height = rows.saturating_sub(1) as usize;
                    self.top_row = self.max_top_row(page_height);
                    self.anchor_source_line = self
                        .wrapped_lines
                        .last()
                        .map(|line| line.source_line)
                        .unwrap_or(0);
                }
                self.status_message = Some("reloaded".to_string());
                self.needs_redraw = true;
            }
            Err(err) => {
                self.status_message = Some(format!("reload failed: {}", err));
                self.needs_redraw = true;
            }
        }

        Ok(())
    }

    fn set_rendered(&mut self, rendered: String) {
        self.anchor_source_line = self.current_source_line();
        self.source_lines = rendered.lines().map(|line| line.to_string()).collect();
        if rendered.ends_with('\n') {
            self.source_lines.push(String::new());
        }
        if self.source_lines.is_empty() {
            self.source_lines.push(String::new());
        }
        self.wrapped_lines.clear();
        self.first_wrap_for_source.clear();
        self.wrapped_cols = 0;
        self.needs_redraw = true;
    }

    fn status_line(&self, page_height: usize) -> String {
        let total_rows = self.wrapped_lines.len();
        let end_row = (self.top_row + page_height).min(total_rows);
        let mut status = format!(
            " see page  rows {}-{} / {}  {}",
            self.top_row.saturating_add(1).min(total_rows.max(1)),
            end_row,
            total_rows,
            STATUS_HELP
        );

        if self.watch_state.is_some() {
            status.push_str("  watching");
            if self.follow_bottom {
                status.push_str("  following");
            }
        }

        if let Some(message) = &self.status_message {
            status.push_str("  ");
            status.push_str(message);
        }

        status
    }

    fn scroll_down(&mut self, count: usize) {
        let (_, rows) = terminal::size().unwrap_or((0, 1));
        let page_height = rows.saturating_sub(1) as usize;
        self.top_row = (self.top_row + count).min(self.max_top_row(page_height));
        self.anchor_source_line = self.current_source_line();
        self.follow_bottom = self.top_row >= self.max_top_row(page_height);
        self.status_message = None;
        self.needs_redraw = true;
    }

    fn scroll_up(&mut self, count: usize) {
        self.top_row = self.top_row.saturating_sub(count);
        self.anchor_source_line = self.current_source_line();
        self.follow_bottom = false;
        self.status_message = None;
        self.needs_redraw = true;
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
}

#[cfg(test)]
mod tests {
    use super::render_capture_args;
    use std::ffi::OsString;

    #[test]
    fn render_capture_args_disable_recursive_modes() {
        let args = vec![
            OsString::from("--page"),
            OsString::from("--watch=true"),
            OsString::from("--watch-interval-ms=1000"),
            OsString::from("README.md"),
        ];

        let filtered = render_capture_args(args);

        assert_eq!(
            filtered,
            vec![
                OsString::from("README.md"),
                OsString::from("--page=false"),
                OsString::from("--watch=false"),
                OsString::from("--render-images=false"),
            ]
        );
    }

    #[test]
    fn render_capture_args_strip_config_overrides_before_appending_false() {
        let args = vec![
            OsString::from("--config=/tmp/see.toml"),
            OsString::from("--pager=false"),
            OsString::from("--page=true"),
            OsString::from("--watch=false"),
            OsString::from("docs/main.md"),
        ];

        let filtered = render_capture_args(args);

        assert_eq!(
            filtered,
            vec![
                OsString::from("--config=/tmp/see.toml"),
                OsString::from("docs/main.md"),
                OsString::from("--page=false"),
                OsString::from("--watch=false"),
                OsString::from("--render-images=false"),
            ]
        );
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

fn file_signature(path: &PathBuf) -> FileSignature {
    match fs::metadata(path) {
        Ok(metadata) => FileSignature {
            exists: true,
            len: metadata.len(),
            modified: metadata.modified().ok(),
        },
        Err(_) => FileSignature {
            exists: false,
            len: 0,
            modified: None,
        },
    }
}
