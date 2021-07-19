use std::num::{NonZeroU32, NonZeroU8};

use mediasoup::router::RouterOptions;
use mediasoup::rtp_parameters::{MimeTypeAudio, MimeTypeVideo, RtcpFeedback, RtpCodecCapability, RtpCodecParametersParameters};

use super::routers::DataSource;

pub struct VideoCodec {
    pub mime_type: MimeTypeVideo,
    pub clock_rate: u32,
    pub parameters: RtpCodecParametersParameters,
    pub rtcp_feedback: Vec<RtcpFeedback>
}

impl VideoCodec {
    pub fn from(codec: &str) -> VideoCodec {
        let mime_type = match codec {
            "H264" => MimeTypeVideo::H264,
            _ => unimplemented!()
        };

        let parameters = match mime_type {
            MimeTypeVideo::H264 => RtpCodecParametersParameters::from([
                ("packetization-mode", 0_u32.into()),
                ("level-asymmetry-allowed", 0_u32.into())
            ]),
            _ => unreachable!()
        };

        let clock_rate = match mime_type {
            MimeTypeVideo::H264 => 90_000,
            _ => unreachable!()
        };

        let rtcp_feedback = match mime_type {
            MimeTypeVideo::H264 => vec! [
                RtcpFeedback::Nack,
                RtcpFeedback::NackPli,
                RtcpFeedback::CcmFir,
                RtcpFeedback::GoogRemb,
                RtcpFeedback::TransportCc,
            ],
            _ => unreachable!()
        };

        VideoCodec {
            mime_type,
            clock_rate,
            parameters,
            rtcp_feedback
        }
    }
}

pub struct AudioCodec {
    pub mime_type: MimeTypeAudio,
    pub clock_rate: u32,
    pub parameters: RtpCodecParametersParameters,
    pub rtcp_feedback: Vec<RtcpFeedback>
}

impl AudioCodec {
    pub fn from(codec: &str) -> AudioCodec {
        let mime_type = match codec {
            "OPUS" => MimeTypeAudio::Opus,
            _ => unimplemented!()
        };

        let parameters = RtpCodecParametersParameters::default();
        let rtcp_feedback = Vec::new();

        let clock_rate = match mime_type {
            MimeTypeAudio::Opus => 48_000,
            _ => unreachable!()
        };

        AudioCodec {
            mime_type,
            clock_rate,
            parameters,
            rtcp_feedback
        }
    }
}

pub fn init_codecs(options: &mut RouterOptions, source: &DataSource) {
    match source {
        DataSource::Ftl(handshake) => {
            if let Some(video) = &handshake.video {
                let VideoCodec { mime_type, clock_rate, parameters, rtcp_feedback }
                    = VideoCodec::from(&video.codec);

                options.media_codecs.push(
                    RtpCodecCapability::Video {
                        mime_type,
                        preferred_payload_type: None,
                        clock_rate: NonZeroU32::new(clock_rate).unwrap(),
                        parameters,
                        rtcp_feedback
                    }
                );
            }

            if let Some(audio) = &handshake.audio {
                let AudioCodec { mime_type, clock_rate, parameters, rtcp_feedback }
                    = AudioCodec::from(&audio.codec);

                options.media_codecs.push(
                    RtpCodecCapability::Audio {
                        mime_type,
                        preferred_payload_type: None,
                        clock_rate: NonZeroU32::new(clock_rate).unwrap(),
                        channels: NonZeroU8::new(2).unwrap(),
                        parameters,
                        rtcp_feedback
                    }
                );
            }
        }
    }
}
