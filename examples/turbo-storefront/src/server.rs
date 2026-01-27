//! Server-side rendering for Spin/WASI.

use leptos::{
    config::get_configuration,
    task::Executor as LeptosExecutor
};
use leptos_wasi::{
    handler::HandlerError,
    prelude::{IncomingRequest, ResponseOutparam, WasiExecutor},
};
use wasi::exports::http::incoming_handler::Guest;
use wasi::http::proxy::export;

use crate::app::{shell, App, GetProducts, GetProduct};

struct TurboServer;

impl Guest for TurboServer {
    fn handle(request: IncomingRequest, response_out: ResponseOutparam) {
        let executor = WasiExecutor::new(leptos_wasi::executor::Mode::Stalled);
        if let Err(e) = LeptosExecutor::init_local_custom_executor(executor.clone()) {
            eprintln!("Executor init error: {e:?}");
            return;
        }
        executor.run_until(async {
            if let Err(e) = handle_request(request, response_out).await {
                eprintln!("Request error: {e:?}");
            }
        })
    }
}

async fn handle_request(
    request: IncomingRequest,
    response_out: ResponseOutparam,
) -> Result<(), HandlerError> {
    use leptos_wasi::prelude::Handler;

    let conf = get_configuration(None).unwrap();
    let leptos_options = conf.leptos_options;

    Handler::build(request, response_out)?
        // Register all server functions
        .with_server_fn::<GetProducts>()
        .with_server_fn::<GetProduct>()
        // Generate routes from App
        .generate_routes(App)
        // Handle with shell
        .handle_with_context(move || shell(leptos_options.clone()), || {})
        .await?;

    Ok(())
}

export!(TurboServer with_types_in wasi);
