use std::fmt::Display;

pub use adventurers_quest::{Quest, QuestProgress, QuestStatus, Reset};

use crate::utils::{Event, BackgroundVariant, Item};


pub struct StepQuest {
    background: BackgroundVariant,
    progress: QuestProgress,
}

impl StepQuest {
    pub fn new(background: BackgroundVariant, steps: usize) -> Self {
        Self {
            background,
            progress: QuestProgress::new(steps),
        }
    }
}

impl Display for StepQuest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "walk on {} {} tile(s).", self.progress.steps, self.background)?;
        match self.progress.progress() {
            Some((a, b)) => write!(f, "({}/{})", a - 1, b)?,
            None => write!(f, "(Completed)")?,
        }
        Ok(())
    }
}

impl Reset for StepQuest {
    fn reset(&mut self) {
        self.progress.reset();
    }
}

impl Quest<Event> for StepQuest {
    fn update(&mut self, event: &Event) {
        if self.is_completed() {
            return;
        }
        match event {
            Event::MoveTo(_, b) => {
                if b == &Some(self.background) {
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

pub struct PickupQuest {
    item: Item,
    progress: QuestProgress,
}

impl PickupQuest {
    pub fn new(item: Item, number: usize) -> Self {
        Self {
            item,
            progress: QuestProgress::new(number),
        }
    }
}

impl Display for PickupQuest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "pickup {} {}(s).", self.progress.steps, self.item)?;
        match self.progress.progress() {
            Some((a, b)) => write!(f, "({}/{})", a - 1, b)?,
            None => write!(f, "(Completed)")?,
        }
        Ok(())
    }
}

impl Reset for PickupQuest {
    fn reset(&mut self) {
        self.progress.reset();
    }
}

impl Quest<Event> for PickupQuest {
    fn update(&mut self, event: &Event) {
        if self.is_completed() {
            return;
        }
        match event {
            Event::Pickup(item) => {
                if item == &self.item {
                    self.progress.next();
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

pub struct CompoundQuest {
    sub_quests: Vec<Box<dyn Quest<Event>>>,
    progress: QuestProgress,
}

impl CompoundQuest {
    pub fn new(sub_quests: Vec<Box<dyn Quest<Event>>>) -> Self {
        let progress = QuestProgress::new(sub_quests.len());
        Self {
            sub_quests,
            progress,
        }
    }
}

impl Display for CompoundQuest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.progress.progress() {
            Some((current, total)) => {
                write!(f, "{}/{}: {}", current, total, self.sub_quests[current - 1])?;
            }
            None => write!(f, "Completed")?,
        }
        Ok(())
    }
}

impl Reset for CompoundQuest {
    fn reset(&mut self) {
        self.progress.reset();
    }
}

impl Quest<Event> for CompoundQuest {
    fn update(&mut self, event: &Event) {
        match self.progress.progress() {
            Some((current, _)) => {
                let sub_quest = &mut self.sub_quests[current - 1];
                sub_quest.update(event);
                if sub_quest.is_completed() {
                    self.progress.next();
                }
            }
            None => {}
        }
    }

    fn status(&self) -> QuestStatus {
        self.progress.status
    }

    fn is_completed(&self) -> bool {
        self.progress.is_completed()
    }
}
