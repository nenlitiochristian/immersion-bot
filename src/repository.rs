use std::ops::Neg;

use crate::{
    constants::{LEADERBOARD_PAGE_SIZE, LOG_ENTRY_PAGE_SIZE},
    Error,
};
use chrono::{DateTime, TimeZone, Utc};
use rusqlite::{params, OptionalExtension, Transaction};
use serenity::all::Timestamp;

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

pub struct SQLiteMetadataRepository<'conn> {
    transaction: &'conn Transaction<'conn>,
}

impl<'conn> SQLiteMetadataRepository<'conn> {
    pub fn new(transaction: &'conn Transaction<'conn>) -> Self {
        SQLiteMetadataRepository { transaction }
    }
}

impl MetadataRepository for SQLiteMetadataRepository<'_> {
    fn get_last_active_status_refresh(&self) -> Result<Option<DateTime<Utc>>, Error> {
        let mut stmt = self.transaction.prepare(
            "
            SELECT last_active_status_refresh
            FROM Metadata
            ",
        )?;

        let rows = stmt.query_map([], |row| {
            // Since we're selecting one column, use index 0
            let time: i64 = row.get(0)?;
            Ok(Utc.timestamp_opt(time, 0).unwrap())
        })?;

        for row in rows {
            return Ok(Some(row?));
        }

        Ok(None)
    }

    fn set_last_active_status_refresh(&mut self, time: DateTime<Utc>) -> Result<(), Error> {
        let sql_update = "UPDATE Metadata SET last_active_status_refresh = ?1";
        let affected = self
            .transaction
            .execute(sql_update, params![time.timestamp()])?;

        // If no row was updated, insert a new row
        if affected == 0 {
            let sql_insert = "INSERT INTO Metadata (last_active_status_refresh) VALUES (?1)";
            self.transaction
                .execute(sql_insert, params![time.timestamp()])?;
        }

        Ok(())
    }
}

pub struct SQLiteCharacterStatisticsRepository<'conn> {
    transaction: &'conn Transaction<'conn>,
}

impl<'conn> SQLiteCharacterStatisticsRepository<'conn> {
    pub fn new(transaction: &'conn Transaction<'conn>) -> Self {
        SQLiteCharacterStatisticsRepository { transaction }
    }

    fn initialize_statistics(
        &mut self,
        user_id: u64,
        name: &str,
    ) -> Result<CharacterStatistics, Error> {
        self.transaction.execute(
            "
        INSERT INTO CharacterStatistics (user_id, total_characters, name)
        VALUES (?1, ?2, ?3)
        ",
            (user_id, 0, name),
        )?;
        Ok(CharacterStatistics::new(user_id, 0, name.to_owned()))
    }
}

impl CharacterStatisticsRepository for SQLiteCharacterStatisticsRepository<'_> {
    fn add_log_entry(
        &mut self,
        user_id: u64,
        name: &str,
        characters: i32,
        time: &DateTime<Utc>,
        notes: Option<String>,
    ) -> Result<CharacterStatistics, Error> {
        let old_statistics = self.get_or_initialize_statistics(user_id, name)?;

        let characters = if characters >= 0 {
            characters
        } else {
            // don't let the total characters be negative
            // by clamping the negative log to current total characters
            characters.clamp(old_statistics.total_characters.neg(), 0)
        };

        // don't insert empty logs (no characters or no notes)
        if characters != 0 || notes.clone().is_some_and(|n| !n.trim().is_empty()) {
            self.transaction.execute(
                "
                INSERT INTO CharacterLogEntry (user_id, characters, time, notes)
                VALUES (?1, ?2, ?3, ?4);
                ",
                (user_id, characters, time.timestamp(), notes),
            )?;
        }

        let new_statistics = CharacterStatistics::new(
            user_id,
            old_statistics.total_characters + characters,
            name.to_owned(),
        );

        self.transaction.execute(
            "
UPDATE CharacterStatistics 
SET total_characters = ?1, name = ?2
WHERE user_id = ?3;
        ",
            (new_statistics.total_characters, name, user_id),
        )?;

        Ok(new_statistics)
    }

    fn set_active_status(
        &mut self,
        user_id: u64,
        active: bool,
        latest_name: Option<&str>,
    ) -> Result<(), Error> {
        if latest_name.is_some() {
            let sql = "UPDATE CharacterStatistics SET is_active = ?1, name = ?2 WHERE user_id = ?3";

            self.transaction
                .execute(sql, params![active, latest_name, user_id])?;
        } else {
            let sql = "UPDATE CharacterStatistics SET is_active = ?1 WHERE user_id = ?2";

            self.transaction.execute(sql, params![active, user_id])?;
        }
        Ok(())
    }

    fn get_paginated_active_users_by_characters(
        &mut self,
        page_number: u64,
    ) -> Result<Vec<CharacterStatistics>, Error> {
        let offset = page_number * LEADERBOARD_PAGE_SIZE;

        let mut stmt = self.transaction.prepare(
            "
                SELECT user_id, total_characters, name
                FROM CharacterStatistics
                WHERE is_active == 1 AND total_characters > 0
                ORDER BY total_characters DESC
                LIMIT ?1 OFFSET ?2;
                ",
        )?;

        let rows = stmt.query_map([LEADERBOARD_PAGE_SIZE, offset], |row| {
            let user_id: u64 = row.get(0)?;
            let total_characters: i32 = row.get(1)?;
            let name: String = row.get(2)?;
            Ok(CharacterStatistics::new(user_id, total_characters, name))
        })?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }

        Ok(result)
    }

    /// Returns a list of users according to the (LEADERBOARD_PAGE_SIZE constant), sorted by the user id.
    fn get_paginated_users_by_id(
        &mut self,
        page_number: u64,
    ) -> Result<Vec<CharacterStatistics>, Error> {
        let offset = page_number * LEADERBOARD_PAGE_SIZE;

        let mut stmt = self.transaction.prepare(
            "
                SELECT user_id, total_characters, name
                FROM CharacterStatistics
                ORDER BY user_id ASC
                LIMIT ?1 OFFSET ?2;
                ",
        )?;

        let rows = stmt.query_map([LEADERBOARD_PAGE_SIZE, offset], |row| {
            let user_id: u64 = row.get(0)?;
            let total_characters: i32 = row.get(1)?;
            let name: String = row.get(2)?;
            Ok(CharacterStatistics::new(user_id, total_characters, name))
        })?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }

        Ok(result)
    }

    fn get_paginated_log_entries_by_time(
        &mut self,
        user_id: u64,
        page_number: u64,
    ) -> Result<Vec<CharacterLogEntry>, Error> {
        let offset = page_number * LOG_ENTRY_PAGE_SIZE;

        let mut stmt = self.transaction.prepare(
            "
                SELECT id, user_id, characters, time, notes
                FROM CharacterLogEntry
                WHERE user_id = ?1
                ORDER BY time DESC
                LIMIT ?2 OFFSET ?3;
            ",
        )?;

        let rows = stmt.query_map([user_id, LOG_ENTRY_PAGE_SIZE, offset], |row| {
            let user_id: u64 = row.get(1)?;
            let characters: i32 = row.get(2)?;
            let time: i64 = row.get(3)?;
            let notes: Option<String> = row.get(4)?;

            Ok(CharacterLogEntry::new(
                user_id,
                characters,
                &Timestamp::from_unix_timestamp(time).expect("Date conversion error!"),
                notes,
            ))
        })?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }

        Ok(result)
    }

    fn exists(&self, user_id: u64) -> Result<bool, Error> {
        let characters = self
            .transaction
            .query_row(
                "
    SELECT total_characters FROM CharacterStatistics
    WHERE user_id = ?1
    ",
                [user_id],
                |row| {
                    let c: i32 = row.get(0)?;
                    Ok(c)
                },
            )
            .optional()?;

        let exists = match characters {
            Some(_) => true,
            None => false,
        };
        Ok(exists)
    }

    fn get_or_initialize_statistics(
        &mut self,
        user_id: u64,
        name: &str,
    ) -> Result<CharacterStatistics, Error> {
        let characters = self
            .transaction
            .query_row(
                "
        SELECT total_characters FROM CharacterStatistics
        WHERE user_id = ?1
        ",
                [user_id],
                |row| {
                    let c: i32 = row.get(0)?;
                    Ok(c)
                },
            )
            .optional()?;

        let characters = match characters {
            Some(c) => c,
            None => self.initialize_statistics(user_id, name)?.total_characters,
        };

        Ok(CharacterStatistics::new(
            user_id,
            characters,
            name.to_owned(),
        ))
    }

    fn get_rank(&mut self, statistics: &CharacterStatistics) -> Result<i32, Error> {
        let mut stmt = self.transaction.prepare(
            "
            SELECT COUNT(*) 
            FROM CharacterStatistics 
            WHERE total_characters > ?1 AND is_active == 1
            ",
        )?;

        let rank_count: i64 = stmt.query_row([statistics.total_characters], |row| row.get(0))?;

        // The rank is one plus the number of users with higher total characters
        let rank = (rank_count + 1) as i32;
        Ok(rank)
    }

    fn get_total_active_users(&mut self) -> Result<u64, Error> {
        let mut stmt = self.transaction.prepare(
            "
            SELECT COUNT(*) 
            FROM CharacterStatistics 
            WHERE is_active == 1 AND total_characters > 0
            ",
        )?;

        let count: u64 = stmt.query_row([], |row| row.get(0))?;
        Ok(count)
    }

    fn get_total_log_entries(&mut self, user_id: u64) -> Result<u64, Error> {
        let mut stmt = self.transaction.prepare(
            "
            SELECT COUNT(*) 
            FROM CharacterLogEntry
            WHERE user_id = ?1
            GROUP BY user_id;
            ",
        )?;

        let count: u64 = stmt.query_row([user_id], |row| row.get(0))?;
        Ok(count)
    }
}
