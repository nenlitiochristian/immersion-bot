use std::collections::HashMap;

use serenity::all::{Role, RoleId};

use crate::model::CharacterStatistics;

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

    pub async fn handle_quiz_roles(ctx: &serenity::client::Context) {}
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
        // Define role requirements
        let requirements = [
            (Roles::Heimin, 100_000, None),
            (Roles::Danshaku, 500_000, Some(QuizRoles::Quiz1)),
            (Roles::Shishaku, 1_000_000, None),
            (Roles::Hakushaku, 2_000_000, None),
            (Roles::SourouKoushaku, 3_500_000, None),
            (Roles::OoyakeKoushaku, 5_000_000, Some(QuizRoles::Quiz2)),
            (Roles::Taikou, 7_500_000, None),
            (Roles::Ousama, 10_000_000, None),
            (Roles::Texnnou, 15_000_000, None),
            (Roles::Chisen, 25_000_000, Some(QuizRoles::Quiz3)),
            (Roles::Texnnou, 50_000_000, Some(QuizRoles::Quiz4)),
            (Roles::Jouzu, 100_000_000, Some(QuizRoles::Quiz5)),
        ];

        // Check for the highest eligible role by iterating from the last element
        for (role, char_count, quiz_requirement) in requirements.iter().rev() {
            if characters >= *char_count
                && (quiz_requirement.is_none() || quiz_roles.contains(&quiz_requirement.unwrap()))
            {
                return Some(role.clone());
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
