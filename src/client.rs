use crate::constant::{ATTACH, CREATE, DELETE, ERROR, IN, OK, OUT, READ, SPACE};
use crate::lexing::Lexer;
use crate::server_client::ServerClient;
use rustupolis::tuple::E::S;
use rustupolis::tuple::{Tuple, E};
use std::collections::HashMap;

pub struct Client {
    server_list: HashMap<String, ServerClient>,
    server_attached: String,
}

impl Client {
    pub fn new() -> Client {
        Client {
            server_attached: String::new(),
            server_list: HashMap::new(),
        }
    }

    pub fn connect(
        &mut self,
        ip_address: String,
        port: String,
        protocol: String,
        server_name: &String,
        key: &str,
    ) {
        let server = ServerClient::new(ip_address, port, protocol, server_name.clone(), key);
        self.server_list.insert(server_name.clone(), server);
    }

    pub fn create(
        &self,
        server_name: &String,
        attributes: Vec<String>,
        tuple_space_name: &String,
        admin_attribute: &String,
    ) {
        let server = self.server_list.get(&*server_name);
        match server {
            None => {}
            Some(server) => {
                if !attributes.is_empty() {
                    let mut attribute_list: String = String::new();
                    for attribute in attributes {
                        attribute_list += &*(attribute + &*" ".to_string());
                    }
                    println!(
                        "{}",
                        server.send_message(
                            String::from(CREATE)
                                + SPACE
                                + &*admin_attribute
                                + SPACE
                                + &*tuple_space_name
                                + SPACE
                                + &*attribute_list
                        )
                    );
                } else {
                    println!(
                        "{}",
                        server.send_message(
                            String::from(CREATE)
                                + SPACE
                                + &*admin_attribute
                                + SPACE
                                + &*tuple_space_name
                        )
                    );
                }
            }
        }
    }

    pub fn in_instr(&mut self, list_tuple: Vec<Tuple>) -> Tuple {
        return self.manage_primitives(IN, list_tuple);
    }

    pub fn out(&mut self, list_tuple: Vec<Tuple>) {
        self.manage_primitives(OUT, list_tuple);
    }

    pub fn read(&mut self, list_tuple: Vec<Tuple>) -> Tuple {
        return self.manage_primitives(READ, list_tuple);
    }

    pub fn attach(
        &mut self,
        server_name: &String,
        attributes: Vec<String>,
        tuple_space_name: &String,
    ) {
        self.server_attached = server_name.clone();
        let server = self.server_list.get(&*self.server_attached);
        match server {
            None => {}
            Some(server) => {
                if !attributes.is_empty() {
                    let mut attribute_list: String = String::new();
                    for attribute in attributes {
                        attribute_list += &*(attribute + &*" ".to_string());
                    }
                    println!(
                        "{}",
                        server.send_message(
                            String::from(ATTACH)
                                + SPACE
                                + &*(tuple_space_name)
                                + SPACE
                                + &*attribute_list
                        )
                    );
                } else {
                    println!(
                        "{}",
                        server.send_message(String::from(ATTACH) + SPACE + &*(tuple_space_name))
                    );
                }
            }
        }
    }

    pub fn delete(
        &self,
        server_name: String,
        delete_attribute: Option<String>,
        tuple_space_name: String,
    ) {
        let server = self.server_list.get(&*server_name);
        match server {
            None => {}
            Some(server) => {
                if let Some(attribute) = delete_attribute {
                    println!(
                        "{}",
                        server.send_message(
                            String::from(DELETE)
                                + SPACE
                                + &*(attribute)
                                + SPACE
                                + &*(tuple_space_name)
                        )
                    );
                } else {
                    println!(
                        "{}",
                        server.send_message(String::from(DELETE) + SPACE + &*(tuple_space_name))
                    );
                }
            }
        }
    }

    pub fn manage_primitives(&mut self, operation: &str, list_tuple: Vec<Tuple>) -> Tuple {
        let server_attached = self.server_attached.clone();
        let server = self.server_list.remove(&*server_attached);
        match server {
            None => {}
            Some(server) => {
                let mut tuple_list: String = String::new();
                for tuple in list_tuple {
                    let request: String = String::from("(");
                    tuple_list += &*(Client::format_tuple(tuple, request) + ")");
                }
                let mut response =
                    server.send_message(String::from(operation) + SPACE + &*tuple_list);
                println!("{}", response);
                let _ = &self.server_list.insert(server_attached, server);
                if response.contains(&String::from(ERROR)) || response.contains(&String::from(OK)) {
                    return Tuple::new(&[]);
                }
                Client::remove_whitespace(&mut response);
                let tuple_list: Vec<Tuple> = Lexer::new(&response).collect();
                if let Some(response) = tuple_list.first() {
                    return response.clone();
                }
            }
        }
        return Tuple::new(&[]);
    }

    fn format_tuple(tuple: Tuple, mut request: String) -> String {
        if !tuple.is_empty() {
            request = match tuple.first() {
                S(value) => request + "\"" + value + "\"",
                E::T(tuple) => {
                    "(".to_owned() + &Client::format_tuple(tuple.clone(), request.clone()) + ")"
                }
                E::I(rest) => request + &*rest.to_string(),
                E::D(rest) => request + &*rest.to_string(),
                E::Any => request + "_",
                E::None => request,
            };
            if !tuple.rest().is_empty() {
                request += ","
            }
            request = Client::format_tuple(tuple.rest(), request.clone())
        }
        return request;
    }

    fn remove_whitespace(s: &mut String) {
        s.retain(|c| !c.is_whitespace());
    }
}
