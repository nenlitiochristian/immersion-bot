use rusqlite::{Connection, OptionalExtension};
use serenity::all::{Timestamp, UserId};

use crate::model::{CharacterLogEntry, CharacterStatistics};

pub trait CharacterStatisticsRepository {
    async fn add_log_entry(
        &mut self,
        user_id: UserId,
        characters: i32,
        time: &Timestamp,
        notes: Option<String>,
    ) -> Result<CharacterStatistics, String>;

    async fn get_statistics(
        &mut self,
        user_id: UserId,
    ) -> Result<Option<CharacterStatistics>, String>;

    /// Returns a list of 15 users, sorted by the amount of characters logged descendingly.
    async fn fetch_paginated_users_by_characters(
        &mut self,
        page_number: usize,
    ) -> Result<Vec<CharacterStatistics>, String>;

    async fn get_log_entries(&mut self, user_id: UserId) -> Result<Vec<CharacterLogEntry>, String>;
}

pub struct SQLiteCharacterStatisticsRepository {
    connection: Connection,
}

impl SQLiteCharacterStatisticsRepository {
    pub fn new(connection: Connection) -> SQLiteCharacterStatisticsRepository {
        SQLiteCharacterStatisticsRepository { connection }
    }

    fn initialize_statistics(&mut self, user_id: UserId) -> Result<CharacterStatistics, String> {
        let id = user_id.get();
        self.connection
            .execute(
                "
        INSERT INTO CharacterStatistics (user_id, total_characters)
        VALUES (?1, ?2)
        ",
                (id, 0),
            )
            .map_err(|e| e.to_string())?;
        Ok(CharacterStatistics::with_total_characters(user_id, 0))
    }
}

impl CharacterStatisticsRepository for SQLiteCharacterStatisticsRepository {
    async fn add_log_entry(
        &mut self,
        user_id: UserId,
        characters: i32,
        time: &Timestamp,
        notes: Option<String>,
    ) -> Result<CharacterStatistics, String> {
        let id = user_id.get();
        let old_statistics = self
            .get_statistics(user_id)
            .await
            .map_err(|e| e.to_string())?;

        let tx = self.connection.transaction();

        let transaction = match tx {
            Err(msg) => return Err(msg.to_string()),
            Ok(tx) => tx,
        };

        transaction
            .execute(
                "
            INSERT INTO CharacterLogEntry (user_id, characters, time, notes)
            VALUES (?1, ?2, ?3, ?4);
            ",
                (id, characters, time.unix_timestamp(), notes),
            )
            .map_err(|e| e.to_string())?;

        let mut new_statistics = match old_statistics {
            None => CharacterStatistics::new(user_id),
            Some(stats) => stats,
        };

        new_statistics.total_characters += characters;

        // INSERT IF NOT EXISTS, UPDATE IF EXISTS
        transaction
            .execute(
                "
    UPDATE CharacterStatistics 
    SET total_characters = ?1
    WHERE user_id = ?2;
        ",
                (new_statistics.total_characters, id),
            )
            .map_err(|e| e.to_string())?;

        transaction.commit().map_err(|e| e.to_string())?;

        Ok(new_statistics)
    }

    async fn fetch_paginated_users_by_characters(
        &mut self,
        page_number: usize,
    ) -> Result<Vec<CharacterStatistics>, String> {
        const PAGE_SIZE: usize = 15;
        let offset = page_number * PAGE_SIZE;

        let mut stmt = self
            .connection
            .prepare(
                "
                SELECT user_id, total_characters
                FROM CharacterStatistics
                ORDER BY total_characters DESC
                LIMIT ?1 OFFSET ?2;
                ",
            )
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map([PAGE_SIZE as i64, offset as i64], |row| {
                let user_id: u64 = row.get(0)?;
                let total_characters: i32 = row.get(1)?;
                Ok(CharacterStatistics::with_total_characters(
                    UserId::from(user_id),
                    total_characters,
                ))
            })
            .map_err(|e| e.to_string())?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| e.to_string())?);
        }

        Ok(result)
    }

    async fn get_log_entries(&mut self, user_id: UserId) -> Result<Vec<CharacterLogEntry>, String> {
        let id = user_id.get();

        let mut stmt = self
            .connection
            .prepare(
                "
                SELECT id, user_id, characters, time, notes
                FROM CharacterLogEntry
                WHERE user_id = ?1
                ORDER BY time DESC;
                ",
            )
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map([id], |row| {
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
            })
            .map_err(|e| e.to_string())?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| e.to_string())?);
        }

        Ok(result)
    }

    async fn get_statistics(
        &mut self,
        user_id: UserId,
    ) -> Result<Option<CharacterStatistics>, String> {
        let id = user_id.get();
        let characters = self
            .connection
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
            .optional()
            .map_err(|e| e.to_string())?;

        let characters = match characters {
            Some(c) => c,
            None => self.initialize_statistics(user_id)?.total_characters,
        };

        Ok(Some(CharacterStatistics::with_total_characters(
            user_id, characters,
        )))
    }
}
