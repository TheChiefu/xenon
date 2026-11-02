//! One user's WebRTC connection to the server
//!
//! https://www.w3.org/TR/webrtc/#rtcpeerconnection-interface

use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::sync::{Arc, Mutex};

use argon2::password_hash::rand_core::{OsRng, RngCore};
use tokio::sync::{mpsc, watch};
use uuid::Uuid;
use webrtc::error::Result;
use webrtc::media_stream::track_local::TrackLocal;
use webrtc::media_stream::track_local::static_rtp::TrackLocalStaticRTP;
use webrtc::peer_connection::{
    register_default_interceptors, MediaEngine, PeerConnection, PeerConnectionBuilder,
    RTCSessionDescription, Registry,
};
use webrtc::rtp_transceiver::{
    RtpSender, RTCRtpReceiverId, RTCRtpTransceiverDirection, RTCRtpTransceiverInit,
};

use ::rtc::rtp_transceiver::rtp_sender::{RTCRtpCodecParameters, RtpCodecKind};

use crate::rtc::events::{
    RTCIceCandidateInit, RTCSdpType, RTCSessionDescriptionInit, RtcCloseCode, RtcServerEvent,
};
use crate::rtc::handler::{self, Handler};
use crate::rtc::media::{self, Media};
use crate::rtc::connections::Connections;
use crate::utils;

// DATA STRUCTS //

/// One track carrying another user's media to this peer
struct Sent {
    track: Arc<TrackLocalStaticRTP>,
    sender: Arc<dyn RtpSender>,
    ssrc: u32
}

/// https://www.w3.org/TR/webrtc/#rtcsignalingstate-enum
#[derive(Clone, Copy, PartialEq, Eq)]
enum Negotiation {
    Idle,
    AwaitingAnswer,
    AwaitingAnswerThenOffer
}

/// One user's connection to the server, and the media forwarded to them from the others in the room
pub struct Peer {
    pub user_id: Uuid,
    pub room_id: Uuid,
    connection: Arc<dyn PeerConnection>,
    receivers: HashMap<RTCRtpReceiverId, Media>,
    sent: Mutex<HashMap<(Uuid, Media), Sent>>,
    outbound: mpsc::UnboundedSender<String>,
    closing: Arc<watch::Sender<Option<RtcCloseCode>>>,
    negotiation: Mutex<Negotiation>
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
    let (outbound, receiver) = mpsc::unbounded_channel();
    let (closing, _) = watch::channel(None);
    let closing = Arc::new(closing);

    let mut media_engine = MediaEngine::default();
    let audio_codec = RTCRtpCodecParameters {
        rtp_codec: media::opus(),
        payload_type: media::OPUS_PAYLOAD_TYPE
    };
    let video_codec = RTCRtpCodecParameters {
        rtp_codec: media::vp8(),
        payload_type: media::VP8_PAYLOAD_TYPE
    };
    media_engine.register_codec(audio_codec, RtpCodecKind::Audio)?;
    media_engine.register_codec(video_codec, RtpCodecKind::Video)?;
    let interceptors = register_default_interceptors(Registry::new(), &mut media_engine)?;

    let addresses = vec![
        SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port).to_string(),
        SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), port).to_string()
    ];

    let handler = Handler {
        user_id,
        room_id,
        connections,
        outbound: outbound.clone(),
        closing: closing.clone()
    };

    let connection = PeerConnectionBuilder::new()
        .with_media_engine(media_engine)
        .with_interceptor_registry(interceptors)
        .with_handler(Arc::new(handler))
        .with_udp_addrs(addresses)
        .build()
        .await?;
    let connection: Arc<dyn PeerConnection> = Arc::new(connection);

    let media = Media::Microphone;
    let receive = RTCRtpTransceiverInit {
        direction: RTCRtpTransceiverDirection::Recvonly,
        streams: Vec::new(),
        send_encodings: Vec::new()
    };
    let transceiver = connection.add_transceiver_from_kind(media.kind(), Some(receive)).await?;
    let receivers = HashMap::from([(RTCRtpReceiverId::from(transceiver.id()), media)]);

    let peer = Peer {
        user_id,
        room_id,
        connection,
        receivers,
        sent: Mutex::new(HashMap::new()),
        outbound,
        closing,
        negotiation: Mutex::new(Negotiation::Idle)
    };

    Ok((Arc::new(peer), receiver))
}

impl Peer {

    /// Add a track carrying another user's microphone
    pub async fn add_sender(&self, sender_id: Uuid) -> Result<()> {
        let media = Media::Microphone;
        let sent = self.send(sender_id, media).await?;
        utils::mutex_lock(&self.sent).insert((sender_id, media), sent);

        Ok(())
    }

    /// Remove the tracks carrying another user's media
    pub async fn remove_sender(&self, sender_id: Uuid) {
        for media in Media::ALL {
            let Some(sent) = utils::mutex_lock(&self.sent).remove(&(sender_id, media)) else {
                continue;
            };

            if let Err(e) = self.connection.remove_track(&sent.sender).await {
                tracing::warn!("could not remove {sender_id}'s {media:?} from {}: {e}", self.user_id);
            }
        }
    }

    /// The media a receiver of this peer's connection carries
    pub fn media_of(&self, receiver_id: RTCRtpReceiverId) -> Option<Media> {
        self.receivers.get(&receiver_id).copied()
    }

    /// The track carrying a user's media to this peer, and its SSRC
    pub fn sent_from(&self, sender_id: Uuid, media: Media) -> Option<(Arc<TrackLocalStaticRTP>, u32)> {
        let sent = utils::mutex_lock(&self.sent);
        let found = sent.get(&(sender_id, media))?;
        Some((found.track.clone(), found.ssrc))
    }

    /// Send an offer, or send one once the offer waiting is answered
    /// https://www.w3.org/TR/webrtc/#dom-rtcpeerconnection-createoffer
    pub async fn negotiate(&self) {
        if !self.start_offer() {
            return;
        }

        let offer = match self.connection.create_offer(None).await {
            Ok(offer) => offer,
            Err(e) => {
                tracing::error!("could not create an offer for {}: {e}", self.user_id);
                self.end(RtcCloseCode::ServerError);
                return;
            }
        };

        let sdp = offer.sdp.clone();

        if let Err(e) = self.connection.set_local_description(offer).await {
            tracing::error!("could not set the offer for {}: {e}", self.user_id);
            self.end(RtcCloseCode::ServerError);
            return;
        }

        let description = RTCSessionDescriptionInit {
            r#type: RTCSdpType::offer,
            sdp
        };
        handler::send(&self.outbound, &RtcServerEvent::Offer { description });
    }

    /// Apply the browser's answer, then send any offer needed meanwhile
    /// https://www.w3.org/TR/webrtc/#dom-rtcpeerconnection-setremotedescription
    pub async fn answer(&self, description: RTCSessionDescriptionInit) {
        if description.r#type != RTCSdpType::answer {
            return;
        }

        let answer = match RTCSessionDescription::answer(description.sdp) {
            Ok(answer) => answer,
            Err(e) => {
                tracing::debug!("{} sent an unreadable answer: {e}", self.user_id);
                return;
            }
        };

        if let Err(e) = self.connection.set_remote_description(answer).await {
            tracing::debug!("could not apply the answer from {}: {e}", self.user_id);
            return;
        }

        let previous = std::mem::replace(&mut *utils::mutex_lock(&self.negotiation), Negotiation::Idle);

        if previous == Negotiation::AwaitingAnswerThenOffer {
            Box::pin(self.negotiate()).await;
        }
    }

    /// https://www.w3.org/TR/webrtc/#dom-rtcpeerconnection-addicecandidate
    pub async fn add_candidate(&self, init: RTCIceCandidateInit) {
        let init = webrtc::peer_connection::RTCIceCandidateInit {
            candidate: init.candidate,
            sdp_mid: init.sdpMid,
            sdp_mline_index: init.sdpMLineIndex,
            username_fragment: init.usernameFragment,
            ..Default::default()
        };

        if let Err(e) = self.connection.add_ice_candidate(init).await {
            tracing::debug!("could not add a candidate from {}: {e}", self.user_id);
        }
    }

    /// Ask for the peer's RTC socket to close, keeping the first code given
    pub fn end(&self, code: RtcCloseCode) {
        handler::end(&self.closing, code);
    }

    /// Wait until the peer's RTC socket should close, returning the code to close with
    pub async fn ended(&self) -> RtcCloseCode {
        let mut receiver = self.closing.subscribe();

        match receiver.wait_for(Option::is_some).await {
            Ok(code) => code.unwrap_or(RtcCloseCode::Normal),
            Err(_) => RtcCloseCode::Normal
        }
    }

    pub async fn close(&self) {
        if let Err(e) = self.connection.close().await {
            tracing::warn!("could not close the connection of {}: {e}", self.user_id);
        }
    }

    // HELPER METHODS //

    /// Add a track carrying one of another user's media
    async fn send(&self, sender_id: Uuid, media: Media) -> Result<Sent> {
        let ssrc = OsRng.next_u32();
        let track = Arc::new(TrackLocalStaticRTP::new(media::track_of(sender_id, media, ssrc)));

        // A new transceiver per track, as add_track would reuse one of this peer's own slots
        // https://www.w3.org/TR/webrtc/#dom-rtcpeerconnection-addtransceiver
        let send = RTCRtpTransceiverInit {
            direction: RTCRtpTransceiverDirection::Sendonly,
            streams: Vec::new(),
            send_encodings: Vec::new()
        };
        let transceiver = self.connection
            .add_transceiver_from_track(track.clone() as Arc<dyn TrackLocal>, Some(send))
            .await?;

        let Some(sender) = transceiver.sender().await? else {
            return Err(webrtc::error::Error::Other(format!("no sender for {sender_id}'s {media:?}")));
        };

        Ok(Sent {
            track,
            sender,
            ssrc
        })
    }

    /// Whether an offer can be sent now, otherwise marks one as needed after the answer
    fn start_offer(&self) -> bool {
        let mut negotiation = utils::mutex_lock(&self.negotiation);

        match *negotiation {
            Negotiation::Idle => {
                *negotiation = Negotiation::AwaitingAnswer;
                true
            }
            _ => {
                *negotiation = Negotiation::AwaitingAnswerThenOffer;
                false
            }
        }
    }
}
