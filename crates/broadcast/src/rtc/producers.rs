use std::num::{NonZeroU32, NonZeroU8};

use mediasoup::direct_transport::DirectTransportOptions;
use mediasoup::producer::{Producer, ProducerOptions};
use mediasoup::router::Router;
use mediasoup::transport::Transport;
use mediasoup::rtp_parameters::{MediaKind, RtpCodecParameters, RtpEncodingParameters, RtpParameters};

use crate::rtc::codecs::{AudioCodec, VideoCodec};

use super::routers::DataSource;

pub async fn init_producers(router: &Router, source: &DataSource) -> Vec<Producer> {
    let mut producers = Vec::new();

    match source {
        DataSource::Ftl(handshake) => {
            // Prepare transport options
            let transport_options = DirectTransportOptions::default();

            // Create direct transport
            let direct_transport = router
                .create_direct_transport(transport_options)
                .await.unwrap();

            // Initialise video producer
            if let Some(video) = &handshake.video {
                let VideoCodec { mime_type, clock_rate, parameters, rtcp_feedback }
                    = VideoCodec::from(&video.codec);

                let mut video_rtp_params = RtpParameters::default();
                video_rtp_params.codecs = vec![
                    RtpCodecParameters::Video {
                        mime_type,
                        payload_type: video.payload_type,
                        clock_rate: NonZeroU32::new(clock_rate).unwrap(),
                        parameters,
                        rtcp_feedback
                    }
                ];

                video_rtp_params.encodings = vec![
                    RtpEncodingParameters {
                        ssrc: Some(video.ssrc),
                        ..RtpEncodingParameters::default()
                    }
                ];

                producers.push(
                    direct_transport.produce(
                        ProducerOptions::new(MediaKind::Video, video_rtp_params)
                    ).await.unwrap()
                );
            }

            // Initialise audio producer
            if let Some(audio) = &handshake.audio {
                let AudioCodec { mime_type, clock_rate, parameters, rtcp_feedback }
                    = AudioCodec::from(&audio.codec);

                let mut audio_rtp_params = RtpParameters::default();
                audio_rtp_params.codecs = vec![
                    RtpCodecParameters::Audio {
                        mime_type,
                        payload_type: audio.payload_type,
                        clock_rate: NonZeroU32::new(clock_rate).unwrap(),
                        channels: NonZeroU8::new(2).unwrap(),
                        parameters,
                        rtcp_feedback,
                    }
                ];
            
                audio_rtp_params.encodings = vec![
                    RtpEncodingParameters {
                        ssrc: Some(audio.ssrc),
                        ..RtpEncodingParameters::default()
                    }
                ];

                producers.push(
                    direct_transport.produce(
                        ProducerOptions::new(MediaKind::Audio, audio_rtp_params)
                    ).await.unwrap()
                );
            }
        }
    }

    return producers;
}
