use poise::CreateReply;
use serenity::all::{CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter};

use crate::{repository::CharacterStatisticsRepository, Context, Error};

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
    let mut repository = ctx.data().character_statistics_repository.lock().await;
    let user_id = ctx.author().id;

    let time = &ctx.created_at();
    let result = repository
        .add_log_entry(user_id, characters, time, notes)
        .await;

    let data = match result {
        Ok(data) => data,
        Err(msg) => {
            return Err(Error::from(msg));
        }
    };

    let response = format!(
        "Logged {} characters. Total characters logged: {}.",
        characters, data.total_characters
    );
    ctx.say(response).await?;
    Ok(())
}

/// Explains how the bot works.
#[poise::command(slash_command)]
pub async fn usage(ctx: Context<'_>) -> Result<(), Error> {
    let embed = CreateEmbed::default()
        .author(CreateEmbedAuthor::new("Bread"))
        .title(format!("Immersion Tracking Bot"))

        .description("This bot is for tracking characters read, **not** for listening immersion. That doesn't mean that listening is not important, but it's implied that you're spending an equal amount of time practicing listening as you are reading.
Reading that you can track includes:

1. Novels
2. VNs
3. Anime with JP subtitles (NOT raw)
4. Podcasts with a script
5. Anything else in a similar vein (if you're unsure, ask an admin)

Do **NOT** estimate your immersion, only log immersion that you are sure of the exact number of characters of. As you log your immersion, you will automatically receive roles based on how much you've done. While we would love to take your word for it, we can't be sure that everyone is honest. To counteract people who might lie about their immersion amount, certain roles will require a kotoba quiz in order to continue. The quiz should be straightforward if you've done the amount of immersion required for that role (with the exception being 天仙 and 上手, whose quizes are intentionally bullshit).")
        .footer(CreateEmbedFooter::new(
            "See /help for a list of commands, /how_to_track for further immersion tracking information, and /roles for roles.",
        ));

    ctx.send(CreateReply::default().embed(embed)).await?;
    Ok(())
}

/// Explains how to track your reading characters.
#[poise::command(slash_command)]
pub async fn how_to_track(ctx: Context<'_>) -> Result<(), Error> {
    let embed = CreateEmbed::default()
        .author(CreateEmbedAuthor::new("Bread"))
        .title(format!("Immersion Tracking Bot"))
        .description("Track **only** characters read (whether that be novels, VNs, games, subtitles, scripts, etc.), **not** raw listening. **Do not** guess how much immersion you've done, only log exact numbers that you're sure of. Whenever possible, your character count should exclude special characters (like punctuation, etc). These rules don't exist to be unnecessarily rigid, they exist to keep everyone on a (measureably) even playing field. Don't spoil things for others.

**Anime**  
It's recommended to create a Japanese-only account on [myanimelist](https://myanimelist.net/) or a similar tracking site, which will show exactly how many episodes you've watched with JP subs. You can download Japanese subtitles from [kitsunekko](https://kitsunekko.net/dirlist.php?dir=subtitles%2Fjapanese%2F) and watch/mine with [animebook](https://cademcniven.com/posts/20210703/). You can find the exact number of characters in the show with [subtitle character counter](https://cademcniven.com/subtitleCharacterCounter.html).

**Novels**  
You can track characters read for novels by reading on [ttu reader](https://ttu-ebook.web.app/). You can convert your ebooks to epub using [calibre](https://calibre-ebook.com/). You can download webnovels to epub with [WebToEpub](https://github.com/dteviot/WebToEpub). You can also read/track webnovels with [Eminent Reader](https://github.com/cademcniven/Eminent-Reader).

**Text Files**  
You can track characters read from text files using [textReader](https://cademcniven.com/projects/textReader/). This is useful for things like [erovoice](https://dl.erovoice.us/) scripts.

**Visual Novels**  
You can track characters read from visual novels using a [texthooker](https://anacreondjt.gitlab.io/texthooker.html). Follow the [TMW Guide](https://learnjapanese.moe/vn/) to learn how to set it up.

**Manga**  
You can track characters read from manga by using [this bookmarklet](https://github.com/kha-white/mokuro/issues/4#issuecomment-1120349063) for [mokuro](https://github.com/kha-white/mokuro).")
        .footer(CreateEmbedFooter::new(
            "See /help for a list of commands, and /usage for an explanation on what I can do.",
        ));

    ctx.send(CreateReply::default().embed(embed)).await?;
    Ok(())
}

/// Shows your latest log history.
#[poise::command(slash_command)]
pub async fn history(ctx: Context<'_>) -> Result<(), Error> {
    let mut repository = ctx.data().character_statistics_repository.lock().await;
    let user_id = ctx.author().id;
    let result = repository.get_log_entries(user_id).await;

    let log_entries = match result {
        Ok(data) => data,
        Err(msg) => {
            return Err(Error::from(msg));
        }
    };

    let mut embed_builder = CreateEmbed::default()
        .author(CreateEmbedAuthor::new("Bread"))
        .title("Immersion Tracking Bot");

    let mut lines = "".to_string();
    for history in log_entries {
        let notes = match history.notes() {
            None => "-",
            Some(x) => &x,
        };
        let time = history.time().format("%Y年%m月%d日").to_string();
        lines += &format!(
            "{}: {} characters | {}\n",
            time,
            history.characters(),
            notes
        );
    }

    embed_builder = embed_builder.description(lines);

    ctx.send(CreateReply::default().embed(embed_builder))
        .await?;
    Ok(())
}

/// Shows the leaderboard.
#[poise::command(slash_command)]
pub async fn leaderboard(
    ctx: Context<'_>,
    #[description = "The page number of the leaderboard to display. Defaults to the first page."]
    page: Option<usize>,
) -> Result<(), Error> {
    let mut repository = ctx.data().character_statistics_repository.lock().await;
    let page_number = page.unwrap_or(1).saturating_sub(1);
    let users = repository
        .fetch_paginated_users_by_characters(page_number)
        .await?;

    let embed_builder = CreateEmbed::default()
        .author(CreateEmbedAuthor::new("Bread"))
        .title("Immersion Tracking Bot");

    let mut line = "".to_owned();
    for (index, user) in users.iter().enumerate() {
        let discord_user = user.get_user_id().to_user(ctx).await?;

        line += &format!(
            "{}. {}: {} characters.",
            index + 1,
            discord_user.display_name(),
            user.total_characters
        );
    }

    ctx.send(CreateReply::default().embed(embed_builder.description(line)))
        .await?;
    Ok(())
}

/// Shows the list of roles available and how to get them.
#[poise::command(slash_command)]
pub async fn roles(ctx: Context<'_>) -> Result<(), Error> {
    let message = "Not implemented yet.";

    ctx.send(CreateReply::default().content(message)).await?;
    Ok(())
}

/// Shows the list of quizzes you need to unlock certain roles.
#[poise::command(slash_command)]
pub async fn quizzes(ctx: Context<'_>) -> Result<(), Error> {
    let message = "Not implemented yet.";

    ctx.send(CreateReply::default().content(message)).await?;
    Ok(())
}

// Vote for something
// #[poise::command(prefix_command, slash_command)]
// pub async fn vote(
//     ctx: Context<'_>,
//     #[description = "What to vote for"] choice: String,
// ) -> Result<(), Error> {
//     // Lock the Mutex in a block {} so the Mutex isn't locked across an await point
//     let num_votes = {
//         let mut hash_map = ctx.data().votes.lock().unwrap();
//         let num_votes = hash_map.entry(choice.clone()).or_default();
//         *num_votes += 1;
//         *num_votes
//     };

//     let response = format!("Successfully voted for {choice}. {choice} now has {num_votes} votes!");
//     ctx.say(response).await?;
//     Ok(())
// }

// #[poise::command(prefix_command, track_edits, aliases("votes"), slash_command)]
// pub async fn getvotes(
//     ctx: Context<'_>,
//     #[description = "Choice to retrieve votes for"] choice: Option<String>,
// ) -> Result<(), Error> {
//     if let Some(choice) = choice {
//         let num_votes = *ctx.data().votes.lock().unwrap().get(&choice).unwrap_or(&0);
//         let response = match num_votes {
//             0 => format!("Nobody has voted for {} yet", choice),
//             _ => format!("{} people have voted for {}", num_votes, choice),
//         };
//         ctx.say(response).await?;
//     } else {
//         let mut response = String::new();
//         for (choice, num_votes) in ctx.data().votes.lock().unwrap().iter() {
//             response += &format!("{}: {} votes", choice, num_votes);
//         }

//         if response.is_empty() {
//             response += "Nobody has voted for anything yet :(";
//         }

//         ctx.say(response).await?;
//     };

//     Ok(())
// }
