//! The HTTP router and the auth extractor.

mod auth;
mod files;
pub mod messages;
mod rooms;
mod search;
mod server;
mod users;

use std::net::{IpAddr, SocketAddr};
use std::time::Duration;

use axum::extract::{ConnectInfo, DefaultBodyLimit, FromRef, FromRequestParts, Request, State};
use axum::http;
use axum::http::header::{AUTHORIZATION, CONTENT_TYPE};
use axum::http::request::Parts;
use axum::http::Method;
use axum::middleware::{self, Next};
use axum::response::Response;
use axum::routing::{delete, get, patch, post, put};
use axum::Router;
use sqlx::SqlitePool;
use tower_http::cors::{AllowOrigin, CorsLayer};
use uuid::Uuid;

use crate::config;
use crate::db;
use crate::error::{AppError, Result};
use crate::ratelimit::AddressLimits;
use crate::sockets::{call, chat};
use crate::state::AppState;
use crate::utils;

/// The authenticated caller.
///
/// Declared as a handler parameter, so axum authenticates before the body runs.
pub struct AuthUser(
    /// Id of the user the session belongs to
    pub Uuid,

    /// The `sessions.token_hash` the request authenticated with
    pub [u8; 32],
);

/// Generic over the router's state, so it survives the state type changing.
impl<S> FromRequestParts<S> for AuthUser
where
    SqlitePool: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self> {

        let token = bearer_token(parts)?;

        // Not a token this server could have issued
        let session_hash = utils::hash_session_token(&token)
            .ok_or(AppError::InvalidCredentials)?;

        // Authenticate
        let pool = SqlitePool::from_ref(state);
        let mut conn = pool.acquire().await?;
        let user_id = db::authenticate(&mut conn, &session_hash)
            .await?
            .ok_or(AppError::InvalidCredentials)?;

        Ok(AuthUser(user_id, session_hash))
    }
}

/// Address of the client that sent the request
pub struct ClientIp(pub IpAddr);

impl<S> FromRequestParts<S> for ClientIp
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self> {

        // Connecting IP, where an IPv6 listener reports an IPv4 client as
        // ::ffff:a.b.c.d and it is converted to a.b.c.d
        let peer = parts.extensions.get::<ConnectInfo<SocketAddr>>()
            .expect("router served with connect info")
            .0
            .ip()
            .to_canonical();

        // Not from a listed proxy, return the client's own IP
        if !config::get().bind.trusted_proxies.contains(&peer) {
            return Ok(ClientIp(peer));
        }

        // Last address in the last X-Forwarded-For header
        let forwarded = parts.headers.get_all("x-forwarded-for").iter().last()
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.rsplit(',').next())
            .and_then(|address| address.trim().parse::<IpAddr>().ok());

        // Forwarded client IP, or the proxy's IP if none was read
        Ok(ClientIp(forwarded.map_or(peer, |address| address.to_canonical())))
    }
}

// Main Router //

/// Builds the router carrying every route on the server.
///
/// # Arguments
///
/// * `state` - Pool and socket registry.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/login", post(auth::login))
        .route("/codes",
            post(auth::create_registration_code)
            .get(auth::list_registration_codes)
            .delete(auth::revoke_registration_code)
        )

        // Me
        .route("/me",
            get(users::get_me)
            .patch(users::update_me)
            .delete(users::delete_me)
        )
        .route("/me/password", patch(users::update_my_password))
        .route("/me/session", delete(users::sign_out))
        .route("/me/preferences",
            get(users::get_preferences)
            .patch(users::update_preferences)
        )
        .route("/me/rooms", get(rooms::list_my_rooms))
        .route("/me/read", get(rooms::list_my_reads))
        .route("/me/invites", get(rooms::list_my_invites))
        .route("/me/invites/{room_id}", delete(rooms::decline_invite))

        // Rooms
        .route("/rooms",
            post(rooms::create_room)
            .get(rooms::list_discoverable_rooms)
        )
        .route("/rooms/{id}",
            get(rooms::get_room)
            .delete(rooms::delete_room)
            .patch(rooms::update)
        )
        .route("/rooms/{id}/join", post(rooms::join_room))
        .route("/rooms/{id}/leave", post(rooms::leave_room))
        .route("/rooms/{id}/read", put(rooms::mark_read))
        .route("/rooms/{id}/messages",
            post(messages::post_message)
            .get(messages::fetch_messages)
        )
        .route("/rooms/{id}/invites", get(rooms::list_invites))
        .route("/rooms/{id}/invites/{user_id}",
            put(rooms::invite_user)
            .delete(rooms::revoke_invite)
        )
        .route("/rooms/{id}/members", get(rooms::list_members))
        .route("/rooms/{id}/members/{user_id}",
            get(rooms::get_member)
            .patch(rooms::patch)
        )
        .route("/rooms/{id}/bans", get(rooms::list_bans))
        .route("/rooms/{id}/bans/{user_id}",
            put(rooms::ban_user)
            .delete(rooms::unban_user)
        )
        .route("/rooms/{id}/call", get(call::socket::ws_handler))

        // Messages
        .route("/messages/{id}",
            get(messages::get_message)
            .delete(messages::delete_message)
            .patch(messages::update_message)
        )

        // Search (QUERY once axum supports it, see /users/lookup)
        .route("/search", post(search::messages))

        // Files
        .route("/files",
            post(files::upload)
            .layer(DefaultBodyLimit::max(files::max_body_bytes()))
        )
        .route("/files/{id}", get(files::download))

        // Users
        //
        // POST /users/lookup looks up users by id or by username prefix. It
        // belongs on QUERY /users: QUERY is the RFC 10008 method (June 2026), a
        // read like GET that carries a body like POST, which this lookup needs
        // since a large room's ids overflow a URL.
        //
        // axum 0.8.9 has no QUERY routing, as MethodFilter only knows the nine
        // older methods. QUERY routing is merged into axum main (tokio-rs/axum#3801)
        // and expected in axum 0.9. Once upgraded, move this to axum's `query` on
        // /users and point the client at QUERY /users.
        //
        // CORS will have to list QUERY as well, see cors_layer.
        .route("/users", post(auth::register))
        .route("/users/lookup", post(users::get_users))
        .route("/users/{id}",
            get(users::get_user)
            .delete(users::delete_user)
        )
        .route("/users/{id}/role", patch(users::set_role))
        .route("/users/{id}/ban",
            put(users::ban_user)
            .delete(users::unban_user)
        )

        // Other
        .route("/ws", get(chat::socket::ws_handler))
        .route("/server", get(server::info))
        .route("/server/version", get(server::version))
        .route("/server/type", get(server::kind))
        .route("/health", get(server::health_check))
        .merge(crate::services::push::router())
        .merge(crate::services::xbox::router())
        .layer(middleware::from_fn_with_state(state.clone(), refuse_blocked))
        .with_state(state)
        .layer(cors_layer())
}

/// Refuses a request from a blocked IP before any route runs
async fn refuse_blocked(
    ClientIp(ip): ClientIp,
    State(address_limits): State<AddressLimits>,
    request: Request,
    next: Next,
) -> Result<Response> {
    address_limits.check(ip)?;
    Ok(next.run(request).await)
}

// Helper Methods //

/// Reports whether an origin may read the server's responses.
fn origin_allowed(origin: &http::HeaderValue, _: &Parts) -> bool {
    let config = config::get();
    let Ok(origin) = origin.to_str() else { return false };

    // A wildcard entry allows any origin
    if config.bind.allowed_origins.iter().any(|allowed| allowed == "*") {
        return true;
    }
    // An explicitly listed origin is allowed
    if config.bind.allowed_origins.iter().any(|allowed| allowed == origin) {
        return true;
    }
    // A server unreachable from any other machine allows any local origin
    if config.binds_loopback() {
        let Some((_, host)) = origin.split_once("://") else { return false };
        let host = host.split(':').next().unwrap_or(host);
        if host == "localhost" || host == "127.0.0.1" {
            return true;
        }
    }
    false
}

fn cors_layer() -> CorsLayer {
    let max_age = config::get().bind.cors_max_age;
    CorsLayer::new()
        .allow_origin(AllowOrigin::predicate(origin_allowed))
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::PUT, Method::DELETE])
        .allow_headers([AUTHORIZATION, CONTENT_TYPE])
        .expose_headers([files::FILE_MIME_HEADER])
        .max_age(Duration::from_secs(max_age))
}

/// Reads the session token a request presents.
///
/// # Arguments
///
/// * `parts` - Head of the request being authenticated.
///
/// # Errors
///
/// Returns `AppError::InvalidCredentials` if no token is presented, or if more
/// than one Authorization header is.
fn bearer_token(parts: &Parts) -> Result<String> {

    let mut values = parts.headers.get_all(http::header::AUTHORIZATION).iter();
    let first = values.next();
    let second = values.next();

    // Repeated headers are joined by HTTP, so two values means two credentials were presented
    // Reject if headers are combined, rather than pick one
    match (first, second) {
        (Some(header), None) => Ok(header
            .to_str()
            .ok()
            .and_then(|value| value.strip_prefix("Bearer "))
            .ok_or(AppError::InvalidCredentials)?
            .to_string()),

        // WebSocket handshakes carry the token as a subprotocol, since a
        // browser cannot set Authorization on one
        (None, None) => parts.headers.get(http::header::SEC_WEBSOCKET_PROTOCOL)
            .and_then(token_from_protocol)
            .ok_or(AppError::InvalidCredentials),

        _ => Err(AppError::InvalidCredentials),
    }
}

/// Reads the session token from a handshake's offered subprotocols, which the
/// client sends as `Bearer, <token>` to mirror the Authorization header.
///
/// # Arguments
///
/// * `header` - The `Sec-WebSocket-Protocol` header value.
fn token_from_protocol(header: &http::HeaderValue) -> Option<String> {
    let mut offered = header.to_str().ok()?.split(',').map(str::trim);
    if offered.next()? != "Bearer" {
        return None;
    }
    let token = offered.next()?;
    (!token.is_empty()).then(|| token.to_string())
}
