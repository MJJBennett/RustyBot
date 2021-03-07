use std::fs::File;
use std::io::Read;
use std::collections::HashMap;
pub use crate::command_tree::*;

enum Resp1Permissions {
    Everyone,
    Operator,
    Names(vec<String>),
}

struct Response {
    hidden: bool,
    originalString: String,
    stringResp: Option<String>,
    name: String,
    permissions: Resp1Permissions
}

impl Response {
    pub fn from(itr: &impl BufRead) -> Result<Response, String>
    {

    }
}

struct Resp1Env {
    command_character: &str,
    commands_file: Option<&str>,
}

impl Resp1Env {
    pub fn new() -> Resp1Env {
        Resp1Env {
            command_character: "!",
            commands_file: None,
        }
    }
}

struct Resp1File {
    env: Resp1Env,
    resps: HashMap<String, Response>,
}

trait Resp1Serializable<T> {
    pub fn from_resp1(path: &Path) -> T;
}

impl Resp1Serializable for CommandTree {
    fn from_resp1(path: &Path) -> CommandTree {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
    }
}
