use async_std::{channel, task};
use async_trait::async_trait;
use ftl_protocol::protocol::FtlHandshakeFinalised;
use hyperspeed_broadcast::rtc::routers::{DataSource, HyperspeedRouter};

use std::sync::RwLock;
use std::collections::HashMap;
use once_cell::sync::OnceCell;

static ROUTERS: OnceCell<RwLock<HashMap<String, HyperspeedRouter>>> = OnceCell::new();

#[async_std::main]
async fn main() -> std::io::Result<()> {
    pretty_env_logger::init();

    let routers = RwLock::new(HashMap::<String, HyperspeedRouter>::new());
    ROUTERS.set(routers).ok();

    use ftl_protocol::server::IngestServer;
    struct MyServer {}

    #[async_trait]
    impl IngestServer for MyServer {
        async fn get_stream_key(&self, channel_id: &str) -> Result<String, ()> {
            match channel_id {
                "77" => Ok("ieDQxSZ7q58EEeLTvja4QKKGzndwUkVQ".to_string()),
                _ => unimplemented!()
            }
        }

        async fn allocate_ingest(&self, channel_id: &str, handshake: FtlHandshakeFinalised) -> Result<u16, ()> {
            let port = match channel_id {
                "77" => 65534,
                _ => unimplemented!()
            };

            let channel_id = channel_id.to_string();
            task::spawn_local(async move {
                let router = HyperspeedRouter::new(
                    channel_id.to_string(),
                    DataSource::Ftl(handshake)
                ).await;

                // ! FIXME: questionable code
                let routers = ROUTERS.get().unwrap();
                let mut routers = routers.write().unwrap();
                routers.insert(channel_id, router);
            });

            Ok(port)
        }
    }

    MyServer {}.launch("127.0.0.1:8084".to_string()).await?;

    Ok(())
}
