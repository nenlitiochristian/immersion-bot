// get roles on request, no need to insert to DB
pub struct UserRoles {
    pub quizzes: Vec<QuizRoles>,
    pub roles: Vec<Roles>,
}

impl UserRoles {}

pub enum QuizRoles {
    Quiz1,
    Quiz2,
    Quiz3,
    Quiz4,
    Quiz5,
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
