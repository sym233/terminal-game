use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs;
use std::time::Duration;

use termgame::{
    run_game, Controller, Game, GameColor as Color, GameEvent, GameSettings, GameStyle as Style,
    KeyCode, SimpleEvent, StyledCharacter,
};

mod utils;
use utils::{Position, Control, MessageType, BackgroundVariant};

const PLAYER_ICON: char = '☻';
const FLAG: char = '⚑';

/// if distance between player and border < padding, move viewport
const VIEW_PADDING: i32 = 2;
const PLAYER_INIT_OXYGEN: i32 = 10;

// #[derive(Debug, Clone, Copy)]
// enum ObjectVariant {
//     Player,
//     Background(BackgroundVariant),
// }

type RawGameMap = HashMap<Position, BackgroundVariant>;

#[derive(Default)]
struct MapLayers {
    player: Position,
    objects: HashMap<Position, char>,
    signs: HashMap<Position, String>,
    backgrounds: HashMap<Position, Color>,
    should_draw: Vec<Position>,
    waters: HashSet<Position>,
    barriers: HashSet<Position>,
}

impl MapLayers {
    fn get(&self, position: &Position) -> Option<StyledCharacter> {
        let mut sc = StyledCharacter::new(' ');

        if let Some(&c) = self.objects.get(position) {
            sc.c = c;
        }

        if self.signs.contains_key(position) {
            sc.c = FLAG;
        }

        if let Some(&color) = self.backgrounds.get(position) {
            sc.style = Some(Style::new().background_color(Some(color)));
        }

        if self.player == *position {
            sc.c = PLAYER_ICON;
        }

        Some(sc)
    }

    fn update_player(&mut self, player: &mut Player) {
        if !player.update_draw {
            return;
        }
        if let Some(position) = player.previous_position.take() {
            self.should_draw.push(position);
        }
        let position = player.position;

        self.player = position;
        player.previous_position = Some(position);
        self.should_draw.push(position);
        player.update_draw = false;
    }
    fn is_barrier(&self, position: &Position) -> bool {
        self.barriers.contains(position)
    }
    fn is_water(&self, position: &Position) -> bool {
        self.waters.contains(position)
    }
    fn get_style_characters(&mut self) -> Vec<(Position, Option<StyledCharacter>)> {
        let positions = self.should_draw.drain(..).collect::<Vec<_>>();
        positions.into_iter().map(|position| (position, self.get(&position))).collect()
    }
}

impl From<&RawGameMap> for MapLayers {
    fn from(raw_game_map: &RawGameMap) -> Self {
        let mut map_layers = MapLayers::default();
        for (position, background) in raw_game_map {
            if background.is_barrier() {
                map_layers.barriers.insert(*position);
            }
            if background.is_water() {
                map_layers.waters.insert(*position);
            }
            if let Some(color) = background.into() {
                map_layers.backgrounds.insert(*position, color);
            }
            if let BackgroundVariant::Object(c) = background {
                map_layers.objects.insert(*position, *c);
            }
            if let BackgroundVariant::Sign(s) = background {
                map_layers.signs.insert(*position, s.clone());
            }
            map_layers.should_draw.push(*position);
        }
        map_layers
    }
}

struct Player {
    update_draw: bool,
    // icon: char,
    position: Position,
    bag: Vec<char>,
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

    fn interact_background(&mut self, map: &MapLayers) {
        if map.is_water(&self.position) {
            self.oxygen -= 1;
            return;
        }
        self.oxygen = PLAYER_INIT_OXYGEN;
    }
}

impl Default for Player {
    fn default() -> Self {
        Player {
            update_draw: true,
            // icon: PLAYER_CHAR,
            position: Position::default(),
            bag: Default::default(),
            previous_position: None,
            oxygen: PLAYER_INIT_OXYGEN,
        }
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
    message: MessageType,
    frame: i32,
    player: Player,
    map_layers: MapLayers,
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
            map_layers: MapLayers::from(&self.game_static.raw_game_map),
            ..Default::default()
        }
    }

    fn update_player_position(&mut self) {
        let GameVar {
            ref control,
            ref mut player,
            ref map_layers,
            ..
        } = self.game_var;
        let move_by = Position::from(control);
        if move_by.is_origin() {
            return;
        }
        let next = player.position + move_by;
        if map_layers.is_barrier(&next) {
            // cannot move into barrier
            return;
        }
        player.move_to(next);
        player.interact_background(map_layers);

        self.update_message_and_status();
    }

    fn update_message_and_status(&mut self) {
        let GameVar {
            ref mut player,
            ref mut map_layers,
            ref mut message,
            ref mut game_status,
            ..
        } = self.game_var;

        if let Some(s) = map_layers.signs.get(&player.position) {
            *message = MessageType::Sign(s.clone());
        } else {
            if let MessageType::Sign(_) = message {
                *message = MessageType::None;
            }
        }

        if let Some(c) = map_layers.objects.remove(&player.position) {
            player.bag.push(c);
            *message = MessageType::Pickup(c);
        } else {
            if let MessageType::Pickup(_) = message {
                *message = MessageType::None;
            }
        }

        if player.oxygen <= 0 {
            *message = MessageType::Death("You died from drown, press Enter to restart".into());
            *game_status = GameStatus::Died;
        }
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
        let bottom = viewport_position.1 + game_height as i32 + message_height as i32 - 3 - y;
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
    }
}

impl Controller for MyGame {
    fn on_start(&mut self, game: &mut Game) {
        self.init(game);

        let GameVar {
            ref mut player,
            ref mut map_layers,
            ..
        } = self.game_var;
        player.move_to(Position(3, 3));

        map_layers.update_player(player);
    }

    fn on_event(&mut self, game: &mut Game, event: GameEvent) {
        let GameVar {
            ref mut control,
            ref mut message,
            ref game_status,
            ref player,
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
            SimpleEvent::Just(key_code) => {
                match key_code {
                    KeyCode::Char(ch) => {
                        match ch {
                            't' => {
                                // debug message
                                if let MessageType::Debug(_) = message {
                                    *message = MessageType::None;
                                } else {
                                    *message = MessageType::Debug(format!(
                                        "player pos: {}",
                                        ron::to_string(&player.position).unwrap()
                                    ));
                                }
                            }
                            'b' => {
                                // check bag
                                if let MessageType::Bag(_) = message {
                                    *message = MessageType::None;
                                } else {
                                    *message = MessageType::Bag(format!("{:?}", player.bag));
                                } 
                            }
                            _ => {}
                        };                    
                    }
                    _ => {}
                };
                control.update(key_code);
            },
            _ => {}
        }
    }

    fn on_tick(&mut self, game: &mut Game) {
        self.update_player_position();
        self.update_viewport_position();

        let GameVar {
            ref mut player,
            ref mut map_layers,
            ref mut control,
            ref viewport_position,
            ref mut message,
            ref mut frame,
            ..
        } = self.game_var;

        map_layers.update_player(player);

        for (Position(x, y), sc) in map_layers.get_style_characters() {
            game.set_screen_char(x, y, sc);
        }

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
        game.set_message((&*message).into());

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
    let game_map = read_map_data()?;

    let mut controller = MyGame::new(game_map);

    run_game(
        &mut controller,
        GameSettings::new()
            .tick_duration(Duration::from_millis(50))
            .quit_event(Some(SimpleEvent::WithControl(KeyCode::Char('c')).into())),
    )?;

    println!("Game Ended!");

    Ok(())
}
