//! The media a user sends the server, and how each is encoded

use uuid::Uuid;
use webrtc::media_stream::MediaStreamTrack;

use ::rtc::peer_connection::configuration::media_engine::{MIME_TYPE_OPUS, MIME_TYPE_VP8};
use ::rtc::rtp_transceiver::rtp_sender::{
    RTCPFeedback, RTCRtpCodec, RTCRtpCodingParameters, RTCRtpEncodingParameters, RtpCodecKind,
};

/// https://www.rfc-editor.org/rfc/rfc3551#section-6
pub const OPUS_PAYLOAD_TYPE: u8 = 111;
pub const VP8_PAYLOAD_TYPE: u8 = 96;
/// https://www.rfc-editor.org/rfc/rfc7587#section-6.1
const OPUS_FORMAT_PARAMETERS: &str = "minptime=10;useinbandfec=1";

// DATA STRUCTS //

/// What a user sends the server, each in a slot of its own
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Media {
    Microphone,
    Camera,
    ScreenAudio,
    ScreenVideo
}

impl Media {
    pub const ALL: [Media; 4] = [
        Media::Microphone,
        Media::Camera,
        Media::ScreenAudio,
        Media::ScreenVideo
    ];

    pub fn kind(self) -> RtpCodecKind {
        match self {
            Media::Microphone | Media::ScreenAudio => RtpCodecKind::Audio,
            Media::Camera | Media::ScreenVideo => RtpCodecKind::Video
        }
    }

    fn name(self) -> &'static str {
        match self {
            Media::Microphone => "microphone",
            Media::Camera => "camera",
            Media::ScreenAudio => "screen_audio",
            Media::ScreenVideo => "screen_video"
        }
    }
}

// PUBLIC METHODS //

/// A track of a user's media. Its stream id is `{user_id}` for the microphone and
/// camera, and `{user_id}:screen` for a screen share.
/// https://www.w3.org/TR/mediacapture-streams/#dom-mediastream-id
pub fn track_of(sender_id: Uuid, media: Media, ssrc: u32) -> MediaStreamTrack {
    let codec = match media.kind() {
        RtpCodecKind::Video => vp8(),
        _ => opus()
    };

    let stream_id = match media {
        Media::Microphone | Media::Camera => sender_id.to_string(),
        Media::ScreenAudio | Media::ScreenVideo => format!("{sender_id}:screen")
    };
    let track_id = format!("{sender_id}-{}", media.name());
    let label = track_id.clone();

    let coding = RTCRtpCodingParameters {
        ssrc: Some(ssrc),
        ..Default::default()
    };
    let encoding = RTCRtpEncodingParameters {
        rtp_coding_parameters: coding,
        codec,
        ..Default::default()
    };

    MediaStreamTrack::new(stream_id, track_id, label, media.kind(), vec![encoding])
}

/// https://www.rfc-editor.org/rfc/rfc7587
pub fn opus() -> RTCRtpCodec {
    RTCRtpCodec {
        mime_type: MIME_TYPE_OPUS.to_owned(),
        clock_rate: 48000,
        channels: 2,
        sdp_fmtp_line: OPUS_FORMAT_PARAMETERS.to_owned(),
        rtcp_feedback: Vec::new()
    }
}

/// https://www.rfc-editor.org/rfc/rfc7741
pub fn vp8() -> RTCRtpCodec {
    RTCRtpCodec {
        mime_type: MIME_TYPE_VP8.to_owned(),
        clock_rate: 90000,
        channels: 0,
        sdp_fmtp_line: String::new(),
        rtcp_feedback: video_feedback()
    }
}

// HELPER METHODS //

/// Retransmission and keyframe requests a video receiver may send
/// https://www.rfc-editor.org/rfc/rfc4585#section-4.2
fn video_feedback() -> Vec<RTCPFeedback> {
    let feedback = [("nack", ""), ("nack", "pli"), ("ccm", "fir")];

    let mut list = Vec::new();
    for (typ, parameter) in feedback {
        list.push(RTCPFeedback {
            typ: typ.to_owned(),
            parameter: parameter.to_owned()
        });
    }

    list
}
