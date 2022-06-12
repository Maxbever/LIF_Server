use std::collections::HashMap;
use std::io;
use std::net::SocketAddr;
use aes_gcm_siv::{Aes128GcmSiv, Key, Nonce}; // Or `Aes128GcmSiv`
use aes_gcm_siv::aead::{Aead, NewAead};

use log::warn;
use mio::{Events, Interest, Poll, Token};
use mio::net::UdpSocket;

use crate::tuple_space::TupleSpace;
use crate::constant::{OK, TUPLE_SPACE_ATTACHED, TUPLE_SPACE_ATTACHED_UPDATED};
use crate::repository::{Repository, RequestResponse};

// A token to allow us to identify which event is for the `UdpSocket`.
const UDP_SOCKET: Token = Token(0);

#[cfg(not(target_os = "wasi"))]
pub(crate) fn launch_server(
    ip_address: &String,
    port: &String,
    repository: &Repository,
    key: &str
) -> io::Result<()> {
    let mut poll = Poll::new()?;
    let mut events = Events::with_capacity(126);

    let address = format!("{}:{}", ip_address, port);
    // Setup the UDP server socket.
    let addr = address.parse().unwrap();

    let mut socket = UdpSocket::bind(addr)?;

    let mut client_list: HashMap<SocketAddr, TupleSpace> = HashMap::new();
    // Register our socket with the token defined above and an interest in being
    // `READABLE`.
    poll.registry()
        .register(&mut socket, UDP_SOCKET, Interest::READABLE)?;

    println!("You can connect to the UDP server using `ncat`:");
    println!("ncat -u {} {}", ip_address, port);

    let mut buf = [0; 1 << 16];

    loop {
        // Poll to check if we have events waiting for us.
        poll.poll(&mut events, None)?;

        // Process each event.
        for event in events.iter() {
            // Validate the token we registered our socket with, in this examples it will only ever
            // be one but we make sure it's valid none the less.
            match event.token() {
                UDP_SOCKET => loop {
                    match socket.recv_from(&mut buf) {
                        Ok((packet_size, source_address)) => {
                                let client = client_list.get(&source_address);
                                let result = repository
                                    .manage_request(&buf[..packet_size], client,key);
                                match result {
                                    RequestResponse::SpaceResponse(new_client) => {
                                        match client_list.insert(source_address, new_client) {
                                            None => {
                                                if let Err(e) = socket.send_to(
                                                    &*encrypt_data(key,TUPLE_SPACE_ATTACHED.as_ref()),
                                                    source_address,
                                                ) {
                                                    println!("{}", e)
                                                }
                                            }
                                            Some(_) => {
                                                if let Err(e) = socket.send_to(
                                                    &*encrypt_data(key,TUPLE_SPACE_ATTACHED_UPDATED.as_ref()),
                                                    source_address,
                                                ) {
                                                    println!("{}", e)
                                                }
                                            }
                                        };
                                    }
                                    RequestResponse::NoResponse(x) => {
                                        if let Err(e) = socket.send_to(&*encrypt_data(key,x.as_ref()), source_address) {
                                            println!("{}", e)
                                        }
                                    }
                                    RequestResponse::OkResponse() => {
                                        if let Err(e) = socket.send_to(&*encrypt_data(key,OK.as_ref()), source_address)
                                        {
                                            println!("{}", e)
                                        }
                                    }
                                    RequestResponse::DataResponse(tuple_list) => {
                                        if let Err(e) =
                                        socket.send_to(&*encrypt_data(key,tuple_list.as_ref()), source_address)
                                        {
                                            println!("{}", e)
                                        }
                                    }
                                }

                        }
                        Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                            // If we get a `WouldBlock` error we know our socket
                            // has no more packets queued, so we can return to
                            // polling and wait for some more.
                            break;
                        }
                        Err(e) => {
                            // If it was any other kind of error, something went
                            // wrong and we terminate with an error.
                            return Err(e);
                        }
                    }
                },
                _ => {
                    // This should never happen as we only registered our
                    // `UdpSocket` using the `UDP_SOCKET` token, but if it ever
                    // does we'll log it.
                    warn!("Got event for unexpected token: {:?}", event);
                }
            }
        }
    }

    fn encrypt_data(key:&str, text:&str) -> Vec<u8> {
        let key = Key::from_slice(key.as_ref());
        let cipher = Aes128GcmSiv::new(key);

        let nonce = Nonce::from_slice(b"unique nonce");

        return cipher.encrypt(nonce,text.as_ref()).expect("encryption failure!");
    }
}
