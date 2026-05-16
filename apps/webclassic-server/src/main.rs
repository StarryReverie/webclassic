use std::error::Error;
use std::net::TcpListener;
use std::path::Path;

use webclassic::runtime::ServerOptions;
use webclassic::web::WebService;

use config::load;
use log::create_log_backend;
use router::build_controller;

mod config;
mod log;
mod router;

fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let config_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "server.toml".to_string());

    let config = load(Path::new(&config_path)).map_err(|e| format!("config error: {}", e))?;

    let log_backend = create_log_backend();

    let controller = build_controller(&config, log_backend)?;
    let service = WebService::boxed(controller);

    let listener = TcpListener::bind(&config.listen)
        .map_err(|e| format!("bind {} failed: {}", config.listen, e))?;

    eprintln!("listening on {}", config.listen);
    ServerOptions::new(service)
        .max_connections(config.max_connections)
        .max_pending(config.max_pending)
        .serve(listener);

    Ok(())
}
