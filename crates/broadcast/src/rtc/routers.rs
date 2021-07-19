use async_std::io;
use async_std::net::UdpSocket;
use log::info;
use mediasoup::producer::{Producer, ProducerId};
use mediasoup::router::{Router, RouterOptions};
use ftl_protocol::protocol::FtlHandshakeFinalised;
use mediasoup::rtp_parameters::MediaKind;

use super::{codecs::init_codecs, workers::WorkerPool, producers::init_producers};

#[derive(Clone)]
pub enum DataSource {
    Ftl(FtlHandshakeFinalised)
}

#[derive(Clone)]
pub struct HyperspeedRouter {
    router: Router,
    channel_id: String,
    producers: Vec<Producer>,
    source: DataSource
}

impl HyperspeedRouter {
    pub async fn new(channel_id: String, source: DataSource) -> HyperspeedRouter {
        let mut options = RouterOptions::default();
        init_codecs(&mut options, &source);

        let router = WorkerPool::get()
            .get_worker()
            .create_router(options)
            .await
            .unwrap();

        let producers = init_producers(&router, &source).await;

        HyperspeedRouter {
            router,
            channel_id,
            producers,
            source
        }
    }

    pub async fn launch(&self, addr: String) -> Result<(), io::Error> {
        match &self.source {
            DataSource::Ftl(_) => {
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
            
                    match packet.header.payload_type {
                        96 => if let Some(video) = video_producer {
                            video.send(packet.marshal().unwrap()).await
                        } else { Ok(()) }
                        97 => if let Some(audio) = audio_producer {
                            audio.send(packet.marshal().unwrap()).await
                        } else { Ok(()) }
                        _ => Ok(())
                    }
                    .unwrap();
                }
            }
        }
    }

    pub fn clone_router(&self) -> Router {
        self.router.clone()
    }

    pub fn get_producer_ids(&self) -> Vec<ProducerId> {
        self.producers.iter()
            .map(|v| v.id().clone())
            .collect()
    }
}
