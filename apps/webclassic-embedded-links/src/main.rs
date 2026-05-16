use std::net::TcpListener;

use webclassic::runtime::ServerOptions;
use webclassic::web::handler::FunctionHandler;
use webclassic::web::protocol::HttpResponse;
use webclassic::web::protocol::util::{Method, StatusCode};
use webclassic::web::{Dispatcher, Route, WebService};

fn main() {
    let service = WebService::new(Dispatcher::new().with(
        Route::by(Method::Get).equal("/"),
        FunctionHandler::new(|_ctx, _interrupt| Some(HttpResponse::new(StatusCode::OK))),
    ));

    let listener = TcpListener::bind("127.0.0.1:3000").unwrap();
    ServerOptions::new(service).serve(listener);
}
