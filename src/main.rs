use std::time::Duration;
use std::io::Result;
use regex::Regex;
use lazy_static::lazy_static;
use async_trait::async_trait;
use futures::{select, FutureExt};
use std::path::Path;
use scopeguard::guard;
use async_std::{
    // TODO use async_channel instead of unstable+slower
    sync::{
        Receiver,
        Sender
    },
    io::BufReader,
    net::TcpStream,
    prelude::*,
    task,
};

use rustybot::command_tree::{CommandTree, CmdValue};

enum Command {
    Stop,
    Continue,
}

// Actually totally unrelated to the above...
enum CommandType {
    Raw,
    PrivMsg,
}

// Temporary until I find the correct way to do this.
trait CaptureExt {
    fn str_at(&self, i: usize) -> String;
}

impl CaptureExt for regex::Captures<'_> {
    fn str_at(&self, i: usize) -> String {
        self.get(i).unwrap().as_str().to_string()
    }
}

#[async_trait]
trait IRCStream {
    async fn send_pass(&mut self, pass: &String) -> ();
    async fn send_nick(&mut self, nick: &String) -> ();
    async fn send_join(&mut self, join: &String) -> ();
    async fn send(&mut self, to_send: String) -> ();
}

#[async_trait]
impl IRCStream for TcpStream {
    // This trait is probably a little unwieldy, but... so cool!
    async fn send_pass(&mut self, pass: &String) {
        self.send(format!("PASS {}", pass)).await;
    }

    async fn send_nick(&mut self, nick: &String) {
        self.send(format!("NICK {}", nick)).await;
    }

    async fn send_join(&mut self, to_join: &String) {
        self.send(format!("JOIN #{}", to_join)).await;
    }

    async fn send(&mut self, to_send: String) {
        let _ = self.write(format!("{}\r\n", to_send).as_bytes()).await; 
    }
}

struct IRCBotClient {
    stream: TcpStream,
    nick: String,
    secret: String,
    reader: BufReader::<TcpStream>,
    sender: Sender::<String>,
    channel: String,
    ct: CommandTree,
}

struct IRCBotMessageSender {
    writer: TcpStream,
    queue: Receiver::<String>,
}

impl IRCBotMessageSender {
    async fn launch_write(&mut self) {
        loop {
            match self.queue.recv().await {
                Ok(s) => {
                    println!("Sending: '{}'", s);
                    self.writer.send(s).await;
                },
                Err(e) => { 
                    println!("Uh oh, queue receive error: {}", e);
                    break;
                }
            }
            task::sleep(Duration::from_millis(100)).await;
        }
    }
}

impl IRCBotClient {
    async fn connect(nick: String, secret: String, channel: String, ct: CommandTree) -> (IRCBotClient, IRCBotMessageSender) {
        // Creates the stream object that will go into the client.
        let stream = TcpStream::connect("irc.chat.twitch.tv:6667").await.unwrap();
        // Get a stream reference to use for reading.
        let reader = BufReader::new(stream.clone());
        let (s, r) = async_std::sync::channel(10); // 10 is capacity of buffer
        (IRCBotClient { 
            stream: stream.clone(),
            nick: nick,
            secret: secret,
            reader: reader,
            sender: s,
            channel: channel,
            ct: ct
        }, IRCBotMessageSender {
            writer: stream,
            queue: r,
        })
        // return the async class for writing back down the TcpStream instead, which contains the
        // receiver + the tcpstream clone
    }

    fn format_twitch(&self, m: String, t: CommandType) -> String {
        match t {
            CommandType::Raw => m,
            CommandType::PrivMsg => format!("PRIVMSG #{} :{}", self.channel, m),
        }
    }

    async fn authenticate(&mut self) -> () {
        println!("Writing password...");
        self.stream.send_pass(&self.secret).await;
        println!("Writing nickname...");
        self.stream.send_nick(&self.nick).await; 
        println!("Writing join command...");
        self.stream.send_join(&self.channel).await; 
    }

    async fn privmsg(&mut self, m: String) -> () {
        self.sender.send(self.format_twitch(m, CommandType::PrivMsg)).await;
    }

    async fn do_elevated(&mut self, mut cmd: String) -> Command {
        if cmd.starts_with("stop") { Command::Stop }
        else if cmd.starts_with("raw") { self.sender.send(cmd.split_off(4)).await; Command::Continue }
        else if cmd.starts_with("say") { self.privmsg(cmd.split_off(4)).await; Command::Continue }
        else { Command::Continue }
    }

    async fn do_command(&mut self, user: String, cmd: String) -> Command {
        let format_str = format!("[Name({}),Command({})] Result: ", user, cmd);
        let mut result = "Command was invalid, disallowed, or skipped.".to_string();
        let _guard = scopeguard::guard((), |()| {
            println!("{}{}", &format_str, &result);
        });
        let node = match self.ct.find(&cmd) {
            Some(x) => x,
            None => return Command::Continue // Not a valid command
        };
        if node.admin_only && user != "desktopfolder" {
            self.privmsg("Naughty naughty, that's not for you!".to_string()).await;
            return Command::Continue;
        }
        let cmd = match &node.value {
            CmdValue::StringResponse(x) => {
                self.privmsg(x.clone()).await;
                return Command::Continue;
            },
            CmdValue::Alias(x) => {
                println!("Wow, {} is an alias!", x);
                return Command::Continue;
            },
            CmdValue::Generic(x) => x
        };
        match cmd.as_str() {
            "meta:help" => self.privmsg("No help for you, good sir!".to_string()).await,
            "meta:stop" => return Command::Stop,
            _ => return Command::Continue
        }
        Command::Continue
    }

    async fn handle_twitch(&mut self, line: &String) -> Command {
        match line.trim() {
            "" => Command::Stop,
            "PING :tmi.twitch.tv" => {
                self.sender.send("PONG :tmi.twitch.tv".to_string()).await;
                Command::Continue
            }
            _ => Command::Continue,
        }
    }

    async fn launch_read(&mut self) -> Result<String> {
        lazy_static! {
            static ref PRIV_RE: Regex = Regex::new(r":(\w*)!\w*@\w*\.tmi\.twitch\.tv PRIVMSG #\w* :\s*(bot |!|~)\s*(.+?)\s*$").unwrap();
        }
        let mut line = String::new();

        loop {
            line.clear();
            match self.reader.read_line(&mut line).await {
                Ok(_) => {
                    println!("[Received] Message: '{}'", line.trim());
                    let (name, command) = match PRIV_RE.captures(line.as_str())
                    {
                        // there must be a better way...
                        Some(caps) => (caps.str_at(1), caps.str_at(3)),
                        None => match self.handle_twitch(&line).await {
                                Command::Stop => return Ok("Stopped due to twitch.".to_string()),
                                _ => continue
                        }
                    };
                    if let Command::Stop = self.do_command(name, command).await { 
                        return Ok("Received stop command.".to_string()); 
                    }
                },
                Err(e) => {
                    println!("Encountered error: {}", e);
                    continue;
                }
            }
        }
    }
}

fn get_file_trimmed(filename: &str) -> String {
    match std::fs::read_to_string(filename) {
        Ok(s) => s.trim().to_string(),
        Err(e) => panic!("Could not open file ({}):\n{}", filename, e)
    }
}

async fn async_main() {
    let nick = get_file_trimmed("auth/user.txt");
    let secret = get_file_trimmed("auth/secret.txt");
    let channel = get_file_trimmed("auth/id.txt");

    println!("Nick: {} | Secret: {} | Channel: {}", nick, secret, channel);

    // Supported commands, loaded from JSON.
    let ct = CommandTree::from_json_file(Path::new("commands.json"));
    //ct.dump_file(Path::new("commands.parsed.json"));
    let (mut client, mut forwarder) = IRCBotClient::connect(nick, secret, channel, ct).await;
    client.authenticate().await;


    select! {
        return_message = client.launch_read().fuse() => match return_message {
            Ok(message) => { println!("Quit (Read): {}", message); },
            Err(error) => { println!("Error (Read): {}", error); }
        },
        () = forwarder.launch_write().fuse() => {}
    }
}

fn main() {
    task::block_on(async_main()) 
}
