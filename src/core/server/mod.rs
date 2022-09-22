//! Modules to start the rest server and pass a
//! [`struct CoreHttpRequest`](crate::core::server::core_http_request::CoreHttpRequest)
//! to all hyper worker threads when an HTTP request is received
//!
pub mod core_http_request;
pub mod core_services;
pub mod run_server;
pub mod start_core_server;
