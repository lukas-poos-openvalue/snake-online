use spacetimedb::{Identity, SpacetimeType, ViewContext};

use crate::{tables::*, Position, BOARD_HEIGHT, BOARD_WIDTH};

#[derive(SpacetimeType)]
enum BoardCellContent {
    None,
    Snake,
    Food,
}

#[derive(SpacetimeType)]
struct PlayerDto {
    identity: Identity,
    name: String,
    is_owner: bool,
    is_active: bool,
}

#[derive(SpacetimeType)]
struct JoinableGameDto {
    game_id: u64,
    name: String,
    owner_name: String,
    player_count: u8,
}

#[derive(SpacetimeType)]
struct ActiveGameDto {
    game_id: u64,
    name: String,
    score: u32,
    state: GameState,
    players: Vec<PlayerDto>,
    board: Vec<Vec<BoardCellContent>>,
}

#[spacetimedb::view(accessor = my_user, public)]
fn my_user(ctx: &ViewContext) -> Option<User> {
    ctx.db.user().identity().find(ctx.sender())
}

#[spacetimedb::view(accessor = my_game, public)]
fn my_game(ctx: &ViewContext) -> Option<Game> {
    let user = ctx.db.user().identity().find(ctx.sender())?;
    ctx.db.game().game_id().find(user.game_id)
}

#[spacetimedb::view(accessor = joinable_game, public)]
fn joinable_game(ctx: &ViewContext) -> Vec<JoinableGameDto> {
    ctx.db
        .game()
        .joinable()
        .filter(true)
        .flat_map(|game| {
            let owner = ctx.db.user().identity().find(game.owner)?;
            let player_count = ctx
                .db
                .user()
                .by_game_id_and_online()
                .filter(game.game_id)
                .count();
            return Some(JoinableGameDto {
                game_id: game.game_id,
                name: game.name,
                owner_name: owner.name.unwrap_or("Annonymous".to_string()),
                player_count: player_count as u8,
            });
        })
        .collect()
}

#[spacetimedb::view(accessor = active_game, public)]
fn active_game(ctx: &ViewContext) -> Option<ActiveGameDto> {
    let current_user = my_user(ctx)?;
    let game = ctx.db.game().game_id().find(current_user.game_id)?;
    let players: Vec<User> = ctx
        .db
        .user()
        .by_game_id_and_online()
        .filter(game.game_id)
        .collect();
    let snake = ctx.db.snake().game_id().find(game.game_id)?;
    let food = ctx.db.food().game_id().find(game.game_id)?;
    let active_player = ctx.db.active_player().game_id().find(game.game_id)?;
    return Some(ActiveGameDto {
        game_id: game.game_id,
        name: game.name,
        score: snake.body.len() as u32 - 3,
        state: game.state,
        players: players
            .iter()
            .map(|p| PlayerDto {
                identity: p.identity,
                name: p.name.clone().unwrap_or("Annonymous".to_string()),
                is_active: active_player.active_player == p.identity,
                is_owner: game.owner == p.identity,
            })
            .collect(),
        board: (0..BOARD_HEIGHT)
            .map(|y| {
                (0..BOARD_WIDTH)
                    .map(|x| {
                        let pos = Position { x, y };
                        if snake.body.contains(&pos) {
                            BoardCellContent::Snake
                        } else if food.position == pos {
                            BoardCellContent::Food
                        } else {
                            BoardCellContent::None
                        }
                    })
                    .collect()
            })
            .collect(),
    });
}
