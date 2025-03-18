use std::ops::Neg;

use crate::{
    constants::{LEADERBOARD_PAGE_SIZE, LOG_ENTRY_PAGE_SIZE},
    Error,
};
use rusqlite::{OptionalExtension, Transaction};
use serenity::all::{Timestamp, UserId};

use crate::model::{CharacterLogEntry, CharacterStatistics};

pub trait CharacterStatisticsRepository {
    fn add_log_entry(
        &mut self,
        user_id: UserId,
        characters: i32,
        time: &Timestamp,
        notes: Option<String>,
    ) -> Result<CharacterStatistics, Error>;

    /// Returns the total logged characters of a user. If the user doesn't exist in the db, this also inserts the user to the db.
    fn get_statistics(&mut self, user_id: UserId) -> Result<CharacterStatistics, Error>;

    fn get_rank(&mut self, statistics: &CharacterStatistics) -> Result<i32, Error>;

    /// Returns a list of users according to the (LEADERBOARD_PAGE_SIZE constant), sorted by the amount of characters logged descendingly.
    fn get_paginated_users_by_characters(
        &mut self,
        page_number: u64,
    ) -> Result<Vec<CharacterStatistics>, Error>;

    fn get_total_users(&mut self) -> Result<u64, Error>;

    /// Returns a list of log entries according to the (LOG_ENTRY_PAGE_SIZE constant), sorted by time created
    fn get_paginated_log_entries_by_time(
        &mut self,
        user_id: UserId,
        page_number: u64,
    ) -> Result<Vec<CharacterLogEntry>, Error>;

    fn get_total_log_entries(&mut self, user_id: UserId) -> Result<u64, Error>;
}

pub struct SQLiteCharacterStatisticsRepository<'conn> {
    transaction: &'conn Transaction<'conn>,
}

impl<'conn> SQLiteCharacterStatisticsRepository<'conn> {
    pub fn new(transaction: &'conn Transaction<'conn>) -> Self {
        SQLiteCharacterStatisticsRepository { transaction }
    }

    fn initialize_statistics(&mut self, user_id: UserId) -> Result<CharacterStatistics, Error> {
        let id = user_id.get();
        self.transaction.execute(
            "
        INSERT INTO CharacterStatistics (user_id, total_characters)
        VALUES (?1, ?2)
        ",
            (id, 0),
        )?;
        Ok(CharacterStatistics::new(user_id, 0))
    }
}

impl CharacterStatisticsRepository for SQLiteCharacterStatisticsRepository<'_> {
    fn add_log_entry(
        &mut self,
        user_id: UserId,
        characters: i32,
        time: &Timestamp,
        notes: Option<String>,
    ) -> Result<CharacterStatistics, Error> {
        let id = user_id.get();
        let old_statistics = self.get_statistics(user_id)?;

        let characters = if characters >= 0 {
            characters
        } else {
            // don't let the total characters be negative
            // by clamping the negative log to current total characters
            characters.clamp(old_statistics.total_characters.neg(), 0)
        };

        self.transaction.execute(
            "
            INSERT INTO CharacterLogEntry (user_id, characters, time, notes)
            VALUES (?1, ?2, ?3, ?4);
            ",
            (id, characters, time.unix_timestamp(), notes),
        )?;

        let new_statistics =
            CharacterStatistics::new(user_id, old_statistics.total_characters + characters);

        self.transaction.execute(
            "
    UPDATE CharacterStatistics 
    SET total_characters = ?1
    WHERE user_id = ?2;
        ",
            (new_statistics.total_characters, id),
        )?;

        Ok(new_statistics)
    }

    fn get_paginated_users_by_characters(
        &mut self,
        page_number: u64,
    ) -> Result<Vec<CharacterStatistics>, Error> {
        let offset = page_number * LEADERBOARD_PAGE_SIZE;

        let mut stmt = self.transaction.prepare(
            "
                SELECT user_id, total_characters
                FROM CharacterStatistics
                ORDER BY total_characters DESC
                LIMIT ?1 OFFSET ?2;
                ",
        )?;

        let rows = stmt.query_map([LEADERBOARD_PAGE_SIZE, offset], |row| {
            let user_id: u64 = row.get(0)?;
            let total_characters: i32 = row.get(1)?;
            Ok(CharacterStatistics::new(
                UserId::from(user_id),
                total_characters,
            ))
        })?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }

        Ok(result)
    }

    fn get_paginated_log_entries_by_time(
        &mut self,
        user_id: UserId,
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

        let rows = stmt.query_map([user_id.get(), LOG_ENTRY_PAGE_SIZE, offset], |row| {
            let user_id: u64 = row.get(1)?;
            let characters: i32 = row.get(2)?;
            let time: i64 = row.get(3)?;
            let notes: Option<String> = row.get(4)?;

            Ok(CharacterLogEntry::new(
                UserId::new(user_id),
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

    fn get_statistics(&mut self, user_id: UserId) -> Result<CharacterStatistics, Error> {
        let id = user_id.get();
        let characters = self
            .transaction
            .query_row(
                "
        SELECT total_characters FROM CharacterStatistics
        WHERE user_id = ?1
        ",
                [id],
                |row| {
                    let c: i32 = row.get(0)?;
                    Ok(c)
                },
            )
            .optional()?;

        let characters = match characters {
            Some(c) => c,
            None => self.initialize_statistics(user_id)?.total_characters,
        };

        Ok(CharacterStatistics::new(user_id, characters))
    }

    fn get_rank(&mut self, statistics: &CharacterStatistics) -> Result<i32, Error> {
        let mut stmt = self.transaction.prepare(
            "
            SELECT COUNT(*) 
            FROM CharacterStatistics 
            WHERE total_characters > ?1
            ",
        )?;

        let rank_count: i64 = stmt.query_row([statistics.total_characters], |row| row.get(0))?;

        // The rank is one plus the number of users with higher total characters
        let rank = (rank_count + 1) as i32;
        Ok(rank)
    }

    fn get_total_users(&mut self) -> Result<u64, Error> {
        let mut stmt = self.transaction.prepare(
            "
            SELECT COUNT(*) 
            FROM CharacterStatistics 
            ",
        )?;

        let count: u64 = stmt.query_row([], |row| row.get(0))?;
        Ok(count)
    }

    fn get_total_log_entries(&mut self, user_id: UserId) -> Result<u64, Error> {
        let mut stmt = self.transaction.prepare(
            "
            SELECT COUNT(*) 
            FROM CharacterLogEntry
            WHERE user_id = ?1
            GROUP BY user_id;
            ",
        )?;

        let count: u64 = stmt.query_row([user_id.get()], |row| row.get(0))?;
        Ok(count)
    }
}
