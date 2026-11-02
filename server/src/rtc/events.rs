//! What the server sends over an RTC socket, and what a client may send back
//!
//! https://www.w3.org/TR/webrtc/#session-negotiation-model
//! https://developer.mozilla.org/en-US/docs/Web/API/WebRTC_API/Signaling_and_video_calling

use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(feature = "ts_bindings")]
use ts_rs::TS;

/// Who is in a room's RTC session
#[derive(Serialize, Clone)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "rtc/events.ts"))]
pub struct RtcRoom {
    pub room_id: Uuid,
    pub participants: HashSet<Uuid>
}

/// https://www.w3.org/TR/webrtc/#dom-rtcsdptype
#[allow(non_camel_case_types)]
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "rtc/events.ts"))]
pub enum RTCSdpType {
    offer,
    answer
}

/// https://www.w3.org/TR/webrtc/#dom-rtcsessiondescriptioninit
#[derive(Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "rtc/events.ts"))]
pub struct RTCSessionDescriptionInit {
    pub r#type: RTCSdpType,
    pub sdp: String
}

/// https://www.w3.org/TR/webrtc/#dom-rtcicecandidateinit
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "rtc/events.ts"))]
pub struct RTCIceCandidateInit {
    pub candidate: String,
    pub sdpMid: Option<String>,
    pub sdpMLineIndex: Option<u16>,
    pub usernameFragment: Option<String>
}

/// What a client sends over its RTC socket
#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "rtc/events.ts"))]
pub enum RtcClientEvent {
    Answer { description: RTCSessionDescriptionInit },
    Candidate { candidate: RTCIceCandidateInit },
    Kick { user_id: Uuid }
}

/// What the server sends over an RTC socket
#[derive(Serialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "rtc/events.ts"))]
pub enum RtcServerEvent {
    Offer { description: RTCSessionDescriptionInit },
    Candidate { candidate: RTCIceCandidateInit }
}

/// Codes the server closes an RTC socket with
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u16)]
#[cfg_attr(feature = "ts_bindings", derive(TS), ts(export, export_to = "rtc/events.ts", repr(enum), rename_all = "snake_case"))]
pub enum RtcCloseCode {
    /// https://www.rfc-editor.org/rfc/rfc6455#section-7.4.1
    Normal = 1000,
    PolicyViolation = 1008,
    ServerError = 1011,

    /// https://www.rfc-editor.org/rfc/rfc6455#section-7.4.2
    Full = 4001,
    Kicked = 4002,
    NoPort = 4003
}
