# A Rust Low Level Chat Application

This project is a demonstration of a chat application written in rust using only the standard library.

## Features
- No third party dependencies
- Uses multiple threads and designed to be thread safe
- Fast and reliable with great error handling mechanisms 
- Low level (raw tcp streams)
- Based on mpsc(multiple producer single consumer) channels

## Get started
Your will need to host the server yourself.
1. clone this repository 
```shell
git clone https://github.com/edilson258/lowchat.git
```

2. running the server (runs on port 8080)
```shell
cd lowchat
cargo run
```
then can use a tcp client to join to the chat

- Example using netcat
```shell
nc 127.0.0.1 8080
```

3. share the server with friends

To be able to invite others to the chat you will need to host the server on a host provider, but for seek of demonstration using
a ssh tunnel through [serveo](https://serveo.net/) is enough.

whith server running locally on port 8080 run the following command
```shell
ssh -R 8081:localhost:8080 serveo.net
```

Note: if get an error like this: `Warning: remote port forwarding failed for listen port 8081` you must try a different port than `8081`.
Now your friends can join the chat through a tcp client 

```shell
nc serveo.net 8081
```

## Contributions 
Feel free to clone and play with it. Pull requests are welcome! 💯
