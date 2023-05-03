use std::fmt::Display;

pub use adventurers_quest::{Quest, QuestProgress, QuestStatus, Reset};

use crate::utils::{Event, BackgroundVariant};


pub struct Q1 {
    progress: QuestProgress,
}

impl Q1 {
    pub fn new() -> Self {
        Self {
            progress: QuestProgress::new(5),
        }
    }
}

impl Display for Q1 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Q1: walk on 5 sands.")?;
        match self.progress.progress() {
            Some((a, b)) => write!(f, "({}/{})", a - 1, b)?,
            None => write!(f, "(Completed)")?,
        }
        Ok(())
    }
}

impl Reset for Q1 {
    fn reset(&mut self) {
        self.progress.reset();
    }
}

impl Quest<Event> for Q1 {
    fn update(&mut self, event: &Event) {
        if self.progress.is_completed() {
            return;
        }
        match event {
            Event::MoveTo(_, b) => {
                if b == &Some(BackgroundVariant::Sand) {
                    self.progress.next();
                } else {
                    self.progress.reset();
                }
            }
            _ => {}
        }
    }

    fn status(&self) -> QuestStatus {
        self.progress.status
    }

    fn is_completed(&self) -> bool {
        self.progress.is_completed()
    }
}
