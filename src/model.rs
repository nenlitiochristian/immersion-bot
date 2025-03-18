use std::sync::Mutex;

use reqwest::Client;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serenity::all::{Timestamp, UserId};

// Custom user data passed to all command functions
pub struct Data {
    pub connection: Mutex<Connection>,
    /// needed to make calls to the kotoba API for quizzes
    pub http_client: Client,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CharacterStatistics {
    user_id: UserId,
    pub total_characters: i32,
}

impl CharacterStatistics {
    pub fn new(user_id: UserId, total_characters: i32) -> CharacterStatistics {
        CharacterStatistics {
            total_characters,
            user_id,
        }
    }

    pub fn get_user_id(&self) -> UserId {
        self.user_id
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CharacterLogEntry {
    user_id: UserId,
    characters: i32,
    time: Timestamp,
    notes: Option<String>,
}

impl CharacterLogEntry {
    pub fn characters(&self) -> i32 {
        self.characters
    }

    pub fn time(&self) -> &Timestamp {
        &self.time
    }

    pub fn notes(&self) -> &Option<String> {
        &self.notes
    }

    pub fn new(
        user_id: UserId,
        characters: i32,
        time: &Timestamp,
        notes: Option<String>,
    ) -> CharacterLogEntry {
        CharacterLogEntry {
            user_id,
            characters,
            time: time.to_owned(),
            notes,
        }
    }
}
