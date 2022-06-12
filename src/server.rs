use crate::{tcp_server, udp_server};
use crate::repository::Repository;

pub enum Protocol {
    TCP,
    UDP,
}

pub struct Server<'a> {
    protocol: Protocol,
    ip_address: &'a String,
    port: &'a String,
    repository: &'a Repository,
    key: &'a str,
}

impl Server<'_> {
    pub fn new<'a>(
        protocol: Protocol,
        ip_address: &'a String,
        port: &'a String,
        repository: &'a Repository,
        key: &'a str,
    ) -> Server<'a> {
        Server {
            protocol,
            ip_address,
            port,
            repository,
            key
        }
    }

    pub fn start_server(&self) -> std::io::Result<()> {
        match &self.protocol {
            Protocol::TCP => {
                tcp_server::launch_server(&self.ip_address, &self.port, &self.repository,&self.key)
            }
            Protocol::UDP => {
                udp_server::launch_server(&self.ip_address, &self.port, &self.repository,&self.key)
            }
        }
    }
}
