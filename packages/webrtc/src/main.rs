use std::collections::HashMap;
use std::net::UdpSocket;

use std::num::{NonZeroU32, NonZeroU8};
use async_std::net::TcpListener;
use async_std::task;
use async_tungstenite::tungstenite::Message;
use mediasoup::consumer::{ConsumerId, ConsumerOptions};
use mediasoup::data_structures::{DtlsParameters, IceCandidate, IceParameters, TransportListenIp};
use mediasoup::direct_transport::DirectTransportOptions;
use mediasoup::producer::{self, Producer, ProducerId, ProducerOptions};
use mediasoup::router::{Router, RouterOptions};
use mediasoup::transport::{Transport, TransportId};
use mediasoup::webrtc_transport::{TransportListenIps, WebRtcTransport, WebRtcTransportOptions, WebRtcTransportRemoteParameters};
use mediasoup::{worker::Worker, worker::WorkerSettings, worker_manager::WorkerManager};
use mediasoup::rtp_parameters::{MediaKind, MimeTypeAudio, MimeTypeVideo, RtcpFeedback, RtpCapabilities, RtpCapabilitiesFinalized, RtpCodecCapability, RtpCodecParameters, RtpCodecParametersParameters, RtpEncodingParameters, RtpEncodingParametersRtx, RtpParameters};
use webrtc_util::Marshal;

use futures::{SinkExt, StreamExt, TryStreamExt};
use serde::{Serialize, Deserialize};

#[async_std::main]
async fn main() -> std::io::Result<()> {
    let socket = UdpSocket::bind("127.0.0.1:65535")?;

    // Create a new worker manager
    let manager = WorkerManager::new();

    // Create a new mediasoup worker
    let mut settings = WorkerSettings::default();
    settings.rtc_ports_range = 21000..=22000;
    let worker = manager.create_worker(settings).await.unwrap();

    // * A worker pool should be globally available,
    // * see Vortex/src/rtc/worker.rs#L16

    // Prepare options for router
    let mut options = RouterOptions::default();


    //#region codecs
    // Initialise Audio codec
    options.media_codecs.push(
        RtpCodecCapability::Audio {
            mime_type: MimeTypeAudio::Opus,
            preferred_payload_type: None,
            clock_rate: NonZeroU32::new(48_000).unwrap(),
            channels: NonZeroU8::new(2).unwrap(),
            parameters: RtpCodecParametersParameters::default(),
            rtcp_feedback: Vec::new(),
        }
    );

    let mut audio_rtp_params = RtpParameters::default();
    audio_rtp_params.codecs = vec![
        RtpCodecParameters::Audio {
            mime_type: MimeTypeAudio::Opus,
            payload_type: 97,
            clock_rate: NonZeroU32::new(48_000).unwrap(),
            channels: NonZeroU8::new(2).unwrap(),
            parameters: RtpCodecParametersParameters::default(),
            rtcp_feedback: Vec::new(),
        }
    ];

    audio_rtp_params.encodings = vec![
        RtpEncodingParameters {
            ssrc: Some(2),
            ..RtpEncodingParameters::default()
        }
    ];

    // Initialise Video codec
    let mut parameters = RtpCodecParametersParameters::from([
        ("packetization-mode", 1_u32.into()),
        ("level-asymmetry-allowed", 1_u32.into())
    ]);

    parameters.insert("profile-level-id", "42001f");

    options.media_codecs.push(
        RtpCodecCapability::Video {
            mime_type: MimeTypeVideo::H264,
            preferred_payload_type: None,
            clock_rate: NonZeroU32::new(90_000).unwrap(),
            parameters: parameters.clone(),
            rtcp_feedback: vec! [
                RtcpFeedback::Nack,
                RtcpFeedback::NackPli,
                RtcpFeedback::CcmFir,
                RtcpFeedback::GoogRemb,
                RtcpFeedback::TransportCc,
            ],
        }
    );

    let mut video_rtp_params = RtpParameters::default();
    video_rtp_params.codecs = vec![
        RtpCodecParameters::Video {
            mime_type: MimeTypeVideo::H264,
            payload_type: 96,
            clock_rate: NonZeroU32::new(90_000).unwrap(),
            parameters,
            rtcp_feedback: vec! [
                RtcpFeedback::Nack,
                RtcpFeedback::NackPli,
                RtcpFeedback::CcmFir,
                RtcpFeedback::GoogRemb,
                RtcpFeedback::TransportCc,
            ],
        }
    ];

    video_rtp_params.encodings = vec![
        RtpEncodingParameters {
            ssrc: Some(1),
            ..RtpEncodingParameters::default()
        }
    ];


    //#region worker
    // Create a mediasoup worker
    let router = worker
        .create_router(options)
        .await.unwrap();

    
    // Prepare transport options
    let transport_options = DirectTransportOptions::default();

    // Create direct transport
    let direct_transport = router
        .create_direct_transport(transport_options)
        .await.unwrap();


    //#region producers
    let video_producer = direct_transport.produce(
        ProducerOptions::new(MediaKind::Video, video_rtp_params)
    ).await.unwrap();

    let audio_producer = direct_transport.produce(
        ProducerOptions::new(MediaKind::Audio, audio_rtp_params)
    ).await.unwrap();

    let producers = vec![ video_producer.id(), audio_producer.id() ];

    let direct_video_producer = match video_producer {
        Producer::Direct(direct) => direct,
        _ => unreachable!()
    };

    let direct_audio_producer = match audio_producer {
        Producer::Direct(direct) => direct,
        _ => unreachable!()
    };


    //#region consumers    
    /*let transport_options =
        WebRtcTransportOptions::new(TransportListenIps::new(TransportListenIp {
            ip: "127.0.0.1".parse().unwrap(),
            announced_ip: None,
        }));

    let consumer_transport = router
        .create_webrtc_transport(transport_options)
        .await.unwrap();*/

    //#region Signalling
    #[derive(Serialize, Deserialize)]
    struct TransportOptions {
        id: TransportId,
        dtls_parameters: DtlsParameters,
        ice_candidates: Vec<IceCandidate>,
        ice_parameters: IceParameters,
    }

    #[derive(Serialize, Deserialize)]
    #[serde(tag = "type")]
    enum ServerboundMessage {
        Begin,
        Init {
            rtp_capabilities: RtpCapabilities
        },
        Connect {
            dtls_parameters: DtlsParameters
        },
        Consume {
            producer_id: ProducerId
        },
        Resume {
            id: ConsumerId
        }
    }

    #[derive(Serialize, Deserialize)]
    #[serde(tag = "type")]
    enum ClientboundMessage {
        Init {
            producers: Vec<ProducerId>,
            transport: TransportOptions,
            router_rtp_capabilities: RtpCapabilitiesFinalized
        },
        Connected,
        Consuming {
            id: ConsumerId,
            producer_id: ProducerId,
            kind: MediaKind,
            rtp_parameters: RtpParameters
        }
    }

    /*let server_init = ClientboundMessage::Init {
        transport: TransportOptions {
            id: direct_transport.id(),
            //dtls_parameters: direct_transport.
        }
    };*/

    task::spawn(async move {
        let try_socket = TcpListener::bind("0.0.0.0:9050").await;
        let listener = try_socket.expect("Failed to bind");

        while let Ok((stream, _)) = listener.accept().await {
            let router = router.clone();
            let producers = producers.clone();

            task::spawn_local(async move {
                let addr = stream.peer_addr().unwrap();
                dbg!(addr);

                let ws = async_tungstenite::accept_async(stream)
                    .await.unwrap();

                let (mut write, mut read) = ws.split();

                let mut client_rtp_capabilities = None;

                let transport_options =
                    WebRtcTransportOptions::new(TransportListenIps::new(TransportListenIp {
                        ip: "127.0.0.1".parse().unwrap(),
                        announced_ip: None,
                    }));

                let consumer_transport = router
                    .create_webrtc_transport(transport_options)
                    .await.unwrap();

                let mut consumers = HashMap::new();
                while let Ok(message) = read.try_next().await {
                    if let Message::Text(text) = message.unwrap() {
                        let msg: ServerboundMessage = serde_json::from_str(&text).unwrap();
                        match msg {
                            ServerboundMessage::Begin => {
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
                            },
                            ServerboundMessage::Init { rtp_capabilities } => {
                                client_rtp_capabilities.replace(rtp_capabilities);
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
                            ServerboundMessage::Consume { producer_id } => {
                                let rtp_capabilities = client_rtp_capabilities.as_ref().unwrap();
                                let mut options = ConsumerOptions::new(producer_id, rtp_capabilities.clone());
                                options.paused = true;

                                let consumer = consumer_transport.consume(options).await.unwrap();
                                let id = consumer.id();
                                let kind = consumer.kind();
                                let rtp_parameters = consumer.rtp_parameters().clone();

                                consumers.insert(id, consumer);
                                
                                write.send(Message::Text(
                                    serde_json::to_string(&ClientboundMessage::Consuming {
                                        id,
                                        producer_id,
                                        kind,
                                        rtp_parameters
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
                        }
                    }
                }
            });
        }
    });
    


    //#region UDP
    // Handle incoming RTP packets.
    let mut buf = [0; 1460];
    loop {
        let (_amt, _src) = socket.recv_from(&mut buf)?;

        use rtp::packet::Packet;
        use webrtc_util::marshal::Unmarshal;

        let packet = Packet::unmarshal(&mut &buf[..]).unwrap();
        // Note from Lightspeed: may fail from Windows OBS clients. Can safely ignore failure.

        match packet.header.payload_type {
            96 => direct_video_producer.send(
                packet.marshal().unwrap()
            ).await,
            97 => direct_audio_producer.send(
                packet.marshal().unwrap()
            ).await,
            _ => Ok(())
        }
        .unwrap();
    }
}
