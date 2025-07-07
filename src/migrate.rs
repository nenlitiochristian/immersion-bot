// use std::{error::Error, fs::File, io::BufReader};

// use crate::repository::CharacterStatisticsRepository;
// use chrono::Utc;
// use serde::{Deserialize, Serialize};

// #[derive(Serialize, Deserialize, Debug)]
// pub struct OldCharacterLog {
//     characters: i32,
//     #[serde(alias = "userID")]
//     user_id: u64,
// }

// pub fn get_json_data(path: &str) -> Result<Vec<OldCharacterLog>, Box<dyn Error + Send + Sync>> {
//     let file = File::open(path)?;
//     let reader = BufReader::new(file);
//     let old_data: Vec<OldCharacterLog> = serde_json::from_reader(reader)?;
//     Ok(old_data)
// }

// pub fn migrate(
//     repo: &mut Box<dyn CharacterStatisticsRepository + '_>,
//     old_data: Vec<OldCharacterLog>,
// ) -> Result<(), Box<dyn Error + Send + Sync>> {
//     for data in old_data.iter() {
//         repo.add_log_entry(
//             data.user_id,
//             "Unknown",
//             data.characters,
//             &Utc::now(),
//             Some("Migrate from previous bot".to_owned()),
//         )?;
//     }
//     Ok(())
// }
