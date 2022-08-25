extern crate chrono;
extern crate log;
extern crate pretty_env_logger;
extern crate prometheus;
extern crate serde;
extern crate serde_json;
extern crate uuid;

use restapi::core::core_config::build_core_config;
use restapi::core::core_config::CoreConfig;
use restapi::core::server::run_server::run_server;

/// main
///
/// Create a [`CoreConfig`](restapi::core::core_config::CoreConfig) and
/// start the server using the configuration. There are
/// many supported environment variables to customize most
/// layers of the stack.
///
/// Feel free to open a github issue to help me figure it out!
///
#[tokio::main]
async fn main() {
    pretty_env_logger::init_timed();

    let label = "server";
    // create the server's config from environment variables
    let core_config: CoreConfig = match build_core_config(&label).await {
        Ok(core_config) => core_config,
        Err(err_msg) => {
            panic!(
                "\
                failed to build core config with err='{err_msg}' \
                stopping"
            );
        }
    };

    run_server(&core_config).await;

    return;
}
