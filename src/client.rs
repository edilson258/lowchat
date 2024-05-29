use std::io::{self, BufRead, BufReader, Write};
use std::net::TcpStream;
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::server::{Event, State};

pub struct Client {
    stream: TcpStream,
    state: Arc<Mutex<State>>,
    server_channel: Sender<Event>,
}

impl Client {
    pub fn make(
        stream: TcpStream,
        server_channel: Sender<Event>,
        state: Arc<Mutex<State>>,
    ) -> Self {
        Self {
            state,
            stream,
            server_channel,
        }
    }

    pub fn run(&mut self) {
        let mut reader = BufReader::new(self.stream.try_clone().unwrap());
        let name = match self.ask_name(&mut reader) {
            Ok(name) => name,
            Err(err) => {
                eprintln!("[Error]: Couldn't ask name: {}", err.to_string());
                return;
            }
        };
        let (tx, rx) = mpsc::channel::<Event>();
        let conn_event = Event::NewConnection(name.clone(), tx);
        if let Err(err) = self.server_channel.send(conn_event) {
            eprintln!(
                "[Error]: Couldn't send connection event to server: {}",
                err.to_string()
            );
            return;
        }
        let server_channel = self.server_channel.clone();
        thread::spawn(move || {
            let mut buf = String::new();
            while reader.read_line(&mut buf).unwrap() > 0 {
                let msg = format!("[{}] {}", name, buf.trim());
                if let Err(err) = server_channel.send(Event::ChatMessage(msg)) {
                    eprintln!(
                        "[Error]: Couldn't send chat message to server: {}",
                        err.to_string()
                    );
                }
                buf.clear();
            }
            if let Err(err) = server_channel.send(Event::Disconnection(name)) {
                eprintln!(
                    "[Error]: Couldn't send isconnection message to server: {}",
                    err.to_string()
                );
            }
        });
        for msg in rx {
            writeln!(self.stream, "{}", msg).unwrap();
            self.stream.flush().unwrap();
        }
    }

    fn ask_name(&mut self, reader: &mut BufReader<TcpStream>) -> io::Result<String> {
        writeln!(self.stream, "Please enter your name:")?;
        let mut buf = String::new();
        reader.read_line(&mut buf)?;
        let mut name = buf.trim();
        loop {
            match self.is_name_valid(name) {
                Err(err) => {
                    writeln!(self.stream, "[Invalid name]: {}", err)?;
                    writeln!(self.stream, "Please enter your name:")?;
                    buf = String::new();
                    reader.read_line(&mut buf)?;
                    name = buf.trim();
                }
                Ok(()) => return Ok(name.to_string()),
            }
        }
    }

    fn is_name_valid(&mut self, name: &str) -> Result<(), String> {
        if name.len() < 5 {
            return Err(format!("Name too short"));
        }
        if name.len() > 25 {
            return Err(format!("Name too long"));
        }
        if !name.is_ascii() {
            return Err(format!("Name must contain ascii digits only"));
        }
        if self.state.lock().unwrap().has_username(&name) {
            return Err(format!("Name already in use"));
        }
        Ok(())
    }
}
