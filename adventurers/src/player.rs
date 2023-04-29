use crate::map::MapLayers;
use crate::utils::Position;

const PLAYER_ICON: char = '☻';
const PLAYER_INIT_OXYGEN: i32 = 10;

pub struct Player {
    pub update_draw: bool,
    pub icon: char,
    pub position: Position,
    pub bag: Vec<char>,
    pub oxygen: i32,
    pub previous_position: Option<Position>,
}

impl Player {
    pub fn move_to(&mut self, position: Position) {
        self.position = position;
        self.update_draw = true;
    }

    pub fn interact_background(&mut self, map: &MapLayers) {
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
            icon: PLAYER_ICON,
            position: Default::default(),
            bag: Default::default(),
            previous_position: None,
            oxygen: PLAYER_INIT_OXYGEN,
        }
    }
}
