use crate::{
    constants::{LEADERBOARD_PAGE_SIZE, LOG_ENTRY_PAGE_SIZE},
    model::{CharacterLogEntry, CharacterStatistics},
    Error,
};
use chrono::{TimeZone, Utc};
use serenity::all::Timestamp;
use sqlx::{Postgres, Transaction};

use super::{CharacterStatisticsRepository, MetadataRepository};

pub struct PostgresMetadataRepository {}

impl PostgresMetadataRepository {
    pub fn new() -> Self {
        Self {}
    }
}

impl<'tx> MetadataRepository<'tx> for PostgresMetadataRepository {
    async fn get_last_active_status_refresh(
        &mut self,

        tx: &mut Transaction<'tx, Postgres>,
    ) -> Result<Option<chrono::DateTime<chrono::Utc>>, Error> {
        let row: Option<(i64,)> = sqlx::query_as(
            r#"SELECT last_active_status_refresh FROM immersion_bot."Metadata" LIMIT 1"#,
        )
        .fetch_optional(&mut **tx)
        .await?;

        Ok(row.map(|(ts,)| Utc.timestamp_opt(ts, 0).unwrap()))
    }

    async fn set_last_active_status_refresh(
        &mut self,

        tx: &mut Transaction<'tx, Postgres>,
        time: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), Error> {
        let affected = sqlx::query!(
            r#"UPDATE immersion_bot."Metadata" SET last_active_status_refresh = $1"#,
            time.timestamp()
        )
        .execute(&mut **tx)
        .await?
        .rows_affected();

        if affected == 0 {
            sqlx::query!(
                r#"INSERT INTO immersion_bot."Metadata" (last_active_status_refresh) VALUES ($1)"#,
                time.timestamp()
            )
            .execute(&mut **tx)
            .await?;
        }

        Ok(())
    }
}

pub struct PostgresCharacterStatisticsRepository {}

impl<'tx> PostgresCharacterStatisticsRepository {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn initialize_statistics(
        &mut self,

        tx: &mut Transaction<'tx, Postgres>,
        user_id: u64,
        name: &str,
    ) -> Result<CharacterStatistics, Error> {
        let uid = user_id as i64;

        sqlx::query!(
            r#"
            INSERT INTO immersion_bot."CharacterStatistics" (user_id, total_characters, name)
            VALUES ($1, $2, $3)
            "#,
            uid,
            0,
            name
        )
        .execute(&mut **tx)
        .await?;

        Ok(CharacterStatistics::new(user_id, 0, name.to_owned()))
    }
}

impl<'tx> CharacterStatisticsRepository<'tx> for PostgresCharacterStatisticsRepository {
    async fn add_log_entry(
        &mut self,
        tx: &mut Transaction<'tx, Postgres>,
        user_id: u64,
        name: &str,
        characters: i32,
        time: i64,
        notes: Option<String>,
    ) -> Result<CharacterStatistics, Error> {
        let old = self.get_or_initialize_statistics(tx, user_id, name).await?;
        let characters = if characters >= 0 {
            characters
        } else {
            characters.clamp(-old.total_characters, 0)
        };

        if characters != 0 || notes.as_ref().map_or(false, |n| !n.trim().is_empty()) {
            sqlx::query!(
                r#"INSERT INTO immersion_bot."CharacterLogEntry" (user_id, characters, time, notes)
                 VALUES ($1, $2, $3, $4)"#,
                user_id as i64,
                characters,
                time,
                notes
            )
            .execute(&mut **tx)
            .await?;
        }

        let new_total = old.total_characters + characters;

        sqlx::query!(
            r#"UPDATE immersion_bot."CharacterStatistics" SET total_characters = $1, name = $2 WHERE user_id = $3"#,
            new_total,
            name,
            user_id as i64
        )
        .execute(&mut **tx)
        .await?;

        Ok(CharacterStatistics::new(
            user_id,
            new_total,
            name.to_string(),
        ))
    }

    async fn exists(
        &mut self,

        tx: &mut Transaction<'tx, Postgres>,
        user_id: u64,
    ) -> Result<bool, Error> {
        let res = sqlx::query_scalar!(
            r#"SELECT 1 FROM immersion_bot."CharacterStatistics" WHERE user_id = $1"#,
            user_id as i64
        )
        .fetch_optional(&mut **tx)
        .await?;
        Ok(res.is_some())
    }

    async fn get_or_initialize_statistics(
        &mut self,

        tx: &mut Transaction<'tx, Postgres>,
        user_id: u64,
        name: &str,
    ) -> Result<CharacterStatistics, Error> {
        let row = sqlx::query!(
            r#"SELECT total_characters FROM immersion_bot."CharacterStatistics" WHERE user_id = $1"#,
            user_id as i64
        )
        .fetch_optional(&mut **tx)
        .await?;

        if let Some(r) = row {
            Ok(CharacterStatistics::new(
                user_id,
                r.total_characters,
                name.to_string(),
            ))
        } else {
            self.initialize_statistics(tx, user_id, name).await
        }
    }

    async fn get_paginated_active_users_by_characters(
        &mut self,

        tx: &mut Transaction<'tx, Postgres>,
        page_number: u64,
    ) -> Result<Vec<CharacterStatistics>, Error> {
        let offset = (page_number * LEADERBOARD_PAGE_SIZE) as i64;
        let rows = sqlx::query!(
            r#"SELECT user_id, total_characters, name
             FROM immersion_bot."CharacterStatistics"
             WHERE is_active = TRUE AND total_characters > 0
             ORDER BY total_characters DESC, user_id ASC
             LIMIT $1 OFFSET $2"#,
            LEADERBOARD_PAGE_SIZE as i64,
            offset
        )
        .fetch_all(&mut **tx)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| CharacterStatistics::new(r.user_id as u64, r.total_characters, r.name))
            .collect())
    }

    async fn get_paginated_log_entries_by_time(
        &mut self,

        tx: &mut Transaction<'tx, Postgres>,
        user_id: u64,
        page_number: u64,
    ) -> Result<Vec<CharacterLogEntry>, Error> {
        let offset = (page_number * LOG_ENTRY_PAGE_SIZE) as i64;
        let rows = sqlx::query!(
            r#"SELECT id, user_id, characters, time, notes
             FROM immersion_bot."CharacterLogEntry"
             WHERE user_id = $1
             ORDER BY time DESC
             LIMIT $2 OFFSET $3"#,
            user_id as i64,
            LOG_ENTRY_PAGE_SIZE as i64,
            offset
        )
        .fetch_all(&mut **tx)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| {
                CharacterLogEntry::new(
                    r.user_id as u64,
                    r.characters,
                    &Timestamp::from_unix_timestamp(r.time).expect("Invalid timestamp"),
                    r.notes,
                )
            })
            .collect())
    }

    async fn get_paginated_users_by_id(
        &mut self,

        tx: &mut Transaction<'tx, Postgres>,
        page_number: u64,
    ) -> Result<Vec<CharacterStatistics>, Error> {
        let offset = (page_number * LEADERBOARD_PAGE_SIZE) as i64;
        let rows = sqlx::query!(
            r#"SELECT user_id, total_characters, name
             FROM immersion_bot."CharacterStatistics"
             ORDER BY user_id ASC
             LIMIT $1 OFFSET $2"#,
            LEADERBOARD_PAGE_SIZE as i64,
            offset
        )
        .fetch_all(&mut **tx)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| CharacterStatistics::new(r.user_id as u64, r.total_characters, r.name))
            .collect())
    }

    async fn get_rank(
        &mut self,

        tx: &mut Transaction<'tx, Postgres>,
        statistics: &CharacterStatistics,
    ) -> Result<i32, Error> {
        let row = sqlx::query_scalar!(
            r#"
            SELECT rank FROM (
                SELECT user_id,
                       RANK() OVER (ORDER BY total_characters DESC, user_id ASC) as rank
                FROM immersion_bot."CharacterStatistics"
                WHERE is_active = TRUE AND total_characters > 0
            ) ranked
            WHERE user_id = $1
            "#,
            statistics.get_user_id() as i64
        )
        .fetch_one(&mut **tx)
        .await?
        .unwrap();

        Ok(row as i32)
    }

    async fn get_total_active_users(
        &mut self,
        tx: &mut Transaction<'tx, Postgres>,
    ) -> Result<u64, Error> {
        let count = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM immersion_bot."CharacterStatistics" WHERE is_active = TRUE AND total_characters > 0"#
        )
        .fetch_one(&mut **tx)
        .await?;

        Ok(count.unwrap_or(0) as u64)
    }

    async fn get_total_log_entries(
        &mut self,

        tx: &mut Transaction<'tx, Postgres>,
        user_id: u64,
    ) -> Result<u64, Error> {
        let count = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM immersion_bot."CharacterLogEntry" WHERE user_id = $1"#,
            user_id as i64
        )
        .fetch_one(&mut **tx)
        .await?;

        Ok(count.unwrap_or(0) as u64)
    }

    async fn set_active_status(
        &mut self,

        tx: &mut Transaction<'tx, Postgres>,
        user_id: u64,
        active: bool,
        latest_name: Option<&str>,
    ) -> Result<(), Error> {
        if let Some(name) = latest_name {
            sqlx::query!(
                r#"UPDATE immersion_bot."CharacterStatistics" SET is_active = $1, name = $2 WHERE user_id = $3"#,
                active,
                name,
                user_id as i64
            )
            .execute(&mut **tx)
            .await?;
        } else {
            sqlx::query!(
                r#"UPDATE immersion_bot."CharacterStatistics" SET is_active = $1 WHERE user_id = $2"#,
                active,
                user_id as i64
            )
            .execute(&mut **tx)
            .await?;
        }

        Ok(())
    }
}
