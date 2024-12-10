use firestore::{errors::FirestoreError, FirestoreDb};
use serenity::{all::UserId, async_trait};

use crate::model::{CharacterLogEntry, CharacterStatistics};

#[async_trait::async_trait]
pub trait CharacterStatisticsRepository {
    async fn add_log_entry(&self, user_id: UserId, entry: CharacterLogEntry) -> Result<(), String>;
    async fn get_statistics(
        &self,
        user_id: UserId,
    ) -> Result<Option<CharacterStatistics>, FirestoreError>;
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
    fn new(firestore: FirestoreDb) -> FirestoreCharacterStatisticsRepository {
        FirestoreCharacterStatisticsRepository { firestore }
    }
}

impl CharacterStatisticsRepository for FirestoreCharacterStatisticsRepository {
    async fn add_log_entry(&self, user_id: UserId, entry: CharacterLogEntry) -> Result<(), String> {
        todo!();
    }

    async fn fetch_paginated_top_users_by_characters(
        &self,
        page_number: usize,
    ) -> Result<Vec<(UserId, CharacterStatistics)>, String> {
        todo!();
    }

    async fn get_statistics(
        &self,
        user_id: UserId,
    ) -> Result<Option<CharacterStatistics>, FirestoreError> {
        let data: Option<CharacterStatistics> = self
            .firestore
            .fluent()
            .select()
            .by_id_in("users")
            .obj()
            .one(format!("{user_id}"))
            .await?;
        Ok(data)
    }
}
