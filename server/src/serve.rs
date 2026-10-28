use std::net::SocketAddr;
use std::time::Duration;

use axum::Router;
use axum_server::tls_rustls::RustlsConfig;

use crate::cli;
use crate::config;

/// How long open connections are given to finish once shutdown begins
const SHUTDOWN_GRACE: Duration = Duration::from_secs(10);

/// Serve over TLS using the configured certificate and key
/// Exits the process if either file cannot be read or parsed.
pub async fn tls(app: Router, bind_addr: SocketAddr) {
    let certificate = config::get().bind.certificate.as_ref().expect("certificate checked by caller");
    let key = config::get().bind.key.as_ref().expect("key checked by caller");

    let tls = match RustlsConfig::from_pem_file(certificate, key).await {
        Ok(tls) => tls,
        Err(e) => {
            eprintln!("could not load certificate \"{certificate}\" and key \"{key}\": {e}");
            std::process::exit(1);
        }
    };

    // Shutdown driven through handle so signal is watched from its own task
    let handle = axum_server::Handle::new();
    tokio::spawn({
        let handle = handle.clone();
        async move {
            shutdown_signal().await;
            handle.graceful_shutdown(Some(SHUTDOWN_GRACE));
        }
    });

    let listener = std::net::TcpListener::bind(bind_addr).expect("failed to bind port to listener");
    listener.set_nonblocking(true).expect("failed to set listener non-blocking");

    tracing::info!("listening on {bind_addr} over TLS");
    cli::READY.notify_one();

    axum_server::from_tcp_rustls(listener, tls)
        .expect("failed to accept on bound listener")
        .handle(handle)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .expect("failed to serve axum application");
}

/// Serve unencrypted (warning if the address is reachable off the host)
pub async fn plaintext(app: Router, bind_addr: SocketAddr) {
    if !config::get().binds_loopback() {
        tracing::warn!(
            "bind.ip is not a loopback address and no certificate is configured, \
             traffic is unencrypted."
        );
    }

    let listener = tokio::net::TcpListener::bind(bind_addr)
        .await
        .expect("failed to bind port to listener");

    tracing::info!("listening on {bind_addr}");
    cli::READY.notify_one();

    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("failed to serve axum application");
}

// Shutdown //

/// Resolve on SIGTERM
#[cfg(unix)]
async fn terminate() {
    use tokio::signal::unix::{signal, SignalKind};
    signal(SignalKind::terminate())
        .expect("failed to install SIGTERM handler")
        .recv()
        .await;
    tracing::info!("Received \"SIGTERM\", terminating program...");
}

#[cfg(not(unix))]
async fn terminate() {
    std::future::pending::<()>().await;
}

/// Resolve on the first shutdown signal
/// Axum polls this alongside serving
async fn shutdown_signal() {
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {},
        _ = terminate() => {},
        _ = cli::EXIT.notified() => {},
    }
    tracing::info!("Shutting down server");
}
