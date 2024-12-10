use std::{collections::HashMap, sync::Mutex};

use serenity::all::{Timestamp, UserId};

// Custom user data passed to all command functions
pub struct Data {
    pub logs: Mutex<HashMap<UserId, CharacterLog>>,
}

pub struct CharacterLog {
    total_characters: i32,
    log_history: Vec<CharacterLogHistory>,
}

impl CharacterLog {
    pub fn total_characters(&self) -> i32 {
        self.total_characters
    }

    pub fn log_history(&self) -> &Vec<CharacterLogHistory> {
        &self.log_history
    }

    pub fn add_log(&mut self, characters: i32, time: &Timestamp, notes: Option<String>) -> () {
        let owned_time = time.clone();
        self.log_history.push(CharacterLogHistory {
            characters,
            time: owned_time,
            notes: notes.into(),
        });
        self.total_characters += characters;
    }

    pub fn new() -> CharacterLog {
        CharacterLog {
            total_characters: 0,
            log_history: Vec::new(),
        }
    }
}

pub struct CharacterLogHistory {
    characters: i32,
    time: Timestamp,
    notes: Option<String>,
}

impl CharacterLogHistory {
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
