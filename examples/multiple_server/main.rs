use rustupolis::tuple::{E, Tuple};
use rustupolis_server::repository::Repository;
use rustupolis_server::server::{Protocol, Server};
use rustupolis_server::server_launcher::ServerLauncher;

fn main() {
    let ip_address = String::from("192.168.1.139");
    let port_tcp = String::from("9000");
    let port_udp = String::from("9001");

    let repository = Repository::new("admin");

    repository.add_tuple_space(String::from("DATA"),vec![String::from("admin")]);
    repository.add_tuple_to_tuple_space(String::from("DATA"), Tuple::new(&[E::str("test")]));

    let server_tcp = Server::new(Protocol::TCP, &ip_address, &port_tcp, &repository);
    let server_udp = Server::new(Protocol::UDP, &ip_address, &port_udp, &repository);

    let server_launcher = ServerLauncher::new(vec![server_tcp, server_udp]);
    server_launcher.launch_server();
}
