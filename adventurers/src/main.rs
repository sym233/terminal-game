use std::error::Error;
use std::time::Duration;

use termgame::{run_game, Controller, Game, GameEvent, GameSettings, KeyCode, SimpleEvent};

mod utils;
use utils::{Control, ForegroundVariant, MessageType, Position};

mod map;
use map::{read_map_data, MapLayers, RawGameMap};

mod player;
use player::Player;

/// if distance between player and border < padding, move viewport
const VIEW_PADDING: i32 = 2;

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

        if let Some(foreground) = map_layers.foregrounds.get(&player.position) {
            match foreground {
                ForegroundVariant::Object(c) => {
                    player.bag.push(*c);
                    *message = MessageType::Pickup(*c);
                    map_layers.remove_foreground(&player.position);
                }
                ForegroundVariant::Sign(s) => {
                    *message = MessageType::Sign(s.clone());
                }
            }
        } else {
            if let MessageType::Sign(_) = message {
                *message = MessageType::None;
            }
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
            }
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

        for (Position(x, y), sc) in map_layers.get_style_characters(&player) {
            game.set_screen_char(x, y, sc);
        }

        control.clear();
        game.set_viewport(<Position>::into(*viewport_position));
        game.set_message((&*message).into());

        *frame += 1;
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // let game_map = read_map_data("../maps/full_game.ron")?;
    let game_map = read_map_data("../maps/testing_game.ron")?;

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
