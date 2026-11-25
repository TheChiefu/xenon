//! The media a user sends the server, and the tracks forwarding everyone else's media to them

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use argon2::password_hash::rand_core::{OsRng, RngCore};
use types::rtc::media::Media;
use uuid::Uuid;
use webrtc::error::Result;
use webrtc::media_stream::track_local::TrackLocal;
use webrtc::media_stream::track_local::static_rtp::TrackLocalStaticRTP;
use webrtc::peer_connection::PeerConnection;
use webrtc::rtp_transceiver::{
    RtpSender, RTCRtpReceiverId, RTCRtpTransceiverDirection, RTCRtpTransceiverInit,
};
use types::rtc::events::RtcStream;

use crate::rtc::media;
use crate::utils;

// DATA STRUCTS //

/// Where packets of another user's media are written, to reach this user
struct Sink {
    track: Arc<TrackLocalStaticRTP>,
    sender: Arc<dyn RtpSender>,
    ssrc: u32,
    stream_id: Uuid
}

pub struct Forwarding {
    connection: Arc<dyn PeerConnection>,
    sources: HashMap<RTCRtpReceiverId, Media>,
    sinks: Mutex<HashMap<(Uuid, Media), Sink>>
}

// PUBLIC METHODS //

impl Forwarding {

    /// Add a source to the connection for each media the user sends
    pub async fn new(connection: Arc<dyn PeerConnection>) -> Result<Forwarding> {
        let mut sources = HashMap::new();

        for media in Media::ALL {
            let receive = RTCRtpTransceiverInit {
                direction: RTCRtpTransceiverDirection::Recvonly,
                streams: Vec::new(),
                send_encodings: Vec::new()
            };
            let transceiver = connection.add_transceiver_from_kind(media::kind(media), Some(receive)).await?;
            sources.insert(RTCRtpReceiverId::from(transceiver.id()), media);
        }

        Ok(Forwarding {
            connection,
            sources,
            sinks: Mutex::new(HashMap::new())
        })
    }

    /// Add the sinks for another user's media
    pub async fn add_sender(&self, sender_id: Uuid) -> Result<()> {
        let microphone_stream = Uuid::now_v7();
        let screen_stream = Uuid::now_v7();

        for media in Media::ALL {
            let stream_id = match media {
                Media::Microphone => microphone_stream,
                Media::ScreenAudio | Media::ScreenVideo => screen_stream
            };

            let sink = self.add_sink(sender_id, media, stream_id).await?;
            utils::mutex_lock(&self.sinks).insert((sender_id, media), sink);
        }

        Ok(())
    }

    /// Remove the sinks for another user's media
    pub async fn remove_sender(&self, sender_id: Uuid) {
        for media in Media::ALL {
            let Some(sink) = utils::mutex_lock(&self.sinks).remove(&(sender_id, media)) else {
                continue;
            };

            if let Err(e) = self.connection.remove_track(&sink.sender).await {
                tracing::warn!("could not remove {sender_id}'s {media:?}: {e}");
            }
        }
    }

    /// The media a source carries
    pub fn media_of(&self, receiver_id: RTCRtpReceiverId) -> Option<Media> {
        self.sources.get(&receiver_id).copied()
    }

    /// The sink for a user's media, and its SSRC
    pub fn sink(&self, sender_id: Uuid, media: Media) -> Option<(Arc<TrackLocalStaticRTP>, u32)> {
        let sinks = utils::mutex_lock(&self.sinks);
        let found = sinks.get(&(sender_id, media))?;
        Some((found.track.clone(), found.ssrc))
    }

    /// The stream each sink belongs to, and whose media it carries
    pub fn streams(&self) -> Vec<RtcStream> {
        let sinks = utils::mutex_lock(&self.sinks);
        let mut streams = Vec::new();

        for ((user_id, media), found) in sinks.iter() {
            streams.push(RtcStream {
                stream_id: found.stream_id,
                user_id: *user_id,
                media: *media
            });
        }

        streams
    }

    // HELPER METHODS //

    /// Add a sink for one of another user's media
    async fn add_sink(&self, sender_id: Uuid, media: Media, stream_id: Uuid) -> Result<Sink> {
        let ssrc = OsRng.next_u32();
        let track = Arc::new(TrackLocalStaticRTP::new(media::track_of(media, stream_id, ssrc)));

        // A new transceiver per track, as add_track would reuse one of the user's own slots
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

        Ok(Sink {
            track,
            sender,
            ssrc,
            stream_id
        })
    }
}
