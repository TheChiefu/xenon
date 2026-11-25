//! The media a user sends the server, and how each is encoded

use types::rtc::media::Media;
use uuid::Uuid;
use webrtc::media_stream::MediaStreamTrack;

use ::rtc::peer_connection::configuration::media_engine::{MIME_TYPE_OPUS, MIME_TYPE_AV1};
use ::rtc::rtp_transceiver::rtp_sender::{
    RTCPFeedback, RTCRtpCodec, RTCRtpCodingParameters, RTCRtpEncodingParameters, RtpCodecKind,
};

/// https://www.rfc-editor.org/rfc/rfc3551#section-6
pub const OPUS_PAYLOAD_TYPE: u8 = 111;
pub const AV1_PAYLOAD_TYPE: u8 = 41;
/// https://www.rfc-editor.org/rfc/rfc7587#section-6.1
const OPUS_FORMAT_PARAMETERS: &str = "minptime=10;useinbandfec=1";

// PUBLIC METHODS //

/// Whether a media is carried as audio or video
pub fn kind(media: Media) -> RtpCodecKind {
    match media {
        Media::Microphone | Media::ScreenAudio => RtpCodecKind::Audio,
        Media::ScreenVideo => RtpCodecKind::Video
    }
}

/// A track of a user's media, in the stream it plays with
/// https://www.w3.org/TR/mediacapture-streams/#dom-mediastream-id
pub fn track_of(media: Media, stream_id: Uuid, ssrc: u32) -> MediaStreamTrack {
    let codec = match kind(media) {
        RtpCodecKind::Video => av1(),
        _ => opus()
    };

    let stream_id = stream_id.to_string();
    let track_id = Uuid::now_v7().to_string();
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

    MediaStreamTrack::new(stream_id, track_id, label, kind(media), vec![encoding])
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

/// https://aomediacodec.github.io/av1-avc/
pub fn av1() -> RTCRtpCodec {
    RTCRtpCodec {
        mime_type: MIME_TYPE_AV1.to_owned(),
        clock_rate: 90000,
        channels: 0,
        sdp_fmtp_line: "profile-id=0".to_owned(),
        rtcp_feedback: video_feedback()
    }
}

// HELPER METHODS //

/// Retransmission and keyframe requests a video receiver may send
/// https://www.rfc-editor.org/rfc/rfc4585#section-4.2
fn video_feedback() -> Vec<RTCPFeedback> {
    vec![
        RTCPFeedback { typ: "nack".to_owned(), parameter: String::new() },
        RTCPFeedback { typ: "nack".to_owned(), parameter: "pli".to_owned() },
        RTCPFeedback { typ: "ccm".to_owned(), parameter: "fir".to_owned() },
    ]
}
