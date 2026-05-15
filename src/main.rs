use std::net::TcpListener;
use std::num::NonZero;
use std::sync::atomic::{AtomicUsize, Ordering};

use webclassic_handler_core::constant::ConstantHandler;
use webclassic_handler_core::function::FunctionHandler;
use webclassic_handler_core::redirect::RedirectHandler;
use webclassic_http::response::HttpResponse;
use webclassic_http::util::{Method, StatusCode};
use webclassic_runtime::ServerOptions;
use webclassic_service::interrupt::Interrupt;
use webclassic_web::controller::{Context, Controller, EmptyController, Filter, FilterExt};
use webclassic_web::dispatcher::{Dispatcher, Route};
use webclassic_web::service::WebService;

fn hello_handler() -> FunctionHandler {
    FunctionHandler::new(|_ctx, _int| {
        Some(
            HttpResponse::new(StatusCode::OK)
                .with_header("content-type", "text/plain".to_string())
                .with_body(b"Hello, World!".to_vec()),
        )
    })
}

fn greet_handler() -> FunctionHandler {
    FunctionHandler::new(|ctx: Context, _int: &Interrupt| {
        let name = ctx
            .request()
            .uri()
            .query()
            .and_then(|q| q.get("name"))
            .and_then(|v| v.first().cloned())
            .unwrap_or_else(|| "stranger".to_string());

        let body = format!("Hello, {}!", name);
        Some(
            HttpResponse::new(StatusCode::OK)
                .with_header("content-type", "text/plain".to_string())
                .with_body(body.into_bytes()),
        )
    })
}

fn echo_handler() -> FunctionHandler {
    FunctionHandler::new(|ctx: Context, _int: &Interrupt| {
        Some(
            HttpResponse::new(StatusCode::OK)
                .with_header("content-type", "application/octet-stream".to_string())
                .with_body(ctx.request().body().to_vec()),
        )
    })
}

struct LoggingFilter {
    counter: AtomicUsize,
}

impl LoggingFilter {
    fn new() -> Self {
        Self {
            counter: AtomicUsize::new(0),
        }
    }
}

impl Filter for LoggingFilter {
    fn filter<C: Controller>(
        &self,
        controller: &C,
        context: Context,
        interrupt: &Interrupt,
    ) -> Option<HttpResponse> {
        let n = self.counter.fetch_add(1, Ordering::Relaxed) + 1;
        eprintln!(
            "[{}] {} {}",
            n,
            context.request().method(),
            context.request().uri().path()
        );
        controller.process(context, interrupt)
    }
}

struct MethodOverrideFilter;

impl Filter for MethodOverrideFilter {
    fn filter<C: Controller>(
        &self,
        controller: &C,
        context: Context,
        interrupt: &Interrupt,
    ) -> Option<HttpResponse> {
        eprintln!(
            "  method-override filter active for {}",
            context.request().uri().path()
        );
        controller.process(context, interrupt)
    }
}

fn main() {
    let log = LoggingFilter::new();

    let controller = Dispatcher::new()
        .with(Route::by(Method::Get).equal("/"), hello_handler())
        .with(Route::by(Method::Get).equal("/greet"), greet_handler())
        .with(
            Route::by(Method::Post).equal("/echo"),
            echo_handler().filtered(MethodOverrideFilter),
        )
        .with(
            Route::by(Method::Get).equal("/health"),
            ConstantHandler::new("ok".to_string()),
        )
        .with(
            Route::by(Method::Get).equal("/old"),
            RedirectHandler::permanent("/".to_string()),
        )
        .with(
            Route::by(Method::Get).prefix("/api"),
            ConstantHandler::new("api".to_string()).filtered(log),
        )
        .fallback(EmptyController::new());

    let service = WebService::new(controller);

    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();

    ServerOptions::new(service)
        .max_connections(NonZero::new(64).unwrap())
        .max_pending(NonZero::new(256).unwrap())
        .serve(listener);
}
