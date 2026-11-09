//! One user in a call: their socket, their connection, and the media forwarded to them

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::sync::Arc;

use tokio::sync::mpsc;
use uuid::Uuid;
use webrtc::error::Result;
use webrtc::peer_connection::{MediaEngine, PeerConnection, PeerConnectionBuilder, Registry};

use ::rtc::interceptor::Slot;
use ::rtc::peer_connection::configuration::interceptor_registry::{
    configure_nack, configure_rtcp_reports, configure_simulcast_extension_headers,
};
use ::rtc::rtp_transceiver::rtp_sender::{RTCRtpCodecParameters, RtpCodecKind};

use crate::rtc::connections::Connections;
use crate::rtc::events::{RTCSessionDescriptionInit, RtcCloseCode};
use crate::rtc::forwarding::Forwarding;
use crate::rtc::handler::Handler;
use crate::rtc::keyframes::Keyframes;
use crate::rtc::link::Link;
use crate::rtc::media;
use crate::rtc::rtcp_forwarder::RtcpForwarder;
use crate::rtc::signal::Signal;

/// After the library's interceptors, so they process inbound RTCP first
const RTCP_FORWARDER_SLOT: usize = Slot::JitterBuffer.slot() + 1_000;

// DATA STRUCTS //

pub struct Peer {
    pub user_id: Uuid,
    pub room_id: Uuid,
    pub signal: Arc<Signal>,
    pub link: Link,
    pub forwarding: Forwarding,
    pub keyframes: Keyframes
}

// PUBLIC METHODS //

/// Open a WebRTC connection for a user on a port, returning the peer and the
/// events its socket writes
pub async fn connect(
    user_id: Uuid,
    room_id: Uuid,
    port: u16,
    connections: Connections,
) -> Result<(Arc<Peer>, mpsc::UnboundedReceiver<String>)> {
    let (signal, receiver) = Signal::new();

    let mut media_engine = MediaEngine::default();
    let audio_codec = RTCRtpCodecParameters {
        rtp_codec: media::opus(),
        payload_type: media::OPUS_PAYLOAD_TYPE
    };
    let video_codec = RTCRtpCodecParameters {
        rtp_codec: media::av1(),
        payload_type: media::AV1_PAYLOAD_TYPE
    };
    media_engine.register_codec(audio_codec, RtpCodecKind::Audio)?;
    media_engine.register_codec(video_codec, RtpCodecKind::Video)?;

    // The default interceptors, without transport-wide congestion control
    let interceptors = configure_nack(Registry::new(), &mut media_engine);
    configure_simulcast_extension_headers(&mut media_engine)?;
    let interceptors = configure_rtcp_reports(interceptors)
        .with(Slot::from(RTCP_FORWARDER_SLOT), RtcpForwarder::default());

    let addresses = vec![
        SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port).to_string(),
        SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), port).to_string()
    ];

    let handler = Handler {
        user_id,
        room_id,
        connections: connections.clone(),
        signal: signal.clone()
    };

    let connection = PeerConnectionBuilder::new()
        .with_media_engine(media_engine)
        .with_interceptor_registry(interceptors)
        .with_handler(Arc::new(handler))
        .with_udp_addrs(addresses)
        .build()
        .await?;
    let connection: Arc<dyn PeerConnection> = Arc::new(connection);

    let peer = Peer {
        user_id,
        room_id,
        signal,
        forwarding: Forwarding::new(connection.clone()).await?,
        keyframes: Keyframes::new(connections),
        link: Link::new(connection)
    };

    Ok((Arc::new(peer), receiver))
}

impl Peer {

    /// Send the user an offer with every stream forwarded to them
    pub async fn negotiate(&self) {
        if let Err(e) = self.link.negotiate(&self.signal, &self.forwarding).await {
            tracing::error!("could not send an offer to {}: {e}", self.user_id);
            self.signal.end(RtcCloseCode::ServerError);
        }
    }

    /// Apply the user's answer, then send any offer needed meanwhile
    pub async fn answer(&self, description: RTCSessionDescriptionInit) {
        match self.link.answer(description).await {
            Ok(true) => self.negotiate().await,
            Ok(false) => {}
            Err(e) => tracing::debug!("could not apply the answer from {}: {e}", self.user_id)
        }
    }
}
