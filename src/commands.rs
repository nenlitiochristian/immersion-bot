use crate::{model::CharacterLog, Context, Error};

/// Show this help menu.
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

/// Log immersion characters.
///
/// Optionally, add a note to keep track of read materials, i.e: `/log_characters characters:4000 notes:Episode 1 of Love Live season 1`
#[poise::command(prefix_command, slash_command)]
pub async fn log_characters(
    ctx: Context<'_>,
    #[description = "The amount of characters read"] characters: i32,
    #[description = "Extra information such as the title of the book or VN"] notes: Option<String>,
) -> Result<(), Error> {
    // Lock the Mutex in a block {} so the Mutex isn't locked across an await point
    // ^ I have no idea what this means lmao
    let total_characters = {
        let mut hash_map = ctx.data().logs.lock().unwrap();
        let user_id = ctx.author().id;
        let character_log = hash_map
            .entry(user_id)
            .or_insert_with(|| CharacterLog::new());

        let time = ctx.created_at();
        character_log.add_log(characters, &time, notes);
        character_log.total_characters()
    };

    let response =
        format!("Logged {characters} characters. Total characters logged: {total_characters}.");
    ctx.say(response).await?;
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
