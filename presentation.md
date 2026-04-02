# SpacetimeDB
```
A real-time backend framework and database for apps and games!
```

---

# Table of Content
- Introduction
  - What is SpacetimeDB?
  - The "Zen" of SpacetimeDB
  - Language Support
  - Key Architecture
- Roundtrip
  - Workflow: spacetime start / generate / build / publish / dev
  - Basics: Tables / Reducers / Subscriptions
  - Essentials: Views / Schedulers / Clients
  - Advanced: Authentication / Authorization
- Demo: Snake-Online

---

# Introduction

## What is SpacetimeDB?
- A database that is also a server (or: a persistent and ACID-compliant application server)
- Provides fully featured relational database system
- Provides means to run application logic on the data
- No separation of database and application server simplifies the entire system immensely

## The "Zen" of SpacetimeDB
- Five core principles make the applications you build stupidly easy
  - Everything is a Table
    - no separate cache, no Redis, no in-memory state
    - Allows stuff like hot-swapping server code and use SQL on the state
  - Everything is persistent
    - Senseful default
    - Too much data? Not really with compression, and you can manage it easily
    - Too slow? Not really, SSDs write up to 15GB/s, you'll be fine
  - Everything is Real-Time
    - No more request-response, just subscribe
    - Real-time updates are baked into the database itself
  - Everything is Transactional
    - Each piece of application logic is atomic
    - No more distributed transactions
  - Everything is Programmable
    - Modules are real code running inside the database
    - Just script everything: Authorization / Validation / Transformation / ...

---

## Language Support
- Server Logic / Database Modules
  - Rust / WASM
  - C# / WASM
  - TypeScript / V8
- Clients
  - Rust
  - C#
  - TypeScript
  - C++ for Unreal Engine

## Key Architecture
- Host: Server that runs database
- Database: 
  - Runs on a host
  - Modules can be published to a database
- Module:
  - Collection of tables / reducers / ...
  - Written in C#, Rust, TypeScript
- Table: Same as in any relational database
- Reducer: 
  - Callable function that interacts with the database
  - Atomic by nature
- Procedure:
  - Callable function that can interact with the database (requires explicit transaction management)
  - Also allows side effects (HTTP requests, IO operations, etc)
- View:
  - Read-only function that just return results
  - Good for access control / visibility / aggregation
- Client:
  - Application that connects to the database
  - Has an identity and a connection id
- Identity:
  - Identifies someone interacting with a database
  - Attached to all interactions with the database
  - Issued using OIDC

---

# Roundtrip

## Workflow

### Start Database Locally
```bash
spacetime start --in-memory
```

### Generate module_bindings
```bash
spacetime generate --lang typescript --module-path backend --out-dir frontend/src/module_bindings
```

### Build Module
```bash
spacetime build --module-path backend
```

### Publish Module
```bash
spacetime publish --server local --module-path backend
```

### Dev Server
```bash
spacetime dev --module-path backend --module-bindings-path frontend/src/module_bindings --client-lang typescript --run "just dev-frontend"
```

---

## Basics

### Tables
Here is how to define it in Rust:
```rust
#[spacetimedb::table(accessor = person, public)]
pub struct Person {
    #[primary_key]
    #[auto_inc]
    id: u32,
    #[index(btree)]
    name: String,
    #[unique]
    email: String,
}
```

Here is how to use it in Rust:
```rust
// Table definition
#[spacetimedb::table(accessor = player, public)]
pub struct Player { /* columns */ }

// Accessor matches name exactly
ctx.db.player().insert(Player { /* ... */ });
```

---

### Reducers
Here is how to define a reducer in Rust:
```rust
use spacetimedb::{reducer, ReducerContext, Table};

#[reducer]
pub fn create_user(ctx: &ReducerContext, name: String, email: String) -> Result<(), String> {
    // Validate input
    if name.is_empty() {
        return Err("Name cannot be empty".to_string());
    }

    // Modify tables
    ctx.db.user().insert(User {
        id: 0, // auto-increment will assign
        name,
        email,
    });

    Ok(())
}
```

---

### Subscriptions
Here is how to subscribe to data in Rust:
```rust
let conn = DbConnection::builder()
    .with_uri("wss://maincloud.spacetimedb.com")
    .with_database_name("chatapp")
    .with_token(creds_store().load().expect("Error loading credentials"))
    .build();

// When a new user joins, print a notification.
conn.db.user().on_insert(on_user_inserted);

// When a user's status changes, print a notification.
conn.db.user().on_update(on_user_updated);

// When a new message is received, print it.
conn.db.message().on_insert(on_message_inserted);
```

---

## Essentials

### Views
```rust
// At-most-one row: return Option<T>
#[view(accessor = my_player, public)]
fn my_player(ctx: &ViewContext) -> Option<Player> {
    ctx.db.player().identity().find(ctx.sender())
}

// Multiple rows: return Vec<T>
#[view(accessor = players_for_level, public)]
fn players_for_level(ctx: &AnonymousViewContext) -> Vec<PlayerAndLevel> {
    ctx.db
        .player_level()
        .level()
        .filter(2u64)
        .flat_map(|player| {
            ctx.db.player().id()
                .find(player.player_id)
                .map(|p| PlayerAndLevel { id: p.id, identity: p.identity, name: p.name, level: player.level, })
        })
        .collect()
}
```

---

### Schedulers
```rust
#[table(accessor = reminder_schedule, scheduled(send_reminder))]
pub struct Reminder {
    #[primary_key]
    #[auto_inc]
    id: u64,
    user_id: u32,
    message: String,
    scheduled_at: ScheduleAt,
}

#[reducer]
fn send_reminder(ctx: &ReducerContext, reminder: Reminder) -> Result<(), String> {
    // Process the scheduled reminder
    Ok(())
}

#[reducer(init)]
fn init(ctx: &ReducerContext) {
    ctx.db.reminder_schedule().insert(Reminder {
        id: 0,
        user_id: 0,
        message: "Game tick".to_string(),
        scheduled_at: ScheduleAt::Interval(Duration::from_millis(50).into()),
    });
}
```

### Clients
```rust
/// Read each line of standard input, and either set our name or send a message as appropriate.
fn user_input_loop(ctx: &DbConnection) {
    for line in std::io::stdin().lines() {
        let Ok(line) = line else {
            panic!("Failed to read from stdin.");
        };
        if let Some(name) = line.strip_prefix("/name ") {
            ctx.reducers
                .set_name_then(name.to_string(), {
                    let name = name.to_string();
                    move |_ctx, result| match result {
                        Err(e) => panic!("Internal error when setting name: {e}"),
                        Ok(Err(e)) => eprintln!("Failed to set name to {name}: {e}"),
                        Ok(Ok(())) => (),
                    }
                })
                .unwrap();
        } else {
            ctx.reducers
                .send_message_then(line.clone(), {
                    move |_ctx, result| match result {
                        Err(e) => panic!("Internal error when sending message: {e}"),
                        Ok(Err(e)) => eprintln!("Failed to send message {line:?}: {e}"),
                        Ok(Ok(())) => (),
                    }
                })
                .unwrap();
        }
    }
}
```

---

## Advanced

### Authentication
- Clients building a connection should pass an OIDC token
  - You can use any identity provider you want (Keycloak, Auth0, ...)
  - If empty, SpacetimeDB will return an identity for you

### Authorization
As mentioned before: Everything is programmable:
```rust
#[spacetimedb::reducer]
fn close_game(ctx: &ReducerContext, game_id: u64) {
    if let Some(game) = ctx.db.game().game_id().find(game_id) {
        if game.owner == ctx.sender() {
            // Update the game to be closed
            // ...

            // Schedule the game cleanup
            // ...
        }
    }
}
```

---

## Demo: Snake Online
Now it is demo time! 
- [GitHub](https://github.com/)
- [Demo](https://snake.luke-homelab.de)
