use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

use crate::player_data::*;

use lazy_static::lazy_static;

lazy_static! {
    static ref PLAYER_PATH: &'static Path = Path::new("players.json");
    static ref GAME_DUMP_PATH: &'static Path = Path::new("gamedump.json");
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Game {
    players: HashMap<String, Player>,
    wagers: HashMap<String, i64>,
}

impl Game {
    pub fn new() -> Game {
        Game {
            players: get_players(&PLAYER_PATH),
            wagers: HashMap::new(),
        }
    }

    pub fn summarize(p: &Player) -> String {
        if p.losses == 0 {
            if p.wins == 0 {
                format!(
                    "Player {} has {} points and has never played </3",
                    p.name, p.cash
                )
            } else {
                format!(
                    "Player {} has {} points and a 100% winrate!",
                    p.name, p.cash
                )
            }
        } else if p.wins == 0 {
            format!(
                "Player {} has {} points and a 0% winrate :(",
                p.name, p.cash
            )
        } else {
            format!(
                "Player {} has {} points and a {:.2}% winrate.",
                p.name,
                p.cash,
                p.wins as f64 * 100 as f64 / (p.wins as f64 + p.losses as f64)
            )
        }
    }

    pub fn status(&self, name: &String) -> String {
        match self.players.get(name) {
            Some(p) => Game::summarize(p),
            None => format!("The player '{}' does not exist; place a wager to join!", name),
        }
    }

    pub fn save(&self) -> bool {
        save_players(&self.players, &PLAYER_PATH)
    }

    pub fn reload(&mut self) {
        self.players = get_players(&PLAYER_PATH);
    }

    pub fn valid_wager(&mut self, wager: &String, user: &String) -> Result<i64, String> {
        // Is it a valid number?
        if let Ok(w) = wager.parse::<i64>() {
            // Is it greater than 4?
            if w < 5 {
                return Err(String::from(
                    "Your wager is too small! (Wagers must be 5 or greater!)",
                ));
            }
            // Is it a valid player?
            let player = self.players.entry(user.clone()).or_insert(Player::new(user.clone()));
            if player.cash < w {
                return Err(format!(
                    "The player '{}' has insufficient funds to make that bet! ({})",
                    user, w
                ));
            }
            // Does the player already have a wager?
            match self.wagers.get(user) {
                Some(i) => Err(format!("The player '{}' has already wagered {}!", user, i)),
                None => Ok(w),
            }
        } else {
            Err(String::from("Your wager needs to be a valid integer!"))
        }
    }

    fn make_bet(&mut self, amount: i64, user: &String) {
        // This function is only called if the wager is valid.
        // Could use typesafety to ensure that, but it doesn't prevent
        // bad use, so this function is private.
        self.players.get_mut(user).unwrap().cash -= amount.abs();
        self.wagers.insert(user.clone(), amount);
    }

    pub fn bet_for(&mut self, user: &String, amount: &String) -> Result<(), String> {
        match self.valid_wager(amount, user) {
            Ok(i) => Ok(self.make_bet(i, user)),
            Err(e) => Err(e),
        }
    }

    pub fn bet_against(&mut self, user: &String, amount: &String) -> Result<(), String> {
        match self.valid_wager(amount, user) {
            Ok(i) => Ok(self.make_bet(-1 * i, user)),
            Err(e) => Err(e),
        }
    }

    pub fn worked(&mut self) -> String {
        // code repetition here & below is sorta bad
        // but that's life
        let mut num_wins: u32 = 0;
        let mut amount_won: i64 = 0;
        let mut num_losses: u32 = 0;
        let mut amount_lost: i64 = 0;
        for (user, wager) in &self.wagers {
            match *wager {
                i if i < 0 => {
                    num_losses += 1;
                    amount_lost -= wager; // beautiful
                    match self.players.get_mut(&*user) {
                        Some(p) => {
                            p.losses += 1;
                        }
                        None => println!("Odd, player {} no longer exists.", user),
                    }
                }
                i if i > 0 => {
                    num_wins += 1;
                    amount_won += wager * 2;
                    match self.players.get_mut(&*user) {
                        Some(p) => {
                            p.cash += wager * 2;
                            p.wins += 1;
                        }
                        None => println!("Odd, player {} no longer exists.", user),
                    }
                }
                _ => println!("Odd, wager for user {} was 0.", user),
            }
        }
        self.wagers.clear();
        if num_wins + num_losses == 0 {
            String::from("Nice work, but nobody was playing...")
        } else if num_wins == 0 {
            format!(
                "Ouch, {} player(s) lost {} points... Ye of little faith!",
                num_losses, amount_lost
            )
        } else if num_losses == 0 {
            format!(
                "Wow! {} player(s) won {} points. Making it easy, eh?",
                num_wins, amount_won
            )
        } else {
            format!(
                "{} player(s) won {} points, while {} player(s) lost {} points!",
                num_wins, amount_won, num_losses, amount_lost
            )
        }
    }

    pub fn failed(&mut self) -> String {
        let mut num_wins: u32 = 0;
        let mut amount_won: i64 = 0;
        let mut num_losses: u32 = 0;
        let mut amount_lost: i64 = 0;
        for (user, wager) in &self.wagers {
            match *wager {
                i if i > 0 => {
                    num_losses += 1;
                    amount_lost += wager; // beautiful
                    match self.players.get_mut(&*user) {
                        Some(p) => {
                            p.losses += 1;
                        }
                        None => println!("Odd, player {} no longer exists.", user),
                    }
                }
                i if i < 0 => {
                    num_wins += 1;
                    amount_won -= wager * 2;
                    match self.players.get_mut(&*user) {
                        Some(p) => {
                            p.cash += wager * -2;
                            p.wins += 1;
                        }
                        None => println!("Odd, player {} no longer exists.", user),
                    }
                }
                _ => println!("Odd, wager for user {} was 0.", user),
            }
        }
        self.wagers.clear();
        if num_wins + num_losses == 0 {
            String::from("You're only hurting yourself...")
        } else if num_wins == 0 {
            format!(
                "Ouch, {} player(s) lost {} points... you've been failed :(",
                num_losses, amount_lost
            )
        } else if num_losses == 0 {
            format!(
                "{} player(s) won {} points. That's ... unfortunate.",
                num_wins, amount_won
            )
        } else {
            format!(
                "{} player(s) won {} points, while {} player(s) lost {} points.",
                num_wins, amount_won, num_losses, amount_lost
            )
        }
    }
}

impl Drop for Game {
    fn drop(&mut self) {
        let file = match match GAME_DUMP_PATH.exists() {
            true => std::fs::OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(&*GAME_DUMP_PATH),
            false => File::create(&*GAME_DUMP_PATH),
        } {
            Ok(file) => file,
            Err(e) => {
                println!("[ERROR] Could not open or create file! {}", e);
                println!(
                    "Emergency dump: {}",
                    serde_json::to_string_pretty(&self).unwrap()
                );
                return;
            }
        };
        match serde_json::to_writer_pretty(file, &self) {
            Ok(_) => {}
            Err(e) => {
                println!("[ERROR] Couldn't save players: {}", e);
            }
        }
    }
}
