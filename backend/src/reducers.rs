use crate::{tables::*, Position};
use std::time::Duration;

use spacetimedb::{rand::seq::SliceRandom, ReducerContext, ScheduleAt, Table};

#[spacetimedb::reducer]
fn set_user_name(ctx: &ReducerContext, name: String) -> Result<(), String> {
    // Validate the parameter
    if name.is_empty() {
        return Err("Name must not be empty".to_string());
    }

    // Find the user and update the name
    if let Some(user) = ctx.db.user().identity().find(ctx.sender()) {
        ctx.db.user().identity().update(User {
            name: Some(name),
            ..user
        });
    } else {
        return Err("Cannot set name for unknown user".to_string());
    }

    Ok(())
}

#[spacetimedb::reducer(client_connected)]
fn identity_connected(ctx: &ReducerContext) {
    if let Some(user) = ctx.db.user().identity().find(ctx.sender()) {
        // Update the existing user as online
        ctx.db.user().identity().update(User {
            online: true,
            ..user
        });
    } else {
        // Create a new user as online
        ctx.db.user().insert(User {
            name: None,
            identity: ctx.sender(),
            online: true,
            game_id: 0,
        });
    }
}

#[spacetimedb::reducer(client_disconnected)]
fn identity_disconnected(ctx: &ReducerContext) {
    if let Some(user) = ctx.db.user().identity().find(ctx.sender()) {
        // Update user as offline
        ctx.db.user().identity().update(User {
            online: false,
            ..user
        });
    }
}

#[spacetimedb::reducer]
fn create_game(ctx: &ReducerContext, name: String) -> Result<(), String> {
    // Validate parameter
    if name.is_empty() {
        return Err("Name must not be empty".to_string());
    }

    // Create the game and associated records
    let game = ctx.db.game().insert(Game::new(ctx.sender(), name));
    ctx.db.active_player().insert(game.provide_active_player());
    let snake = ctx.db.snake().insert(game.provide_snake());
    ctx.db
        .food()
        .insert(game.provide_food(&mut ctx.rng(), &snake));
    ctx.db
        .next_direction()
        .insert(game.provide_next_direction());

    // Update the user
    let user = ctx
        .db
        .user()
        .identity()
        .find(ctx.sender())
        .ok_or("No user")?;
    ctx.db.user().identity().update(User {
        game_id: game.game_id,
        ..user
    });

    Ok(())
}

#[spacetimedb::reducer]
fn join_game(ctx: &ReducerContext, game_id: u64) -> Result<(), String> {
    if let Some(user) = ctx.db.user().identity().find(ctx.sender()) {
        // Update user game id
        ctx.db.user().identity().update(User {
            game_id: game_id,
            ..user
        });
    }
    Ok(())
}

#[spacetimedb::reducer]
pub fn game_tick(ctx: &ReducerContext, game: Game) -> Result<(), String> {
    let game_id = game.game_id;

    // Check for game over
    let game = ctx.db.game().game_id().find(game_id).ok_or("no game")?;
    if game.state != GameState::Running {
        return Ok(());
    }

    // Get the snake
    let snake = ctx
        .db
        .snake()
        .game_id()
        .find(game_id)
        .ok_or("No snake for game")?;

    // Assert that the body is not empty
    let mut next_body = snake.body.clone();
    if next_body.is_empty() {
        return Err("Snake body must not be empty".to_string());
    }

    // Determine the next head position based on the next direction
    let next_direction = ctx
        .db
        .next_direction()
        .game_id()
        .find(game_id)
        .ok_or("No next direction for game")?;
    ctx.db.next_direction().game_id().update(NextDirection {
        previous: next_direction.next,
        ..next_direction
    });
    let next_head = next_direction.next.apply_to(next_body.first().unwrap());

    // Check alive (next head position not in body)
    if next_body.contains(&next_head) {
        ctx.db.game().game_id().update(Game {
            state: GameState::GameOver,
            ..game
        });
        return Ok(());
    }
    next_body.insert(0, next_head.clone());

    // Check food (next head position on food)
    let food = ctx
        .db
        .food()
        .game_id()
        .find(game_id)
        .ok_or("No food for game")?;
    if food.position == next_head {
        if let Some(next_food_position) = Position::pick_random(&mut ctx.rng(), &next_body) {
            // Random position is available; update food position
            ctx.db.food().game_id().update(Food {
                game_id: game_id,
                position: next_food_position,
            });
        } else {
            // No more positions available; Win!
            ctx.db.game().game_id().update(Game {
                state: GameState::Win,
                ..game
            });
            return Ok(());
        }
    } else {
        next_body.pop();
    }

    // Update the snake
    ctx.db.snake().game_id().update(Snake {
        body: next_body,
        ..snake
    });

    Ok(())
}

#[spacetimedb::reducer]
pub fn pick_next_active_player(
    ctx: &ReducerContext,
    active_player: ActivePlayer,
) -> Result<(), String> {
    // select the snake to verify that it is alive
    let game = ctx
        .db
        .game()
        .game_id()
        .find(active_player.game_id)
        .ok_or("no game")?;
    if game.state != GameState::Running {
        return Ok(());
    }

    // select all the players that are in the game
    let players: Vec<User> = ctx
        .db
        .user()
        .by_game_id_and_online()
        .filter((active_player.game_id, true))
        .collect();
    if players.len() == 1 && players.first().unwrap().identity == active_player.active_player {
        return Ok(());
    }

    // pick another player at random to be the active player
    let mut next_player_id = active_player.active_player;
    while next_player_id == active_player.active_player {
        if let Some(next_player) = players.choose(&mut ctx.rng()) {
            next_player_id = next_player.identity;
        }
    }
    ctx.db.active_player().game_id().update(ActivePlayer {
        active_player: next_player_id,
        ..active_player
    });

    return Ok(());
}

#[spacetimedb::reducer]
fn set_next_direction(
    ctx: &ReducerContext,
    game_id: u64,
    direction: Direction,
) -> Result<(), String> {
    let active_player = ctx
        .db
        .active_player()
        .game_id()
        .find(game_id)
        .ok_or("no active player")?;
    if active_player.active_player != ctx.sender() {
        return Err("Not active player".to_string());
    }
    let next_direction = ctx
        .db
        .next_direction()
        .game_id()
        .find(game_id)
        .ok_or("No next direction")?;
    if next_direction.previous.is_opposite(direction) {
        return Ok(());
    }
    ctx.db.next_direction().game_id().update(NextDirection {
        next: direction,
        ..next_direction
    });

    Ok(())
}

#[spacetimedb::reducer]
fn restart_game(ctx: &ReducerContext, game_id: u64) -> Result<(), String> {
    if let Some(game) = ctx.db.game().game_id().find(game_id) {
        if [GameState::Running, GameState::Closed].contains(&game.state) {
            return Err("Game cannot be started right now!".to_string());
        }

        if game.owner == ctx.sender() {
            // Reset the game objects if necessary
            if game.state != GameState::Idle {
                let snake = ctx.db.snake().game_id().update(game.provide_snake());
                ctx.db
                    .food()
                    .game_id()
                    .update(game.provide_food(&mut ctx.rng(), &snake));
                ctx.db
                    .next_direction()
                    .game_id()
                    .update(game.provide_next_direction());
            }

            // Update the game to be running
            ctx.db.game().game_id().update(Game {
                state: GameState::Running,
                ..game
            });
        }
    }
    Ok(())
}

#[spacetimedb::reducer]
fn close_game(ctx: &ReducerContext, game_id: u64) {
    if let Some(game) = ctx.db.game().game_id().find(game_id) {
        if game.owner == ctx.sender() {
            // Update the game to be closed
            ctx.db.game().game_id().update(Game {
                state: GameState::Closed,
                joinable: false,
                ..game
            });

            // Schedule the game cleanup
            ctx.db.game_cleanup().insert(GameCleanup {
                scheduled_id: 0,
                scheduled_at: ScheduleAt::Time(ctx.timestamp + Duration::from_mins(1)),
                game_id: game.game_id,
            });
        }
    }
}

#[spacetimedb::reducer]
fn exit_game(ctx: &ReducerContext, game_id: u64) -> Result<(), String> {
    // Get the game and check if the user is not the owner (cannot leave in that case)
    let game = ctx.db.game().game_id().find(game_id).ok_or("no game")?;
    if game.owner == ctx.sender() {
        return Err("cannot exit the game as the owner of the game".to_string());
    }

    // Update the user
    let user = ctx
        .db
        .user()
        .identity()
        .find(ctx.sender())
        .ok_or("no user")?;
    ctx.db.user().identity().update(User { game_id: 0, ..user });

    // If the user is the active player, pick another active player
    if let Some(active_player) = ctx.db.active_player().game_id().find(game_id) {
        if active_player.active_player == ctx.sender() {
            pick_next_active_player(ctx, active_player)?;
        }
    }

    Ok(())
}

#[spacetimedb::reducer]
pub fn remove_closed_games(ctx: &ReducerContext, game_cleanup: GameCleanup) {
    // Remove all records related to the game
    ctx.db.game().game_id().delete(game_cleanup.game_id);
    ctx.db
        .active_player()
        .game_id()
        .delete(game_cleanup.game_id);
    ctx.db.food().game_id().delete(game_cleanup.game_id);
    ctx.db.snake().game_id().delete(game_cleanup.game_id);
    ctx.db
        .next_direction()
        .game_id()
        .delete(game_cleanup.game_id);

    // Update all players and reset the game id
    for player in ctx
        .db
        .user()
        .by_game_id_and_online()
        .filter(game_cleanup.game_id)
    {
        ctx.db.user().identity().update(User {
            game_id: 0u64,
            ..player
        });
    }
}

#[spacetimedb::reducer]
fn update_game_speed(ctx: &ReducerContext, game_id: u64, interval_ms: u64) {
    if let Some(game) = ctx.db.game().game_id().find(game_id) {
        ctx.db.game().game_id().update(Game {
            scheduled_at: ScheduleAt::Interval(Duration::from_millis(interval_ms).into()),
            ..game
        });
    }
}
