use std::error::Error;
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::Arc;

use webclassic::runtime::ServerOptions;
use webclassic::web::WebService;

use webclassic_embedded_links::router;
use webclassic_embedded_links::state::AppState;
use webclassic_embedded_links::template;

fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let static_dir = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/static"));

    let state = Arc::new(AppState::new());
    let env = template::create_env(&static_dir)?;

    let dispatcher = router::build_router(state, env, static_dir);
    let service = WebService::new(dispatcher);

    let listener = TcpListener::bind("127.0.0.1:3000")?;
    ServerOptions::new(service).serve(listener);

    Ok(())
}
