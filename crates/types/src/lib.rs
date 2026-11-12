//! Types sent between the server and its clients

pub mod appearance;
pub mod auth;
pub mod error;
pub mod events;
pub mod messages;
pub mod platforms;
pub mod preferences;
pub mod rooms;
pub mod routes;
pub mod services;
pub mod users;

#[cfg(feature = "rtc")]
pub mod rtc;
