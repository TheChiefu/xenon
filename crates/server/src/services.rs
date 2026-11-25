//! Outside services the server links to, and the HTTP client they share

#[cfg(any(feature = "push", feature = "links"))]
pub mod client;

#[cfg(feature = "push")]
pub mod push;

#[cfg(feature = "links")]
pub mod links;
