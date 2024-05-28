use std::collections::HashMap;
use std::io::{BufRead, BufReader, Error, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self};

struct Session {
    peers: HashMap<String, Sender<String>>,
}

impl Session {
    fn make() -> Self {
        let session = Session {
            peers: HashMap::new(),
        };
        session
    }

    fn broadcast(&self, msg: &str) {
        for (name, chan) in &self.peers {
            if let Err(err) = chan.send(msg.to_string()) {
                eprintln!("[Error]: Couldn't broadcast to {}: {:?}", name, err);
            }
        }
    }
}

struct Client {
    name: String,
    stream: TcpStream,
    session: Arc<Mutex<Session>>,
}

impl Client {
    fn make(stream: TcpStream, session: Arc<Mutex<Session>>) -> Self {
        let client = Client {
            name: String::new(),
            stream,
            session,
        };

        client
    }

    fn main_loop(&mut self) -> std::io::Result<()> {
        self.ask_name()?;

        let (tx, rx) = mpsc::channel();

        // Notify everbody that a new client has joined!
        {
            println!("[INFO]: {} is connected", &self.name);
            let mut session = self.session.lock().unwrap();
            session.peers.insert(self.name.clone(), tx);
            session.broadcast(&format!("[{}] is ONLINE", &self.name));
        }

        let mut reader = BufReader::new(self.stream.try_clone()?);
        let session_clone = self.session.clone();
        let name_cloned = self.name.clone();

        thread::spawn(move || {
            let mut buf = String::new();
            while reader.read_line(&mut buf).unwrap() > 0 {
                let msg = format!("[{}] {}", name_cloned, buf.trim());
                {
                    let session = session_clone.lock().unwrap();
                    session.broadcast(&msg);
                }
                buf.clear();
            }
        });

        let mut writer = self.stream.try_clone()?;
        for msg in rx {
            writeln!(writer, "{}", msg)?;
            writer.flush()?;
        }

        Ok(())
    }

    fn ask_name(&mut self) -> std::io::Result<()> {
        let mut reader = BufReader::new(self.stream.try_clone()?);
        writeln!(self.stream, "Please enter your username:")?;
        let mut buf = String::new();
        reader.read_line(&mut buf)?;
        self.name = buf.trim().to_string();
        Ok(())
    }
}

fn main() -> Result<(), Error> {
    let ipv4 = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    let address = SocketAddr::new(ipv4, 8080);
    let listener = TcpListener::bind(address)?;

    let session = Arc::new(Mutex::new(Session::make()));

    println!("Listening for incoming connection on port 8080...");

    for stream in listener.incoming() {
        if let Err(err) = stream {
            eprintln!("[Error]: Couldn't accept connection: {:?}", err);
            continue;
        }

        let session = session.clone();
        thread::spawn(move || {
            if let Err(err) = Client::make(stream.unwrap(), session).main_loop() {
                eprintln!("[Error]: {:?}", err);
            }
        });
    }

    Ok(())
}
