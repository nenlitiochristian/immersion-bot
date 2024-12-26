use std::error::Error;

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
    ) -> Result<CharacterStatistics, Box<dyn Error + Sync + Send>>;

    fn get_statistics(
        &mut self,
        user_id: UserId,
    ) -> Result<Option<CharacterStatistics>, Box<dyn Error + Sync + Send>>;

    fn get_rank(&mut self, statistics: &CharacterStatistics) -> Result<i32, crate::Error>;

    /// Returns a list of 15 users, sorted by the amount of characters logged descendingly.
    fn fetch_paginated_users_by_characters(
        &mut self,
        page_number: usize,
    ) -> Result<Vec<CharacterStatistics>, Box<dyn Error + Sync + Send>>;

    fn get_total_users(&mut self) -> Result<usize, crate::Error>;

    fn get_log_entries(
        &mut self,
        user_id: UserId,
    ) -> Result<Vec<CharacterLogEntry>, Box<dyn Error + Sync + Send>>;
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
        user_id: UserId,
    ) -> Result<CharacterStatistics, Box<dyn Error + Sync + Send>> {
        let id = user_id.get();
        self.transaction.execute(
            "
        INSERT INTO CharacterStatistics (user_id, total_characters)
        VALUES (?1, ?2)
        ",
            (id, 0),
        )?;
        Ok(CharacterStatistics::with_total_characters(user_id, 0))
    }
}

impl CharacterStatisticsRepository for SQLiteCharacterStatisticsRepository<'_> {
    fn add_log_entry(
        &mut self,
        user_id: UserId,
        characters: i32,
        time: &Timestamp,
        notes: Option<String>,
    ) -> Result<CharacterStatistics, Box<dyn Error + Sync + Send>> {
        let id = user_id.get();
        let old_statistics = self.get_statistics(user_id)?;

        self.transaction.execute(
            "
            INSERT INTO CharacterLogEntry (user_id, characters, time, notes)
            VALUES (?1, ?2, ?3, ?4);
            ",
            (id, characters, time.unix_timestamp(), notes),
        )?;

        let mut new_statistics = match old_statistics {
            None => CharacterStatistics::new(user_id),
            Some(stats) => stats,
        };

        new_statistics.total_characters += characters;

        // INSERT IF NOT EXISTS, UPDATE IF EXISTS
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

    fn fetch_paginated_users_by_characters(
        &mut self,
        page_number: usize,
    ) -> Result<Vec<CharacterStatistics>, Box<dyn Error + Sync + Send>> {
        const PAGE_SIZE: usize = 15;
        let offset = page_number * PAGE_SIZE;

        let mut stmt = self.transaction.prepare(
            "
                SELECT user_id, total_characters
                FROM CharacterStatistics
                ORDER BY total_characters DESC
                LIMIT ?1 OFFSET ?2;
                ",
        )?;

        let rows = stmt.query_map([PAGE_SIZE as i64, offset as i64], |row| {
            let user_id: u64 = row.get(0)?;
            let total_characters: i32 = row.get(1)?;
            Ok(CharacterStatistics::with_total_characters(
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

    fn get_log_entries(
        &mut self,
        user_id: UserId,
    ) -> Result<Vec<CharacterLogEntry>, Box<dyn Error + Sync + Send>> {
        let id = user_id.get();

        let mut stmt = self.transaction.prepare(
            "
                SELECT id, user_id, characters, time, notes
                FROM CharacterLogEntry
                WHERE user_id = ?1
                ORDER BY time DESC;
                ",
        )?;

        let rows = stmt.query_map([id], |row| {
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

    fn get_statistics(
        &mut self,
        user_id: UserId,
    ) -> Result<Option<CharacterStatistics>, Box<dyn Error + Sync + Send>> {
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

        Ok(Some(CharacterStatistics::with_total_characters(
            user_id, characters,
        )))
    }

    fn get_rank(&mut self, statistics: &CharacterStatistics) -> Result<i32, crate::Error> {
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

    fn get_total_users(&mut self) -> Result<usize, crate::Error> {
        let mut stmt = self.transaction.prepare(
            "
            SELECT COUNT(*) 
            FROM CharacterStatistics 
            ",
        )?;

        let count: usize = stmt.query_row([], |row| row.get(0))?;
        Ok(count)
    }
}
