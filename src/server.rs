use core::fmt;
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};

pub type Username = String;
pub type Message = String;
pub type ServerChan = Receiver<Event>;
pub type UserChan = Sender<Event>;

#[derive(Debug)]
pub enum Event {
    ChatMessage(Message),
    NewConnection(Username, UserChan),
    Disconnection(Username),
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ChatMessage(msg) => write!(f, "{}", msg),
            Self::NewConnection(name, _) => write!(f, "[{}] is connected", name),
            Self::Disconnection(name) => write!(f, "[{}] disconnected", name),
        }
    }
}

pub struct State {
    clients: HashMap<String, UserChan>,
}

impl State {
    pub fn make() -> Self {
        Self {
            clients: HashMap::new(),
        }
    }

    pub fn has_username(&mut self, name: &str) -> bool {
        self.clients.contains_key(name)
    }
}

pub struct Server {
    channel: ServerChan,
    state: Arc<Mutex<State>>,
}

impl Server {
    pub fn make(channel: ServerChan, state: Arc<Mutex<State>>) -> Self {
        Self { channel, state }
    }

    pub fn run(&mut self) {
        loop {
            let event = match self.channel.recv() {
                Ok(event) => event,
                Err(err) => {
                    eprintln!("[Error]: Couldn't recv event: {}", err.to_string());
                    continue;
                }
            };
            self.handle_event(event);
        }
    }

    fn handle_event(&mut self, event: Event) {
        println!("{}", &event);
        match event {
            Event::ChatMessage(msg) => self.broadcast_message(msg),
            Event::NewConnection(name, chan) => self.handle_connection_event(name, chan),
            Event::Disconnection(name) => self.handle_disconnection_event(name),
        }
    }

    fn broadcast_message(&mut self, msg: String) {
        for (_, chan) in &self.state.lock().unwrap().clients {
            chan.send(Event::ChatMessage(msg.clone())).unwrap();
        }
    }

    fn handle_connection_event(&mut self, name: String, chan: UserChan) {
        self.state
            .lock()
            .unwrap()
            .clients
            .insert(name.clone(), chan);
        self.broadcast_message(format!("[{}] is online", name));
    }

    fn handle_disconnection_event(&mut self, name: String) {
        self.state.lock().unwrap().clients.remove(&name);
        self.broadcast_message(format!("[{}] is offline", name));
    }
}
