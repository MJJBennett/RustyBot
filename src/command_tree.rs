use std::fmt;
use std::fs::File;
use std::io::Read;
use std::collections::HashMap;
use std::path::Path;
use serde_json::*;
use serde::{Serialize, Deserialize};

/* CommandTree - A (strange) tree implementation.
 *
 * Goals of this data structure:
 *  - Lookup time should be more or less as fast as is reasonable
 *  - Lookups should be case insensitive (lazily for now)
 *  - Insertion and deletion time are nearly irrelevant
 *  - Lookup should return partial matches if there is no part of the search string that is 'wrong'
 *
 *  16/08 - Unlikely to follow through with most of this. Will probably use a KV store instead.
 *  Then can use trees for subcommands. Not a big deal to be missing autocomplete, & can still have
 *  case insensitivity (as lazy as it would have been before).
 *  
 *  JSON should look like:
 *
 *  {
 *      "command": { "subcommands": { "x": { "value": CommandValue } }
 */

#[derive(Debug, Serialize, Deserialize)]
pub enum CmdValue {
    // We support string responses - eg "sayhi" => "hi!"
    StringResponse(String),
    // Otherwise, it requires code implementation
    // I will do the translation in this file, although generally
    // I think the translation is too codebase-specific and this is 
    // just a library and not core implementation.
    Bet(String), // Contains 
    Alias(String),
    Shoutout,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandNode {
    value: CmdValue,
    admin_only: bool,
    subcommands: HashMap<String, CommandNode>,
    hidden: bool,
}

impl CommandNode {
    pub fn new(value: CmdValue) -> CommandNode {
        CommandNode { value: value, admin_only: false, subcommands: HashMap::new(), hidden: false }
    }

    pub fn new_easter(value: CmdValue) -> CommandNode {
        CommandNode { value: value, admin_only: false, subcommands: HashMap::new(), hidden: true }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandTree {
    commands: HashMap<String, CommandNode>,
}

impl CommandTree {
    pub fn from_json_file(filename: &Path) -> CommandTree {
        let mut file = File::open(filename).expect(&format!("Could not open file: {}", filename.display()));
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect(&format!("Could not read file {} to string.", filename.display()));
        CommandTree::from_json(serde_json::from_str(&contents).unwrap())
    }

    pub fn from_json(json: serde_json::Value) -> CommandTree {
        serde_json::from_value(json).unwrap()
    }

    pub fn setup_new(path: &Path) -> CommandTree {
        match path.exists() {
            // yes, it's a race condition
            // only overwrites though. not a big deal.
            true => panic!("Cannot setup new command tree; path already exists!"),
            false => {
                let mut ct = CommandTree { commands: HashMap::new() };
                ct.commands.insert("json".to_string(), 
                                   CommandNode::new_easter(
                                       CmdValue::StringResponse("The truth is alterable. The truth never has been altered. JSON is the best data format. JSON has always been the best data format.".to_string())));


                serde_json::to_writer_pretty(&File::create(path).unwrap(), &ct).unwrap();               

                return ct;
            }
        }
    }
}
