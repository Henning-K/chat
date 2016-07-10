
// From: "Rust in Detail: Writing Scalable Chat Service from Scratch"
// at: https://nbaksalyar.github.io/2015/07/10/writing-chat-in-rust.html

extern crate mio;

use std::net::SocketAddr;

use mio::*;
use mio::tcp::*;

use std::collections::HashMap;

struct WebSocketServer {
    socket: TcpListener,
    clients: HashMap<Token, TcpStream>,
    token_counter: usize,
}

const SERVER_TOKEN: Token = Token(0);

impl Handler for WebSocketServer {
    // Traits can have useful default implementations, so in fact the handler interface
    // requires us to provide only two things: concrete types for timeouts and messages.
    // We're not ready to cover these fancy details, and we wouldn't get to them anytime
    // soon, so let's get along with the defaults from the mio examples:
    type Timeout = usize;
    type Message = ();

    fn ready(&mut self,
             event_loop: &mut EventLoop<WebSocketServer>,
             token: Token,
             events: EventSet) {
        match token {
            SERVER_TOKEN => {
                let client_socket = match self.socket.accept() {
                    Err(e) => {
                        println!("Accept error: {}", e);
                        return;
                    }
                    Ok(None) => unreachable!("Accept has returned 'None'"),
                    Ok(Some((sock, addr))) => sock,
                };

                // the token_counter is here to have a unique, non-overlapping key
                // for use in the HashMap which stores client connections.
                self.token_counter += 1;
                let new_token = Token(self.token_counter);

                self.clients.insert(new_token, client_socket);
                event_loop.register(&self.clients[&new_token],
                                    new_token,
                                    EventSet::readable(),
                                    PollOpt::edge() | PollOpt::oneshot())
                          .unwrap();
            }
        }
    }
}

fn main() {
    let mut event_loop = EventLoop::new().unwrap();
    // Create a new instance of our handler struct:
    let mut handler = WebSocketServer;
    // ... and provide the event loop with a mutable reference to it:
    event_loop.run(&mut handler).unwrap();

    let address = "0.0.0.0:10_000".parse::<SocketAddr>().unwrap();
    let server_socket = TcpListener::bind(&address).unwrap();

    event_loop.register(&server_socket,
                        Token(0),
                        EventSet::readable(),
                        PollOpt::edge())
              .unwrap();
}
