use std::sync::LazyLock;

use chrono::Duration;

use crate::roles::{QuizRequirement, QuizRoles};

pub const KOTOBA_BOT_ID: u64 = 251239170058616833;

/// in seconds -> 2 hours
pub const USER_ACTIVE_STATUS_REFRESH_INTERVAL: i64 = Duration::hours(2).num_seconds();

pub const CONGRATULATE_NEW_ROLE_CHANNEL_IDS: [u64; 1] = [735507346624741387];

pub const QUIZ_TIME_LIMIT: i32 = 20000;
pub const QUIZ_FONT: &str = "Eishiikaisho";

pub const LEADERBOARD_PAGE_SIZE: u64 = 15;
pub const LOG_ENTRY_PAGE_SIZE: u64 = 15;

pub static QUIZ_REQUIREMENTS: LazyLock<Vec<QuizRequirement>> = LazyLock::new(|| {
    vec![
        QuizRequirement {
            quiz_role: QuizRoles::Quiz1,
            score_limit: 15,
            max_missed_questions: 4,
            unique_ids: vec!["281ebf61-e0aa-429e-a09f-f5b56079ee46".to_string()],
        },
        QuizRequirement {
            quiz_role: QuizRoles::Quiz2,
            score_limit: 20,
            max_missed_questions: 4,
            unique_ids: vec!["8982a22e-314d-4a08-a026-12e497299bb1".to_string()],
        },
        QuizRequirement {
            quiz_role: QuizRoles::Quiz3,
            score_limit: 20,
            max_missed_questions: 4,
            unique_ids: vec!["14c54eb0-f77d-4611-b974-c1e109ef09da".to_string()],
        },
        QuizRequirement {
            quiz_role: QuizRoles::Quiz4,
            score_limit: 30,
            max_missed_questions: 4,
            unique_ids: vec![
                "2bef521f-512c-490d-924d-b00086c10f2d".to_string(),
                "animals".to_string(),
                "birds".to_string(),
                "bugs".to_string(),
                "countries".to_string(),
                "fish".to_string(),
                "plants".to_string(),
                "vegetables".to_string(),
                "yojijukugo".to_string(),
            ],
        },
        QuizRequirement {
            quiz_role: QuizRoles::Quiz5,
            score_limit: 100,
            max_missed_questions: 4,
            unique_ids: vec!["stations_japan".to_string()],
        },
    ]
});
