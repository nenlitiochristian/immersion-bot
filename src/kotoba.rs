use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct QuizData {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(rename = "sessionName")]
    pub session_name: String,
    #[serde(rename = "startTime")]
    pub start_time: String,
    #[serde(rename = "endTime")]
    pub end_time: String,
    pub participants: Vec<Participant>,
    #[serde(rename = "discordServerIconUri")]
    pub discord_server_icon_uri: Option<String>,
    #[serde(rename = "discordServerName")]
    pub discord_server_name: String,
    #[serde(rename = "discordChannelName")]
    pub discord_channel_name: String,
    pub scores: Vec<Score>,
    pub settings: Settings,
    pub decks: Vec<Deck>,
    #[serde(rename = "isLoaded")]
    pub is_loaded: bool,
    #[serde(rename = "rawStartCommand")]
    pub raw_start_command: String,
    pub questions: Vec<Question>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Participant {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(rename = "discordUser")]
    pub discord_user: DiscordUser,
    pub admin: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DiscordUser {
    pub id: String,
    pub avatar: String,
    pub username: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Score {
    pub user: String,
    pub score: i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Settings {
    #[serde(rename = "isConquest")]
    pub is_conquest: bool,
    #[serde(rename = "scoreLimit")]
    pub score_limit: i32,
    #[serde(rename = "unansweredQuestionLimit")]
    pub unanswered_question_limit: i32,
    #[serde(rename = "answerTimeLimitInMs")]
    pub answer_time_limit_in_ms: i32,
    #[serde(rename = "newQuestionDelayAfterUnansweredInMs")]
    pub new_question_delay_after_unanswered_in_ms: i32,
    #[serde(rename = "newQuestionDelayAfterAnsweredInMs")]
    pub new_question_delay_after_answered_in_ms: i32,
    #[serde(rename = "additionalAnswerWaitTimeInMs")]
    pub additional_answer_wait_time_in_ms: i32,
    #[serde(rename = "fontSize")]
    pub font_size: i32,
    #[serde(rename = "fontColor")]
    pub font_color: String,
    #[serde(rename = "backgroundColor")]
    pub background_color: String,
    pub font: String,
    #[serde(rename = "maxMissedQuestions")]
    pub max_missed_questions: i32,
    pub shuffle: bool,
    #[serde(rename = "serverSettings")]
    pub server_settings: ServerSettings,
    #[serde(rename = "inlineSettings")]
    pub inline_settings: InlineSettings,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ServerSettings {
    #[serde(rename = "bgColor")]
    pub bg_color: String,
    #[serde(rename = "fontFamily")]
    pub font_family: String,
    pub color: String,
    pub size: i32,
    #[serde(rename = "additionalAnswerWaitWindow")]
    pub additional_answer_wait_window: f64,
    #[serde(rename = "answerTimeLimit")]
    pub answer_time_limit: i32,
    #[serde(rename = "conquestAndInfernoEnabled")]
    pub conquest_and_inferno_enabled: bool,
    #[serde(rename = "internetDecksEnabled")]
    pub internet_decks_enabled: bool,
    #[serde(rename = "delayAfterAnsweredQuestion")]
    pub delay_after_answered_question: f64,
    #[serde(rename = "delayAfterUnansweredQuestion")]
    pub delay_after_unanswered_question: i32,
    #[serde(rename = "scoreLimit")]
    pub score_limit: i32,
    #[serde(rename = "unansweredQuestionLimit")]
    pub unanswered_question_limit: i32,
    #[serde(rename = "maxMissedQuestions")]
    pub max_missed_questions: i32,
    pub shuffle: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct InlineSettings {
    #[serde(rename = "fontFamily")]
    pub font_family: String,
    #[serde(rename = "delayAfterUnansweredQuestion")]
    pub delay_after_unanswered_question: i32,
    #[serde(rename = "delayAfterAnsweredQuestion")]
    pub delay_after_answered_question: i32,
    #[serde(rename = "additionalAnswerWaitWindow")]
    pub additional_answer_wait_window: i32,
    pub aliases: Vec<String>,
    #[serde(rename = "maxMissedQuestions")]
    pub max_missed_questions: i32,
    #[serde(rename = "answerTimeLimit")]
    pub answer_time_limit: i32,
    #[serde(rename = "scoreLimit")]
    pub score_limit: i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Deck {
    pub name: String,
    #[serde(rename = "shortName")]
    pub short_name: String,
    #[serde(rename = "uniqueId")]
    pub unique_id: String,
    pub mc: bool,
    #[serde(rename = "internetDeck")]
    pub internet_deck: bool,
    #[serde(rename = "appearanceWeight")]
    pub appearance_weight: i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Question {
    #[serde(rename = "deckUniqueId")]
    pub deck_unique_id: String,
    pub question: String,
    pub answers: Vec<String>,
    pub comment: String,
    #[serde(rename = "canCopyToCustomDeck")]
    pub can_copy_to_custom_deck: bool,
    #[serde(rename = "questionCreationStrategy")]
    pub question_creation_strategy: String,
    pub instructions: String,
    #[serde(rename = "linkQuestion")]
    pub link_question: bool,
    pub uri: String,
    #[serde(rename = "correctAnswerers")]
    pub correct_answerers: Vec<String>,
}