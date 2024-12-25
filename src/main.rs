#![warn(clippy::str_to_string)]

mod commands;
mod constants;
mod kotoba;
mod model;
mod repository;
mod roles;
mod utils;

use dotenv::dotenv;
use model::Data;
use poise::serenity_prelude as serenity;
use reqwest::Client;
use roles::QuizRoles;
use rusqlite::Connection;
use std::{
    env::var,
    sync::{Arc, Mutex},
    time::Duration,
};

// Types used by all command functions
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    // This is our custom error handler
    // They are many errors that can occur, so we only handle the ones we want to customize
    // and forward the rest to the default handler
    match error {
        poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {:?}", error),
        poise::FrameworkError::Command { error, ctx, .. } => {
            println!("Error in command `{}`: {:?}", ctx.command().name, error,);
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                println!("Error while handling error: {}", e)
            }
        }
    }
}

async fn setup_discord_bot(data: Data) {
    // FrameworkOptions contains all of poise's configuration option in one struct
    // Every option can be omitted to use its default value
    let options = poise::FrameworkOptions {
        commands: vec![
            commands::help(),
            commands::log_characters(),
            commands::history(),
            commands::usage(),
            commands::how_to_track(),
            commands::roles(),
            commands::leaderboard(),
            commands::quizzes(),
        ],
        prefix_options: poise::PrefixFrameworkOptions {
            // commands only, no prefix messages
            prefix: None,
            edit_tracker: Some(Arc::new(poise::EditTracker::for_timespan(
                Duration::from_secs(3600),
            ))),
            ..Default::default()
        },
        // The global error handler for all error cases that may occur
        on_error: |error| Box::pin(on_error(error)),
        // This code is run before every command
        pre_command: |ctx| {
            Box::pin(async move {
                println!("Executing command {}...", ctx.command().qualified_name);
            })
        },
        // This code is run after a command if it was successful (returned Ok)
        post_command: |ctx| {
            Box::pin(async move {
                println!("Executed command {}!", ctx.command().qualified_name);
            })
        },
        // Every command invocation must pass this check to continue execution
        command_check: Some(|ctx| {
            Box::pin(async move {
                if ctx.author().id == 123456789 {
                    return Ok(false);
                }
                Ok(true)
            })
        }),

        // Enforce command checks even for owners (enforced by default)
        // Set to true to bypass checks, which is useful for testing
        skip_checks_for_owners: false,
        event_handler: |ctx, event, framework, data| {
            Box::pin(event_handler(ctx, event, framework, data))
        },
        ..Default::default()
    };

    let framework = poise::Framework::builder()
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                println!("Logged in as {}", _ready.user.name);
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(data)
            })
        })
        .options(options)
        .build();

    let token: String = var("DISCORD_TOKEN")
        .expect("Missing `DISCORD_TOKEN` env var, see README for more information.");
    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;

    client.unwrap().start().await.unwrap()
}

async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::Message { new_message } => {
            QuizRoles::handle_quiz_roles(ctx, new_message, data).await?;
        }
        _ => {}
    }
    Ok(())
}

fn setup_sqlite_connection() -> rusqlite::Result<Connection> {
    let connection = Connection::open("perdition.db")?;

    // Setup migration
    connection.execute(
        "
-- Create the CharacterStatistics table
CREATE TABLE IF NOT EXISTS CharacterStatistics (
    user_id INTEGER PRIMARY KEY, -- the discord id of the user
    total_characters INTEGER NOT NULL
);    
    ",
        (),
    )?;

    connection.execute(
        "
-- Create the CharacterLogEntry table
CREATE TABLE IF NOT EXISTS CharacterLogEntry (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL, -- Foreign key linking to CharacterStatistics
    characters INTEGER NOT NULL,
    time INTEGER NOT NULL, -- Store timestamp as Unix timestamp (64bits in SQLite)
    notes TEXT, -- Optional field for notes
    FOREIGN KEY (user_id) REFERENCES CharacterStatistics (user_id)
);
    ",
        (),
    )?;

    Ok(connection)
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let connection = setup_sqlite_connection().expect("Failed to open an SQLite connection!");
    let http_client = Client::new();

    let data = Data {
        connection: Mutex::new(connection),
        http_client,
    };
    setup_discord_bot(data).await
}
