use std::{io, time::Duration};

use ratatui::crossterm::event::{self, Event, KeyEvent, KeyEventKind};

pub struct InputEvents;

impl InputEvents {
    pub fn next() -> io::Result<Option<KeyEvent>> {
        if event::poll(Duration::from_millis(16))? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => return Ok(Some(key)),
                _ => {}
            }
        }
        Ok(None)
    }
}
