use mio::net::TcpListener;
use mio::{Events, Interest, Poll, Token};
use std::collections::HashMap;
use std::io::{self};

// Setup some tokens to allow us to identify which event is for which socket.
const SERVER: Token = Token(0);

pub mod client;
pub mod entity;
pub mod protocol;
pub mod server;
pub mod util;

#[cfg(not(target_os = "wasi"))]
fn main() -> io::Result<()> {
    use std::{
        rc::Rc,
        sync::{Arc, Mutex},
    };

    use client::{interrupted, Client};
    use server::Server;

    simple_logger::SimpleLogger::new().init().unwrap();

    // Create a poll instance.
    let mut poll = Poll::new()?;
    // Create storage for events.
    let mut events = Events::with_capacity(128);

    // Setup the TCP server socket.
    let addr = "127.0.0.1:25565".parse().unwrap();
    let mut listener = TcpListener::bind(addr)?;

    // Register the server with poll we can receive events for it.
    poll.registry()
        .register(&mut listener, SERVER, Interest::READABLE)?;

    // Unique token for each incoming connection.
    let mut unique_token = Token(SERVER.0 + 1);

    log::info!("You now can connect to the server");

    let mut server = Arc::new(Mutex::new(Server::new()));

    loop {
        if let Err(err) = poll.poll(&mut events, None) {
            if interrupted(&err) {
                continue;
            }
            return Err(err);
        }

        for event in events.iter() {
            match event.token() {
                SERVER => loop {
                    // Received an event for the TCP server socket, which
                    // indicates we can accept an connection.
                    let (mut connection, address) = match listener.accept() {
                        Ok((connection, address)) => (connection, address),
                        Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                            // If we get a `WouldBlock` error we know our
                            // listener has no more incoming connections queued,
                            // so we can return to polling and wait for some
                            // more.
                            break;
                        }
                        Err(e) => {
                            // If it was any other kind of error, something went
                            // wrong and we terminate with an error.
                            return Err(e);
                        }
                    };

                    log::info!("Accepted connection from: {}", address);

                    let token = next(&mut unique_token);
                    poll.registry().register(
                        &mut connection,
                        token,
                        Interest::READABLE.add(Interest::WRITABLE),
                    )?;

                    let rc_server = Arc::clone(&server);
                    let mut guard = server.try_lock().unwrap();
                    guard.new_client(rc_server, connection, token);
                },
                
                token => {
                    // Maybe received an event for a TCP connection.
                    let done = if let Some(client) = server.try_lock().unwrap().connections.get_mut(&token) {
                        client.poll(poll.registry(), event).unwrap();
                        client.closed
                    } else {
                        // Sporadic events happen, we can safely ignore them.
                        false
                    };
                    if done {
                        if let Some(mut client) = server.try_lock().unwrap().connections.remove(&token) {
                            poll.registry().deregister(&mut client.connection)?;
                        }
                    }
                }
            }
        }
    }
}

fn next(current: &mut Token) -> Token {
    let next = current.0;
    current.0 += 1;
    Token(next)
}

#[cfg(target_os = "wasi")]
fn main() {
    panic!("can't bind to an address with wasi")
}
