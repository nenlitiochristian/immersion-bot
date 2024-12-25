use std::collections::HashMap;

use serenity::all::{Message, Role, RoleId, UserId};

use crate::{
    constants::{self, QUIZ_FONT, QUIZ_TIME_LIMIT},
    kotoba::QuizData,
    model::{CharacterStatistics, Data},
};

// get roles on request, no need to insert to DB
pub struct UserRoles {
    pub quizzes: Vec<QuizRoles>,
    pub roles: Vec<Roles>,
}

impl UserRoles {
    pub fn new(user_roles: &Vec<RoleId>, guild_roles: &HashMap<RoleId, Role>) -> UserRoles {
        let mut quizzes: Vec<QuizRoles> = Vec::new();
        let mut roles: Vec<Roles> = Vec::new();

        for id in user_roles {
            if let Some(guild_role) = guild_roles.get(&id) {
                // Try to parse as a quiz role
                if let Some(quiz_role) = QuizRoles::from_string(&guild_role.name) {
                    quizzes.push(quiz_role);
                }
                // Otherwise, try as a general role
                else if let Some(role) = Roles::from_string(&guild_role.name) {
                    roles.push(role);
                }
            }
        }

        UserRoles { quizzes, roles }
    }

    /// Updates the user roles based on currently possessed quiz roles and character count
    pub async fn update_role(
        &self,
        ctx: crate::Context<'_>,
        statistics: &CharacterStatistics,
    ) -> Result<(), crate::Error> {
        let characters = statistics.total_characters;
        let new_role = Roles::from_characters_and_quiz_roles(&self.quizzes, characters);

        // user shouldn't have any roles, do nothing
        if new_role.is_none() && self.roles.is_empty() {
            return Ok(());
        }

        let new_role = new_role.unwrap();
        // user's role didn't change, do nothing
        if self.roles.iter().any(|role| role == &new_role) {
            return Ok(());
        }

        // user role changed, we need to clear the old ones
        let guild = ctx.guild().unwrap().clone(); // Ensure we are in a guild
        let user = ctx.author_member().await.unwrap();
        for role in &self.roles {
            let guild_role = guild.role_by_name(&role.to_string()).unwrap();
            user.remove_role(ctx, guild_role.id).await?;
        }

        let guild_role = guild.role_by_name(&new_role.to_string()).unwrap();
        user.add_role(ctx, guild_role.id).await?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum QuizRoles {
    Quiz1,
    Quiz2,
    Quiz3,
    Quiz4,
    Quiz5,
}

#[derive(Debug)]
pub struct QuizRequirement {
    pub quiz_role: QuizRoles,
    pub quiz_name: &'static str,
    pub score_limit: i32,
    pub max_missed_questions: i32,
}

pub static QUIZ_REQUIREMENTS: [QuizRequirement; 5] = [
    QuizRequirement {
        quiz_role: QuizRoles::Quiz1,
        quiz_name: "pq_1",
        score_limit: 15,
        max_missed_questions: 4,
    },
    QuizRequirement {
        quiz_role: QuizRoles::Quiz2,
        quiz_name: "pq_2",
        score_limit: 20,
        max_missed_questions: 4,
    },
    QuizRequirement {
        quiz_role: QuizRoles::Quiz3,
        quiz_name: "pq_3",
        score_limit: 20,
        max_missed_questions: 4,
    },
    QuizRequirement {
        quiz_role: QuizRoles::Quiz4,
        quiz_name: "pq_4",
        score_limit: 30,
        max_missed_questions: 4,
    },
    QuizRequirement {
        quiz_role: QuizRoles::Quiz5,
        quiz_name: "stations_full",
        score_limit: 100,
        max_missed_questions: 4,
    },
];

impl QuizRoles {
    pub fn to_string(&self) -> String {
        let string = match self {
            Self::Quiz1 => "Quiz 1",
            Self::Quiz2 => "Quiz 2",
            Self::Quiz3 => "Quiz 3",
            Self::Quiz4 => "Quiz 4",
            Self::Quiz5 => "Quiz 5",
        };
        string.to_string()
    }

    pub fn from_string(input: &str) -> Option<QuizRoles> {
        match input {
            "Quiz 1" => Some(Self::Quiz1),
            "Quiz 2" => Some(Self::Quiz2),
            "Quiz 3" => Some(Self::Quiz3),
            "Quiz 4" => Some(Self::Quiz4),
            "Quiz 5" => Some(Self::Quiz5),
            _ => None, // Return `None` for invalid strings
        }
    }

    pub async fn handle_quiz_roles(
        ctx: &serenity::client::Context,
        message: &Message,
        data: &Data,
    ) -> Result<(), crate::Error> {
        if message.embeds.is_empty() {
            return Ok(());
        }

        // make sure the embed is from kotoba
        // the id is hard coded to make sure it's really kotoba
        if message.author.id.get() != constants::KOTOBA_BOT_ID {
            return Ok(());
        }

        for embed in message.embeds.iter() {
            for field in embed.fields.iter() {
                // find the game report API
                if field.name != "Game Report" || !field.value.contains("https://kotobaweb.com/") {
                    continue;
                }

                // fetch the data from kotoba
                let url_start = field.value.find('(').unwrap() + 1;
                let url_end = field.value.find(')').unwrap();
                let substring = &field.value[url_start..url_end];
                let api_url = substring.replace("dashboard", "api");

                // deserialize and validate the data
                let response = data.http_client.get(api_url).send().await?;
                if response.status().is_success() {
                    let quiz_data = response.json::<QuizData>().await?;

                    if quiz_data.decks.len() != 1 {
                        // we don't care, it's not our quiz deck, there needs to be only 1 for the attempt to be valid
                        continue;
                    }

                    let quiz_name = &quiz_data.decks[0].short_name;

                    // we only care if the quiz is one of the quizzes needed for the role
                    let current_quiz = QUIZ_REQUIREMENTS
                        .iter()
                        .find(|requirement| requirement.quiz_name == quiz_name);
                    if current_quiz.is_none() {
                        continue;
                    }
                    let current_quiz = current_quiz.unwrap();

                    //if it is indeed our deck, then we want to make sure there's only one participant
                    if quiz_data.scores.len() != 1 || quiz_data.participants.len() != 1 {
                        message
                            .reply(ctx, "Only one participant is allowed.")
                            .await?;
                        continue;
                    }

                    let quiz_score_limit = &quiz_data.settings.score_limit;
                    let quiz_max_missed_questions = &quiz_data.settings.max_missed_questions;
                    let quiz_font = &quiz_data.settings.font;
                    let quiz_time_limit = &quiz_data.settings.answer_time_limit_in_ms;
                    let quiz_score = &quiz_data.scores[0].score;
                    let quiz_user = &quiz_data.participants[0].discord_user.id;

                    // Since the player didn't reach the score needed, we just ignore it
                    if quiz_score < &current_quiz.score_limit {
                        continue;
                    }

                    if &current_quiz.max_missed_questions != quiz_max_missed_questions
                        || &current_quiz.score_limit != quiz_score_limit
                        || quiz_font != QUIZ_FONT
                        || quiz_time_limit != &QUIZ_TIME_LIMIT
                    {
                        message.reply(ctx, "Quiz settings were incorrect.").await?;
                        continue;
                    }

                    // Actually give the role to the member
                    let user_id = UserId::new(quiz_user.parse::<u64>()?);
                    let guild_id = message.guild_id.unwrap();
                    let guild = message.guild(&ctx.cache).unwrap().clone();
                    let role = guild
                        .role_by_name(&current_quiz.quiz_role.to_string())
                        .unwrap();
                    let member = guild_id.member(ctx, user_id).await?;
                    member.add_role(ctx, role.id).await?;
                } else {
                    message
                        .reply(
                            ctx,
                            "Failed to get quiz results from kotoba, tag an admin for help.",
                        )
                        .await?;
                    continue;
                }
            }
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Roles {
    Heimin,
    Danshaku,
    Shishaku,
    Hakushaku,
    SourouKoushaku,
    OoyakeKoushaku,
    Taikou,
    Ousama,
    Texnnou,
    Chisen,
    Jouzu,
}

#[derive(Debug, Clone)]
pub struct RoleRequirement {
    pub role: Roles,
    pub characters: i32,
    pub quiz_role: Option<QuizRoles>,
}

static ROLE_REQUIREMENTS: [RoleRequirement; 12] = [
    RoleRequirement {
        role: Roles::Heimin,
        characters: 100_000,
        quiz_role: None,
    },
    RoleRequirement {
        role: Roles::Danshaku,
        characters: 500_000,
        quiz_role: Some(QuizRoles::Quiz1),
    },
    RoleRequirement {
        role: Roles::Shishaku,
        characters: 1_000_000,
        quiz_role: None,
    },
    RoleRequirement {
        role: Roles::Hakushaku,
        characters: 2_000_000,
        quiz_role: None,
    },
    RoleRequirement {
        role: Roles::SourouKoushaku,
        characters: 3_500_000,
        quiz_role: None,
    },
    RoleRequirement {
        role: Roles::OoyakeKoushaku,
        characters: 5_000_000,
        quiz_role: Some(QuizRoles::Quiz2),
    },
    RoleRequirement {
        role: Roles::Taikou,
        characters: 7_500_000,
        quiz_role: None,
    },
    RoleRequirement {
        role: Roles::Ousama,
        characters: 10_000_000,
        quiz_role: None,
    },
    RoleRequirement {
        role: Roles::Texnnou,
        characters: 15_000_000,
        quiz_role: None,
    },
    RoleRequirement {
        role: Roles::Chisen,
        characters: 25_000_000,
        quiz_role: Some(QuizRoles::Quiz3),
    },
    RoleRequirement {
        role: Roles::Texnnou,
        characters: 50_000_000,
        quiz_role: Some(QuizRoles::Quiz4),
    },
    RoleRequirement {
        role: Roles::Jouzu,
        characters: 100_000_000,
        quiz_role: Some(QuizRoles::Quiz5),
    },
];

impl Roles {
    pub fn to_string(&self) -> String {
        let string = match self {
            Self::Heimin => "平民",
            Self::Danshaku => "男爵",
            Self::Shishaku => "子爵",
            Self::Hakushaku => "伯爵",
            Self::SourouKoushaku => "候爵",
            Self::OoyakeKoushaku => "公爵",
            Self::Taikou => "大公",
            Self::Ousama => "王様",
            Self::Texnnou => "天皇",
            Self::Chisen => "智仙",
            Self::Jouzu => "上手",
        };
        string.to_string()
    }

    pub fn from_characters_and_quiz_roles(
        quiz_roles: &Vec<QuizRoles>,
        characters: i32,
    ) -> Option<Roles> {
        // Check for the highest eligible role by iterating from the last element
        for requirement in ROLE_REQUIREMENTS.iter().rev() {
            if characters >= requirement.characters
                && (requirement.quiz_role.is_none()
                    || quiz_roles.contains(&requirement.quiz_role.unwrap()))
            {
                return Some(requirement.role.clone());
            }
        }
        None
    }

    pub fn from_string(input: &str) -> Option<Roles> {
        match input {
            "平民" => Some(Self::Heimin),
            "男爵" => Some(Self::Danshaku),
            "子爵" => Some(Self::Shishaku),
            "伯爵" => Some(Self::Hakushaku),
            "候爵" => Some(Self::SourouKoushaku),
            "公爵" => Some(Self::OoyakeKoushaku),
            "大公" => Some(Self::Taikou),
            "王様" => Some(Self::Ousama),
            "天皇" => Some(Self::Texnnou),
            "智仙" => Some(Self::Chisen),
            "上手" => Some(Self::Jouzu),
            _ => None, // Return `None` for invalid strings
        }
    }
}
