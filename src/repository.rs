use aes_gcm::aead::{Aead, NewAead};
use aes_gcm::{Aes128Gcm, Key, Nonce}; // Or `Aes128Gcm`
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

use futures::executor;
use rustupolis::space::Space;
use rustupolis::store::SimpleStore;
use rustupolis::tuple;
use rustupolis::tuple::{Tuple, E};

use crate::constant::{
    ATTACH, CREATE, DELETE, EMPTY_REQUEST, IN, NO_MATCHING_TUPLE_FOUND, NO_PERMISSION,
    NO_TUPLE_SPACE_ATTACHED, OUT, PERMISSION, READ, REQUEST_DOESNT_EXIST, TUPLE_IS_EMPTY,
    TUPLE_SPACE_NOT_FOUND,
};
use crate::lexing::Lexer;
use crate::repository::RequestResponse::{DataResponse, NoResponse, OkResponse, SpaceResponse};
use crate::tuple_space::TupleSpace;

pub struct Repository {
    tuple_spaces: Arc<RwLock<HashMap<String, Arc<Mutex<Space<SimpleStore>>>>>>,
    permission_tuple_space: Arc<Mutex<Space<SimpleStore>>>,
}

pub enum RequestResponse {
    SpaceResponse(TupleSpace),
    DataResponse(String),
    OkResponse(),
    NoResponse(String),
}

impl Repository {
    pub fn new(admin_attribute: &str) -> Repository {
        let permission = Arc::new(Mutex::new(Space::new(SimpleStore::new())));
        let new_repository = Repository {
            tuple_spaces: Arc::new(RwLock::new(HashMap::with_capacity(128))),
            permission_tuple_space: permission.clone(),
        };
        new_repository
            .tuple_spaces
            .write()
            .unwrap()
            .insert(String::from(PERMISSION), permission);
        let mut permission_tuple_space = new_repository.permission_tuple_space.lock().unwrap();
        let result = executor::block_on(permission_tuple_space.tuple_out(tuple!(
            E::str(CREATE),
            E::T(tuple!(E::str(admin_attribute)))
        )));
        drop(permission_tuple_space);
        new_repository.add_permission_list(vec![String::from(admin_attribute)], PERMISSION);
        match result {
            Ok(_) => new_repository,
            Err(error) => {
                panic!("{}", error)
            }
        }
    }

    pub fn add_tuple_space(&self, name: String, attributes: Vec<String>) {
        self.tuple_spaces.write().unwrap().insert(
            name.clone(),
            Arc::new(Mutex::new(Space::new(SimpleStore::new()))),
        );
        self.add_permission_list(attributes, name.as_str());
    }

    pub fn remove_tuple_space(&self, name: &str) {
        self.tuple_spaces.write().unwrap().remove(name);
    }

    pub fn add_tuple_to_tuple_space(&self, tuple_space: String, tuple: Tuple) {
        let tuple_spaces = self.tuple_spaces.read().unwrap();
        let tuple_space = tuple_spaces.get(&*tuple_space).unwrap();
        let mut space = tuple_space.lock().unwrap();
        let mut vec: Vec<E> = Vec::new();
        let formatted_tuple = Repository::format_tuple(tuple, &mut vec);

        executor::block_on(space.tuple_out(Tuple::from_vec(formatted_tuple.clone())))
            .expect("ERROR - When out a value");
    }

    fn format_tuple(tuple: Tuple, formatted_tuple: &mut Vec<E>) -> &Vec<E> {
        if !tuple.is_empty() {
            formatted_tuple.push(match tuple.first() {
                E::S(value) => E::S("\"".to_owned() + value + "\""),
                E::T(tuple) => E::T(Tuple::from_vec(Repository::format_tuple(tuple.clone(), &mut formatted_tuple.clone()).clone())),
                E::I(rest) => E::I(*rest),
                E::D(rest) => E::D(*rest),
                E::Any => E::Any,
                E::None => E::None,
            });
            let mut vec: Vec<E> = Vec::new();
            let rest = Repository::format_tuple(tuple.rest().clone(), &mut vec);
            formatted_tuple.append(&mut rest.clone());
        }
        return formatted_tuple;
    }

    pub fn remove_tuple_to_tuple_space(&self, tuple_space: String, tuple: Tuple) {
        let tuple_spaces = self.tuple_spaces.read().unwrap();
        let tuple_space = tuple_spaces.get(&*tuple_space).unwrap();
        let mut space = tuple_space.lock().unwrap();
        executor::block_on(space.tuple_in(tuple)).expect("ERROR - When in a value");
    }

    pub fn check_permission(
        &self,
        action: &str,
        attributes: &Vec<String>,
        tuple_space_name: Option<&str>,
    ) -> bool {
        let mut permission_space = self.permission_tuple_space.lock().unwrap();
        return match action {
            CREATE => {
                match executor::block_on(permission_space.tuple_rd(tuple!(E::str(action), E::Any)))
                {
                    None => false,
                    Some(tuple) => {
                        if tuple.is_empty() {
                            return false;
                        }
                        let rest = tuple.rest();
                        Repository::compare_attributes(rest.first(), attributes)
                    }
                }
            }
            _ => {
                match executor::block_on(permission_space.tuple_rd(tuple!(
                    E::str(tuple_space_name.unwrap()),
                    E::str(action),
                    E::Any
                ))) {
                    None => false,
                    Some(tuple) => {
                        if tuple.is_empty() {
                            return false;
                        }
                        let rest = tuple.rest().rest();
                        Repository::compare_attributes(rest.first(), attributes)
                    }
                }
            }
        };
    }

    fn compare_attributes(attributes_permission: &E, attributes_client: &Vec<String>) -> bool {
        if let E::T(tuple) = attributes_permission {
            let mut attributes_permission_list = Vec::with_capacity(156);
            if let E::S(attribute) = tuple.first() {
                attributes_permission_list.push(String::from(attribute));
            }
            while !tuple.rest().is_empty() {
                if let E::S(attribute) = tuple.first() {
                    attributes_permission_list.push(String::from(attribute));
                }
            }

            if attributes_client
                .iter()
                .filter(|&x| attributes_permission_list.contains(&x))
                .count()
                > 0
            {
                return true;
            }
            return false;
        }
        return false;
    }

    pub fn add_permission_list(&self, attributes: Vec<String>, tuple_space_name: &str) {
        if attributes.len() == 1 {
            let attribute = attributes.first().unwrap();
            self.add_permission(attribute, DELETE, tuple_space_name);
            self.add_permission(attribute, READ, tuple_space_name);
            self.add_permission(attribute, IN, tuple_space_name);
            self.add_permission(attribute, OUT, tuple_space_name);
        } else if attributes.len() == 4 {
            self.add_permission(&attributes[0], READ, tuple_space_name);
            self.add_permission(&attributes[1], IN, tuple_space_name);
            self.add_permission(&attributes[2], OUT, tuple_space_name);
            self.add_permission(&attributes[3], DELETE, tuple_space_name);
        }
    }

    pub fn add_permission(&self, attribute: &String, action: &str, tuple_space_name: &str) {
        let mut permission_space = self.permission_tuple_space.lock().unwrap();
        match executor::block_on(permission_space.tuple_out(tuple!(
            E::str(tuple_space_name),
            E::str(action),
            E::T(tuple!(E::S(attribute.clone())))
        ))) {
            Ok(_) => {}
            Err(error) => {
                println!("{}", error)
            }
        }
    }

    fn decrypt_data(key: &str, text: &[u8]) -> Vec<u8> {
        let key = Key::from_slice(key.as_ref());
        let cipher = Aes128Gcm::new(key);
        let nonce = Nonce::from_slice(b"unique nonce"); // 96-bits; unique per message
        return match cipher.decrypt(nonce, text) {
            Ok(test) => test,
            Err(_) => {
                panic!();
            }
        };
    }

    pub fn manage_request(
        &self,
        request: &[u8],
        client_option: Option<&TupleSpace>,
        key: &str,
    ) -> RequestResponse {
        let request_str = &*Repository::decrypt_data(key, request).clone();
        let request = std::str::from_utf8(request_str).unwrap();
        println!("Decrypted request: {}", &request);
        let words: Vec<&str> = request.split_whitespace().collect();
        if words.len() != 0 {
            match words[0] {
                CREATE => {
                    let attribute_to_create = String::from(words[1]).replace("\"", "");
                    if self.check_permission(CREATE, &vec![attribute_to_create], None) {
                        let mut attributes_list: Vec<String> = Vec::new();
                        for index in 3..words.len() {
                            attributes_list.push(String::from(words[index]));
                        }
                        self.add_tuple_space(String::from(words[2]), attributes_list);
                        OkResponse()
                    } else {
                        NoResponse(String::from(NO_PERMISSION))
                    }
                }
                DELETE => {
                    let attribute_to_delete = String::from(words[1]);
                    // TODO check attributes
                    if self.check_permission(DELETE, &vec![attribute_to_delete], Some(words[2])) {
                        self.remove_tuple_space(words[2]);
                        OkResponse()
                    } else {
                        NoResponse(String::from(NO_PERMISSION))
                    }
                }
                ATTACH => {
                    let tuple_spaces = self.tuple_spaces.read().unwrap();
                    let tuple_space_found = tuple_spaces.get(words[1]);
                    match tuple_space_found {
                        None => NoResponse(String::from(TUPLE_SPACE_NOT_FOUND)),
                        Some(tuple_space_ref) => {
                            let mut attributes_list: Vec<String> = Vec::new();
                            for index in 2..words.len() {
                                attributes_list.push(String::from(words[index]));
                            }
                            SpaceResponse(TupleSpace::new(
                                tuple_space_ref.clone(),
                                attributes_list,
                                words[1],
                            ))
                        }
                    }
                }
                OUT => {
                    if let Some(client) = client_option {
                        if self.check_permission(
                            OUT,
                            client.attributes(),
                            Some(client.tuple_space_name()),
                        ) {
                            let param_list = words[1..].join(" ");
                            let tuple_list: Vec<Tuple> = Lexer::new(&param_list).collect();
                            for tuple in tuple_list {
                                if !tuple.is_empty() {
                                    if tuple.is_defined() {
                                        let mut space = client.tuple_space().lock().unwrap();
                                        if let Err(error) =
                                            executor::block_on(space.tuple_out(tuple))
                                        {
                                            eprintln!(
                                                "Cannot push tuple into space! Encountered error {:?}",
                                                error
                                            );
                                        } else {
                                            println!(
                                                "pushed tuple(s) {} into tuple space",
                                                param_list
                                            );
                                        }
                                    } else {
                                        eprintln!("Cannot push tuple into space! The given tuple is ill-defined.");
                                    }
                                }
                            }
                            OkResponse()
                        } else {
                            NoResponse(String::from(NO_PERMISSION))
                        }
                    } else {
                        NoResponse(String::from(NO_TUPLE_SPACE_ATTACHED))
                    }
                }
                READ => {
                    if let Some(client) = client_option {
                        if self.check_permission(
                            READ,
                            client.attributes(),
                            Some(client.tuple_space_name()),
                        ) {
                            let param_list = words[1..].join(" ");
                            let mut tuples: Vec<Tuple> = Lexer::new(&param_list).collect();
                            let mut response: RequestResponse = NoResponse(String::from(""));
                            let mut tuple_list: String = String::new();
                            let mut nb_tuples = 0;
                            for i in (0..tuples.len()).rev() {
                                let rd_tup: Tuple = tuples.remove(i);
                                if !rd_tup.is_empty() {
                                    let mut space = client.tuple_space().lock().unwrap();
                                    if let Some(match_tup) =
                                        executor::block_on(space.tuple_rd(rd_tup))
                                    {
                                        if match_tup.is_empty() {
                                            response =
                                                NoResponse(String::from(NO_MATCHING_TUPLE_FOUND));
                                        } else {
                                            println!("reading tuples {} from space", match_tup);
                                            tuple_list += &*match_tup.to_string();
                                            nb_tuples += 1;
                                            if i != 0 {
                                                tuple_list.push_str(", ");
                                            }
                                        }
                                    }
                                } else {
                                    response = NoResponse(String::from(TUPLE_IS_EMPTY));
                                }
                            }
                            if tuple_list.eq(&String::from("(")) {
                                response
                            } else {
                                if nb_tuples > 1 {
                                    DataResponse("(".to_owned() + &tuple_list + ")")
                                } else {
                                    DataResponse(tuple_list)
                                }
                            }
                        } else {
                            NoResponse(String::from(NO_PERMISSION))
                        }
                    } else {
                        NoResponse(String::from(NO_TUPLE_SPACE_ATTACHED))
                    }
                }
                IN => {
                    if let Some(client) = client_option {
                        if self.check_permission(
                            IN,
                            client.attributes(),
                            Some(client.tuple_space_name()),
                        ) {
                            let param_list = words[1..].join(" ");
                            let mut tuples: Vec<Tuple> = Lexer::new(&param_list).collect();
                            let mut response: RequestResponse = NoResponse(String::from(""));
                            let mut tuple_list: String = String::new();
                            let mut nb_tuples = 0;
                            for i in (0..tuples.len()).rev() {
                                let rd_tup: Tuple = tuples.remove(i);
                                if !rd_tup.is_empty() {
                                    let mut space = client.tuple_space().lock().unwrap();
                                    dbg!("pulling in tuple matching {} from space", &rd_tup);
                                    if let Some(match_tup) =
                                        executor::block_on(space.tuple_in(rd_tup))
                                    {
                                        if match_tup.is_empty() {
                                            response =
                                                NoResponse(String::from(NO_MATCHING_TUPLE_FOUND));
                                        } else {
                                            tuple_list += &*match_tup.to_string();
                                            nb_tuples += 1;
                                            if i != 0 {
                                                tuple_list.push_str(", ");
                                            }
                                        }
                                    }
                                } else {
                                    response = NoResponse(String::from(TUPLE_IS_EMPTY));
                                }
                            }
                            if tuple_list.is_empty() {
                                response
                            } else {
                                if nb_tuples > 1 {
                                    DataResponse("(".to_owned() + &tuple_list + ")")
                                } else {
                                    DataResponse(tuple_list)
                                }
                            }
                        } else {
                            NoResponse(String::from(NO_PERMISSION))
                        }
                    } else {
                        NoResponse(String::from(NO_TUPLE_SPACE_ATTACHED))
                    }
                }
                _ => NoResponse(String::from(REQUEST_DOESNT_EXIST)),
            }
        } else {
            NoResponse(String::from(EMPTY_REQUEST))
        }
    }
}
