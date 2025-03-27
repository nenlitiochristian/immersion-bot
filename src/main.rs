#![warn(clippy::str_to_string)]

mod commands;
mod constants;
mod kotoba;
mod migrate;
mod model;
mod repository;
mod roles;
mod utils;

use ::serenity::all::{Member, PartialGuild, UserId};
use chrono::Utc;
use constants::USER_ACTIVE_STATUS_REFRESH_INTERVAL;
use dotenv::dotenv;
use migrate::{get_json_data, migrate};
use model::Data;
use poise::serenity_prelude as serenity;
use repository::{
    CharacterStatisticsRepository, MetadataRepository, SQLiteCharacterStatisticsRepository,
    SQLiteMetadataRepository,
};
use reqwest::Client;
use roles::QuizRoles;
use rusqlite::Connection;
use std::{
    collections::HashMap,
    env::{self, var},
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
            commands::edit_characters(),
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
        post_command: |ctx| Box::pin(async move {}),
        // Every command invocation must pass this check to continue execution
        // command_check: Some(|ctx| Box::pin(async move { Ok(true) })),

        // Enforce command checks even for owners (enforced by default)
        // Set to true to bypass checks, which is useful for testing
        skip_checks_for_owners: false,
        event_handler: |ctx, event, framework, _| Box::pin(event_handler(ctx, event, framework)),
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
    let intents = serenity::GatewayIntents::non_privileged()
        | serenity::GatewayIntents::MESSAGE_CONTENT
        | serenity::GatewayIntents::GUILD_MEMBERS;

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;

    client.unwrap().start().await.unwrap()
}

async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    framework: poise::FrameworkContext<'_, Data, Error>,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::Ready { data_about_bot } => {
            println!("Running ready event");
            for guild in &data_about_bot.guilds {
                let partial_guild = guild.id.to_partial_guild(ctx).await?;
                refresh_active_users(ctx, framework.user_data, partial_guild).await?;
            }
        }
        serenity::FullEvent::GuildMemberAddition { new_member } => {
            let mut conn = framework.user_data.connection.lock().unwrap();
            let tx = conn.transaction()?;
            let mut repository = SQLiteCharacterStatisticsRepository::new(&tx);
            if repository.exists(new_member.user.id.get())? {
                repository.set_active_status(new_member.user.id.get(), true)?;
                println!("{} returned", new_member.display_name());
            }
            tx.commit()?;
        }
        serenity::FullEvent::GuildMemberRemoval {
            guild_id: _,
            user,
            member_data_if_available: _,
        } => {
            let mut conn = framework.user_data.connection.lock().unwrap();
            let tx = conn.transaction()?;
            let mut repository = SQLiteCharacterStatisticsRepository::new(&tx);
            repository.set_active_status(user.id.get(), false)?;
            println!("{} left", user.display_name());
            tx.commit()?;
        }
        serenity::FullEvent::Message { new_message } => {
            let result = QuizRoles::handle_quiz_roles(ctx, new_message, framework.user_data).await;
            if result.is_err() {
                println!("Handle quiz role error: {}", result.unwrap_err());
            }
        }
        _ => {}
    }
    Ok(())
}

async fn refresh_active_users(
    ctx: &serenity::Context,
    user_data: &Data,
    guild: PartialGuild,
) -> Result<(), Error> {
    println!("Reloading active users...");
    let should_refresh = {
        let mut conn = user_data.connection.lock().unwrap();
        let tx = conn.transaction()?;
        let repository = SQLiteMetadataRepository::new(&tx);
        let last_refresh = repository.get_last_active_status_refresh()?;
        tx.commit()?;
        match last_refresh {
            None => true,
            Some(last_refresh) => {
                let elapsed = Utc::now().timestamp() - last_refresh.timestamp();
                elapsed > USER_ACTIVE_STATUS_REFRESH_INTERVAL
            }
        }
    };

    if !should_refresh {
        println!("No need to reload, 2 hours haven't passed");
        return Ok(());
    }

    let mut after: Option<UserId> = None;
    let mut members: HashMap<UserId, Member> = HashMap::with_capacity(2500);
    loop {
        let temp_members = guild.members(ctx, None, after).await?;
        if temp_members.is_empty() {
            break;
        }
        after = Some(temp_members.last().unwrap().user.id);
        for m in temp_members.into_iter() {
            members.insert(m.user.id, m);
        }
    }

    let mut conn = user_data.connection.lock().unwrap();
    let tx = conn.transaction()?;
    let mut repository = SQLiteCharacterStatisticsRepository::new(&tx);

    let mut page_number = 0;
    loop {
        let users = repository.get_paginated_users_by_id(page_number)?;
        if users.is_empty() {
            break;
        }
        for u in users.iter() {
            let is_active = members.contains_key(&UserId::from(u.get_user_id()));
            repository.set_active_status(u.get_user_id(), is_active)?;
        }
        page_number += 1;
    }

    let mut metadata_repository = SQLiteMetadataRepository::new(&tx);
    metadata_repository.set_last_active_status_refresh()?;

    tx.commit()?;
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
    total_characters INTEGER NOT NULL,
    is_active INTEGER NOT NULL DEFAULT 1 -- 1 = TRUE, 0 = FALSE
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

    // Setup migration
    connection.execute(
        "
CREATE TABLE IF NOT EXISTS Metadata (
    last_active_status_refresh INTEGER NOT NULL
);    
        ",
        (),
    )?;

    Ok(connection)
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let mut connection = setup_sqlite_connection().expect("Failed to open an SQLite connection!");
    let http_client = Client::new();

    // migrate old json data (if needed)
    let args: Vec<String> = env::args().collect();
    if args.len() > 2 && args[1] == "--migrate" {
        let path = &args[2];
        println!("Migrating file: {}", path);
        let result = handle_migrate(&mut connection, path);
        match result {
            Err(error) => {
                println!("Failed to migrate json data: {error}");
            }
            _ => (),
        };
    }

    let data = Data {
        connection: Mutex::new(connection),
        http_client,
    };
    setup_discord_bot(data).await
}

fn handle_migrate(connection: &mut Connection, path: &str) -> Result<(), Error> {
    let old_data = get_json_data(path)?;
    migrate(connection, old_data)
}
