use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::ops::{Add, AddAssign};
use std::time::Duration;
use termgame::{
    run_game, Controller, Game, GameColor as Color, GameEvent, GameSettings, GameStyle as Style,
    KeyCode, Message, SimpleEvent, StyledCharacter, ViewportLocation,
};

use serde::{Deserialize, Serialize};

const PLAYER_ICON: char = '☻';

/// if distance between player and border < padding, move viewport
const VIEW_PADDING: i32 = 2;
const PLAYER_INIT_OXYGEN: i32 = 10;

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
    fn player(self, player: bool) -> Self {
        Self { player, ..self }
    }
    fn background(self, background: Option<BackgroundVariant>) -> Self {
        Self {
            background: background,
            ..self
        }
    }
    fn is_water(&self) -> bool {
        if let Some(b) = &self.background {
            b.is_water()
        } else {
            false
        }
    }

    fn is_barrier(&self) -> bool {
        if let Some(b) = &self.background {
            b.is_barrier()
        } else {
            false
        }
    }
}

impl Into<StyledCharacter> for &Place {
    fn into(self) -> StyledCharacter {
        let mut c = StyledCharacter::new(' ');
        if self.player {
            c.c = PLAYER_ICON;
        }
        c.style = self.background.as_ref().map(<&BackgroundVariant>::into);
        return c;
    }
}

#[derive(Copy, Clone, Default, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
struct Position(i32, i32);

impl Into<ViewportLocation> for Position {
    fn into(self) -> ViewportLocation {
        ViewportLocation {
            x: self.0,
            y: self.1,
        }
    }
}

impl Position {
    fn is_origin(&self) -> bool {
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

struct Player {
    update_draw: bool,
    // icon: char,
    position: Position,
    oxygen: i32,
    previous_position: Option<Position>,
}

impl Player {
    fn move_to(&mut self, position: Position) {
        self.position = position;
        self.update_draw = true;
    }

    // fn move_by(&mut self, x: i32, y: i32) {
    //     self.position += &Position(x, y);
    //     self.update_draw = true;
    // }

    fn interact_background(&mut self, map: &MapPlace) {
        if map.is_water(self.position) {
            self.oxygen -= 1;
            return;
        }
        self.oxygen = PLAYER_INIT_OXYGEN;
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
enum BackgroundVariant {
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
    fn is_barrier(&self) -> bool {
        self == &BackgroundVariant::Barrier
    }
    fn is_water(&self) -> bool {
        self == &BackgroundVariant::Water
    }
}

impl Into<Color> for &BackgroundVariant {
    fn into(self) -> Color {
        match self {
            &BackgroundVariant::Grass => Color::Green,
            &BackgroundVariant::Sand => Color::LightYellow,
            &BackgroundVariant::Rock => Color::DarkGray,
            &BackgroundVariant::Cinderblock => Color::LightRed,
            &BackgroundVariant::Flowerbush => Color::LightMagenta,
            &BackgroundVariant::Barrier => Color::Black,
            &BackgroundVariant::Water => Color::LightBlue,
            _ => panic!("Unknown background"),
        }
    }
}

impl Into<Style> for &BackgroundVariant {
    fn into(self) -> Style {
        Style::new().background_color(Some(self.into()))
    }
}

type RawGameMap = HashMap<Position, BackgroundVariant>;

impl Default for Player {
    fn default() -> Self {
        Player {
            update_draw: true,
            // icon: PLAYER_CHAR,
            position: Position::default(),
            previous_position: None,
            oxygen: PLAYER_INIT_OXYGEN,
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
                place.player = false;
            }
            self.should_draw.push(position);
        }
        self.map
            .entry(player.position)
            .and_modify(|place| place.player = true)
            .or_insert_with(|| Place::default().player(true));
        player.previous_position = Some(player.position);
        player.update_draw = false;
        self.should_draw.push(player.position);
    }

    fn draw(&mut self, game: &mut Game) {
        for position in self.should_draw.drain(..) {
            let Position(x, y) = position;
            let place = self.map.get(&position);
            game.set_screen_char(x, y, place.map(<&Place>::into));
        }
    }

    fn is_water(&self, position: Position) -> bool {
        if let Some(place) = self.map.get(&position) {
            place.is_water()
        } else {
            false
        }
    }

    fn is_barrier(&self, position: Position) -> bool {
        if let Some(place) = self.map.get(&position) {
            place.is_barrier()
        } else {
            false
        }
    }
}

impl From<&RawGameMap> for MapPlace {
    fn from(raw_game_map: &RawGameMap) -> Self {
        let mut map_place: MapPlace = Default::default();
        for (&position, variant) in raw_game_map {
            match variant {
                // unimplemented
                &BackgroundVariant::Object(_) | &BackgroundVariant::Sign(_) => continue,
                _ => {}
            }
            map_place
                .map
                .entry(position)
                .and_modify(|place| {
                    place.background = Some(variant.clone());
                })
                .or_insert_with(|| Place::default().background(Some(variant.clone())));
            map_place.should_draw.push(position);
        }
        map_place
    }
}

#[derive(Default)]
enum GameStatus {
    #[default]
    Running,
    Died,
}

#[derive(Default)]
struct GameVar {
    game_status: GameStatus,
    control: Control,
    viewport_position: Position,
    message: Option<(String, String)>,
    show_message: bool,
    frame: i32,
    player: Player,
    map_place: MapPlace,
}

struct GameStatic {
    raw_game_map: RawGameMap,
    screen_size: (u16, (u16, u16)),
}

struct MyGame {
    game_var: GameVar,
    game_static: GameStatic,
}

impl MyGame {
    fn new(raw_game_map: RawGameMap) -> Self {
        let game_static = GameStatic {
            raw_game_map,
            screen_size: Default::default(),
        };
        Self {
            game_var: Default::default(),
            game_static,
        }
    }

    fn init(&mut self, game: &Game) {
        self.game_static.screen_size = game.screen_size();
        self.game_var = GameVar {
            map_place: MapPlace::from(&self.game_static.raw_game_map),
            ..Default::default()
        }
    }

    fn update_player_position(&mut self) {
        let GameVar {
            ref control,
            ref mut player,
            ref mut map_place,
            ..
        } = self.game_var;
        let move_by = Position::from(control);
        if move_by.is_origin() {
            return;
        }
        let next = player.position + move_by;
        if map_place.is_barrier(next) {
            // cannot move into barrier
            return;
        }
        player.move_to(next);
        player.interact_background(map_place);
    }
    fn update_viewport_position(&mut self) {
        let GameStatic {
            screen_size: (width, (game_height, message_height)),
            ..
        } = self.game_static;
        let GameVar {
            ref player,
            ref mut viewport_position,
            ..
        } = self.game_var;
        let Position(x, y) = player.position;

        let left = x - viewport_position.0;
        let top = y - viewport_position.1;
        let right = viewport_position.0 + width as i32 - 2 - x;
        let bottom = viewport_position.1 + game_height as i32 + message_height as i32
            - 3
            - y;
        if left < VIEW_PADDING {
            viewport_position.0 -= 1;
        }
        if top < VIEW_PADDING {
            viewport_position.1 -= 1;
        }
        if right < VIEW_PADDING {
            viewport_position.0 += 1;
        }
        if bottom < VIEW_PADDING {
            viewport_position.1 += 1;
        }
        // self.text = format!("top: {top}, left: {left}, bottom: {bottom}, right: {right}");
    }
}

impl Controller for MyGame {
    fn on_start(&mut self, game: &mut Game) {
        self.init(game);

        let GameVar {
            ref mut player,
            ref mut map_place,
            ..
        } = self.game_var;
        player.move_to(Position(3, 3));

        map_place
            .update_player(player);
    }

    fn on_event(&mut self, game: &mut Game, event: GameEvent) {
        let GameVar {
            ref mut control,
            ref mut show_message,
            ref game_status,
            ..
        } = self.game_var;
        match game_status {
            GameStatus::Died => {
                match event.into() {
                    SimpleEvent::Just(KeyCode::Enter) => {
                        self.init(game);
                        self.on_start(game);
                    }
                    _ => {}
                }
                return;
            }
            _ => {}
        }

        match event.into() {
            SimpleEvent::Just(key_code) => match key_code {
                KeyCode::Char(ch) => match ch {
                    't' => {
                        // self.text = format!("vp: {:?}", game.screen_size());
                        *show_message = !*show_message;
                    }
                    _ => {}
                },
                // KeyCode::Enter => {
                //     self.show_text = !self.show_text;
                // }
                KeyCode::Left => {
                    control.left = true;
                }
                KeyCode::Right => {
                    control.right = true;
                }
                KeyCode::Up => {
                    control.up = true;
                }
                KeyCode::Down => {
                    control.down = true;
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn on_tick(&mut self, game: &mut Game) {
        self.update_player_position();
        self.update_viewport_position();

        let GameVar {
            ref mut player,
            ref mut map_place,
            ref mut control,
            ref viewport_position,
            ref mut message,
            ref mut frame,
            ref mut show_message,
            ref mut game_status,
            ..
        } = self.game_var;

        map_place
            .update_player(player);
        map_place.draw(game);

        control.clear();
        game.set_viewport(<Position>::into(*viewport_position));

        // let f = format!(
        //     "player on {}, oxygen {:2}.",
        //     if self.map_place
        //         .map
        //         .get(&self.player.position)
        //         .map(|p| p.water)
        //         .unwrap() { "water" } else { "other" },
        //     self.player.oxygen
        // );
        // for (i, ch) in f.chars().enumerate() {
        //     game.set_screen_char(30 + i as i32, 10, Some(StyledCharacter::new(ch)));
        // }

        // Debug Message:
        *message = Some((
            "Test".into(),
            format!(
                "Pos: {}",
                ron::to_string(&player.position).unwrap()
            ),
        ));

        if player.oxygen <= 0 {
            *message = Some((
                "You Died".into(),
                "You died from drown, press Enter to restart.".into(),
            ));
            *show_message = true;
            *game_status = GameStatus::Died;
        }

        if *show_message {
            if let Some((title, text)) = &message {
                let msg = Message::new(text.clone()).title(title.clone());
                game.set_message(Some(msg));
            }
        } else {
            game.set_message(None);
        }

        *frame += 1;
    }
}

fn read_map_data() -> Result<RawGameMap, Box<dyn Error>> {
    // let path = "../maps/full_game.ron";
    let path = "../maps/testing_game.ron";
    let content = fs::read_to_string(path)?;
    let game_map = ron::from_str::<RawGameMap>(&content)?;
    Ok(game_map)
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("Welcome to AdventureRS");
    println!("To get started, you should read the termgame documentation,");
    println!("and try out getting a termgame UI to appear on your terminal.");

    let game_map = read_map_data()?;

    let mut controller = MyGame::new(game_map);

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
