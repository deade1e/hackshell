use std::path::Path;

use crossterm::{
    cursor, event::{self, EventStream}, execute, terminal
};
use futures::StreamExt;
use tokio::{
    fs::OpenOptions,
    io::{self, AsyncReadExt, AsyncWriteExt, stderr},
};

#[derive(Default)]
pub struct Context {
    history: Vec<String>,
    history_pos: usize,
    ci: String,
    ci_pos: usize,
}

pub struct Readline {
    ctx: Context,
    history_file: Option<tokio::fs::File>,
}

pub enum Event {
    Line(String),
    Ctrlc,
    Eof,
    Tab,
}

impl Readline {
    pub async fn new(history_file: Option<&Path>) -> io::Result<Self> {
        let mut rl = Self {
            ctx: Default::default(),
            history_file: match history_file {
                Some(path) => Some(
                    OpenOptions::new()
                        .read(true)
                        .write(true)
                        .create(true)
                        .truncate(true)
                        .open(path)
                        .await
                        .unwrap(),
                ),
                None => None,
            },
        };

        rl.history_load().await?;

        Ok(rl)
    }

    pub async fn readline(&mut self, prompt: &str) -> Result<Event, io::Error> {
        terminal::enable_raw_mode()?;

        let mut reader = EventStream::new();

        self.print_current_line(prompt).await?;

        loop {
            let event = reader
                .next()
                .await
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "No event"))??;

            if let event::Event::Key(key) = event {
                if key.kind != event::KeyEventKind::Press {
                    continue;
                }

                // Checking if it's CTRL+C or CTRL+D
                match key {
                    event::KeyEvent {
                        code: event::KeyCode::Char('c'),
                        modifiers: event::KeyModifiers::CONTROL,
                        kind: _,
                        state: _,
                    } => {
                        terminal::disable_raw_mode()?;
                        return Ok(Event::Ctrlc);
                    }
                    event::KeyEvent {
                        code: event::KeyCode::Char('d'),
                        modifiers: event::KeyModifiers::CONTROL,
                        kind: _,
                        state: _,
                    } => {
                        terminal::disable_raw_mode()?;
                        return Ok(Event::Eof);
                    }
                    _ => {}
                }

                match key.code {
                    event::KeyCode::Up => {
                        self.on_up_arrow(prompt).await?;
                    }
                    event::KeyCode::Down => {
                        self.on_down_arrow(prompt).await?;
                    }
                    event::KeyCode::Left => {
                        self.on_left_arrow(prompt).await?;
                    }
                    event::KeyCode::Right => {
                        self.on_right_arrow(prompt).await?;
                    }
                    event::KeyCode::Delete => {
                        self.on_canc(prompt).await?;
                    }
                    event::KeyCode::Backspace => {
                        self.on_backspace(prompt).await?;
                    }
                    event::KeyCode::Char(c) => {
                        self.insert_ci(c, prompt).await?;
                    }
                    event::KeyCode::Tab => {
                        terminal::disable_raw_mode()?;
                        return Ok(Event::Tab);
                    }
                    event::KeyCode::Enter => {
                        terminal::disable_raw_mode()?;
                        return Ok(Event::Line(self.on_enter().await?));
                    }
                    _ => {}
                }
            }
        }
    }

    async fn insert_ci(&mut self, what: char, prompt: &str) -> io::Result<()> {
        self.ci_insert_pos(what).await;

        if self.ctx.ci_pos != self.ctx.ci.len() {
            Self::clear_current_line().await?;
            let _ = self.print_current_line(prompt).await;
        } else {
            Self::write_flush(format!("{}", what)).await?;
        }

        Ok(())
    }

    async fn on_left_arrow(&mut self, prompt: &str) -> io::Result<()> {
        if self.ctx.ci_pos > 0 {
            self.ctx.ci_pos -= 1;

            Self::clear_current_line().await?;
            let _ = self.print_current_line(prompt).await;
        }

        Ok(())
    }

    async fn on_right_arrow(&mut self, prompt: &str) -> io::Result<()> {
        if self.ctx.ci_pos < self.ctx.ci.len() {
            self.ctx.ci_pos += 1;

            Self::clear_current_line().await?;
            let _ = self.print_current_line(prompt).await;
        }

        Ok(())
    }

    async fn on_up_arrow(&mut self, prompt: &str) -> io::Result<()> {
        if self.ctx.history_pos > 0 {
            self.ctx.history_pos -= 1;
            self.set_ci(self.ctx.history[self.ctx.history_pos].clone())
                .await;

            Self::clear_current_line().await?;
            let _ = self.print_current_line(prompt).await;
        }

        Ok(())
    }

    async fn on_down_arrow(&mut self, prompt: &str) -> io::Result<()> {
        if self.ctx.history_pos < self.ctx.history.len() {
            self.ctx.history_pos += 1;
            self.set_ci(
                self.ctx
                    .history
                    .get(self.ctx.history_pos)
                    .cloned()
                    .unwrap_or_default(),
            )
            .await;

            Self::clear_current_line().await?;
            let _ = self.print_current_line(prompt).await;
        }

        Ok(())
    }

    async fn on_enter(&mut self) -> io::Result<String> {
        Self::write_flush("\r\n".to_string()).await?; // Move to the next line

        if !self.ctx.ci.is_empty() {
            self.history_push(self.ctx.ci.clone()).await;
        }

        self.reset_history_pos().await;
        let r = self.ctx.ci.clone();

        self.ctx.ci.clear(); // error on purpose

        self.ctx.ci_pos = 0;

        Ok(r)
    }

    async fn on_backspace(&mut self, prompt: &str) -> io::Result<()> {
        if self.ci_remove_pos().await {
            Self::clear_current_line().await?;
            let _ = self.print_current_line(prompt).await;
        }

        Ok(())
    }

    async fn on_canc(&mut self, prompt: &str) -> io::Result<()> {
        if self.ci_remove_pos_right().await {
            Self::clear_current_line().await?;
            let _ = self.print_current_line(prompt).await;
        }

        Ok(())
    }

    async fn _current_input_pop(&mut self) {
        self.ctx.ci.pop();
    }

    async fn _current_input_push(&mut self, what: char) {
        self.ctx.ci.push(what);
    }

    async fn ci_insert_pos(&mut self, what: char) {
        self.ctx.ci.insert(self.ctx.ci_pos, what);
        self.ctx.ci_pos += 1;
    }

    // Returns where to update the current line or not
    async fn ci_remove_pos(&mut self) -> bool {
        // If there is nothing to delete or the position is already zero.
        if self.ctx.ci.is_empty() || self.ctx.ci_pos == 0 {
            return false;
        }

        self.ctx.ci_pos -= 1;
        self.ctx.ci.remove(self.ctx.ci_pos);

        true
    }

    // Returns where to update the current line or not
    async fn ci_remove_pos_right(&mut self) -> bool {
        // If there is nothing to delete or the position is already at the extreme right.
        if self.ctx.ci.is_empty() || self.ctx.ci_pos == self.ctx.ci.len() {
            return false;
        }

        self.ctx.ci.remove(self.ctx.ci_pos);

        true
    }

    async fn set_ci(&mut self, what: String) {
        self.ctx.ci_pos = what.len();
        self.ctx.ci = what;
    }

    async fn reset_history_pos(&mut self) {
        self.ctx.history_pos = self.ctx.history.len(); // Reset history position
        // History file truncate
    }

    async fn history_load(&mut self) -> std::io::Result<()> {
        if let Some(file) = self.history_file.as_mut() {
            let mut content = String::new();
            file.read_to_string(&mut content).await?;

            self.ctx.history = content.lines().map(|s| s.to_string()).collect();
        }

        self.reset_history_pos().await;

        Ok(())
    }

    async fn history_push(&mut self, what: String) {
        self.ctx.history.push(what.clone());

        // Add the history item to the history file
        if let Some(file) = self.history_file.as_mut() {
            file.write_all(format!("{}\n", what).as_bytes())
                .await
                .unwrap();
            file.flush().await.unwrap();
        }
    }

    async fn write_flush(what: String) -> std::io::Result<()> {
        let mut stderr = stderr();

        stderr.write_all(what.as_bytes()).await?;
        stderr.flush().await
    }

    async fn print_current_line(&self, prompt: &str) -> std::io::Result<()> {
        let mut stderr = stderr();

        stderr
            .write_all(format!("\r{}{}", prompt, self.ctx.ci).as_bytes())
            .await?;
        stderr.flush().await?;
        Self::move_cursor_col(prompt.len() as u16 + self.ctx.ci_pos as u16).await?;

        Ok(())
    }

    async fn clear_current_line() -> std::io::Result<()> {
        execute!(std::io::stderr(), terminal::Clear(terminal::ClearType::CurrentLine)) // Ugly
    }

    async fn move_cursor_col(col: u16) -> std::io::Result<()> {
        execute!(std::io::stderr(), cursor::MoveToColumn(col))
    }
}
