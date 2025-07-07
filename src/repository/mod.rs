pub mod postgres_db;
// pub mod sqlite_db;

use crate::Error;
use chrono::{DateTime, Utc};
use sqlx::{Postgres, Transaction};

use crate::model::{CharacterLogEntry, CharacterStatistics};

pub trait CharacterStatisticsRepository<'tx> {
    async fn add_log_entry(
        &mut self,
        tx: &mut Transaction<'tx, Postgres>,
        user_id: u64,
        name: &str,
        characters: i32,
        time: i64,
        notes: Option<String>,
    ) -> Result<CharacterStatistics, Error>;

    /// Checks if a user has logged before. Doesn't add the user to the db.
    async fn exists(
        &mut self,
        tx: &mut Transaction<'tx, Postgres>,
        user_id: u64,
    ) -> Result<bool, Error>;

    /// Returns the total logged characters of a user. If the user doesn't exist in the db, this also inserts the user to the db.
    async fn get_or_initialize_statistics(
        &mut self,
        tx: &mut Transaction<'tx, Postgres>,
        user_id: u64,
        name: &str,
    ) -> Result<CharacterStatistics, Error>;

    async fn get_rank(
        &mut self,
        tx: &mut Transaction<'tx, Postgres>,
        statistics: &CharacterStatistics,
    ) -> Result<i32, Error>;

    /// Inactive means that the user has left the server and won't be shown in the leaderboards
    /// Because the user could already be gone when we change the active status, we don't always know their latest_name
    /// None for latest_name means that we don't change it in the db
    async fn set_active_status(
        &mut self,
        tx: &mut Transaction<'tx, Postgres>,
        user_id: u64,
        active: bool,
        latest_name: Option<&str>,
    ) -> Result<(), Error>;

    /// Returns a list of active users according to the (LEADERBOARD_PAGE_SIZE constant), sorted by the amount of characters logged descendingly.
    async fn get_paginated_active_users_by_characters(
        &mut self,
        tx: &mut Transaction<'tx, Postgres>,
        page_number: u64,
    ) -> Result<Vec<CharacterStatistics>, Error>;

    /// Returns a list of users according to the (LEADERBOARD_PAGE_SIZE constant), sorted by the user id.
    async fn get_paginated_users_by_id(
        &mut self,
        tx: &mut Transaction<'tx, Postgres>,
        page_number: u64,
    ) -> Result<Vec<CharacterStatistics>, Error>;

    async fn get_total_active_users(
        &mut self,
        tx: &mut Transaction<'tx, Postgres>,
    ) -> Result<u64, Error>;

    /// Returns a list of log entries according to the (LOG_ENTRY_PAGE_SIZE constant), sorted by time created
    async fn get_paginated_log_entries_by_time(
        &mut self,
        tx: &mut Transaction<'tx, Postgres>,
        user_id: u64,
        page_number: u64,
    ) -> Result<Vec<CharacterLogEntry>, Error>;

    async fn get_total_log_entries(
        &mut self,
        tx: &mut Transaction<'tx, Postgres>,
        user_id: u64,
    ) -> Result<u64, Error>;
}

pub trait MetadataRepository<'tx> {
    async fn get_last_active_status_refresh(
        &mut self,
        tx: &mut Transaction<'tx, Postgres>,
    ) -> Result<Option<DateTime<Utc>>, Error>;
    async fn set_last_active_status_refresh(
        &mut self,
        tx: &mut Transaction<'tx, Postgres>,
        time: DateTime<Utc>,
    ) -> Result<(), Error>;
}
