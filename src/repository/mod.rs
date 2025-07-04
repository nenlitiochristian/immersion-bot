pub mod sqlite_db;

use crate::Error;
use chrono::{DateTime, Utc};

use crate::model::{CharacterLogEntry, CharacterStatistics};

pub trait CharacterStatisticsRepository {
    fn add_log_entry(
        &mut self,
        user_id: u64,
        name: &str,
        characters: i32,
        time: &DateTime<Utc>,
        notes: Option<String>,
    ) -> Result<CharacterStatistics, Error>;

    /// Checks if a user has logged before. Doesn't add the user to the db.
    fn exists(&self, user_id: u64) -> Result<bool, Error>;

    /// Returns the total logged characters of a user. If the user doesn't exist in the db, this also inserts the user to the db.
    fn get_or_initialize_statistics(
        &mut self,
        user_id: u64,
        name: &str,
    ) -> Result<CharacterStatistics, Error>;

    fn get_rank(&mut self, statistics: &CharacterStatistics) -> Result<i32, Error>;

    /// Inactive means that the user has left the server and won't be shown in the leaderboards
    /// Because the user could already be gone when we change the active status, we don't always know their latest_name
    /// None for latest_name means that we don't change it in the db
    fn set_active_status(
        &mut self,
        user_id: u64,
        active: bool,
        latest_name: Option<&str>,
    ) -> Result<(), Error>;

    /// Returns a list of active users according to the (LEADERBOARD_PAGE_SIZE constant), sorted by the amount of characters logged descendingly.
    fn get_paginated_active_users_by_characters(
        &mut self,
        page_number: u64,
    ) -> Result<Vec<CharacterStatistics>, Error>;

    /// Returns a list of users according to the (LEADERBOARD_PAGE_SIZE constant), sorted by the user id.
    fn get_paginated_users_by_id(
        &mut self,
        page_number: u64,
    ) -> Result<Vec<CharacterStatistics>, Error>;

    fn get_total_active_users(&mut self) -> Result<u64, Error>;

    /// Returns a list of log entries according to the (LOG_ENTRY_PAGE_SIZE constant), sorted by time created
    fn get_paginated_log_entries_by_time(
        &mut self,
        user_id: u64,
        page_number: u64,
    ) -> Result<Vec<CharacterLogEntry>, Error>;

    fn get_total_log_entries(&mut self, user_id: u64) -> Result<u64, Error>;
}

pub trait MetadataRepository {
    fn get_last_active_status_refresh(&self) -> Result<Option<DateTime<Utc>>, Error>;
    fn set_last_active_status_refresh(&mut self, time: DateTime<Utc>) -> Result<(), Error>;
}
