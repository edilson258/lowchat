mod client;
mod server;

use std::io::Error;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

use crate::client::Client;
use crate::server::{Server, State};

fn main() -> Result<(), Error> {
    let ipv4 = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    let address = SocketAddr::new(ipv4, 8080);
    let listener = TcpListener::bind(address)?;
    let (tx, rx) = mpsc::channel();
    let state = Arc::new(Mutex::new(State::make()));
    let mut server = Server::make(rx, state.clone());
    thread::spawn(move || {
        server.run();
    });
    println!("Listening for incoming connection on port 8080...");
    for stream in listener.incoming() {
        if let Err(err) = stream {
            eprintln!("[Error]: Couldn't accept connection: {:?}", err);
            continue;
        }
        let server_channel = tx.clone();
        let state = state.clone();
        thread::spawn(move || {
            Client::make(stream.unwrap(), server_channel, state).run();
        });
    }
    Ok(())
}
