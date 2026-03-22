use spacetimedb::{rand::Rng, Identity, ScheduleAt, SpacetimeType};

use crate::{
    reducers::{game_tick, pick_next_active_player, remove_closed_games},
    Position, BOARD_HEIGHT, BOARD_WIDTH, PLAYER_TURN_DURATION, TICK_INTERVAL,
};

#[derive(SpacetimeType, PartialEq, Copy, Clone)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(SpacetimeType, PartialEq)]
pub enum GameState {
    Idle,
    Running,
    GameOver,
    Win,
    Closed,
}

impl Direction {
    pub fn apply_to(self, org: &Position) -> Position {
        let (offset_x, offset_y) = match self {
            Direction::Up => (0, -1i8),
            Direction::Down => (0, 1),
            Direction::Left => (-1i8, 0),
            Direction::Right => (1, 0),
        };
        Position {
            x: match org.x as i8 + offset_x {
                v if v < 0 => BOARD_WIDTH,
                v if v >= BOARD_WIDTH as i8 => 0,
                v => v as u8,
            },
            y: match org.y as i8 + offset_y {
                v if v < 0 => BOARD_HEIGHT,
                v if v >= BOARD_HEIGHT as i8 => 0,
                v => v as u8,
            },
        }
    }

    pub fn is_opposite(self, other: Direction) -> bool {
        match self {
            Direction::Up => other == Direction::Down,
            Direction::Down => other == Direction::Up,
            Direction::Left => other == Direction::Right,
            Direction::Right => other == Direction::Left,
        }
    }
}

#[spacetimedb::table(accessor = user, index(accessor = by_game_id_and_online, btree(columns = [game_id, online])))]
pub struct User {
    #[primary_key]
    pub identity: Identity,
    pub name: Option<String>,
    pub online: bool,
    pub game_id: u64,
}

#[spacetimedb::table(accessor = game, scheduled(game_tick))]
pub struct Game {
    #[primary_key]
    #[auto_inc]
    pub game_id: u64,
    pub scheduled_at: ScheduleAt,
    #[index(btree)]
    pub owner: Identity,
    pub name: String,
    pub state: GameState,
    #[index(btree)]
    pub joinable: bool,
}

impl Game {
    pub fn new(owner: Identity, name: String) -> Self {
        Game {
            game_id: 0,
            scheduled_at: ScheduleAt::Interval(TICK_INTERVAL.into()),
            owner: owner,
            name: name,
            state: GameState::Idle,
            joinable: true,
        }
    }

    pub fn provide_active_player(&self) -> ActivePlayer {
        ActivePlayer {
            game_id: self.game_id,
            scheduled_at: ScheduleAt::Interval(PLAYER_TURN_DURATION.into()),
            active_player: self.owner,
        }
    }

    pub fn provide_food<R>(&self, mut rng: &mut R, snake: &Snake) -> Food
    where
        R: Rng + ?Sized,
    {
        Food {
            game_id: self.game_id,
            position: Position::pick_random(&mut rng, &snake.body).unwrap(),
        }
    }

    pub fn provide_snake(&self) -> Snake {
        Snake {
            game_id: self.game_id,
            body: vec![
                Position {
                    x: (BOARD_WIDTH / 2),
                    y: (BOARD_HEIGHT / 2),
                },
                Position {
                    x: (BOARD_WIDTH / 2) - 1,
                    y: (BOARD_HEIGHT / 2),
                },
                Position {
                    x: (BOARD_WIDTH / 2) - 2,
                    y: (BOARD_HEIGHT / 2),
                },
            ],
        }
    }

    pub fn provide_next_direction(&self) -> NextDirection {
        NextDirection {
            game_id: self.game_id,
            previous: Direction::Right,
            next: Direction::Right,
        }
    }
}

#[spacetimedb::table(accessor = game_cleanup, scheduled(remove_closed_games))]
pub struct GameCleanup {
    #[primary_key]
    #[auto_inc]
    pub scheduled_id: u64,
    pub scheduled_at: ScheduleAt,
    pub game_id: u64,
}

#[spacetimedb::table(accessor = active_player, scheduled(pick_next_active_player))]
pub struct ActivePlayer {
    #[primary_key]
    pub game_id: u64,
    pub scheduled_at: ScheduleAt,
    pub active_player: Identity,
}

#[spacetimedb::table(accessor = food)]
pub struct Food {
    #[primary_key]
    pub game_id: u64,
    pub position: Position,
}

#[spacetimedb::table(accessor = snake)]
#[derive(Clone)]
pub struct Snake {
    #[primary_key]
    pub game_id: u64,
    pub body: Vec<Position>,
}

#[spacetimedb::table(accessor = next_direction)]
pub struct NextDirection {
    #[primary_key]
    pub game_id: u64,
    pub previous: Direction,
    pub next: Direction,
}
