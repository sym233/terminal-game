use std::collections::HashMap;
use std::error::Error;
use std::ops::{Add, AddAssign};
use std::time::Duration;
use termgame::{
    run_game, Controller, Game, GameColor as Color, GameEvent, GameSettings, GameStyle as Style,
    KeyCode, Message, SimpleEvent, StyledCharacter, ViewportLocation,
};

const CHESS_PAWN: char = '♟';

/// if distance between player and border < padding, move viewport
const VIEW_PADDING: i32 = 2;

// #[derive(Debug, Clone, Copy)]
// enum ObjectVariant {
//     Player,
//     Background(BackgroundVariant),
// }

#[derive(Default)]
struct Place {
    player: bool,
    background: Option<BackgroundVariant>,
}

impl Place {
    fn set_player(&mut self) {
        self.player = true;
    }
    fn remove_player(&mut self) {
        self.player = false;
    }
    fn player(self) -> Self {
        Self {
            player: true,
            ..Default::default()
        }
    }
    fn background(self, background: Option<BackgroundVariant>) -> Self {
        Self {
            background: background,
            ..Default::default()
        }
    }
}

impl Into<StyledCharacter> for &Place {
    fn into(self) -> StyledCharacter {
        let mut c = StyledCharacter::new(' ');
        if self.player {
            c.c = 'A';
        }
        c.style = self.background.map(BackgroundVariant::into);
        return c;
    }
}

#[derive(Copy, Clone, Default, Debug, Eq, Hash, PartialEq)]
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
    previous_position: Option<Position>,
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
#[derive(Debug, Clone, Copy)]
enum BackgroundVariant {
    Grass,
    Sand,
    Rocks,
    Cinderblock,
    Flowers,
}

impl Into<Color> for BackgroundVariant {
    fn into(self) -> Color {
        match self {
            Self::Grass => Color::Green,
            Self::Sand => Color::LightYellow,
            Self::Rocks => Color::DarkGray,
            Self::Cinderblock => Color::LightRed,
            Self::Flowers => Color::LightMagenta,
        }
    }
}

impl Into<Style> for BackgroundVariant {
    fn into(self) -> Style {
        Style::new().background_color(Some(self.into()))
    }
}

#[derive(Debug)]
struct BackgroundBlock {
    // update_draw: bool,
    variant: BackgroundVariant,
    position: Position,
    previous_position: Option<Position>,
}

impl BackgroundBlock {
    fn new(variant: BackgroundVariant, position: Position) -> Self {
        BackgroundBlock {
            // update_draw: true,
            variant,
            position,
            previous_position: None,
        }
    }
}

impl Default for Player {
    fn default() -> Self {
        Player {
            update_draw: true,
            // icon: PLAYER_CHAR,
            icon: 'A',
            position: Position::default(),
            previous_position: None,
        }
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
struct MapPlace {
    should_draw: Vec<Position>,
    map: HashMap<Position, Place>,
}

impl MapPlace {
    fn update_player(&mut self, player: &mut Player) {
        if !player.update_draw {
            return;
        }
        if let Some(position) = player.previous_position.take() {
            if let Some(place) = self.map.get_mut(&position) {
                place.remove_player();
            }
            self.should_draw.push(position);
        }
        self.map
            .entry(player.position)
            .and_modify(|place| place.set_player())
            .or_insert_with(|| Place::default().player());
        player.previous_position = Some(player.position);
        player.update_draw = false;
        self.should_draw.push(player.position);
    }
    fn update_background(&mut self, background: &mut BackgroundBlock) {
        if let Some(position) = background.previous_position.take() {
            if let Some(place) = self.map.get_mut(&position) {
                place.background = None;
            }
            self.should_draw.push(position);
        }
        self.map
            .entry(background.position)
            .and_modify(|place| place.background = Some(background.variant))
            .or_insert_with(|| Place::default().background(Some(background.variant)));
        self.should_draw.push(background.position);
    }

    fn draw(&mut self, game: &mut Game) {
        for position in self.should_draw.drain(..) {
            let Position(x, y) = position;
            let place = self.map.get(&position);
            game.set_screen_char(x, y, place.map(<&Place>::into));
        }
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
    blocks: Vec<BackgroundBlock>,
    map_place: MapPlace,
}

impl MyGame {
    fn init(&mut self) {
        self.blocks = vec![
            BackgroundBlock::new(BackgroundVariant::Grass, Position(5, 6)),
            BackgroundBlock::new(BackgroundVariant::Sand, Position(6, 8)),
            BackgroundBlock::new(BackgroundVariant::Sand, Position(6, 9)),
            BackgroundBlock::new(BackgroundVariant::Rocks, Position(5, 10)),
            BackgroundBlock::new(BackgroundVariant::Rocks, Position(5, 11)),
            BackgroundBlock::new(BackgroundVariant::Cinderblock, Position(9, 20)),
            BackgroundBlock::new(BackgroundVariant::Cinderblock, Position(9, 19)),
            BackgroundBlock::new(BackgroundVariant::Flowers, Position(10, 20)),
            BackgroundBlock::new(BackgroundVariant::Flowers, Position(11, 20)),
        ];
    }

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

impl Controller for MyGame {
    fn on_start(&mut self, game: &mut Game) {
        self.player.move_to(3, 3);
        self.viewport_size = game.screen_size();

        self.map_place.update_player(&mut self.player);
        for b in &mut self.blocks {
            self.map_place.update_background(b);
        }
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

        self.map_place.update_player(&mut self.player);
        self.map_place.draw(game);

        self.control.clear();
        game.set_viewport(self.viewport_position.into());

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
    controller.init();

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
