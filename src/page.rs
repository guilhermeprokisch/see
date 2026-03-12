use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{execute, queue};
use std::env;
use std::io::{self, Write};
use std::process::{Command, Stdio};
use std::time::Duration;

const STATUS_HELP: &str = "q quit  j/k scroll  arrows scroll  PgUp/PgDn page  g/G top/bottom";

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

struct InternalPager {
    lines: Vec<String>,
    top_line: usize,
}

impl InternalPager {
    fn new(rendered: String) -> Self {
        let mut lines: Vec<String> = rendered.lines().map(|line| line.to_string()).collect();
        if rendered.ends_with('\n') {
            lines.push(String::new());
        }

        Self { lines, top_line: 0 }
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

            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                let (_, rows) = terminal::size()?;
                let page_height = rows.saturating_sub(1) as usize;

                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('j') | KeyCode::Down => self.scroll_down(1),
                    KeyCode::Char('k') | KeyCode::Up => self.scroll_up(1),
                    KeyCode::PageDown | KeyCode::Char(' ') => self.scroll_down(page_height.max(1)),
                    KeyCode::PageUp => self.scroll_up(page_height.max(1)),
                    KeyCode::Char('g') => self.top_line = 0,
                    KeyCode::Char('G') => self.top_line = self.max_top_line(page_height),
                    _ => {}
                }
            }
        }
    }

    fn draw(&self, stdout: &mut io::Stdout) -> io::Result<()> {
        let (cols, rows) = terminal::size()?;
        let content_rows = rows.saturating_sub(1) as usize;
        let last_line = (self.top_line + content_rows).min(self.lines.len());

        queue!(stdout, MoveTo(0, 0), Clear(ClearType::All))?;

        for (row, line) in self.lines[self.top_line..last_line].iter().enumerate() {
            queue!(stdout, MoveTo(0, row as u16))?;
            write!(stdout, "{}", line)?;
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

    fn status_line(&self, page_height: usize) -> String {
        let total_lines = self.lines.len();
        let end_line = (self.top_line + page_height).min(total_lines);
        format!(
            " see page  {}-{} / {}  {}",
            self.top_line.saturating_add(1).min(total_lines.max(1)),
            end_line,
            total_lines,
            STATUS_HELP
        )
    }

    fn scroll_down(&mut self, count: usize) {
        let (_, rows) = terminal::size().unwrap_or((0, 1));
        let page_height = rows.saturating_sub(1) as usize;
        self.top_line = (self.top_line + count).min(self.max_top_line(page_height));
    }

    fn scroll_up(&mut self, count: usize) {
        self.top_line = self.top_line.saturating_sub(count);
    }

    fn max_top_line(&self, page_height: usize) -> usize {
        self.lines.len().saturating_sub(page_height)
    }
}
