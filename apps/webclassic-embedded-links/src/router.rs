use std::path::PathBuf;
use std::sync::Arc;

use minijinja::Environment;
use webclassic::web::handler::StaticDirectoryHandler;
use webclassic::web::protocol::util::Method;
use webclassic::web::{Dispatcher, Route};

use crate::handler;
use crate::state::AppState;

pub fn build_router(
    state: Arc<AppState>,
    env: Environment<'static>,
    static_dir: PathBuf,
) -> Dispatcher {
    Dispatcher::new()
        .with(
            Route::by(Method::Post).equal("/shorten"),
            handler::shorten_handler(Arc::clone(&state)),
        )
        .with(
            Route::by(Method::Get).equal("/list"),
            handler::list_handler(Arc::clone(&state), env),
        )
        .with(
            Route::by(Method::Get).prefix("/s"),
            handler::redirect_handler(state),
        )
        .with(
            Route::by(Method::Get).prefix("/"),
            StaticDirectoryHandler::new(static_dir).with_index_file("index.html"),
        )
}
