use crate::server::Server;

pub struct ServerLauncher<'a> {
    server_list: Vec<Server<'a>>,
}

impl ServerLauncher<'_> {
    pub fn new(server_list: Vec<Server>) -> ServerLauncher {
        ServerLauncher {
            server_list
        }
    }

    pub fn new_one_server(server: Server) -> ServerLauncher {
        ServerLauncher {
            server_list: vec![server]
        }
    }

    pub fn launch_server(&self) {
        crossbeam::scope(|scope| {
            for server in &self.server_list {
                scope.spawn(move |_| match server.start_server() {
                    Ok(_) => {
                        println!("{}", "OK ")
                    }
                    Err(error) => {
                        println!("{}", error)
                    }
                });
            }
        })
            .unwrap();
    }
}