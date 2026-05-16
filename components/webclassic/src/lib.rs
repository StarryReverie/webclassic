pub mod runtime {
    pub use webclassic_runtime::ServerOptions;
}

pub mod service {
    pub use webclassic_service::interrupt;
    pub use webclassic_service::request;
    pub use webclassic_service::response;

    pub use webclassic_service::interrupt::Interrupt;
    pub use webclassic_service::request::Request;
    pub use webclassic_service::response::Response;
    pub use webclassic_service::service::{RunServiceError, Service};
}

pub mod web {
    pub mod protocol {
        pub use webclassic_http::request;
        pub use webclassic_http::response;
        pub use webclassic_http::util;

        pub use webclassic_http::request::HttpRequest;
        pub use webclassic_http::response::HttpResponse;
    }

    pub mod handler {
        pub use webclassic_handler_cgi::CgiHandler;
        pub use webclassic_handler_core::constant::ConstantHandler;
        pub use webclassic_handler_core::function::FunctionHandler;
        pub use webclassic_handler_core::redirect::RedirectHandler;
        pub use webclassic_handler_static::directory::StaticDirectoryHandler;
        pub use webclassic_handler_static::file::StaticFileHandler;
    }

    pub use webclassic_web::controller;
    pub use webclassic_web::dispatcher;

    pub use webclassic_web::controller::{Controller, Filter, FilterExt};
    pub use webclassic_web::dispatcher::{Dispatcher, Route};
    pub use webclassic_web::service::WebService;
}
