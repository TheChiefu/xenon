//! What the WebRTC library reports about one peer's connection
//!
//! https://www.w3.org/TR/webrtc/#event-summary

use std::net::IpAddr;
use std::sync::Arc;

use tokio::sync::{mpsc, watch};
use url::{Host, Url};
use uuid::Uuid;
use webrtc::media_stream::track_local::TrackLocal;
use webrtc::media_stream::track_remote::{TrackRemote, TrackRemoteEvent};
use webrtc::peer_connection::{
    PeerConnectionEventHandler, RTCIceCandidate, RTCPeerConnectionIceEvent, RTCPeerConnectionState,
};

use crate::config;
use crate::rtc::connections::{self, Connections};
use crate::rtc::events::{RTCIceCandidateInit, RtcCloseCode, RtcServerEvent};
use crate::rtc::media::Media;
use crate::sockets;

// DATA STRUCTS //

pub struct Handler {
    pub user_id: Uuid,
    pub room_id: Uuid,
    pub connections: Connections,
    pub outbound: mpsc::UnboundedSender<String>,
    pub closing: Arc<watch::Sender<Option<RtcCloseCode>>>
}

#[async_trait::async_trait]
impl PeerConnectionEventHandler for Handler {

    /// Send the browser this server address, and a copy with the public IP for callers outside the server's network
    /// https://www.w3.org/TR/webrtc/#dom-rtcpeerconnection-onicecandidate
    async fn on_ice_candidate(&self, event: RTCPeerConnectionIceEvent) {
        self.send_candidate(&event.candidate);

        let Ok(host) = event.candidate.address.parse::<IpAddr>() else {
            return;
        };

        for address in public_addresses().await {
            if address.is_ipv4() == host.is_ipv4() && address != host {
                let mut candidate = event.candidate.clone();
                candidate.address = address.to_string();
                self.send_candidate(&candidate);
            }
        }
    }

    /// https://www.w3.org/TR/webrtc/#dom-rtcpeerconnection-onconnectionstatechange
    async fn on_connection_state_change(&self, state: RTCPeerConnectionState) {
        if state == RTCPeerConnectionState::Failed {
            end(&self.closing, RtcCloseCode::Normal);
        }
    }

    /// https://www.w3.org/TR/webrtc/#dom-rtcpeerconnection-ontrack
    async fn on_track(&self, track: Arc<dyn TrackRemote>) {
        tokio::spawn(forward(self.connections.clone(), self.room_id, self.user_id, track));
    }
}

impl Handler {

    /// Send a candidate to the browser
    fn send_candidate(&self, candidate: &RTCIceCandidate) {
        let init = match candidate.to_json() {
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
    connections: Connections,
    room_id: Uuid,
    sender_id: Uuid,
    track: Arc<dyn TrackRemote>,
) {
    let mut media: Option<Media> = None;

    while let Some(event) = track.poll().await {
        match event {
            TrackRemoteEvent::OnOpen(init) => {
                if let Some(peer) = connections::peer(&connections, sender_id) {
                    media = peer.media_of(init.receiver_id);
                }
            }
            TrackRemoteEvent::OnRtpPacket(packet) => {
                let Some(media) = media else {
                    continue;
                };

                for peer in connections::in_room(&connections, room_id) {
                    if peer.user_id == sender_id {
                        continue;
                    }

                    let Some((track, ssrc)) = peer.sent_from(sender_id, media) else {
                        continue;
                    };

                    let mut packet = packet.clone();
                    packet.header.ssrc = ssrc;

                    // Extension ids are negotiated per connection, and the sender's are
                    // rejected on the recipient's connection
                    // https://www.rfc-editor.org/rfc/rfc8285#section-5
                    packet.header.extension = false;
                    packet.header.extension_profile = 0;
                    packet.header.extensions.clear();

                    if let Err(e) = track.write_rtp(packet).await {
                        tracing::debug!("could not forward {media:?} from {sender_id} to {}: {e}", peer.user_id);
                    }
                }
            }
            _ => {}
        }
    }
}

/// Look up the server's public IPv4 and IPv6 addresses
async fn public_addresses() -> Vec<IpAddr> {
    let Some(public_url) = &config::get().network.public_url else {
        return Vec::new();
    };

    let Ok(url) = Url::parse(public_url) else {
        return Vec::new();
    };

    match url.host() {
        Some(Host::Ipv4(address)) => vec![IpAddr::V4(address)],
        Some(Host::Ipv6(address)) => vec![IpAddr::V6(address)],
        Some(Host::Domain(domain)) => match tokio::net::lookup_host((domain, 0)).await {
            Ok(addresses) => addresses.map(|address| address.ip()).collect(),
            Err(e) => {
                tracing::error!("could not resolve network.public_url, advertising only the machine's own addresses: {e}");
                Vec::new()
            }
        },
        None => Vec::new()
    }
}
