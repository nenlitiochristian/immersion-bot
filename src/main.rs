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
use model::Data;
use poise::serenity_prelude as serenity;
use repository::{
    postgres_db::{PostgresCharacterStatisticsRepository, PostgresMetadataRepository},
    CharacterStatisticsRepository, MetadataRepository,
};
use reqwest::Client;
use roles::QuizRoles;
use sqlx::{postgres::PgPoolOptions, Executor, PgPool};
use std::{collections::HashMap, env::var, sync::Arc, time::Duration};
use tokio::sync::Mutex;

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
            commands::rank(),
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
        post_command: |ctx| {
            Box::pin(async move {
                let guild: PartialGuild = ctx.partial_guild().await.unwrap();
                let user_data = ctx.data();
                let result = refresh_active_users(ctx.serenity_context(), user_data, &guild).await;
                match result {
                    Ok(_) => (),
                    Err(error) => println!("Error occured when refreshing active users: {}", error),
                }
            })
        },
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
                refresh_active_users(ctx, framework.user_data, &partial_guild).await?;
            }
        }
        serenity::FullEvent::GuildMemberAddition { new_member } => {
            let conn = framework.user_data.connection.lock().await;
            let mut tx = conn.begin().await?;
            {
                let mut repository = PostgresCharacterStatisticsRepository::new();
                if repository.exists(&mut tx, new_member.user.id.get()).await? {
                    repository
                        .set_active_status(
                            &mut tx,
                            new_member.user.id.get(),
                            true,
                            Some(new_member.user.display_name()),
                        )
                        .await?;
                    println!("{} returned", new_member.display_name());
                }
            }
            tx.commit().await?;
        }
        serenity::FullEvent::GuildMemberRemoval {
            guild_id: _,
            user,
            member_data_if_available: _,
        } => {
            let conn = framework.user_data.connection.lock().await;
            let mut tx = conn.begin().await?;
            {
                let mut repository = PostgresCharacterStatisticsRepository::new();
                repository
                    .set_active_status(&mut tx, user.id.get(), false, Some(user.display_name()))
                    .await?;
                println!("{} left", user.display_name());
            }
            tx.commit().await?;
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
    guild: &PartialGuild,
) -> Result<(), Error> {
    println!("Reloading active users...");
    let should_refresh = {
        let conn = user_data.connection.lock().await;
        let mut tx = conn.begin().await?;
        let last_refresh = {
            let mut repository = PostgresMetadataRepository::new();
            repository.get_last_active_status_refresh(&mut tx).await?
        };
        tx.commit().await?;

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

    let conn = user_data.connection.lock().await;
    let mut tx = conn.begin().await?;
    let mut repository = PostgresCharacterStatisticsRepository::new();

    let mut page_number = 0;
    loop {
        let users = repository
            .get_paginated_users_by_id(&mut tx, page_number)
            .await?;
        if users.is_empty() {
            break;
        }
        for u in users.iter() {
            let member = members.get(&UserId::from(u.get_user_id()));
            let name = match member {
                None => None,
                Some(member) => Some(member.user.display_name()),
            };
            repository
                .set_active_status(&mut tx, u.get_user_id(), member.is_some(), name)
                .await?;
        }
        page_number += 1;
    }

    let mut tx = conn.begin().await?;
    let mut metadata_repository = PostgresMetadataRepository::new();
    metadata_repository
        .set_last_active_status_refresh(&mut tx, Utc::now())
        .await?;
    println!("Done reloading active users");
    Ok(())
}

async fn setup_postgres_connection() -> Result<PgPool, sqlx::Error> {
    let token: String = var("DATABASE_URL").expect("Missing `POSTGRES_TOKEN` env var.");
    let client = PgPoolOptions::new()
        .max_connections(1)
        .connect(&token)
        .await?;

    client
        .execute(
            r#"
    -- Create the CharacterStatistics table
CREATE TABLE IF NOT EXISTS immersion_bot."CharacterStatistics" (
    user_id BIGINT PRIMARY KEY, -- the discord id of the user
    total_characters INTEGER NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE, -- 1 = TRUE, 0 = FALSE
    name TEXT NOT NULL DEFAULT 'UNKNOWN'    
);

-- Create the CharacterLogEntry table
CREATE TABLE IF NOT EXISTS immersion_bot."CharacterLogEntry" (
    id SERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL, -- Foreign key linking to CharacterStatistics
    characters INTEGER NOT NULL,
    time BIGINT NOT NULL, -- Store timestamp as Unix timestamp (64bits in SQLite)
    notes TEXT, -- Optional field for notes
    FOREIGN KEY (user_id) REFERENCES immersion_bot."CharacterStatistics" (user_id)
);

-- Create the Metadata table
CREATE TABLE IF NOT EXISTS immersion_bot."Metadata" (
    last_active_status_refresh BIGINT NOT NULL
);"#,
        )
        .await?;

    Ok(client)
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    // let mut connection = setup_sqlite_connection().expect("Failed to open an SQLite connection!");
    let connection = setup_postgres_connection().await.unwrap();
    let http_client = Client::new();

    let data = Data {
        connection: Mutex::new(connection),
        http_client,
    };
    setup_discord_bot(data).await
}

// fn handle_migrate(
//     connection: &mut Box<dyn CharacterStatisticsRepository + '_>,
//     path: &str,
// ) -> Result<(), Error> {
//     let old_data = get_json_data(path)?;
//     migrate(connection, old_data)
// }
