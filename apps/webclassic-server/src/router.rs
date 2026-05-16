use std::error::Error;

use webclassic::web::handler::{CgiHandler, StaticDirectoryHandler};
use webclassic::web::protocol::util::Method;
use webclassic::web::{Dispatcher, Route};

use crate::config::Config;

pub fn build_dispatcher(config: &Config) -> Result<Dispatcher, Box<dyn Error + Send + Sync>> {
    let mut dispatcher = Dispatcher::new();

    for entry in &config.cgi {
        let handler = CgiHandler::new(entry.program.clone(), entry.args.clone());
        let methods = entry.parse_methods()?;
        for method in methods {
            dispatcher = dispatcher.with(Route::by(method).prefix(&entry.prefix), handler.clone());
        }
    }

    let mut static_handler = StaticDirectoryHandler::new(config.content.root.clone());
    if let Some(ref index) = config.content.index {
        static_handler = static_handler.with_index_file(index);
    }
    dispatcher = dispatcher.with(Route::by(Method::Get).prefix("/"), static_handler);

    Ok(dispatcher)
}
