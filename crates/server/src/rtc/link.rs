//! One user's WebRTC connection to the server, and the offers and answers that change it
//!
//! https://www.w3.org/TR/webrtc/#rtcpeerconnection-interface

use std::sync::{Arc, Mutex};

use webrtc::error::{Error, Result};
use webrtc::peer_connection::{PeerConnection, RTCSessionDescription};
use types::rtc::events::{RTCIceCandidateInit, RTCSdpType, RTCSessionDescriptionInit, RtcServerEvent};

use crate::rtc::forwarding::Forwarding;
use crate::rtc::signal::Signal;
use crate::utils;

// DATA STRUCTS //

/// https://www.w3.org/TR/webrtc/#rtcsignalingstate-enum
#[derive(Clone, Copy, PartialEq, Eq)]
enum Negotiation {
    Idle,
    AwaitingAnswer,
    AwaitingAnswerThenOffer
}

pub struct Link {
    connection: Arc<dyn PeerConnection>,
    negotiation: Mutex<Negotiation>
}

// PUBLIC METHODS //

impl Link {

    pub fn new(connection: Arc<dyn PeerConnection>) -> Link {
        Link {
            connection,
            negotiation: Mutex::new(Negotiation::Idle)
        }
    }

    /// Send an offer, or send one once the offer waiting is answered
    /// https://www.w3.org/TR/webrtc/#dom-rtcpeerconnection-createoffer
    pub async fn negotiate(&self, signal: &Signal, forwarding: &Forwarding) -> Result<()> {
        if !self.start_offer() {
            return Ok(());
        }

        let offer = self.connection.create_offer(None).await?;
        let sdp = offer.sdp.clone();
        self.connection.set_local_description(offer).await?;

        let description = RTCSessionDescriptionInit {
            r#type: RTCSdpType::offer,
            sdp
        };
        let streams = forwarding.streams();
        signal.send(&RtcServerEvent::Offer { description, streams });

        Ok(())
    }

    /// Apply the browser's answer, returning whether another offer is due
    /// https://www.w3.org/TR/webrtc/#dom-rtcpeerconnection-setremotedescription
    pub async fn answer(&self, description: RTCSessionDescriptionInit) -> Result<bool> {
        if description.r#type != RTCSdpType::answer {
            return Err(Error::Other("the description is not an answer".to_owned()));
        }

        let answer = RTCSessionDescription::answer(description.sdp)?;
        self.connection.set_remote_description(answer).await?;

        let previous = std::mem::replace(&mut *utils::mutex_lock(&self.negotiation), Negotiation::Idle);

        Ok(previous == Negotiation::AwaitingAnswerThenOffer)
    }

    /// https://www.w3.org/TR/webrtc/#dom-rtcpeerconnection-addicecandidate
    pub async fn add_candidate(&self, init: RTCIceCandidateInit) -> Result<()> {
        let init = webrtc::peer_connection::RTCIceCandidateInit {
            candidate: init.candidate,
            sdp_mid: init.sdpMid,
            sdp_mline_index: init.sdpMLineIndex,
            username_fragment: init.usernameFragment,
            ..Default::default()
        };

        self.connection.add_ice_candidate(init).await
    }

    pub async fn close(&self) -> Result<()> {
        self.connection.close().await
    }

    // HELPER METHODS //

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
