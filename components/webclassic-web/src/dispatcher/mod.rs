#[allow(clippy::module_inception)]
mod dispatcher;
mod route;

pub use dispatcher::Dispatcher;
pub use route::{MatchMetric, PathPattern, Route, RouteBuilder};
