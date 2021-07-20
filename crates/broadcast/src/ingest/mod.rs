use log::info;
use async_std::io;
use async_std::net::UdpSocket;
use mediasoup::producer::Producer;
use mediasoup::rtp_parameters::MediaKind;

use crate::rtc::routers::{DataSource, HyperspeedRouter};

impl HyperspeedRouter {
    pub async fn launch_ingest(&self, addr: String) -> Result<(), io::Error> {
        match &self.source {
            DataSource::Ftl(data) => {
                let video_producer = self.producers
                    .iter()
                    .find(|v| v.kind() == MediaKind::Video)
                    .map(|v| match v {
                        Producer::Direct(direct) => direct,
                        _ => unreachable!()
                    });
                
                let audio_producer = self.producers
                    .iter()
                    .find(|v| v.kind() == MediaKind::Audio)
                    .map(|v| match v {
                        Producer::Direct(direct) => direct,
                        _ => unreachable!()
                    });

                    let video_payload_type = data.video.as_ref().map(|v| v.payload_type).unwrap_or(96);
                    let audio_payload_type = data.video.as_ref().map(|v| v.payload_type).unwrap_or(97);
                
                info!("Created new UDP server on {}", addr);
                let socket = UdpSocket::bind(addr).await?;
                let mut buf = vec![0u8; 4096];
                loop {
                    let (amt, _src) = socket.recv_from(&mut buf).await?;
                    // ! FIXME: we should validate _src is the same as the address of the FTL peer
            
                    use rtp::packet::Packet;
                    use webrtc_util::marshal::{Marshal, Unmarshal};
            
                    let packet = Packet::unmarshal(&mut &buf[..amt]).unwrap(); // ! FIXME
                    // Note from Lightspeed: may fail from Windows OBS clients.
                    // Can safely ignore failure.

                    if video_payload_type == packet.header.payload_type {
                        if let Some(video) = video_producer {
                            video.send(packet.marshal().unwrap()).await.unwrap();
                        }
                    } else if audio_payload_type == packet.header.payload_type {
                        if let Some(audio) = audio_producer {
                            audio.send(packet.marshal().unwrap()).await.unwrap();
                        }
                    }
                }
            }
        }
    }
}