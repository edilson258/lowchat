use core::fmt;
use std::io::{Error, Read, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::spawn;

enum Message {
    ClientConnected(TcpStream),
    ClientDisconnec(TcpStream),
    ChatMessage([u8; 512]),
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ChatMessage(msg) => write!(f, "[Message]: {}", String::from_utf8_lossy(msg)),
            Self::ClientConnected(stream) => write!(f, "[Connected] {:#?}", stream),
            Self::ClientDisconnec(stream) => write!(f, "[Disconnected] {:#?}", stream),
        }
    }
}

struct Server {
    clients: Vec<TcpStream>,
}

impl Server {
    fn make() -> Self {
        Self { clients: vec![] }
    }
    fn main_loop(&mut self, recv: Receiver<Message>) {
        for msg in recv {
            match msg {
                Message::ClientConnected(stream) => self.handle_connected(stream),
                Message::ClientDisconnec(stream) => self.handle_disconnec(stream),
                Message::ChatMessage(msg) => self.handle_chat_message(msg),
            }
        }
    }

    fn handle_connected(&mut self, stream: TcpStream) {
        self.clients.push(stream);
    }

    fn handle_disconnec(&mut self, stream: TcpStream) {
        self.clients
            .retain(|s| s.peer_addr().unwrap() != stream.peer_addr().unwrap());
    }

    fn handle_chat_message(&mut self, msg: [u8; 512]) {
        self.clients.iter().for_each(|mut s| {
            s.write(&msg).unwrap();
        })
    }
}

fn client(stream: TcpStream, senr: Sender<Message>) {
    let mut stream1 = match stream.try_clone() {
        Ok(stream1) => stream1,
        Err(err) => {
            eprintln!("[Error]: Couldn't clone client's stream: {:?}", err);
            return;
        }
    };

    if let Err(err) = senr.send(Message::ClientConnected(stream)) {
        eprintln!("[Error]: Couldn't send connection event: {:?}", err);
        return;
    }

    let mut buffer: [u8; 512] = [0; 512];
    loop {
        match stream1.read(&mut buffer) {
            Ok(0) => {
                senr.send(Message::ClientDisconnec(stream1)).unwrap();
                break;
            }
            Ok(_) => {
                senr.send(Message::ChatMessage(buffer)).unwrap();
                buffer = [0; 512];
            }
            Err(err) => {
                eprintln!("[Error]: Couldn't read from client: {:?}", err);
                break;
            }
        }
    }
}

fn main() -> Result<(), Error> {
    let ipv4 = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    let address = SocketAddr::new(ipv4, 8080);
    let listener = TcpListener::bind(address)?;

    let mut server = Server::make();

    let (sndr, recv) = mpsc::channel::<Message>();

    spawn(move || {
        server.main_loop(recv);
    });

    for stream in listener.incoming() {
        if let Err(err) = stream {
            eprintln!("[Error]: Couldn't accept connection: {:?}", err);
            continue;
        }

        let sndr = mpsc::Sender::clone(&sndr);

        spawn(move || {
            client(stream.unwrap(), sndr);
        });
    }

    Ok(())
}
