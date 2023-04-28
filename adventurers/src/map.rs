use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs::read_to_string;
use std::path::Path;

use termgame::StyledCharacter;

use crate::player::Player;
use crate::utils::{
    BackgroundVariant, ForegroundVariant, MapObjectVariant, Position, RawMapObject,
};

const PLAYER_ICON: char = '☻';
const FLAG: char = '⚑';

pub type RawGameMap = HashMap<Position, RawMapObject>;

pub fn read_map_data<P: AsRef<Path>>(path: P) -> Result<RawGameMap, Box<dyn Error>> {
    let content = read_to_string(path)?;
    let game_map = ron::from_str::<RawGameMap>(&content)?;
    Ok(game_map)
}

#[derive(Default)]
pub struct MapLayers {
    pub player: Position,
    pub foregrounds: HashMap<Position, ForegroundVariant>,
    pub backgrounds: HashMap<Position, BackgroundVariant>,
    pub should_draw: Vec<Position>,
    pub waters: HashSet<Position>,
    pub barriers: HashSet<Position>,
}

impl MapLayers {
    /// render a position into StyledCharacter
    pub fn get(&self, position: &Position) -> Option<StyledCharacter> {
        let mut sc = StyledCharacter::new(' ');
        if let Some(foreground) = self.foregrounds.get(position) {
            match foreground {
                ForegroundVariant::Object(c) => {
                    sc.c = *c;
                }
                ForegroundVariant::Sign(_) => {
                    sc.c = FLAG;
                }
            }
        }

        if let Some(background) = self.backgrounds.get(position) {
            sc.style = Some(background.into());
        }

        if self.player == *position {
            sc.c = PLAYER_ICON;
        }

        Some(sc)
    }

    pub fn update_player(&mut self, player: &mut Player) {
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
    pub fn is_barrier(&self, position: &Position) -> bool {
        self.barriers.contains(position)
    }
    pub fn is_water(&self, position: &Position) -> bool {
        self.waters.contains(position)
    }
    pub fn get_style_characters(&mut self) -> Vec<(Position, Option<StyledCharacter>)> {
        let positions = self.should_draw.drain(..).collect::<Vec<_>>();
        positions
            .into_iter()
            .map(|position| (position, self.get(&position)))
            .collect()
    }
    pub fn remove_foreground(&mut self, position: &Position) {
        self.foregrounds.remove(position);
        self.should_draw.push(*position);
    }
}

impl From<&RawGameMap> for MapLayers {
    fn from(raw_game_map: &RawGameMap) -> Self {
        let mut map_layers = MapLayers::default();
        for (position, map_object) in raw_game_map {
            match map_object.into() {
                MapObjectVariant::Foreground(f) => {
                    map_layers.foregrounds.insert(*position, f);
                }
                MapObjectVariant::Background(b) => {
                    if b.is_barrier() {
                        map_layers.barriers.insert(*position);
                    }

                    if b.is_water() {
                        map_layers.waters.insert(*position);
                    }
                    map_layers.backgrounds.insert(*position, b);
                }
            }
            map_layers.should_draw.push(*position);
        }
        map_layers
    }
}
