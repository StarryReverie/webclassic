#[cfg(feature = "service")]
pub mod service {
    pub use webclassic_service::interrupt;
    pub use webclassic_service::request;
    pub use webclassic_service::response;

    pub use webclassic_service::interrupt::Interrupt;
    pub use webclassic_service::request::Request;
    pub use webclassic_service::response::Response;
    pub use webclassic_service::service::{RunServiceError, Service};
}

#[cfg(feature = "runtime")]
pub mod runtime {
    pub use webclassic_runtime::ServerOptions;
}

#[cfg(feature = "web")]
pub mod web {
    pub mod protocol {
        pub use webclassic_http::request;
        pub use webclassic_http::response;
        pub use webclassic_http::util;

        pub use webclassic_http::request::HttpRequest;
        pub use webclassic_http::response::HttpResponse;
    }

    pub mod handler {
        #[cfg(feature = "web-handler-core")]
        pub use webclassic_handler_core::constant::ConstantHandler;
        #[cfg(feature = "web-handler-core")]
        pub use webclassic_handler_core::function::FunctionHandler;
        #[cfg(feature = "web-handler-core")]
        pub use webclassic_handler_core::redirect::RedirectHandler;

        #[cfg(feature = "web-handler-static")]
        pub use webclassic_handler_static::directory::StaticDirectoryHandler;
        #[cfg(feature = "web-handler-static")]
        pub use webclassic_handler_static::file::StaticFileHandler;

        #[cfg(feature = "web-handler-cgi")]
        pub use webclassic_handler_cgi::CgiHandler;
    }

    pub mod filter {
        #[cfg(feature = "web-filter-core")]
        pub use webclassic_filter_core::error_page::ErrorPageFilter;
        #[cfg(feature = "web-filter-core")]
        pub use webclassic_filter_core::head::HeadFilter;
    }

    pub use webclassic_web::controller;
    pub use webclassic_web::dispatcher;

    pub use webclassic_web::controller::{Controller, Filter, FilterExt};
    pub use webclassic_web::dispatcher::{Dispatcher, Route};
    pub use webclassic_web::service::WebService;
}
