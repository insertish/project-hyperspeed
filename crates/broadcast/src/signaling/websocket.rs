use std::collections::HashMap;

use async_tungstenite::tungstenite::Message;
use futures::{StreamExt, TryStreamExt, SinkExt};
use async_std::net::TcpListener;
use mediasoup::{consumer::ConsumerOptions, data_structures::TransportListenIp, producer::ProducerId, router::Router, webrtc_transport::{TransportListenIps, WebRtcTransportOptions, WebRtcTransportRemoteParameters}};
use mediasoup::transport::Transport;
use async_trait::async_trait;
use async_std::task;
use log::info;

use crate::signaling::messages::{ClientboundMessage, Consume, ServerboundMessage, TransportOptions};

pub struct StreamInformation {
    pub router: Router,
    pub producers: Vec<ProducerId>
}

#[async_trait]
pub trait SignalingServer {
    async fn launch(&'static self, addr: &'static str, announced_ip: &'static str) {
        let try_socket = TcpListener::bind(addr).await;
        let listener = try_socket.expect("Failed to bind");

        while let Ok((stream, _)) = listener.accept().await {
            task::spawn_local(async move {
                let addr = stream.peer_addr().unwrap();
                info!("User connected: {}", addr);

                let ws = async_tungstenite::accept_async(stream)
                    .await.unwrap();

                let (mut write, mut read) = ws.split();
                let mut channel_id = None;
                'outer: while let Ok(message) = read.try_next().await {
                    if let Message::Text(text) = message.unwrap() {
                        let msg: ServerboundMessage = serde_json::from_str(&text).unwrap();
                        match msg {
                            ServerboundMessage::Begin { channel_id: cid } => {
                                channel_id = Some(cid);
                                break 'outer;
                            },
                            _ => {}
                        }
                    }
                }

                if channel_id.is_none() {
                    return;
                }

                let channel_id = channel_id.unwrap();
                let stream_info = self.get_stream(channel_id).await;

                if stream_info.is_none() {
                    // ! FIXME: throw error; not live here
                    dbg!("not live");
                    return;
                }

                let StreamInformation { router, producers } = stream_info.unwrap();
                let transport_options =
                    WebRtcTransportOptions::new(TransportListenIps::new(TransportListenIp {
                        ip: "0.0.0.0".parse().unwrap(),
                        announced_ip: Some(announced_ip.parse().unwrap()),
                    }));

                let consumer_transport = router
                    .create_webrtc_transport(transport_options)
                    .await.unwrap();

                write.send(Message::Text(
                    serde_json::to_string(&ClientboundMessage::Init {
                        transport: TransportOptions {
                            id: consumer_transport.id(),
                            dtls_parameters: consumer_transport.dtls_parameters(),
                            ice_candidates: consumer_transport.ice_candidates().clone(),
                            ice_parameters: consumer_transport.ice_parameters().clone()
                        },
                        router_rtp_capabilities: router.rtp_capabilities().clone(),
                        producers: producers.clone()
                    })
                    .unwrap()
                ))
                .await.unwrap();

                // Client data
                let mut consumers = HashMap::new();
                let mut client_rtp_capabilities = None;

                while let Ok(message) = read.try_next().await {
                    if let Message::Text(text) = message.unwrap() {
                        let msg: ServerboundMessage = serde_json::from_str(&text).unwrap();
                        match msg {
                            ServerboundMessage::Begin { .. } => {},
                            ServerboundMessage::Init { rtp_capabilities } => {
                                client_rtp_capabilities = Some(rtp_capabilities);
                            },
                            ServerboundMessage::Connect { dtls_parameters } => {
                                consumer_transport
                                    .connect(WebRtcTransportRemoteParameters { dtls_parameters })
                                    .await
                                    .unwrap();
                                
                                write.send(Message::Text(
                                    serde_json::to_string(&ClientboundMessage::Connected)
                                    .unwrap()
                                ))
                                .await.unwrap();
                            },
                            ServerboundMessage::Consume => {
                                let mut consume = vec![];
                                for producer_id in &producers {
                                    let rtp_capabilities = client_rtp_capabilities.as_ref().unwrap();
                                    let mut options = ConsumerOptions::new(*producer_id, rtp_capabilities.clone());
                                    options.paused = true;

                                    let consumer = consumer_transport.consume(options).await.unwrap();
                                    
                                    let id = consumer.id();
                                    let kind = consumer.kind();
                                    let rtp_parameters = consumer.rtp_parameters().clone();

                                    consumers.insert(id, consumer);
                                    consume.push(Consume {
                                        id,
                                        producer_id: *producer_id,
                                        kind,
                                        rtp_parameters
                                    });
                                }
                                
                                write.send(Message::Text(
                                    serde_json::to_string(&ClientboundMessage::Consuming {
                                        consume
                                    })
                                    .unwrap()
                                ))
                                .await.unwrap();
                            },
                            ServerboundMessage::Resume { id } => {
                                if let Some(consumer) = consumers.get(&id).cloned() {
                                    consumer.resume().await.unwrap();
                                }
                            }
                            _ => {}
                        }
                    }
                }
            });
        }
    }

    async fn get_stream(&self, channel_id: String) -> Option<StreamInformation>;
}
