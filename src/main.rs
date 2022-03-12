use std::net::{SocketAddr, IpAddr, Ipv4Addr};
use async_std::task;
use async_trait::async_trait;
use ftl_protocol::protocol::FtlHandshakeFinalised;
use hyperspeed_broadcast::rtc::workers::WorkerPool;
use hyperspeed_broadcast::signaling::websocket::StreamInformation;
use hyperspeed_broadcast::rtc::routers::{DataSource, HyperspeedRouter};

use async_std::channel::Receiver;
use std::sync::{RwLock};
use std::collections::HashMap;
use once_cell::sync::OnceCell;

static ROUTERS: OnceCell<RwLock<HashMap<String, HyperspeedRouter>>> = OnceCell::new();

#[async_std::main]
async fn main() -> std::io::Result<()> {
    pretty_env_logger::init();
    WorkerPool::init().await;

    let routers = RwLock::new(HashMap::<String, HyperspeedRouter>::new());
    ROUTERS.set(routers).ok();

    use ftl_protocol::server::IngestServer;
    struct MyIngestServer {}

    #[async_trait]
    impl IngestServer for MyIngestServer {
        async fn get_stream_key(&self, channel_id: &str) -> Result<String, ()> {
            match channel_id {
                "77" => Ok("ieDQxSZ7q58EEeLTvja4QKKGzndwUkVQ".to_string()),
                "78" => Ok("ieDQxSZ7q58EEeLTvja4QKKGzndwUkVQ".to_string()),
                "7543" => Ok("Uf3Orxx4I5qzXEsM8amlaArdUg1Buhfk".to_string()),
                _ => unimplemented!()
            }
        }

        async fn allocate_ingest(&self, channel_id: &str, handshake: FtlHandshakeFinalised, stop_receiver: Receiver<()>) -> Result<u16, ()> {
            let port = match channel_id {
                "77" => 65534,
                "78" => 65535,
                "7543" => 65535,
                _ => unimplemented!()
            };

            let channel_id = channel_id.to_string();
            task::spawn_local(async move {
                let router = HyperspeedRouter::new(
                    channel_id.to_string(),
                    DataSource::Ftl(handshake),
                    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), port)
                ).await;

                // ! FIXME: questionable code
                // I mean really, this shouldn't fail unless if we manage to somehow poison
                // the ROUTERS cell. If we're careful we probably won't need to replace this.
                let routers = ROUTERS.get().unwrap();
                let mut routers = routers.write().unwrap();
                routers.insert(channel_id.to_string(), router.clone());
                drop(routers);

                // Launch UDP ingest server
                stop_receiver.recv().await.unwrap();
                
                // and drop it
                let routers = ROUTERS.get().unwrap();
                let mut routers = routers.write().unwrap();
                routers.remove(&channel_id);
                drop(routers);
            });

            Ok(port)
        }
    }

    use hyperspeed_broadcast::signaling::websocket::SignalingServer;
    struct MySignalingServer {}

    #[async_trait]
    impl SignalingServer for MySignalingServer {
        async fn get_stream(&self, channel_id: String) -> Option<StreamInformation> {
            let routers = ROUTERS.get().unwrap();
            let routers = routers.read().unwrap();

            if let Some(router) = routers.get(&channel_id) {
                Some(StreamInformation {
                    router: router.clone_router(),
                    producers: router.get_producer_ids()
                })
            } else {
                None
            }
        }
    }

    task::spawn(MySignalingServer {}.launch("0.0.0.0:9050", "192.168.0.10"));
    MyIngestServer {}.launch("0.0.0.0:8084".to_string()).await?;

    Ok(())
}
