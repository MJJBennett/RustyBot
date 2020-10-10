use std::fs::File;
use std::io::Read;
use std::collections::{HashMap};
use std::path::Path;
use serde::{Serialize, Deserialize};

/* Player data storage
 *
 * All player data is stored in a hashmap of <Player Name, Player Data>.
 * This data is saved in a file, and should be intermittently saved to the file.
 * One question could be - why would there be Serde defaults? It's not like we'll
 * be manually editing the JSON file. Definitely something to think about.
 *
 * Future:
 *   - It may be nice to hash usernames (with an near-zero collision algorithm)
 *      This would allow the stored data to never link usernames to any kind of data,
 *      although it's true that there is no personal data stored here.
 *      In this case, player names would be loaded on the fly when matched
 */

fn default_cash() -> i64 { 1000 }
fn get_zero() -> i64 { 0 }

#[derive(Debug, Serialize, Deserialize)]
pub struct Player {
    pub name: String,
    #[serde(default = "default_cash")]
    pub cash: i64,
    #[serde(default = "get_zero")]
    pub wins: i64,
    #[serde(default = "get_zero")]
    pub losses: i64,
}

impl Player {
    pub fn new(name: String) -> Player {
        Player {
            name: name,
            cash: default_cash(),
            wins: 0,
            losses: 0,
        }
    }
}

pub fn save_players(val: &HashMap<String, Player>, path: &Path) -> bool {
    let file = match match path.exists() {
        true => std::fs::OpenOptions::new().write(true).truncate(true).open(path),
        false => File::create(path)
    } {
        Ok(file) => file,
        Err(e) => {
            println!("Could not open or create file! {}", e);
            return false;
        }
    };
    match serde_json::to_writer_pretty(file, val) {
        Ok(_) => true,
        Err(e) => {
            println!("Couldn't save players: {}", e);
            false
        }
    }
}

pub fn get_players(path: &Path) -> HashMap<String, Player> {
    if !path.exists() { return HashMap::new(); }
    let mut f = File::open(path).expect("Could not open player file.");
    let mut contents = String::new();
    f.read_to_string(&mut contents).expect("Could not read player file to string.");
    serde_json::from_str(&contents).unwrap()
}

#[cfg(test)]
mod player_data_tests {
    use super::*;

    #[test]
    fn test_new_file() {
        let hm = std::collections::HashMap::new();
        let path = std::path::Path::new("test_player_data1.json");
        assert!(save_players(&hm, path));
    }

    #[test]
    fn test_load() {
        let mut hm = std::collections::HashMap::new();
        let path = std::path::Path::new("test_player_data2.json");

        let player = super::new(String::from("mjb"));
        hm.insert(player.name.clone(), player);

        assert!(save_players(&hm, path)); 

        let res = get_players(path);
        assert!(res.contains_key("mjb"));
    }
}
