#[macro_use]
extern crate log;
extern crate nix;

extern crate pos_api;

use log::LevelFilter;
use nix::sys::signal;
use nix::sys::signal::Signal;
use nix::unistd::Pid;
use std::env;
use std::process::{Child, Command};
use std::time::Duration;
use tokio::time::sleep;

struct Guard(Child);

/// Child process guard
impl Drop for Guard {
    fn drop(&mut self) {
        let pid = self.0.id() as i32;
        match signal::kill(Pid::from_raw(pid), Signal::SIGINT) {
            Err(e) => debug!("could not kill child process {}: {}", pid, e),
            Ok(_) => debug!("killed process {}", pid),
        }
    }
}

#[tokio::test]
async fn test_api() {
    let _ = env_logger::builder()
        .is_test(false)
        .filter_level(LevelFilter::Info)
        .try_init();

    let path = env::current_dir().unwrap();
    info!("Path: {:?}", path);

    let server_path = "../../target/debug/pos-service";
    let server = Command::new(server_path).spawn().unwrap();
    let guard = Guard(server);

    sleep(Duration::from_secs(5)).await;

    info!("{}", guard.0.id());
}
