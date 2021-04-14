#[macro_use]
extern crate log;
extern crate hex;
extern crate pos_api;
extern crate pos_compute;

mod api;
mod pos_task;
mod server;

use crate::server::{Init, PosServer, SetConfig, StartGrpcService};
use chrono::prelude::*;
use clap::{App, Arg};
use config::Config;
use env_logger::fmt::Color;
use env_logger::Builder;
use log::*;
use std::env;
use std::io::Write;
use tokio::signal;
use xactor::*;

const DEFAULT_GRPC_PORT: u32 = 6667;
const DEFAULT_HOST: &str = "[::1]";
const DEFAULT_INDEXED_PER_CYCLE: u64 = 9 * 128 * 1024;
const DEFAULT_BITS_PER_INDEX: u32 = 8;
const DEFAULT_SALT: &str = "114a00005de29b0aaad6814e5f33d357686da48923e8e4864ee5d6e20053e886";

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    init_logging();
    let mut config = get_default_config();
    let args = App::new("Pos Server")
        .version("0.1.0")
        .author("Aviv Eyal <a@spacemesh.io>")
        .about("Creates proofs of space using compute providers such as gpus")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .takes_value(true)
                .value_name("FILE")
                .help("provide server configuration file")
                .takes_value(true),
        )
        .get_matches();

    if let Some(conf_file) = args.value_of("config") {
        config
            .merge(config::File::with_name(conf_file).required(false))
            .unwrap();
    }

    start_server(config).await?;

    // block app until it is terminated
    signal::ctrl_c()
        .await
        .expect("failed to listen for ctrl-c signal");

    info!("got signal - terminating app");

    Ok(())
}

pub(crate) async fn start_server(config: Config) -> Result<()> {
    // init the server (one-time per process, pre config)
    let server = PosServer::from_registry().await?;
    server.call(Init {}).await??;

    // set server config
    let salt = hex::decode(config.get_str("salt").unwrap()).unwrap();
    use pos_api::api::Config;
    server
        .call(SetConfig(Config {
            // default config
            data_dir: config.get_str("data_dir").unwrap(),
            indexes_per_compute_cycle: config.get_int("indexes_per_cycle").unwrap() as u64,
            bits_per_index: config.get_int("bits_per_index").unwrap() as u32,
            salt,
            n: config.get_int("n").unwrap() as u32,
            r: config.get_int("r").unwrap() as u32,
            p: config.get_int("p").unwrap() as u32,
        }))
        .await??;

    info!("server starting...");

    server
        .call(StartGrpcService {
            port: config.get_int("port").unwrap() as u32,
            host: config.get_str("host").unwrap(),
        })
        .await??;

    info!("server running.");

    // do_providers_list();
    //do_benchmark();
    Ok(())
}

pub(crate) fn init_logging() {
    let mut builder = Builder::new();

    builder
        .format_level(true)
        .format_timestamp(None)
        .format(move |buf, record| {
            let level_style = buf.default_level_style(record.level());

            let now: DateTime<Local> = Local::now();
            let date_format = format!("{}", now.to_rfc3339(),);

            let mut date_style = buf.style();
            date_style.set_color(Color::Yellow).set_bold(true);

            let mut file_name_style = buf.style();
            file_name_style.set_color(Color::Blue);

            let file_name = format!(
                "{} {}",
                record.file().unwrap().split('/').last().unwrap(),
                record.line().unwrap()
            );

            writeln!(
                buf,
                "{} {} {} {}",
                date_style.value(date_format),
                level_style.value(record.level()),
                file_name_style.value(file_name),
                record.args(),
            )
        })
        .filter(None, LevelFilter::Info);

    if env::var("RUST_LOG").is_ok() {
        builder.parse_filters(&env::var("RUST_LOG").unwrap());
    }

    builder.init();
}

pub(crate) fn get_default_config() -> config::Config {
    let mut config = Config::default();
    config
        .set_default("data_dir", "./pos")
        .unwrap()
        .set_default("log_file", "./log.txt")
        .unwrap()
        .set_default("indexes_per_cycle", DEFAULT_INDEXED_PER_CYCLE.to_string())
        .unwrap()
        .set_default("bits_per_index", DEFAULT_BITS_PER_INDEX.to_string())
        .unwrap()
        .set_default("salt", DEFAULT_SALT)
        .unwrap()
        .set_default("port", DEFAULT_GRPC_PORT.to_string())
        .unwrap()
        .set_default("host", DEFAULT_HOST)
        .unwrap()
        .set_default("n", 512.to_string())
        .unwrap()
        .set_default("r", 1.to_string())
        .unwrap()
        .set_default("p", 1.to_string())
        .unwrap()
        .clone()
}
