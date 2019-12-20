use std::fs::File;
use std::io::copy;
use std::path::Path;
use std::process::Child;
use std::process::Command;

const MOCK_FILENAME: &str = "CouchbaseMock.jar";
const MOCK_DOWNLOADPATH: &str =
    "https://github.com/couchbase/CouchbaseMock/releases/download/1.5.22/CouchbaseMock-1.5.22.jar";

pub struct MockServer {
    server_handle: Child,
}

impl MockServer {
    pub fn start() -> Self {
        let tests_dir = format!("{}/tests", env!("CARGO_MANIFEST_DIR"));

        if !mock_is_present(&tests_dir) {
            println!(">> Mock not present, downloading into {}", tests_dir);
            download_mock(&tests_dir);
        }

        // start server in background
        let server_handle = Command::new("java")
            .arg("-jar")
            .arg(format!("{}/{}", tests_dir, MOCK_FILENAME))
            .spawn()
            .expect("Could not spawn mock!");

        println!(">> Spawned Mock");

        Self { server_handle }
    }

    pub fn stop(&mut self) {
        // todo: this should probably use the admin port top stop and then wait instead
        // of kill
        self.server_handle
            .kill()
            .expect("Error while killing the server handle");
        println!(">> Stopped Mock");
    }
}

fn mock_is_present(tests_dir: &str) -> bool {
    Path::new(&format!("{}/{}", tests_dir, MOCK_FILENAME)).exists()
}

fn download_mock(tests_dir: &str) {
    let mut response = reqwest::get(MOCK_DOWNLOADPATH).expect("Could not locate download file!");
    let mut destination = File::create(&format!("{}/{}", tests_dir, MOCK_FILENAME))
        .expect("Could not create mock file!");
    copy(&mut response, &mut destination).expect("Could not copy mock!");
}
