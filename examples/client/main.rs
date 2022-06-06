use rustupolis::tuple;
use rustupolis::tuple::{E, Tuple};
use rustupolis_server::client::Client;

fn main() {
    let mut client = Client::new();
    let server_tcp_name = String::from("TCP_server");
    let server_udp_name = String::from("UDP_server");
    let admin_attribute = String::from("admin");
    let tuple_space_name = String::from("data");
    let tuple_space_name_mean = String::from("tuple_space_mean");
    let attribute = String::from("attribute");

    client.connect(String::from("127.0.0.1"),String::from("9000"), String::from("tcp"),&server_tcp_name);
    client.connect(String::from("127.0.0.1"),String::from("9001"), String::from("udp"),&server_udp_name);

    client.create(&server_tcp_name,vec![attribute.clone()],&tuple_space_name,&admin_attribute);
    client.create(&server_udp_name,vec![attribute.clone()],&tuple_space_name_mean,&admin_attribute);

    client.attach(&server_tcp_name,vec![attribute.clone()],&tuple_space_name);

    client.out(vec![tuple![ E::str("\"temp\""), E::I(21),], tuple![ E::str("temp"), E::I(23),], tuple![ E::str("temp"), E::I(29),], tuple![ E::str("temp"), E::I(25),], tuple![ E::str("temp"), E::I(20),]]);

   // let data = client.in_instr(vec![tuple![ E::str("temp"), E::Any], tuple![ E::str("temp"), E::Any], tuple![ E::str("temp"), E::Any], tuple![ E::str("temp"),E::Any], tuple![ E::str("temp"), E::Any]]);
    //dbg!(data);
    let sum = 0;
}