//! Relays viewers' keyframe requests back to the user sending the media
//!
//! https://www.rfc-editor.org/rfc/rfc4585#section-6.3.1

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use tokio::task::JoinHandle;
use uuid::Uuid;
use webrtc::media_stream::track_local::{TrackLocal, TrackLocalEvent};
use webrtc::media_stream::track_local::static_rtp::TrackLocalStaticRTP;
use webrtc::media_stream::track_remote::TrackRemote;

use ::rtc::rtcp::payload_feedbacks::picture_loss_indication::PictureLossIndication;

use crate::rtc::connections::{self, Connections};
use crate::rtc::forwarding::Forwarding;
use crate::rtc::media::Media;
use crate::rtc::rtcp_forwarder;
use crate::utils;

/// The shortest time between two keyframe requests sent to a user for the same media
const REQUEST_INTERVAL: Duration = Duration::from_secs(1);

// DATA STRUCTS //

/// A track the user sends the server, and the SSRC its packets arrive with
struct Source {
    track: Arc<dyn TrackRemote>,
    ssrc: u32,
    requested_at: Option<Instant>
}

pub struct Keyframes {
    connections: Connections,

    /// The media this user sends, to request keyframes of
    sources: Mutex<HashMap<Media, Source>>,

    /// Tasks relaying this user's requests for another user's media, keyed by that user
    relays: Mutex<HashMap<(Uuid, Media), JoinHandle<()>>>
}

// PUBLIC METHODS //

impl Keyframes {

    pub fn new(connections: Connections) -> Keyframes {
        Keyframes {
            connections,
            sources: Mutex::new(HashMap::new()),
            relays: Mutex::new(HashMap::new())
        }
    }

    /// Record the track one of this user's media arrives on
    pub fn open_source(&self, media: Media, track: Arc<dyn TrackRemote>, ssrc: u32) {
        utils::mutex_lock(&self.sources).insert(media, Source {
            track,
            ssrc,
            requested_at: None
        });
    }

    /// Forget the track one of this user's media arrived on, unless another has replaced it
    pub fn close_source(&self, media: Media, track: &Arc<dyn TrackRemote>) {
        let mut sources = utils::mutex_lock(&self.sources);
        if sources.get(&media).is_some_and(|source| Arc::ptr_eq(&source.track, track)) {
            sources.remove(&media);
        }
    }

    /// Relay this user's keyframe requests for another user's media to that user
    pub fn relay_sender(&self, forwarding: &Forwarding, sender_id: Uuid) {
        for media in Media::ALL {
            let Some((track, _)) = forwarding.sink(sender_id, media) else {
                continue;
            };

            let relay = tokio::spawn(relay(self.connections.clone(), sender_id, media, track));
            if let Some(previous) = utils::mutex_lock(&self.relays).insert((sender_id, media), relay) {
                previous.abort();
            }
        }
    }

    /// Stop relaying this user's keyframe requests for another user's media
    pub fn stop_sender(&self, sender_id: Uuid) {
        let mut relays = utils::mutex_lock(&self.relays);
        for media in Media::ALL {
            if let Some(relay) = relays.remove(&(sender_id, media)) {
                relay.abort();
            }
        }
    }
}

impl Drop for Keyframes {
    fn drop(&mut self) {
        for (_, relay) in utils::mutex_lock(&self.relays).drain() {
            relay.abort();
        }
    }
}

// HELPER METHODS //

impl Keyframes {

    /// Ask this user's browser for a keyframe of their media, at most once per interval
    async fn request(&self, media: Media) {
        let (track, ssrc) = {
            let mut sources = utils::mutex_lock(&self.sources);
            let Some(source) = sources.get_mut(&media) else {
                return;
            };

            let now = Instant::now();
            if source.requested_at.is_some_and(|at| now.duration_since(at) < REQUEST_INTERVAL) {
                return;
            }

            source.requested_at = Some(now);
            (source.track.clone(), source.ssrc)
        };

        let request = PictureLossIndication {
            sender_ssrc: 0,
            media_ssrc: ssrc
        };

        if let Err(e) = track.write_rtcp(vec![Box::new(request)]).await {
            tracing::debug!("could not request a keyframe of {media:?}: {e}");
        }
    }
}

/// Pass the keyframe requests a viewer sends about a sink on to the user whose media it carries
async fn relay(
    connections: Connections,
    sender_id: Uuid,
    media: Media,
    track: Arc<TrackLocalStaticRTP>,
) {
    while let Some(event) = track.poll().await {
        let TrackLocalEvent::OnRtcpPacket(packets) = event else {
            continue;
        };

        if !packets.iter().any(rtcp_forwarder::is_keyframe_request) {
            continue;
        }

        if let Some(sender) = connections::peer(&connections, sender_id) {
            sender.keyframes.request(media).await;
        }
    }
}
