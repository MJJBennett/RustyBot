#![recursion_limit="1024"]
use async_std::net::{TcpStream};
use async_std::fs;
use std::time::Duration;
use async_std::task;
use async_std::io::{BufReader, BufWriter, Read};
use regex::Regex;
use async_std::prelude::*;
use async_trait::async_trait;
use std::sync::Arc;
use futures::{select, FutureExt};

trait CaptureExt {
    fn str_at(&self, i: usize) -> String;
}

impl CaptureExt for regex::Captures {
    fn str_at(&self, i: usize) {
        &self.get(i).unwrap().as_str().to_string()
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
        &self.send(format!("PASS {}", pass)).await;
    }

    async fn send_nick(&mut self, nick: &String) {
        &self.send(format!("NICK {}", nick)).await;
    }

    async fn send_join(&mut self, to_join: &String) {
        &self.send(format!("JOIN #{}", to_join)).await;
    }

    async fn send(&mut self, to_send: String) {
        let _ = &self.write(format!("{}\r\n", to_send).as_bytes()).await; 
    }
}

struct EasyReader {
    line: String, 
    // Use direct <TcpStream> for now, could template EasyReader later
    reader: BufReader<TcpStream>
}

struct IRCBotClient {
    stream: TcpStream,
    nick: String,
    secret: String,
    channel: String,
    reader: EasyReader,
}

impl EasyReader {
    fn new(stream: TcpStream) -> EasyReader {
        EasyReader { line: String::new(), reader: BufReader::new(stream) }
    }

    async fn read_line(&mut self) -> std::io::Result<usize> {
        self.line.clear();
        self.reader.read_line(&mut self.line).await
    }
}

impl IRCBotClient {
    async fn connect(nick: String, secret: String, channel: String) -> IRCBotClient {
        // Creates the stream object that will go into the client.
        let stream = TcpStream::connect("irc.chat.twitch.tv:6667").await.unwrap();
        // Get a stream reference to use for reading.
        let reader = EasyReader::new(stream.clone());
        IRCBotClient { 
            stream: stream,
            nick: nick,
            secret: secret,
            channel: channel,
            reader: reader,
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

    async fn send(&mut self, msg: String) -> () {
        self.stream.send(format!("{}", msg));
    }

    async fn do_command(&mut self, user: String, cmd: String) -> () {

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

    let mut client: IRCBotClient = IRCBotClient::connect(nick, secret, channel).await;
    client.authenticate().await;     

    println!("Starting loop.");

    //:desktopfolder!desktopfolder@desktopfolder.tmi.twitch.tv PRIVMSG #desktopfolder :~cmd
    let priv_re = Regex::new(r":(\w*)!\w*@\w*\.tmi\.twitch\.tv PRIVMSG #\w* :\s*(bot |!|~)\s*(.+?)\s*$").unwrap();

    /* Tasks in Rust
     * Spawning tasks puts them into the event loop.
     */

    async fn is_async() {
        println!("Are we async yet?!");
        task::sleep(Duration::from_secs(1)).await;
        println!("Are we async yet?!");
        task::sleep(Duration::from_secs(1)).await;
        println!("Are we async yet?!");
        task::sleep(Duration::from_secs(1)).await;
        println!("Are we async yet?!");
        task::sleep(Duration::from_secs(1)).await;
        println!("Are we async yet?!");
        task::sleep(Duration::from_secs(1)).await;
        println!("Are we async yet?!");
        task::sleep(Duration::from_secs(1)).await;
        println!("Are we async yet?!");
        task::sleep(Duration::from_secs(1)).await;
        println!("Are we async yet?!");
        task::sleep(Duration::from_secs(1)).await;
        println!("Are we async yet?!");
        task::sleep(Duration::from_secs(1)).await;
        println!("Are we async yet?!");
        task::sleep(Duration::from_secs(1)).await;
        println!("We're async!");
    };

    task::spawn(is_async());

    select! {
        return_message = client.launch_read() => match return_message {
            Ok(message) => { println!("Quit: {}", message); },
            Err(error) => { println!("Error: {}", error); }
        }
        line = client.reader.read_line().fuse() => match line {
            Ok(line) => {
                let line = &client.reader.line;
                println!("[Received] Message: {}", line.trim());
                let (name, command) = match priv_re.captures(line.as_str())
                {
                    // there must be a better way...
                    Some(caps) => (caps.str_at(1), caps.str_at(3)),
                    None => continue
                };
                println!("[Parsed Command] Name: {} | Command: '{}'", name, command);
                client.do_command(name, command).await;
            },
            Err(e) => {
                println!("Encountered error: {}", e);
                continue;
            }
        }
    }
}

fn main() {
    task::block_on(async_main())
}
