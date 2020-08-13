use std::time::Duration;
use std::io::Result;
use regex::Regex;
use lazy_static::lazy_static;
use async_trait::async_trait;
use futures::{select, FutureExt};
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
}

struct IRCBotMessageSender {
    writer: TcpStream,
    queue: Receiver::<String>,
}

impl IRCBotMessageSender {
    async fn launch_write(&mut self) {
        loop {
            println!("Awaiting writes...");
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
    async fn connect(nick: String, secret: String, channel: String) -> (IRCBotClient, IRCBotMessageSender) {
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
            channel: channel
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
        println!("[Parsed Command] Name: {} | Command: '{}'", user, cmd);
        if user == "desktopfolder" && cmd.starts_with("Stop") { return Command::Stop; }

        // Ideally, load a tree from JSON or similar
        if cmd == "bnb" { 
            self.privmsg("Brand new bot!".to_string()).await; 
            Command::Continue
        }
        else if user == "desktopfolder" {
            self.do_elevated(cmd).await
        }
        else { Command::Continue }
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
                        None => {
                            if let Command::Stop = self.handle_twitch(&line).await { break; }
                            continue;
                        }
                    };
                    if let Command::Stop = self.do_command(name, command).await { break; }
                },
                Err(e) => {
                    println!("Encountered error: {}", e);
                    continue;
                }
            }
        }
        Ok("Finished reading.".to_string())
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

    let (mut client, mut forwarder) = IRCBotClient::connect(nick, secret, channel).await;
    client.authenticate().await;

    select! {
        return_message = client.launch_read().fuse() => match return_message {
            Ok(message) => { println!("Quit (Read): {}", message); },
            Err(error) => { println!("Error (Read): {}", error); }
        },
        () = forwarder.launch_write().fuse() => {}
    }
}

fn main() { task::block_on(async_main()) }
