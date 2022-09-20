use crate::core::core_config::CoreConfig;
use crate::core::server::start_core_server::start_core_server;

/// run_server
///
/// v1 (current version) - Wrapper around
///    [`start_core_server`](crate::core::server::start_core_server::start_core_server)
///
/// v2 - State machine in a `loop` for flushing
///      caches and connectivity (postgres client db threadpool)
///
/// # Arguments
///
/// * `config` - [`CoreConfig`](crate::core::core_config::CoreConfig)
///
pub async fn run_server(config: &CoreConfig) -> bool {
    // boot up the server
    match start_core_server(config).await {
        Ok(_) => {
            info!("{} - run_server.start_core_server done", config.label);
            true
        }
        Err(hyper_error) => {
            let err_msg = format!("{hyper_error}");
            panic!(
                "{} - run_server.start_core_server failed with \
                err='{err_msg}'",
                config.label
            );
        }
    };
    false
}
