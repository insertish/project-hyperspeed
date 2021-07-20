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

impl FtlHandshake {
    /// Given a FTL attribute, insert it into the Handshake structure.
    pub fn insert(&mut self, key: String, value: String) -> Result<(), FtlError> {
        // ! FIXME: causing panics here, try not doing that
        match key.as_ref() {
            "ProtocolVersion" => {
                let mut parts = value.split('.');
                self.protocol_version = Some((
                    parts.next().unwrap().parse().unwrap(),
                    parts.next().unwrap().parse().unwrap()
                ));
            }
            "VendorName" => self.vendor.name = Some(value),
            "VendorVersion" => self.vendor.version = Some(value),
            "Video" |
            "Audio" => match value.as_ref() {
                "true" => {
                    if key == "Video" {
                        self.video = Some(Video::default());
                    } else {
                        self.audio = Some(Audio::default());
                    }
                },
                "false" => {},
                _ => panic!("Failed to deserialise boolean.")
            }
            "VideoCodec" |
            "VideoHeight" |
            "VideoWidth" |
            "VideoPayloadType" |
            "VideoIngestSSRC" => if let Some(mut video) = self.video.as_mut() {
                match key.as_ref() {
                    "VideoCodec" => video.codec = Some(value),
                    "VideoHeight" => video.height = Some(value.parse().unwrap()),
                    "VideoWidth" => video.width = Some(value.parse().unwrap()),
                    "VideoPayloadType" => video.payload_type = Some(value.parse().unwrap()),
                    "VideoIngestSSRC" => video.ssrc = Some(value.parse().unwrap()),
                    _ => unreachable!()
                }
            }
            "AudioCodec" |
            "AudioPayloadType" |
            "AudioIngestSSRC" => if let Some(mut audio) = self.audio.as_mut() {
                match key.as_ref() {
                    "AudioCodec" => audio.codec = Some(value),
                    "AudioPayloadType" => audio.payload_type = Some(value.parse().unwrap()),
                    "AudioIngestSSRC" => audio.ssrc = Some(value.parse().unwrap()),
                    _ => unreachable!()
                }
            }
            _ => {}
        }

        Ok(())
    }
}

//#region Finalised Handshake.
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
//#endregion

#[cfg(test)]
mod tests {
    #[test]
    fn full_test() {
        use crate::protocol::{FtlCommand, FtlHandshake};

        // Start constructing handshake somewhere in your code.
        let mut handshake = FtlHandshake::default();

        // Example incoming command.
        let command = FtlCommand::Attribute {
            key: "ProtocolVersion".to_string(),
            value: "0.9".to_string()
        };

        // Match attribute and insert it into handshake.
        if let FtlCommand::Attribute { key, value } = command {
            handshake.insert(key, value).unwrap();
            // You should handle any errors here,
            // but we know this isn't going to fail.
        }

        // Once we have the minimum amount of information,
        // (see the note under FTL handshakes on the protocol page)
        // we can finalise the handshake, this verifies all data is
        // correct, such as the protocol version and ensuring if A/V
        // streams are enabled that they have all fields present.
        let handshake = handshake.finalise().unwrap();
        assert_eq!(handshake.protocol_version.1, 9);
    }
}
