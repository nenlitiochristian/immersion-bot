use std::{future::Future, time::Instant};

use poise::CreateReply;
use serenity::all::{Color, CreateEmbed, CreateEmbedFooter, UserId};

use crate::{
    constants::{LEADERBOARD_PAGE_SIZE, LOG_ENTRY_PAGE_SIZE},
    repository::{CharacterStatisticsRepository, SQLiteCharacterStatisticsRepository},
    roles::{Roles, UserRoles},
    utils::format_with_commas,
    Context, Error,
};

/// Shows this help menu.
#[poise::command(track_edits, slash_command)]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help about"]
    #[autocomplete = "poise::builtins::autocomplete_command"]
    command: Option<String>,
) -> Result<(), Error> {
    poise::builtins::help(
        ctx,
        command.as_deref(),
        poise::builtins::HelpConfiguration {
            extra_text_at_bottom: "This is a list of commands available in the bot. For an explanation on how to use the bot, try /usage.",
            ..Default::default()
        },
    )
    .await?;
    Ok(())
}

/// Logs immersion characters.
///
/// Optionally, add a note to keep track of read materials, i.e: `/log_characters characters:4000 notes:Episode 1 of Love Live season 1`
#[poise::command(slash_command)]
pub async fn log_characters(
    ctx: Context<'_>,
    #[description = "The amount of characters read"] characters: i32,
    #[description = "Extra information such as the title of the book or VN"] notes: Option<String>,
) -> Result<(), Error> {
    let (data, rank) = {
        let mut connection = ctx.data().connection.lock().unwrap();
        let tx = connection.transaction().map_err(|e| e.to_string())?;
        let mut repository = SQLiteCharacterStatisticsRepository::new(&tx);

        let user_id = ctx.author().id;
        let name = ctx.author().display_name();

        let time = &ctx.created_at();
        let data = repository.add_log_entry(user_id.get(), name, characters, time, notes)?;
        let rank = repository.get_rank(&data)?;
        tx.commit()?;

        (data, rank)
    };

    let user = ctx.author_member().await.unwrap().into_owned();
    let guild = ctx.guild().unwrap().to_owned();
    let user_roles = &user.roles;
    let guild_roles = &guild.roles.clone();
    let roles = UserRoles::new(user_roles, guild_roles);
    let new_role = roles.update_role(ctx, &guild, &user, &data).await?;
    if let Some(new_role) = new_role {
        // role changed, if it's higher give a congratulations message
        if roles.roles.iter().all(|r| &new_role > r) {
            ctx.say(format!(
                "Congratulations {} for obtaining role: {}",
                user.user.display_name(),
                new_role.to_string()
            ))
            .await?;
        }
    }

    let current_role = Roles::from_characters_and_quiz_roles(&roles.quizzes, data.total_characters);
    let current_role_message = match current_role {
        Some(role) => format!("Current role is {}", role.to_string()),
        None => "You currently don't have a role".to_owned(),
    };

    let next_role_message = if let Some(requirement) =
        Roles::next_role_requirement(&roles.quizzes, data.total_characters)
    {
        let condition = data.total_characters < requirement.characters;
        let message = if condition {
            format!(
                "{} more characters",
                format_with_commas(requirement.characters - data.total_characters)
            )
        } else {
            format!("to pass {}", requirement.quiz_role.unwrap().to_string())
        };
        format!("For {} you need {}.", requirement.role.to_string(), message)
    } else {
        "You already have the highest role.".to_owned()
    };

    let embed = create_base_embed()
        .title(format!(
            "{} logged {} characters!",
            user.user.display_name(),
            format_with_commas(characters),
        ))
        .description(format!(
            "Total characters logged: {}",
            format_with_commas(data.total_characters)
        ))
        .field(
            format!("You are currently rank {} on the leaderboard", rank),
            format!("{}. {}", current_role_message, next_role_message),
            false,
        );

    ctx.send(CreateReply::default().embed(embed)).await?;
    Ok(())
}

/// Admin-only command to change any member's logs
#[poise::command(slash_command, default_member_permissions = "ADMINISTRATOR")]
pub async fn edit_characters(
    ctx: Context<'_>,
    #[description = "The targeted member"] user_id: UserId,
    #[description = "The amount of characters read"] characters: i32,
    #[description = "Extra information such as the title of the book or VN"] notes: Option<String>,
) -> Result<(), Error> {
    let name = user_id.to_user(ctx).await?.display_name().to_owned();
    let (data, rank) = {
        let mut connection = ctx.data().connection.lock().unwrap();
        let tx = connection.transaction().map_err(|e| e.to_string())?;
        let mut repository = SQLiteCharacterStatisticsRepository::new(&tx);

        let time = &ctx.created_at();
        let data = repository.add_log_entry(user_id.get(), &name, characters, time, notes)?;
        let rank = repository.get_rank(&data)?;
        tx.commit()?;

        (data, rank)
    };

    let guild = ctx.guild().unwrap().to_owned();
    let member = guild.member(ctx, user_id).await?.into_owned();
    let user_roles = &member.roles;
    let guild_roles = &guild.roles.clone();
    let roles = UserRoles::new(user_roles, guild_roles);
    let new_role = roles.update_role(ctx, &guild, &member, &data).await?;
    if let Some(new_role) = new_role {
        // role changed, if it's higher give a congratulations message
        if roles.roles.iter().all(|r| &new_role > r) {
            ctx.say(format!(
                "Congratulations {} for obtaining role: {}",
                member.user.display_name(),
                new_role.to_string()
            ))
            .await?;
        }
    }

    let current_role = Roles::from_characters_and_quiz_roles(&roles.quizzes, data.total_characters);
    let current_role_message = match current_role {
        Some(role) => format!("Current role is {}", role.to_string()),
        None => "You currently don't have a role".to_owned(),
    };

    let next_role_message = if let Some(requirement) =
        Roles::next_role_requirement(&roles.quizzes, data.total_characters)
    {
        let condition = data.total_characters < requirement.characters;
        let message = if condition {
            format!(
                "{} more characters",
                format_with_commas(requirement.characters - data.total_characters)
            )
        } else {
            format!("to pass {}", requirement.quiz_role.unwrap().to_string())
        };
        format!("For {} you need {}.", requirement.role.to_string(), message)
    } else {
        "You already have the highest role.".to_owned()
    };

    let embed = create_base_embed()
        .title(format!(
            "{} logged {} characters!",
            member.user.display_name(),
            format_with_commas(characters),
        ))
        .description(format!(
            "Total characters logged: {}",
            format_with_commas(data.total_characters)
        ))
        .field(
            format!("You are currently rank {} on the leaderboard", rank),
            format!("{}. {}", current_role_message, next_role_message),
            false,
        );

    ctx.send(CreateReply::default().embed(embed)).await?;
    Ok(())
}

fn create_base_embed() -> CreateEmbed {
    CreateEmbed::default()
        .footer(CreateEmbedFooter::new(
            "See /help for a list of commands and /usage for an explanation on what I can do.",
        ))
        .color(Color::from_rgb(225, 178, 28))
}

/// Explains how the bot works.
#[poise::command(slash_command)]
pub async fn usage(ctx: Context<'_>) -> Result<(), Error> {
    let embed = create_base_embed()
        .title("How to use this bot")
        .description(
"This bot is for tracking characters read, **not** for listening immersion. That doesn't mean that listening is not important, but it's implied that you're spending an equal amount of time practicing listening as you are reading.
Reading that you can track includes:

1. Novels
2. VNs
3. Anime with JP subtitles (NOT raw)
4. Podcasts with a script
5. Anything else in a similar vein (if you're unsure, ask an admin)

Do **NOT** estimate your immersion, only log immersion that you are sure of the exact number of characters of. As you log your immersion, you will automatically receive roles based on how much you've done. While we would love to take your word for it, we can't be sure that everyone is honest. To counteract people who might lie about their immersion amount, certain roles will require a kotoba quiz in order to continue. The quiz should be straightforward if you've done the amount of immersion required for that role (with the exception being 天仙 and 上手, whose quizes are intentionally bullshit).");

    ctx.send(CreateReply::default().embed(embed)).await?;
    Ok(())
}

/// Explains how to track your reading characters.
#[poise::command(slash_command)]
pub async fn how_to_track(ctx: Context<'_>) -> Result<(), Error> {
    let embed = create_base_embed().title("How to track characters read")
        .description("Track **only** characters read (whether that be novels, VNs, games, subtitles, scripts, etc.), **not** raw listening. **Do not** guess how much immersion you've done, only log exact numbers that you're sure of. Whenever possible, your character count should exclude special characters (like punctuation, etc). These rules don't exist to be unnecessarily rigid, they exist to keep everyone on a (measureably) even playing field. Don't spoil things for others.

**Anime**  
It's recommended to create a Japanese-only account on [myanimelist](https://myanimelist.net/) or a similar tracking site, which will show exactly how many episodes you've watched with JP subs. You can download Japanese subtitles from [jimaku](https://jimaku.cc/) and watch/mine with [animebook](https://cademcniven.com/posts/20210703/). You can find the exact number of characters in the show with [subtitle character counter](https://cademcniven.com/subtitleCharacterCounter.html).

**Novels**  
You can track characters read for novels by reading on [ttu reader](https://ttu-ebook.web.app/). You can convert your ebooks to epub using [calibre](https://calibre-ebook.com/). You can download webnovels to epub with [WebToEpub](https://github.com/dteviot/WebToEpub). 

**Web Novels**  
You can use [this userscript](https://greasyfork.org/en/scripts/512137-japanese-reading-tracker) to track characters read in popular Japanese novel websites like [Syosetu](https://syosetu.com) and [Kakuyomu](https://kakuyomu.jp).

**Visual Novels**  
You can track characters read from visual novels using a [texthooker](https://renji-xd.github.io/texthooker-ui/). Follow the [TMW Guide](https://learnjapanese.moe/vn/) to learn how to set it up.

**Manga**  
You can track characters read from manga by using [mokuro reader](https://reader.mokuro.app/), or which is a reader for [mokuro](https://github.com/kha-white/mokuro) files.");

    ctx.send(CreateReply::default().embed(embed)).await?;
    Ok(())
}

async fn make_history_embed_by_page(
    ctx: Context<'_>,
    page: u64,
    user_id: u64,
) -> Result<CreateEmbed, Error> {
    let (log_entries, total_count) = {
        let mut connection = ctx.data().connection.lock().unwrap();
        let tx = connection.transaction().map_err(|e| e.to_string())?;
        let mut repository = SQLiteCharacterStatisticsRepository::new(&tx);

        let entries = repository.get_paginated_log_entries_by_time(user_id, page)?;
        let total_entry_count = repository.get_total_log_entries(user_id)?;
        tx.commit()?;

        (entries, total_entry_count)
    };

    let embed_builder = create_base_embed().title(format!(
        "Log history (Page {} of {})",
        page + 1,
        (total_count / LOG_ENTRY_PAGE_SIZE) + 1
    ));

    let mut lines = "".to_string();
    for history in log_entries {
        let notes = match history.notes() {
            None => "-",
            Some(x) => x,
        };
        let time = history.time().format("%Y年%m月%d日").to_string();
        lines += &format!(
            "{}: {} characters | {}\n",
            time,
            format_with_commas(history.characters()),
            notes
        );
    }

    Ok(embed_builder.description(lines))
}

/// Shows yours or other people's latest log history.
#[poise::command(slash_command)]
pub async fn history(
    ctx: Context<'_>,
    #[description = "The user you want to check"] user: Option<UserId>,
) -> Result<(), Error> {
    let user_id = user.unwrap_or(ctx.author().id).get();
    let exists = {
        let mut connection = ctx.data().connection.lock().unwrap();
        let tx = connection.transaction()?;
        let repository = SQLiteCharacterStatisticsRepository::new(&tx);
        repository.exists(user_id)?
    };

    if !exists {
        let embed = create_base_embed().description("The user hasn't made any logs.");
        ctx.send(CreateReply::default().embed(embed).ephemeral(true))
            .await?;
        return Ok(());
    }

    let length = {
        let mut connection = ctx.data().connection.lock().unwrap();
        let tx = connection.transaction().map_err(|e| e.to_string())?;
        let mut repository = SQLiteCharacterStatisticsRepository::new(&tx);

        let entries = repository.get_total_log_entries(user_id)?;
        tx.commit()?;

        entries.div_ceil(LOG_ENTRY_PAGE_SIZE)
    };

    paginate(ctx, None, user_id, make_history_embed_by_page, length).await?;

    Ok(())
}

async fn make_leaderboard_embed_by_page(
    ctx: Context<'_>,
    page: u64,
    custom_context_data: (u64, String),
) -> Result<CreateEmbed, Error> {
    let user_id = custom_context_data.0;
    let user_name = custom_context_data.1.as_str();
    let start = Instant::now();
    let (users, rank, users_count, stats) = {
        let mut connection = ctx.data().connection.lock().unwrap();
        let tx = connection.transaction().map_err(|e| e.to_string())?;
        let mut repository = SQLiteCharacterStatisticsRepository::new(&tx);

        let users = repository.get_paginated_active_users_by_characters(page)?;
        let stats = repository.get_or_initialize_statistics(user_id, user_name)?;
        let rank = repository.get_rank(&stats)?;

        let users_count = repository.get_total_active_users()?;
        tx.commit()?;
        (users, rank, users_count, stats)
    };

    // there are 15 data per page
    let total_pages = users_count.div_ceil(LEADERBOARD_PAGE_SIZE);
    let embed_builder = create_base_embed()
        .title(format!(
            "Leaderboard (Page {} of {})",
            page + 1,
            total_pages
        ))
        .description(format!(
            "{} is currently rank {} of {}, with {} total characters.",
            user_name,
            rank,
            users_count,
            format_with_commas(stats.total_characters)
        ));

    let mut line = "".to_owned();
    for (index, u) in users.iter().enumerate() {
        let index: u64 = index.try_into().unwrap();
        let is_bold = user_id == u.get_user_id();
        let formatted = if is_bold {
            format!(
                "{}. **{}: {} characters**\n",
                index + (page * LEADERBOARD_PAGE_SIZE) + 1,
                u.name,
                format_with_commas(u.total_characters)
            )
        } else {
            format!(
                "{}. {}: {} characters\n",
                index + (page * LEADERBOARD_PAGE_SIZE) + 1,
                u.name,
                format_with_commas(u.total_characters)
            )
        };

        line += &formatted;
    }

    if line.is_empty() {
        line = format!("No users found for page {}", page + 1)
    }

    let duration = start.elapsed();
    println!("Execution time: {:?}", duration);

    Ok(embed_builder.field("Top Immersers", line, false))
}

/// Shows you where you are on the leaderboard. Can also be used to check other people's rank.
#[poise::command(slash_command)]
pub async fn rank(
    ctx: Context<'_>,
    #[description = "The user you want to check"] user: Option<UserId>,
) -> Result<(), Error> {
    let (user_id, display_name) = match user {
        None => (
            ctx.author().id.get(),
            ctx.author().display_name().to_owned(),
        ),
        Some(id) => {
            let user = id.to_user(ctx).await?;
            (id.get(), user.display_name().to_owned())
        }
    };

    let exists = {
        let mut connection = ctx.data().connection.lock().unwrap();
        let tx = connection.transaction()?;
        let repository = SQLiteCharacterStatisticsRepository::new(&tx);
        repository.exists(user_id)?
    };

    if !exists {
        let embed = create_base_embed().description("The user hasn't made any logs.");
        ctx.send(CreateReply::default().embed(embed).ephemeral(true))
            .await?;
        return Ok(());
    }

    let (total_pages, my_page) = {
        let mut connection = ctx.data().connection.lock().unwrap();
        let tx = connection.transaction().map_err(|e| e.to_string())?;
        let mut repository = SQLiteCharacterStatisticsRepository::new(&tx);

        let users_count = repository.get_total_active_users()?;
        let stats = repository.get_or_initialize_statistics(user_id, &display_name)?;
        let rank: u64 = repository.get_rank(&stats)?.try_into()?;
        tx.commit()?;

        let pages = users_count.div_ceil(LEADERBOARD_PAGE_SIZE);
        let my_page = rank.div_ceil(LEADERBOARD_PAGE_SIZE) - 1;
        (pages, my_page)
    };

    paginate(
        ctx,
        Some(my_page),
        (user_id, display_name),
        make_leaderboard_embed_by_page,
        total_pages,
    )
    .await?;

    Ok(())
}

/// Shows the leaderboard.
#[poise::command(slash_command)]
pub async fn leaderboard(ctx: Context<'_>) -> Result<(), Error> {
    let total_pages = {
        let mut connection = ctx.data().connection.lock().unwrap();
        let tx = connection.transaction().map_err(|e| e.to_string())?;
        let mut repository = SQLiteCharacterStatisticsRepository::new(&tx);

        let users_count = repository.get_total_active_users()?;
        tx.commit()?;

        users_count.div_ceil(LEADERBOARD_PAGE_SIZE)
    };

    paginate(
        ctx,
        None,
        (
            ctx.author().id.get(),
            ctx.author().display_name().to_owned(),
        ),
        make_leaderboard_embed_by_page,
        total_pages,
    )
    .await?;

    Ok(())
}

/// Shows the list of roles available and how to get them.
#[poise::command(slash_command)]
pub async fn roles(ctx: Context<'_>) -> Result<(), Error> {
    let embed = create_base_embed().title("Roles").description(
        "平民 - 100,000 characters
男爵 - 500,000 characters (must pass quiz 1)
子爵 - 1,000,000 characters
伯爵 - 2,000,000 characters
侯爵 - 3,500,000 characters
公爵 - 5,000,000 characters (must pass quiz 2)
大公 - 7,500,000 characters
王様 - 10,000,000 characters
天皇 - 15,000,000 characters
地仙 - 25,000,000 characters (must pass quiz 3)
天仙 - 50,000,000 characters (must pass quiz 4)
上手 - 100,000,000 characters (must pass quiz 5)",
    );

    ctx.send(CreateReply::default().embed(embed)).await?;
    Ok(())
}

/// Shows the list of quizzes you need to unlock certain roles.
#[poise::command(slash_command)]
pub async fn quizzes(ctx: Context<'_>) -> Result<(), Error> {
    let embed = create_base_embed()
        .title("Quizzes")
        .description("Certain roles require you to pass a quiz (see /roles for more info). You're allowed to take the quiz as many times as you want. Take the quiz in #kotoba or #kotoba2. Quizzes must be taken in order (you can't skip quiz 1 and 2 by doing 3 first). 
        
        **Commands**
        Quiz 1 (男爵): `k!quiz pq_1 15 nd mmq=4 font=5 atl=20`
Quiz 2 (公爵): `k!quiz pq_2 20 nd mmq=4 font=5 atl=20`
Quiz 3 (地仙): `k!quiz pq_3 20 nd mmq=4 font=5 atl=20`
Quiz 4 (天仙): `k!quiz pq_4+animals+bugs+fish+plants+birds+vegetables+yojijukugo+countries 30 nd mmq=4 font=5 atl=20`
Quiz 5 (上手): `k!quiz stations_full 100 nd mmq=4 font=5 atl=20`");

    ctx.send(CreateReply::default().embed(embed)).await?;
    Ok(())
}

pub async fn paginate<'a, F, Fut, CustomContextData>(
    ctx: Context<'a>,
    page_start: Option<u64>,
    custom_context_data: CustomContextData,
    page_fetch_function: F,
    length: u64,
) -> Result<(), Error>
where
    F: Fn(Context<'a>, u64, CustomContextData) -> Fut,
    Fut: Future<Output = Result<CreateEmbed, Error>>,
    CustomContextData: Clone,
{
    let page_start = page_start.unwrap_or(0u64);
    // Define some unique identifiers for the navigation buttons
    let ctx_id = ctx.id();
    let prev_button_id = format!("{}prev", ctx_id);
    let next_button_id = format!("{}next", ctx_id);

    // Send the embed with the first page as content
    let reply = {
        let components = serenity::builder::CreateActionRow::Buttons(vec![
            serenity::builder::CreateButton::new(&prev_button_id).label("◀️"),
            serenity::builder::CreateButton::new(&next_button_id).label("▶️"),
        ]);

        CreateReply::default()
            .embed(page_fetch_function(ctx, page_start, custom_context_data.clone()).await?)
            .components(vec![components])
    };

    ctx.send(reply).await?;

    // Loop through incoming interactions with the navigation buttons
    let mut current_page = page_start;
    while let Some(press) = serenity::collector::ComponentInteractionCollector::new(ctx)
        // We defined our button IDs to start with `ctx_id`. If they don't, some other command's
        // button was pressed
        .filter(move |press| press.data.custom_id.starts_with(&ctx_id.to_string()))
        // Timeout when no navigation button has been pressed for 24 hours
        .timeout(std::time::Duration::from_secs(3600 * 24))
        .await
    {
        // Depending on which button was pressed, go to next or previous page
        if press.data.custom_id == next_button_id {
            current_page += 1;
            if current_page >= length {
                current_page = 0;
            }
        } else if press.data.custom_id == prev_button_id {
            current_page = current_page.checked_sub(1).unwrap_or(length - 1);
        } else {
            // This is an unrelated button interaction
            continue;
        }

        // Update the message with the new page contents
        press
            .create_response(
                ctx.serenity_context(),
                serenity::builder::CreateInteractionResponse::UpdateMessage(
                    serenity::builder::CreateInteractionResponseMessage::new().embed(
                        page_fetch_function(ctx, current_page, custom_context_data.clone()).await?,
                    ),
                ),
            )
            .await?;
    }

    Ok(())
}
