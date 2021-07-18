use rtp::packet::Packet;
use webrtc_util::marshal::Unmarshal;

use webrtc::media::rtp::rtp_codec::RTPCodecCapability;
use webrtc::media::track::track_local::TrackLocalWriter;
use webrtc::media::track::track_local::track_local_static_rtp::TrackLocalStaticRTP;

use bytes::Bytes;
use std::convert::TryInto;

use std::net::UdpSocket;

fn main() -> std::io::Result<()> {
    let socket = UdpSocket::bind("127.0.0.1:65535")?;

    let mut videoCodec = RTPCodecCapability::default();
    videoCodec.mime_type = "video/h264".to_string();
    let videoTrack = TrackLocalStaticRTP::new(videoCodec, "video".to_string(), "hyperspeed".to_string());
    
    let mut audioCodec = RTPCodecCapability::default();
    audioCodec.mime_type = "audio/opus".to_string();
    let audioTrack = TrackLocalStaticRTP::new(audioCodec, "audio".to_string(), "hyperspeed".to_string());

    let mut buf = [0; 1460];
    loop {
        let (amt, src) = socket.recv_from(&mut buf)?;

        let packet = Packet::unmarshal(&mut &buf[..]).unwrap();
        // Note from Lightspeed: may fail from Windows OBS clients. Can safely ignore failure.

        match packet.header.payload_type {
            96 => videoTrack.write_rtp(&packet),
            97 => audioTrack.write_rtp(&packet),
            _ => Ok(0)
        }
        .unwrap();
    }
}
