use std::ops::AddAssign;
use std::time::Duration;
use std::{error::Error, ops::Add};
use termgame::{
    run_game, Controller, Game, GameEvent, GameSettings, KeyCode, Message, SimpleEvent,
    StyledCharacter, ViewportLocation,
};

const CHESS_PAWN: char = '♟';

/// if distance between player and border < padding, move viewport
const VIEW_PADDING: i32 = 2;

trait Draw {
    fn draw(&mut self, game: &mut Game);
}

#[derive(Copy, Clone, Default, Debug)]
struct Position(i32, i32);

impl Into<ViewportLocation> for Position {
    fn into(self) -> ViewportLocation {
        ViewportLocation {
            x: self.0,
            y: self.1,
        }
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

struct Player {
    update_draw: bool,
    icon: char,
    position: Position,
    drawn: Option<Position>,
}

impl Player {
    // fn new() -> Self {
    //     Self::default()
    // }

    fn move_to(&mut self, x: i32, y: i32) {
        self.position = Position(x, y);
        self.update_draw = true;
    }

    fn move_by(&mut self, x: i32, y: i32) {
        self.position += &Position(x, y);
        self.update_draw = true;
    }
}

impl Default for Player {
    fn default() -> Self {
        Player {
            update_draw: true,
            // icon: PLAYER_CHAR,
            icon: 'A',
            position: Position::default(),
            drawn: None,
        }
    }
}

impl Draw for Player {
    fn draw(&mut self, game: &mut Game) {
        if !self.update_draw {
            return;
        }
        if let Some(Position(x, y)) = self.drawn {
            game.set_screen_char(x, y, None);
        }
        let Position(x, y) = self.position;
        game.set_screen_char(x, y, Some(StyledCharacter::new(self.icon)));
        self.drawn = Some(self.position);
        self.update_draw = false;
    }
}

#[derive(Default)]
struct Control {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
}

impl Control {
    fn clear(&mut self) {
        *self = Self::default();
    }
}

#[derive(Default)]
struct MyGame {
    control: Control,
    // test: bool,
    viewport_size: (u16, (u16, u16)),
    viewport_position: Position,
    text: String,
    show_text: bool,
    frame: i32,
    player: Player,
}

impl MyGame {
    fn update_player_position(&mut self) {
        if self.control.left {
            self.player.move_by(-1, 0);
        }
        if self.control.right {
            self.player.move_by(1, 0);
        }
        if self.control.up {
            self.player.move_by(0, -1);
        }
        if self.control.down {
            self.player.move_by(0, 1);
        }
    }
    fn update_viewport_position(&mut self) {
        let pos = &self.player.position;
        let left = pos.0 - self.viewport_position.0;
        let top = pos.1 - self.viewport_position.1;
        let right =
            self.viewport_position.0 + self.viewport_size.0 as i32 - 2 - self.player.position.0;
        let bottom = self.viewport_position.1
            + self.viewport_size.1 .0 as i32
            + self.viewport_size.1 .1 as i32
            - 3
            - self.player.position.1;
        if left < VIEW_PADDING {
            self.viewport_position.0 -= 1;
        }
        if top < VIEW_PADDING {
            self.viewport_position.1 -= 1;
        }
        if right < VIEW_PADDING {
            self.viewport_position.0 += 1;
        }
        if bottom < VIEW_PADDING {
            self.viewport_position.1 += 1;
        }
        self.text = format!("top: {top}, left: {left}, bottom: {bottom}, right: {right}");
    }
}

impl Draw for MyGame {
    fn draw(&mut self, game: &mut Game) {
        self.player.draw(game);
        game.set_viewport(self.viewport_position.into());
    }
}

impl Controller for MyGame {
    fn on_start(&mut self, game: &mut Game) {
        self.player.move_to(3, 3);
        self.viewport_size = game.screen_size();

        game.set_screen_char(5, 6, Some(StyledCharacter::new('a')));
        game.set_screen_char(20, 16, Some(StyledCharacter::new('b')));
        game.set_screen_char(23, 10, Some(StyledCharacter::new('c')));
        game.set_screen_char(12, 14, Some(StyledCharacter::new('d')));
    }

    fn on_event(&mut self, game: &mut Game, event: GameEvent) {
        match event.into() {
            SimpleEvent::Just(key_code) => match key_code {
                KeyCode::Char(ch) => match ch {
                    't' => {
                        self.text = format!("vp: {:?}", game.screen_size());
                    }
                    _ => {}
                },
                KeyCode::Enter => {
                    self.show_text = !self.show_text;
                }

                KeyCode::Left => {
                    self.control.left = true;
                }
                KeyCode::Right => {
                    self.control.right = true;
                }
                KeyCode::Up => {
                    self.control.up = true;
                }
                KeyCode::Down => {
                    self.control.down = true;
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn on_tick(&mut self, game: &mut Game) {

        self.update_player_position();
        self.update_viewport_position();

        self.draw(game);

        self.control.clear();

        // let f = format!("{:>8}", self.frame);
        // for (i, ch) in f.chars().enumerate() {
        //     game.set_screen_char(10 + i as i32, 10, Some(StyledCharacter::new(ch)));
        // }
        if self.show_text {
            game.set_message(Some(Message::new(self.text.clone())));
        } else {
            game.set_message(None);
        }

        self.frame += 1;
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("Welcome to AdventureRS");
    println!("To get started, you should read the termgame documentation,");
    println!("and try out getting a termgame UI to appear on your terminal.");

    let mut controller = MyGame::default();

    run_game(
        &mut controller,
        GameSettings::new()
            // The below are the defaults, but shown so you can edit them.
            .tick_duration(Duration::from_millis(50))
            .quit_event(Some(SimpleEvent::WithControl(KeyCode::Char('c')).into())),
    )?;

    println!("Game Ended!");

    Ok(())
}
