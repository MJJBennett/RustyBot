use std::net::{TcpStream};
use std::io::{Read, Write};
use std::fs;
use std::{thread, time};
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use regex::Regex;

trait IRCStream {
    fn send_pass(&mut self, pass: &String) -> ();
    fn send_nick(&mut self, nick: &String) -> ();
    fn send_join(&mut self, join: &String) -> ();
    fn send(&mut self, to_send: String) -> ();
}

impl IRCStream for TcpStream {
    // This trait is probably a little unwieldy, but... so cool!
    fn send_pass(&mut self, pass: &String) {
        &self.send(format!("PASS {}", pass));
    }

    fn send_nick(&mut self, nick: &String) {
        &self.send(format!("NICK {}", nick));
    }

    fn send_join(&mut self, to_join: &String) {
        &self.send(format!("JOIN #{}", to_join));
    }

    fn send(&mut self, to_send: String) {
        let _ = &self.write(format!("{}\r\n", to_send).as_bytes()); 
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

    fn read_line(&mut self) -> std::io::Result<usize> {
        self.line.clear();
        self.reader.read_line(&mut self.line)
    }
}

impl IRCBotClient {
    fn connect(nick: String, secret: String, channel: String) -> IRCBotClient {
        // Creates the stream object that will go into the client.
        let stream = TcpStream::connect("irc.chat.twitch.tv:6667").unwrap();
        // Get a stream reference to use for reading.
        let reader = EasyReader::new(stream.try_clone().expect("Failed to get a stream reader."));
        IRCBotClient { 
            stream: stream,
            nick: nick,
            secret: secret,
            channel: channel,
            reader: reader,
        }
    }

    fn authenticate(&mut self) -> () {
        println!("Writing password...");
        self.stream.send_pass(&self.secret);
        println!("Writing nickname...");
        self.stream.send_nick(&self.nick); 
        println!("Writing join command...");
        self.stream.send_join(&self.channel); 
    }
}

fn get_file_trimmed(filename: &str) -> String {
    match fs::read_to_string(filename) {
        Ok(s) => s.trim().to_string(),
        Err(e) => panic!("Could not open file ({}):\n{}", filename, e)
    }
}

fn main()
{
    let nick = get_file_trimmed("auth/user.txt");
    let secret = get_file_trimmed("auth/secret.txt");
    let channel = get_file_trimmed("auth/id.txt");

    println!("Nick: {} | Secret: {} | Channel: {}", nick, secret, channel);

    let mut client = IRCBotClient::connect(nick, secret, channel);
    client.authenticate();     

    println!("Starting loop.");

    //:desktopfolder!desktopfolder@desktopfolder.tmi.twitch.tv PRIVMSG #desktopfolder :~cmd
    let priv_re = Regex::new(r":(\w*)!\w*@\w*\.tmi\.twitch\.tv PRIVMSG #\w* :\s*(bot |!|~)\s*(.+?)\s*$").unwrap();

    loop {
        if let Err(e) = client.reader.read_line() {
            println!("Encountered error: {}", e);
            continue;
        }

        let line = &client.reader.line;
        println!("[Received] Message: {}", line.trim());
        let (name, command) = match priv_re.captures(line.as_str())
        {
            Some(caps) => (caps.get(1).unwrap().as_str(), caps.get(3).unwrap().as_str()),
            None => continue
        };
        println!("[Parsed Command] Name: {} | Command: '{}'", name, command);

        let t = time::Duration::from_secs(1);
        thread::sleep(t);
    }
}
