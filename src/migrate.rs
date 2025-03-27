use std::{error::Error, fs::File, io::BufReader};

use chrono::Utc;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::repository::{CharacterStatisticsRepository, SQLiteCharacterStatisticsRepository};

#[derive(Serialize, Deserialize, Debug)]
pub struct OldCharacterLog {
    characters: i32,
    #[serde(alias = "userID")]
    user_id: u64,
}

pub fn get_json_data(path: &str) -> Result<Vec<OldCharacterLog>, Box<dyn Error + Send + Sync>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let old_data: Vec<OldCharacterLog> = serde_json::from_reader(reader)?;
    Ok(old_data)
}

pub fn migrate(
    connection: &mut Connection,
    old_data: Vec<OldCharacterLog>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let tx = connection.transaction()?;
    let mut repo = SQLiteCharacterStatisticsRepository::new(&tx);

    for data in old_data.iter() {
        repo.add_log_entry(
            data.user_id,
            data.characters,
            &Utc::now(),
            Some("Migrate from previous bot".to_owned()),
        )?;
    }
    tx.commit()?;
    Ok(())
}
