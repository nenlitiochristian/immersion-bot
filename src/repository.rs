use firestore::FirestoreDb;
use serenity::all::{Timestamp, UserId};

use crate::model::CharacterStatistics;

pub trait CharacterStatisticsRepository {
    async fn add_log_entry(
        &self,
        user_id: UserId,
        characters: i32,
        time: &Timestamp,
        notes: Option<String>,
    ) -> Result<CharacterStatistics, String>;

    async fn get_statistics(&self, user_id: UserId) -> Result<Option<CharacterStatistics>, String>;

    /// Each page contains 15 users
    async fn fetch_paginated_top_users_by_characters(
        &self,
        page_number: usize,
    ) -> Result<Vec<(UserId, CharacterStatistics)>, String>;
}

pub struct FirestoreCharacterStatisticsRepository {
    firestore: FirestoreDb,
}

// The firestore db:
// users/{userId} : { total_characters: i32, history: Vec<CharacterLogEntry>}

impl FirestoreCharacterStatisticsRepository {
    pub fn new(firestore: FirestoreDb) -> FirestoreCharacterStatisticsRepository {
        FirestoreCharacterStatisticsRepository { firestore }
    }
}

impl CharacterStatisticsRepository for FirestoreCharacterStatisticsRepository {
    async fn add_log_entry(
        &self,
        user_id: UserId,
        characters: i32,
        time: &Timestamp,
        notes: Option<String>,
    ) -> Result<CharacterStatistics, String> {
        let previous_data = self.get_statistics(user_id).await?;

        let mut data = match previous_data {
            Some(data) => data,
            None => CharacterStatistics::new(),
        };

        data.add_log(characters, time, notes);

        let result = self
            .firestore
            .fluent()
            .update()
            .in_col("users")
            .document_id(user_id.to_string())
            .object(&data)
            .execute()
            .await;

        match result {
            Err(msg) => Err(msg.to_string()),
            Ok(data) => Ok(data),
        }
    }

    async fn fetch_paginated_top_users_by_characters(
        &self,
        page_number: usize,
    ) -> Result<Vec<(UserId, CharacterStatistics)>, String> {
        todo!();
    }

    async fn get_statistics(&self, user_id: UserId) -> Result<Option<CharacterStatistics>, String> {
        let result = self
            .firestore
            .fluent()
            .select()
            .by_id_in("users")
            .obj()
            .one(user_id.to_string())
            .await;

        match result {
            Err(msg) => Err(msg.to_string()),
            Ok(data) => Ok(data),
        }
    }
}
