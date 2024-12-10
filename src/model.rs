use crate::repository::FirestoreCharacterStatisticsRepository;
use serde::{Deserialize, Serialize};
use serenity::all::Timestamp;

// Custom user data passed to all command functions
pub struct Data {
    pub repository: FirestoreCharacterStatisticsRepository,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CharacterStatistics {
    total_characters: i32,
    history: Vec<CharacterLogEntry>,
}

impl CharacterStatistics {
    pub fn total_characters(&self) -> i32 {
        self.total_characters
    }

    pub fn history(&self) -> &Vec<CharacterLogEntry> {
        &self.history
    }

    pub fn add_log(&mut self, characters: i32, time: &Timestamp, notes: Option<String>) -> () {
        let owned_time = time.clone();
        self.history.push(CharacterLogEntry {
            characters,
            time: owned_time,
            notes: notes.into(),
        });
        self.total_characters += characters;
    }

    pub fn new() -> CharacterStatistics {
        CharacterStatistics {
            total_characters: 0,
            history: Vec::new(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CharacterLogEntry {
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
}
