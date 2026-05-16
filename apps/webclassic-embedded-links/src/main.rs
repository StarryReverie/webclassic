use std::net::TcpListener;
use std::sync::Arc;

use webclassic::runtime::ServerOptions;
use webclassic::web::handler::FunctionHandler;
use webclassic::web::protocol::HttpResponse;
use webclassic::web::protocol::util::{Method, StatusCode};
use webclassic::web::{Dispatcher, Route, WebService};

use webclassic_embedded_links::state::AppState;

fn main() {
    let _state = Arc::new(AppState::new());

    let service = WebService::new(Dispatcher::new().with(
        Route::by(Method::Get).equal("/"),
        FunctionHandler::new(move |_ctx, _interrupt| Some(HttpResponse::new(StatusCode::OK))),
    ));

    let listener = TcpListener::bind("127.0.0.1:3000").unwrap();
    ServerOptions::new(service).serve(listener);
}
