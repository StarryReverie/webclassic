use std::sync::Arc;

use minijinja::Environment;
use webclassic::service::Interrupt;
use webclassic::web::controller::Context;
use webclassic::web::handler::FunctionHandler;
use webclassic::web::protocol::HttpResponse;
use webclassic::web::protocol::util::StatusCode;

use crate::state::AppState;

pub fn redirect_handler(state: Arc<AppState>) -> FunctionHandler {
    FunctionHandler::new(move |context: Context, _: &Interrupt| {
        let segments = context.route_tail();
        if segments.len() != 1 {
            return Some(HttpResponse::new(StatusCode::NOT_FOUND));
        }
        let code = &segments[0];
        match state.resolve(code) {
            Some(url) => {
                let response = HttpResponse::new(StatusCode::FOUND)
                    .with_header("location", url)
                    .with_body(Vec::new());
                Some(response)
            }
            None => Some(HttpResponse::new(StatusCode::NOT_FOUND)),
        }
    })
}

pub fn shorten_handler(state: Arc<AppState>) -> FunctionHandler {
    FunctionHandler::new(move |context: Context, _: &Interrupt| {
        let body = context.request().body();
        let url = String::from_utf8_lossy(body)
            .trim()
            .trim_start_matches("url=")
            .to_string();

        if url.is_empty() {
            return Some(HttpResponse::new(StatusCode::BAD_REQUEST));
        }

        state.shorten(url);
        let response = HttpResponse::new(StatusCode::FOUND)
            .with_header("location", "/list".to_string())
            .with_body(Vec::new());
        Some(response)
    })
}

pub fn list_handler(state: Arc<AppState>, env: Environment<'static>) -> FunctionHandler {
    FunctionHandler::new(move |_: Context, _: &Interrupt| {
        let entries = state.list();
        let links: Vec<_> = entries
            .into_iter()
            .map(|(code, url)| minijinja::context! { code, url })
            .collect();

        let tmpl = env.get_template("list").unwrap();
        match tmpl.render(minijinja::context! { links }) {
            Ok(html) => {
                let response = HttpResponse::new(StatusCode::OK)
                    .with_header("content-type", "text/html; charset=utf-8".to_string())
                    .with_body(html.into_bytes());
                Some(response)
            }
            Err(_) => Some(HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR)),
        }
    })
}
