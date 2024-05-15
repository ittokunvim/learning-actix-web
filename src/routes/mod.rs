pub mod application;
pub mod server;
pub mod extractors;
pub mod handlers;
pub mod errors;
pub mod url_dispatch;
pub mod testing;

pub use application::init_routes as application_routes;
pub use server::init_routes as server_routes;
pub use extractors::init_routes as extractor_routes;
pub use handlers::init_routes as handler_routes;
pub use errors::init_routes as error_routes;
pub use url_dispatch::init_routes as url_dispatch_routes;
pub use testing::init_routes as testing_routes;
