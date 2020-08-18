use std::fmt;
use std::fs::File;
use std::io::Read;
use std::collections::{HashMap, HashSet};
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
 * haha yes this is very incomplete you're correct!
 */

// lol
fn get_true_lol() -> bool { true }
fn get_false_lol() -> bool { false }
fn default_ver() -> String { "0.0.0".to_string() }

#[derive(Debug, Serialize, Deserialize)]
pub enum CmdValue {
    // We support string responses - eg "sayhi" => "hi!"
    StringResponse(String),
    // Aliases are automatically parsed and translated
    Alias(String),
    // Otherwise, it requires code implementation
    Generic(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandNode {
    value: CmdValue,
    #[serde(default = "get_false_lol")]
    admin_only: bool,
    #[serde(default = "HashMap::new")]
    subcommands: HashMap<String, CommandNode>,
    #[serde(default = "get_false_lol")]
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
    #[serde(default = "default_ver")]
    version: String,
    #[serde(default = "HashMap::new")]
    commands: HashMap<String, CommandNode>,
}

impl CommandTree {
    pub fn find(&self, key: &String) -> Option<&CommandNode> {
        match self.commands.get(key) {
            Some(node) => match &node.value {
                CmdValue::Alias(k) => {
                    let mut h = HashSet::new();
                    h.insert(key);
                    self.find_recurse(&k, h)
                },
                _ => Some(node)
            },
            None => None
        }
    }

    pub fn find_recurse(&self, key: &String, mut prev: HashSet<&String>) -> Option<&CommandNode> {
        match self.commands.get(key) {
            Some(node) => match &node.value {
                CmdValue::Alias(k) => {
                    if prev.insert(k) { self.find_recurse(&k, prev) }
                    else { None }
                }
                _ => Some(node)
            },
            None => None
        }
    }

    pub fn validate(ct: &CommandTree) -> bool {
        for (key, value) in &ct.commands {
            for c in key.chars() {
                if c.is_uppercase() {
                    return false
                }
            }
        }
        true
    }

    pub fn from_json_file(filename: &Path) -> CommandTree {
        let mut file = File::open(filename).expect(&format!("Could not open file: {}", filename.display()));
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect(&format!("Could not read file {} to string.", filename.display()));
        CommandTree::from_json(serde_json::from_str(&contents).unwrap())
    }

    pub fn from_json(json: serde_json::Value) -> CommandTree {
        serde_json::from_value(json).unwrap()
    }

    pub fn dump_file(&self, path: &Path) {
        serde_json::to_writer_pretty(&File::create(path).unwrap(), &self).unwrap()
    }

    pub fn setup_new(path: &Path) -> CommandTree {
        match path.exists() {
            // yes, it's a race condition
            // only overwrites though. not a big deal.
            true => panic!("Cannot setup new command tree; path already exists!"),
            false => {
                let mut ct = CommandTree { commands: HashMap::new(), version: default_ver() };
                ct.commands.insert("json".to_string(), 
                                   CommandNode::new_easter(
                                       CmdValue::StringResponse("The truth is alterable. The truth never has been altered. JSON is the best data format. JSON has always been the best data format.".to_string())));

                serde_json::to_writer_pretty(&File::create(path).unwrap(), &ct).unwrap();

                return ct;
            }
        }
    }
}
