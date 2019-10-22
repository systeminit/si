use lazy_static::lazy_static;
use si_ssh_key::{
    data::{self, Db},
    error,
    service::Service,
    settings::Settings,
    ssh_key::client::SshKeyClient,
    ssh_key::server::SshKeyServer,
};
use tokio::runtime::Builder;
use tonic::transport::{Channel, Server};

use std::env;
use std::thread;
use std::time::Duration;

lazy_static! {
    pub static ref SETTINGS: Settings = {
        env::set_var("RUN_ENV", "testing");
        Settings::new().expect("Failed to load settings")
    };
    pub static ref SERVER: bool = {
        run_server();
        true
    };
}

pub fn get_connected_client() -> SshKeyClient<Channel> {
    let bind_to = format!("http://[::1]:{}", SETTINGS.service.port);
    SshKeyClient::connect(bind_to).expect("Cannot connect client to server")
}

pub fn run_server() {
    thread::spawn(move || {
        let runtime = Builder::new()
            .panic_handler(|err| std::panic::resume_unwind(err))
            .build()
            .expect("Cannot start server runtime");
        runtime.block_on(async {
            let db = Db::new(&SETTINGS).expect("Cannot init database");

            let mut data = data::migration_data();
            for d in data.iter_mut() {
                db.migrate_component(d).await.expect("Error migrating data");
            }

            let service = Service::new(db).expect("Error creating new Service");

            let bind_to = format!("[::1]:{}", SETTINGS.service.port);

            let addr = bind_to.parse().unwrap();

            Server::builder()
                .serve(addr, SshKeyServer::new(service))
                .await
                .map_err(error::Error::TonicError)
                .expect("Error with server building");
        });
    });
    std::thread::sleep(Duration::from_secs(1));
    true;
}
