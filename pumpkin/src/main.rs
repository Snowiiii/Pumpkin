use mio::net::TcpListener;
use mio::{Events, Interest, Poll, Token};
use std::io::{self};

use client::Client;
use commands::handle_command;
use configuration::AdvancedConfiguration;

use std::{
    collections::HashMap,
    rc::Rc,
    thread,
};

use client::interrupted;
use configuration::BasicConfiguration;
use server::Server;

// Setup some tokens to allow us to identify which event is for which socket.
const SERVER: Token = Token(0);

pub mod client;
pub mod commands;
pub mod configuration;
pub mod entity;
pub mod server;
pub mod util;

#[cfg(not(target_os = "wasi"))]
fn main() -> io::Result<()> {
    use std::{cell::RefCell, time::Instant};

    let time = Instant::now();
    let basic_config = BasicConfiguration::load("configuration.toml");

    let advanced_configuration = AdvancedConfiguration::load("features.toml");

    simple_logger::SimpleLogger::new().init().unwrap();

    // Create a poll instance.
    let mut poll = Poll::new()?;
    // Create storage for events.
    let mut events = Events::with_capacity(128);

    // Setup the TCP server socket.

    let addr = format!(
        "{}:{}",
        basic_config.server_address, basic_config.server_port
    )
    .parse()
    .unwrap();

    let mut listener = TcpListener::bind(addr)?;

    // Register the server with poll we can receive events for it.
    poll.registry()
        .register(&mut listener, SERVER, Interest::READABLE)?;

    // Unique token for each incoming connection.
    let mut unique_token = Token(SERVER.0 + 1);

    let use_console = advanced_configuration.commands.use_console;

    let mut connections: HashMap<Token, Rc<RefCell<Client>>> = HashMap::new();

    let mut server = Server::new((
        basic_config,
        advanced_configuration,
    ));
    log::info!("Started Server took {}ms", time.elapsed().as_millis());
    log::info!("You now can connect to the server");

    if use_console {
        thread::spawn(move || {
            let stdin = std::io::stdin();
            loop {
                let mut out = String::new();
                stdin
                    .read_line(&mut out)
                    .expect("Failed to read console line");
                handle_command(&mut commands::CommandSender::Console, out);
            }
        });
    }
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
                    if let Err(e) = connection.set_nodelay(true) {
                            log::warn!("failed to set TCP_NODELAY {e}");
                        }

                    log::info!("Accepted connection from: {}", address);

                    let token = next(&mut unique_token);
                    poll.registry().register(
                        &mut connection,
                        token,
                        Interest::READABLE.add(Interest::WRITABLE),
                    )?;
                    let rc_token = Rc::new(token);
                    let client = Rc::new(RefCell::new(Client::new(
                        Rc::clone(&rc_token),
                        connection,
                        addr,
                    )));
                    server
                        .add_client(rc_token, Rc::clone(&client));
                    connections.insert(token, client);
                },

                token => {
                    // Maybe received an event for a TCP connection.
                    let done = if let Some(client) = connections.get_mut(&token) {
                        let mut client = client.borrow_mut();
                        client.poll(&mut server, event);
                        client.closed
                    } else {
                        // Sporadic events happen, we can safely ignore them.
                        false
                    };
                    if done {
                        if let Some(client) = connections.remove(&token) {
                            let mut client = client.borrow_mut();
                            server.remove_client(&token);
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
