extern crate core;

pub use rustupolis::tuple;
pub use rustupolis::tuple::E;
mod tuple_space;
mod constant;
mod lexing;
pub mod repository;
pub mod server;
pub mod server_launcher;
pub mod client;
mod tcp_server;
mod udp_server;
mod server_client;