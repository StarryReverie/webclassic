use std::error::Error;
use std::net::TcpListener;
use std::path::Path;

use webclassic::runtime::ServerOptions;
use webclassic::web::WebService;

use config::load;
use router::build_dispatcher;

mod config;
mod router;

fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let config_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "server.toml".to_string());

    let config = load(Path::new(&config_path)).map_err(|e| format!("config error: {}", e))?;

    let dispatcher = build_dispatcher(&config)?;
    let service = WebService::new(dispatcher);

    let listener = TcpListener::bind(&config.listen)
        .map_err(|e| format!("bind {} failed: {}", config.listen, e))?;

    eprintln!("listening on {}", config.listen);
    ServerOptions::new(service).serve(listener);

    Ok(())
}
