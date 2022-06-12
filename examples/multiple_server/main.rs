use rustupolis::tuple::{E, Tuple};
use rustupolis_server::repository::Repository;
use rustupolis_server::server::{Protocol, Server};
use rustupolis_server::server_launcher::ServerLauncher;

fn main() {
    let ip_address = String::from("127.0.0.1");
    let port_tcp = String::from("9000");
    let port_udp = String::from("9001");

    let repository = Repository::new("admin");
    let key = "an example very ";

    repository.add_tuple_space(String::from("DATA"),vec![String::from("admin")]);

    repository.add_tuple_to_tuple_space(String::from("DATA"), Tuple::new(&[E::str("test")]));
    repository.remove_tuple_to_tuple_space(String::from("DATA"), Tuple::new(&[E::Any]));

    let server_tcp = Server::new(Protocol::TCP, &ip_address, &port_tcp, &repository,key);
    let server_udp = Server::new(Protocol::UDP, &ip_address, &port_udp, &repository,key);

    let server_launcher = ServerLauncher::new(vec![server_tcp, server_udp]);
    server_launcher.launch_server();
}
