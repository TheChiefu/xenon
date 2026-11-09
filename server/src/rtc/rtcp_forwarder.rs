//! Passes viewers' keyframe requests on to the application, so they can be relayed to the sender

use std::collections::VecDeque;
use std::time::Instant;

use ::rtc::interceptor::{Attribute, Interceptor, Packet, StreamInfo, TaggedPacket};
use ::rtc::rtcp::payload_feedbacks::full_intra_request::FullIntraRequest;
use ::rtc::rtcp::payload_feedbacks::picture_loss_indication::PictureLossIndication;
use ::rtc::sansio::Protocol;
use ::rtc::shared::error::Error;

// DATA STRUCTS //

#[derive(Default)]
pub struct RtcpForwarder {
    read_queue: VecDeque<TaggedPacket>,
    write_queue: VecDeque<TaggedPacket>
}

// PUBLIC METHODS //

pub fn is_keyframe_request(packet: &Box<dyn ::rtc::rtcp::Packet>) -> bool {
    let payload = packet.as_any();
    payload.is::<PictureLossIndication>() || payload.is::<FullIntraRequest>()
}

impl Protocol<TaggedPacket, TaggedPacket, ()> for RtcpForwarder {
    type Rout = TaggedPacket;
    type Wout = TaggedPacket;
    type Eout = ();
    type Error = Error;
    type Time = Instant;

    fn handle_read(&mut self, mut msg: TaggedPacket) -> Result<(), Self::Error> {
        if let Packet::Rtcp(packets) = &msg.message.packet {
            let requests: Vec<Box<dyn ::rtc::rtcp::Packet>> = packets
                .iter()
                .filter(|packet| is_keyframe_request(packet))
                .cloned()
                .collect();

            if requests.is_empty() {
                return Ok(());
            }

            msg.message.packet = Packet::Rtcp(requests);
            msg.message.add(Attribute::DeliverToApplication);
        }

        self.read_queue.push_back(msg);
        Ok(())
    }

    fn poll_read(&mut self) -> Option<Self::Rout> {
        self.read_queue.pop_front()
    }

    fn handle_write(&mut self, msg: TaggedPacket) -> Result<(), Self::Error> {
        self.write_queue.push_back(msg);
        Ok(())
    }

    fn poll_write(&mut self) -> Option<Self::Wout> {
        self.write_queue.pop_front()
    }
}

impl Interceptor for RtcpForwarder {
    fn bind_local_stream(&mut self, _info: &StreamInfo) {}
    fn unbind_local_stream(&mut self, _info: &StreamInfo) {}
    fn bind_remote_stream(&mut self, _info: &StreamInfo) {}
    fn unbind_remote_stream(&mut self, _info: &StreamInfo) {}
}
