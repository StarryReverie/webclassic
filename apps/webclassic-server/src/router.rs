use std::error::Error;

use webclassic::web::controller::Controller;
use webclassic::web::filter::{ErrorPageFilter, HeadFilter};
use webclassic::web::handler::{CgiHandler, StaticDirectoryHandler};
use webclassic::web::protocol::util::{Method, StatusCode};
use webclassic::web::{Dispatcher, FilterExt, Route};

use crate::config::Config;

pub fn build_controller(
    config: &Config,
) -> Result<Box<dyn Controller>, Box<dyn Error + Send + Sync>> {
    let dispatcher = build_dispatcher(config)?;

    let mut error_filter = ErrorPageFilter::new();
    for (code_str, path) in &config.error_pages {
        let code: u16 = code_str
            .parse()
            .map_err(|_| format!("invalid status code in error_pages: '{}'", code_str))?;
        let full_path = config.content.root.join(path);
        let html = std::fs::read_to_string(&full_path)
            .map_err(|e| format!("failed to read error page '{}': {}", full_path.display(), e))?;
        let status = StatusCode::new(code);
        error_filter = error_filter.with_page(status, html);
    }

    let controller = dispatcher
        .filtered(HeadFilter::new())
        .filtered(error_filter);
    Ok(Box::new(controller))
}

fn build_dispatcher(config: &Config) -> Result<Dispatcher, Box<dyn Error + Send + Sync>> {
    let mut dispatcher = Dispatcher::new();

    for entry in &config.cgi {
        let handler = match &entry.interpreter {
            Some(interp) => {
                let mut full_args = vec![entry.script.to_string_lossy().to_string()];
                full_args.extend_from_slice(&entry.args);
                CgiHandler::new(interp.clone(), full_args)
            }
            None => CgiHandler::new(entry.script.clone(), entry.args.clone()),
        };
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
