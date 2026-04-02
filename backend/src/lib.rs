use std::{sync::LazyLock, time::Duration};

use spacetimedb::{
    rand::{seq::SliceRandom, Rng},
    SpacetimeType,
};

const BOARD_WIDTH: u8 = 10;
const BOARD_HEIGHT: u8 = 10;
const PLAYER_TURN_DURATION: Duration = Duration::from_secs(2);
const TICK_INTERVAL: Duration = Duration::from_millis(250);

#[derive(SpacetimeType, PartialEq, Clone)]
pub struct Position {
    pub x: u8,
    pub y: u8,
}

impl Position {
    pub fn pick_random<R>(
        mut rng: &mut R,
        unavailable_positions: &Vec<Position>,
    ) -> Option<Position>
    where
        R: Rng + ?Sized,
    {
        let possible_positions: Vec<&Position> = BOARD_POSITIONS
            .iter()
            .filter(|pos| !unavailable_positions.contains(pos))
            .collect();
        if possible_positions.is_empty() {
            return None;
        }
        let pick = *(possible_positions.choose(&mut rng).unwrap());
        Some(pick.clone())
    }
}

static BOARD_POSITIONS: LazyLock<Vec<Position>> = LazyLock::new(|| {
    (0..BOARD_WIDTH)
        .into_iter()
        .flat_map(|x| {
            (0..BOARD_HEIGHT)
                .into_iter()
                .map(move |y| Position { x: x, y: y })
        })
        .collect()
});

pub mod reducers;
pub mod tables;
pub mod views;
