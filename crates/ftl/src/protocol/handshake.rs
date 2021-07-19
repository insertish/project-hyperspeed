use super::FtlError;

#[derive(Default, Debug, Clone)]
pub struct Vendor {
    pub name: Option<String>,
    pub version: Option<String>,
}

#[derive(Default, Debug, Clone)]
pub struct Video {
    pub codec: Option<String>,
    pub height: Option<isize>,
    pub width: Option<isize>,
    pub payload_type: Option<u8>,
    pub ssrc: Option<u32>,
}

#[derive(Default, Debug, Clone)]
pub struct Audio {
    pub codec: Option<String>,
    pub payload_type: Option<u8>,
    pub ssrc: Option<u32>,
}

#[derive(Default, Debug, Clone)]
pub struct FtlHandshake {
    pub protocol_version: Option<(isize, isize)>,
    pub vendor: Vendor,
    pub video: Option<Video>,
    pub audio: Option<Audio>,
}

// Finalised versions of handshake.

#[derive(Debug, Clone)]
pub struct KnownVideo {
    pub codec: String,
    pub height: isize,
    pub width: isize,
    pub payload_type: u8,
    pub ssrc: u32,
}

#[derive(Debug, Clone)]
pub struct KnownAudio {
    pub codec: String,
    pub payload_type: u8,
    pub ssrc: u32,
}

#[derive(Debug, Clone)]
pub struct FtlHandshakeFinalised {
    pub protocol_version: (isize, isize),
    pub vendor: Vendor,
    pub video: Option<KnownVideo>,
    pub audio: Option<KnownAudio>,
}

impl FtlHandshake {
    pub fn finalise(self) -> Result<FtlHandshakeFinalised, FtlError> {
        Ok(FtlHandshakeFinalised {
            protocol_version: if let Some((major, minor)) = self.protocol_version {
                if major != 0 && minor != 9 {
                    return Err(FtlError::UnsupportedProtocolVersion)
                }

                (major, minor)
            } else {
                return Err(FtlError::InvalidProtocolVersion)
            },
            vendor: self.vendor,
            video: if let Some(video) = self.video {
                match (video.codec, video.width, video.height, video.payload_type, video.ssrc) {
                    (Some(codec), Some(width), Some(height), Some(payload_type), Some(ssrc)) =>
                        Some(KnownVideo { codec, width, height, payload_type, ssrc }),
                    _ => return Err(FtlError::MissingCodecInformation)
                }
            } else { None },
            audio: if let Some(audio) = self.audio {
                match (audio.codec, audio.payload_type, audio.ssrc) {
                    (Some(codec), Some(payload_type), Some(ssrc)) =>
                        Some(KnownAudio { codec, payload_type, ssrc }),
                    _ => return Err(FtlError::MissingCodecInformation)
                }
            } else { None }
        })
    }
}
