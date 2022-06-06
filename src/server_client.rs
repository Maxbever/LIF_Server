use crate::constant;
use crate::constant::{STOP_SERVER, TIMEOUT};
use constant::{TCP, UDP};
use std::io::{BufRead, Write};
use std::net::{Ipv4Addr, TcpStream, UdpSocket};
use std::str::from_utf8;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use std::time::Duration;
use std::{io, thread};

pub struct ServerClient {
    mpsc_channel: (Sender<String>, Receiver<String>),
}

impl ServerClient {
    pub fn new(ip_address: String, port: String, protocol: String, server_name:String) -> ServerClient {
        let address = format!("{}:{}", ip_address, port);
        let addr: String = address.parse().unwrap();
        let (tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel();
        let (tx_response, rx_response): (Sender<String>, Receiver<String>) = mpsc::channel();

        match protocol.as_str() {
            TCP => {
                match thread::Builder::new().name(server_name).spawn(move || {
                    match TcpStream::connect(addr) {
                        Ok(mut server) => {
                            ServerClient::read_response_tcp(&mut server, &tx_response);
                            loop {
                                match &rx.try_recv() {
                                    Ok(message) => {
                                        if STOP_SERVER.eq(message.as_str()) {
                                            break;
                                        }
                                        match server.write(message.as_bytes()) {
                                            Ok(_) => {
                                                println!("{} sent", message)
                                            }
                                            Err(e) => {
                                                eprintln!("{}", e)
                                            }
                                        };
                                        ServerClient::read_response_tcp(&mut server, &tx_response);
                                    }
                                    Err(TryRecvError::Disconnected) => {
                                        println!("Terminating.");
                                        break;
                                    }
                                    Err(_) => {}
                                };
                            }
                        }
                        Err(error) => {
                            panic!("{}", error);
                        }
                    };
                }) {
                    Ok(_) => {}
                    Err(error) => {
                        panic!("{}", error);
                    }
                };
            }
            UDP => {
                match thread::Builder::new().name(server_name).spawn(move || {
                    match UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)) {
                        Ok(mut server) => {
                            match server.connect(addr) {
                                Ok(_) => loop {
                                    match &rx.try_recv() {
                                        Ok(message) => {
                                            if STOP_SERVER.eq(message.as_str()) {
                                                break;
                                            }
                                            match server.send(message.as_bytes()) {
                                                Ok(_) => {
                                                    println!("{} sent", message)
                                                }
                                                Err(e) => {
                                                    eprintln!("{}", e)
                                                }
                                            };
                                            ServerClient::read_response_udp(&mut server, &tx_response);
                                        }
                                        Err(TryRecvError::Disconnected) => {
                                            println!("Terminating.");
                                            break;
                                        }
                                        Err(_) => {}
                                    };
                                },
                                Err(error) => {
                                    panic!("{}", error)
                                }
                            };
                        }
                        Err(error) => {
                            println!("UDP socket error : {}", error);
                        }
                    };
                }) {
                    Ok(_) => {}
                    Err(error) => {
                        panic!("{}", error);
                    }
                };
            }
            _ => {}
        };

        loop {
            match rx_response.recv_timeout(Duration::from_secs(TIMEOUT)) {
                Ok(response) => {
                    println!("{}", response);
                    break;
                }
                Err(_) => {
                    break;
                }
            };
        }

        ServerClient {
            mpsc_channel: (tx, rx_response),
        }
    }

    pub fn read_response_tcp(server: &mut TcpStream, tx_response: &Sender<String>) {
        let mut reader = io::BufReader::new(server);
        loop {
            match reader.fill_buf() {
                Ok(response) => {
                    if response.len() != 0 {
                        let text = from_utf8(&response).unwrap();
                        match tx_response.send(text.parse().unwrap()) {
                            Ok(_) => {}
                            Err(error) => {eprintln!("{}",error)}
                        };
                        break;
                    }
                }
                Err(error) => {
                    eprintln!("{}", error)
                }
            }
        }
    }

    pub fn read_response_udp(server: &mut UdpSocket, tx_response: &Sender<String>) {
        let mut buf = [0; 64];
        loop {
            match server.recv(&mut buf) {
                Ok(response) => {
                    if response != 0 {
                        let text = from_utf8(&buf).unwrap().trim_matches(char::from(0));
                        match tx_response.send(text.parse().unwrap()) {
                            Ok(_) => {}
                            Err(error) => {eprintln!("{}",error)}
                        };
                        break;
                    }
                }
                Err(error) => {
                    eprintln!("{}", error)
                }
            }
        }
    }

    pub fn send_message(&self, message: String) -> String {
        let (tx, rx_response) = &self.mpsc_channel;
        match tx.send(message) {
            Ok(_) => {}
            Err(error) => {
                eprintln!("{}", error)
            }
        };

        loop {
            match rx_response.recv_timeout(Duration::from_secs(TIMEOUT)) {
                Ok(response) => {
                    return response;
                }
                Err(_) => {}
            };
        }
    }
}
