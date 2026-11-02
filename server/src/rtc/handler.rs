//! What the WebRTC library reports about one peer's connection
//!
//! https://www.w3.org/TR/webrtc/#event-summary

use std::sync::Arc;

use tokio::sync::{mpsc, watch};
use uuid::Uuid;
use webrtc::media_stream::track_local::TrackLocal;
use webrtc::media_stream::track_remote::{TrackRemote, TrackRemoteEvent};
use webrtc::peer_connection::{
    PeerConnectionEventHandler, RTCPeerConnectionIceEvent, RTCPeerConnectionState,
};

use crate::rtc::events::{RTCIceCandidateInit, RtcCloseCode, RtcServerEvent};
use crate::rtc::media::Media;
use crate::rtc::registry::{self, Registry};
use crate::sockets;

// DATA STRUCTS //

pub struct Handler {
    pub user_id: Uuid,
    pub room_id: Uuid,
    pub registry: Registry,
    pub outbound: mpsc::UnboundedSender<String>,
    pub closing: Arc<watch::Sender<Option<RtcCloseCode>>>
}

#[async_trait::async_trait]
impl PeerConnectionEventHandler for Handler {

    /// https://www.w3.org/TR/webrtc/#dom-rtcpeerconnection-onicecandidate
    async fn on_ice_candidate(&self, event: RTCPeerConnectionIceEvent) {
        let init = match event.candidate.to_json() {
            Ok(init) => init,
            Err(e) => {
                tracing::warn!("could not read a candidate for {}: {e}", self.user_id);
                return;
            }
        };

        let candidate = RTCIceCandidateInit {
            candidate: init.candidate,
            sdpMid: init.sdp_mid,
            sdpMLineIndex: init.sdp_mline_index,
            usernameFragment: init.username_fragment
        };

        send(&self.outbound, &RtcServerEvent::Candidate { candidate });
    }

    /// https://www.w3.org/TR/webrtc/#dom-rtcpeerconnection-onconnectionstatechange
    async fn on_connection_state_change(&self, state: RTCPeerConnectionState) {
        if state == RTCPeerConnectionState::Failed {
            end(&self.closing, RtcCloseCode::Normal);
        }
    }

    /// https://www.w3.org/TR/webrtc/#dom-rtcpeerconnection-ontrack
    async fn on_track(&self, track: Arc<dyn TrackRemote>) {
        tokio::spawn(forward(self.registry.clone(), self.room_id, self.user_id, track));
    }
}

// PUBLIC METHODS //

/// Queue an event for a peer's RTC socket
pub fn send(outbound: &mpsc::UnboundedSender<String>, event: &RtcServerEvent) {
    if let Some(text) = sockets::encode(event) {
        let _ = outbound.send(text);
    }
}

/// Ask for a peer's RTC socket to close, keeping the first code given
pub fn end(closing: &watch::Sender<Option<RtcCloseCode>>, code: RtcCloseCode) {
    closing.send_if_modified(|current| {
        if current.is_some() {
            return false;
        }
        *current = Some(code);
        true
    });
}

// HELPER METHODS //

/// Copy a user's incoming media to everyone else in the room until the track ends
async fn forward(
    registry: Registry,
    room_id: Uuid,
    sender_id: Uuid,
    track: Arc<dyn TrackRemote>,
) {
    let mut media: Option<Media> = None;

    while let Some(event) = track.poll().await {
        match event {
            TrackRemoteEvent::OnOpen(init) => {
                if let Some(peer) = registry::peer(&registry, sender_id) {
                    media = peer.media_of(init.receiver_id);
                }
            }
            TrackRemoteEvent::OnRtpPacket(packet) => {
                let Some(media) = media else {
                    continue;
                };

                for peer in registry::peers(&registry, room_id) {
                    if peer.user_id == sender_id {
                        continue;
                    }

                    let Some((track, ssrc)) = peer.sent_from(sender_id, media) else {
                        continue;
                    };

                    let mut packet = packet.clone();
                    packet.header.ssrc = ssrc;

                    if let Err(e) = track.write_rtp(packet).await {
                        tracing::debug!("could not forward {media:?} from {sender_id} to {}: {e}", peer.user_id);
                    }
                }
            }
            _ => {}
        }
    }
}
