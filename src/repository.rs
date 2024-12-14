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
    ) -> Result<Vec<(UserId, CharacterStatistics)>, String>;

    async fn get_log_entries(&mut self, user_id: UserId) -> Result<Vec<CharacterLogEntry>, String>;
}

pub struct SQLiteCharacterStatisticsRepository {
    connection: Connection,
}

impl SQLiteCharacterStatisticsRepository {
    pub fn new(connection: Connection) -> SQLiteCharacterStatisticsRepository {
        SQLiteCharacterStatisticsRepository { connection }
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

        transaction.execute(
            "
            INSERT INTO CharacterLogEntry (user_id, characters, time, notes)
            VALUES (?1, ?2, ?3, ?4);
            ",
            (id, characters, time.unix_timestamp(), notes),
        );

        let mut new_statistics = match old_statistics {
            None => CharacterStatistics::new(user_id),
            Some(stats) => stats,
        };

        new_statistics.total_characters += characters;

        // INSERT IF NOT EXISTS, UPDATE IF EXISTS
        transaction.execute(
            "
    INSERT INTO CharacterStatistics (user_id, total_characters)
    VALUES (?1, ?2)
    ON CONFLICT(user_id) DO UPDATE SET
    total_characters = total_characters + excluded.total_characters;
        ",
            (new_statistics.total_characters, id),
        );

        transaction.commit().map_err(|e| e.to_string())?;

        Ok(new_statistics)
    }

    async fn fetch_paginated_users_by_characters(
        &mut self,
        page_number: usize,
    ) -> Result<Vec<(UserId, CharacterStatistics)>, String> {
        todo!()
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
            None => return Ok(None),
        };

        Ok(Some(CharacterStatistics::with_total_characters(
            user_id, characters,
        )))
    }
}
