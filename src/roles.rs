use crate::model::CharacterStatistics;

// get roles on request, no need to insert to DB
pub struct UserRoles {
    pub quizzes: Vec<QuizRoles>,
    pub roles: Vec<Roles>,
}

impl UserRoles {
    pub async fn new(ctx: crate::Context<'_>) -> UserRoles {
        // we do the async task first so that we don't borrow any data accross an await point
        let user = ctx.author_member().await.unwrap();
        let mut quizzes: Vec<QuizRoles> = Vec::new();
        let mut roles: Vec<Roles> = Vec::new();

        // the bot may only be used inside the server, assume that guild must exist
        let guild = ctx.guild().unwrap();
        let guild_roles = &guild.roles;

        for id in &user.roles {
            if let Some(guild_role) = guild_roles.get(id) {
                // try to parse as a quiz role
                if let Some(quiz_role) = QuizRoles::from_string(&guild_role.name) {
                    quizzes.push(quiz_role);
                }
                // otherwise, try as a general role
                else if let Some(role) = Roles::from_string(&guild_role.name) {
                    roles.push(role);
                }
            }
        }

        UserRoles { quizzes, roles }
    }

    /// Updates the user roles based on currently possessed quiz roles and character count
    pub async fn update_role(&self, ctx: crate::Context<'_>, statistics: &CharacterStatistics) {}
}

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
}

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
