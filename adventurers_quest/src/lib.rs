use std::fmt::Display;

pub trait Quest<Event>: Display + Reset {
    fn update(&mut self, event: &Event);
    fn status(&self) -> QuestStatus;
    fn is_completed(&self) -> bool;
}

pub trait Reset {
    fn reset(&mut self);
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum QuestStatus {
    Pending(usize),
    Completed,
}

pub struct QuestProgress {
    /// max steps to complete quest, min = 1
    pub steps: usize,
    /// status begin with 1, to complete the 1st quest step
    pub status: QuestStatus,
}

impl QuestProgress {
    pub fn new(steps: usize) -> Self {
        Self {
            steps,
            status: QuestStatus::Pending(1),
        }
    }

    /// return (a, b), attempting the ath step of total b steps
    pub fn progress(&self) -> Option<(usize, usize)> {
        match self.status {
            QuestStatus::Pending(n) => Some((n, self.steps)),
            QuestStatus::Completed => None,
        }
    }

    /// return true if progress went to next step
    pub fn next(&mut self) -> bool {
        use QuestStatus::*;
        match &mut self.status {
            Pending(step) => {
                if *step < self.steps {
                    *step += 1;
                } else {
                    self.status = Completed;
                }
                true
            }
            Completed => false,
        }
    }

    pub fn is_completed(&self) -> bool {
        self.status == QuestStatus::Completed
    }
}

impl Reset for QuestProgress {
    fn reset(&mut self) {
        self.status = QuestStatus::Pending(1);
    }
}
