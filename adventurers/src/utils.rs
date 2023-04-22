use std::ops::{Add, AddAssign};

use serde::{Deserialize, Serialize};
use termgame::{KeyCode, ViewportLocation, Message, GameColor, GameStyle};

#[derive(Copy, Clone, Default, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct Position(pub i32, pub i32);

impl Into<ViewportLocation> for Position {
    fn into(self) -> ViewportLocation {
        ViewportLocation {
            x: self.0,
            y: self.1,
        }
    }
}

impl Position {
    pub fn is_origin(&self) -> bool {
        self.0 == 0 && self.1 == 0
    }
}

impl Add for Position {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Position(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl<'a> Add<&'a Position> for &'a Position {
    type Output = Position;
    fn add(self, rhs: Self) -> Self::Output {
        Position(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl AddAssign<&Position> for Position {
    fn add_assign(&mut self, rhs: &Self) {
        self.0 += rhs.0;
        self.1 += rhs.1;
    }
}

#[derive(Default)]
pub struct Control {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
}

impl Control {
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    pub fn update(&mut self, key_code: KeyCode) {
        match key_code {
            KeyCode::Left => {
                self.left = true;
            }
            KeyCode::Right => {
                self.right = true;
            }
            KeyCode::Up => {
                self.up = true;
            }
            KeyCode::Down => {
                self.down = true;
            }
            _ => {}
        }
    }
}

impl From<&Control> for Position {
    fn from(control: &Control) -> Self {
        let mut x = 0;
        let mut y = 0;
        if control.left {
            x -= 1;
        }
        if control.right {
            x += 1;
        }
        if control.up {
            y -= 1;
        }
        if control.down {
            y += 1;
        }
        return Position(x, y);
    }
}

#[derive(Clone, Default)]
pub enum MessageType {
    Death(String),
    Sign(String),
    Debug(String),
    Pickup(char),
    Bag(String),
    #[default]
    None,
}

impl Into<Option<(String, String)>> for &MessageType {
    fn into(self) -> Option<(String, String)> {
        Some(match self.clone() {
            MessageType::Sign(s) => ("You saw a message on the sign".into(), s),
            MessageType::Death(s) => ("You died".into(), s),
            MessageType::Pickup(c) => ("Pick up an object".into(), format!("You pick up '{c}'")),
            MessageType::Bag(s) => ("Your bag has".into(), s),
            MessageType::Debug(s) => ("Debug".into(), s),
            MessageType::None => return None,
        })
    }
}

impl Into<Option<Message>> for &MessageType {
    fn into(self) -> Option<Message> {
        if let Some((title, text)) = Into::<Option<(String, String)>>::into(self) {
            Some(Message::new(text).title(title))
        } else {
            None
        }
    }
}


#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BackgroundVariant {
    Grass,
    Sand,
    Rock,
    Cinderblock,
    Flowerbush,
    Barrier,
    Water,
    Sign(String),
    Object(char),
}

impl BackgroundVariant {
    pub fn is_barrier(&self) -> bool {
        self == &BackgroundVariant::Barrier
    }
    pub fn is_water(&self) -> bool {
        self == &BackgroundVariant::Water
    }
}

impl Into<Option<GameColor>> for &BackgroundVariant {
    fn into(self) -> Option<GameColor> {
        Some(match self {
            &BackgroundVariant::Grass => GameColor::Green,
            &BackgroundVariant::Sand => GameColor::LightYellow,
            &BackgroundVariant::Rock => GameColor::DarkGray,
            &BackgroundVariant::Cinderblock => GameColor::LightRed,
            &BackgroundVariant::Flowerbush => GameColor::LightMagenta,
            &BackgroundVariant::Barrier => GameColor::Black,
            &BackgroundVariant::Water => GameColor::LightBlue,
            _ => return None,
        })
    }
}

impl Into<GameStyle> for &BackgroundVariant {
    fn into(self) -> GameStyle {
        GameStyle::new().background_color(self.into())
    }
}
