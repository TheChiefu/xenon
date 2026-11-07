# Screen share freezes: RTCP feedback is not forwarded

## Symptom

Watching a user's screen share works for a few seconds, then freezes on the
viewer's side for a minute or two until the share is stopped and restarted.

Server logs during the freeze:

```
WARN rtc::peer_connection::handler: SrtpHandler.handle_read got error: srtp ssrc=595800960 index=26427: duplicated
WARN rtc_ice::agent::agent_proto: [controlling]: Discarded message, not a valid local candidate from UDP:224.0.0.251:5353
```

## Root cause

The server is a media **relay**, but it only relays **RTP** (media packets),
not **RTCP** (control feedback).

Flow today:

1. Sender's browser → RTP + RTCP → server (sender's peer connection)
2. `forward()` in `src/rtc/handler.rs` copies only `OnRtpPacket` events to
   each other peer, rewriting the SSRC to the sink's SSRC
3. Viewer's browser → RTP + RTCP → server (viewer's peer connection)
4. `forward()` copies only `OnRtpPacket` events back to the sender

The viewer's browser recovers from packet loss with RTCP feedback:

- **NACK** — "I missed packets N, M; please retransmit"
- **PLI / FIR** — "I lost sync; please send a keyframe"

These RTCP packets arrive at the server but are consumed by the server's own
interceptor chain and **never forwarded to the sender's browser**:

- The NACK responder tries to retransmit from the server's own buffer, which
  is empty (the server is a relay, it does not buffer media)
- PLI/FIR are dropped — the server has no video encoder to keyframe

So the sender never retransmits and never sends a keyframe. The viewer's
decoder starves after the last keyframe and the picture freezes. Restarting
the share works because a fresh share starts with a keyframe.

### Log evidence

- `srtp ... duplicated` — the sender's browser *is* retransmitting (its own
  NACK responder is working), but the server's SRTP handler has already seen
  those sequence numbers, so the copies are dropped. The retransmissions are
  for the server's own (pointless) NACKs, not for the viewer's.
- `224.0.0.251:5353` — mDNS multicast noise on the UDP port; unrelated.

## Proposed fix

All server-side. Three parts:

### 1. Capture inbound RTCP per peer

Add a custom interceptor (modeled on webrtc-rs's own `rtcp-processing`
example, `examples/rtcp-processing/rtcp-processing.rs` in the `webrtc` crate)
that:

- runs **last** in the interceptor chain (highest slot, e.g. `Slot::from(14_000)`)
- filters inbound RTCP to NACK (`TransportLayerNack`), PLI
  (`PictureLossIndication`) and FIR (`FullIntraRequest`)
- marks those packets with `Attribute::DeliverToApplication` so they reach
  the application via `TrackRemoteEvent::OnRtcpPacket`
- forwards them to a per-peer `mpsc` channel

Register it in `peer.rs` after `register_default_interceptors`:

```rust
let interceptors = register_default_interceptors(Registry::new(), &mut media_engine)?;
let interceptors = interceptors.with(Slot::from(14_000), RtcpForwarder::new(tx));
```

### 2. Record the sender's original SSRC

In `forwarding.rs`, add `sender_ssrc: Mutex<Option<u32>>` to `Sink`. In
`forward()` in `handler.rs`, when the first RTP packet from a sender arrives,
record `packet.header.ssrc` (the original, before the rewrite to the sink's
SSRC). This gives the reverse mapping: sink SSRC → sender SSRC.

### 3. Forward RTCP to the sender with SSRC translation

Spawn a task per peer that:

1. reads RTCP packets from the channel
2. rewrites the SSRC from the sink's SSRC back to the sender's original SSRC
3. calls `write_rtcp()` on the sender's track (the `Arc<dyn TrackRemote>`
   from the sender's `on_track`)

This delivers the viewer's NACK/PLI/FIR to the sender's browser, which can
then retransmit the missing packets or send a keyframe.

## Files touched

- `src/rtc/forwarding.rs` — `sender_ssrc` on `Sink`, accessor
- `src/rtc/handler.rs` — record original SSRC in `forward()`, spawn the RTCP
  forwarder task
- `src/rtc/peer.rs` — register the interceptor, wire the channel
- `src/rtc/rtcp_forwarder.rs` (new) — the interceptor

## Notes

- `TrackRemote::write_rtcp()` is already part of the `webrtc` 0.21 API
- The interceptor must be registered per peer connection (it is, since
  `peer.rs` builds one registry per connection)
- NACKs for the sender's *own* media (viewer → server) are the only ones that
  matter; the server's own NACK responder can stay, it is harmless
