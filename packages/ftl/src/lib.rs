use std::str::FromStr;

use async_std::{io, task};
use async_std::net::TcpListener;
use async_std::prelude::*;

#[derive(Debug)]
pub enum FtlCommand {
    HMAC,
    Connect {
        channel_id: String,
        stream_key: String
    },
    Dot,
    Attribute {
        key: String,
        value: String
    },
    Ping {
        channel_id: String
    },
    Disconnect,
}

#[derive(Debug)]
pub enum FtlError {
    NoLabel
}

impl FromStr for FtlCommand {
    type Err = FtlError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "HMAC" => Ok(FtlCommand::HMAC),
            "." => Ok(FtlCommand::Dot),
            "DISCONNECT" => Ok(FtlCommand::Disconnect),
            s => {
                if &s[..4] == "PING" {
                    Ok(FtlCommand::Ping { channel_id: s[5..].to_string() })
                } else if &s[..7] == "CONNECT" {
                    let parts = &mut s[8..].split(" ");

                    Ok(FtlCommand::Connect {
                        channel_id: parts.next().unwrap().to_string(),
                        stream_key: parts.next().unwrap().to_string(),
                    })
                } else {
                    if s.contains(':') {
                        let mut parts = s.split(':').map(|v| v.trim());

                        return Ok(FtlCommand::Attribute {
                            key: parts.next().unwrap().to_string(),
                            value: parts.next().unwrap().to_string(),
                        })
                    }

                    unimplemented!()
                }
            }
        }
    }
}

pub async fn launch() -> Result<(), io::Error> {
    let listener = TcpListener::bind("127.0.0.1:8084").await?;

    while let Ok((stream, _)) = listener.accept().await {
        task::spawn(async move {
            let (reader, writer) = &mut (&stream, &stream);

            let mut b = reader.bytes();
            let mut buf = Vec::with_capacity(1024);

            let hmac_payload = generate_hmac();

            #[derive(Default, Debug)]
            struct Vendor {
                pub name: Option<String>,
                pub version: Option<String>
            }

            #[derive(Default, Debug)]
            struct Video {
                pub codec: Option<String>,
                pub height: Option<isize>,
                pub width: Option<isize>,
                pub payload_type: Option<u8>,
                pub ssrc: Option<u32>,
            }

            #[derive(Default, Debug)]
            struct Audio {
                pub codec: Option<String>,
                pub payload_type: Option<u8>,
                pub ssrc: Option<u32>,
            }

            #[derive(Default, Debug)]
            struct FtlHandshake {
                pub protocol_version: Option<(isize, isize)>,
                pub vendor: Vendor,
                pub video: Option<Video>,
                pub audio: Option<Audio>
            }

            let mut handshake = FtlHandshake::default();

            while let Some(byte) = b.next().await {
                if let Ok(byte) = byte {
                    if byte == b'\n' {
                        if buf.len() > 0 {
                            if let Ok(payload) = std::str::from_utf8(&buf) {
                                match FtlCommand::from_str(payload).unwrap() {
                                    FtlCommand::HMAC => {
                                        writer.write(b"200 ").await.unwrap();
                                        writer.write(hmac_payload.as_bytes()).await.unwrap();
                                        writer.write(b"\n").await.unwrap();
                                    }
                                    FtlCommand::Connect { channel_id, stream_key } => {
                                        let known_key = match channel_id.as_ref() {
                                            "77" => hex::decode("6965445178535a377135384545654c54766a6134514b4b477a6e6477556b5651").unwrap(),
                                            _ => unimplemented!()
                                        };

                                        // Key starts with $, omit and decode.
                                        let client_hash = hex::decode(stream_key[1..].to_string()).unwrap();
                                        let key = ring::hmac::Key::new(ring::hmac::HMAC_SHA512, &known_key);

                                        ring::hmac::verify(&key, &decode(hmac_payload.clone().into_bytes()).unwrap(), &client_hash.as_slice()).unwrap();

                                        writer.write(b"200\n").await.unwrap();
                                    }
                                    FtlCommand::Attribute { key, value } => {
                                        match key.as_ref() {
                                            "ProtocolVersion" => {
                                                let mut parts = value.split('.');
                                                handshake.protocol_version = Some((
                                                    parts.next().unwrap().parse().unwrap(),
                                                    parts.next().unwrap().parse().unwrap()
                                                ));
                                            }
                                            "VendorName" => handshake.vendor.name = Some(value),
                                            "VendorVersion" => handshake.vendor.version = Some(value),
                                            "Video" |
                                            "Audio" => match value.as_ref() {
                                                "true" => {
                                                    if key == "Video" {
                                                        handshake.video = Some(Video::default());
                                                    } else {
                                                        handshake.audio = Some(Audio::default());
                                                    }
                                                },
                                                "false" => {},
                                                _ => panic!("Failed to deserialise boolean.")
                                            }
                                            "VideoCodec" |
                                            "VideoHeight" |
                                            "VideoWidth" |
                                            "VideoPayloadType" |
                                            "VideoIngestSSRC" => if let Some(mut video) = handshake.video.as_mut() {
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
                                            "AudioIngestSSRC" => if let Some(mut audio) = handshake.audio.as_mut() {
                                                match key.as_ref() {
                                                    "AudioCodec" => audio.codec = Some(value),
                                                    "AudioPayloadType" => audio.payload_type = Some(value.parse().unwrap()),
                                                    "AudioIngestSSRC" => audio.ssrc = Some(value.parse().unwrap()),
                                                    _ => unreachable!()
                                                }
                                            }
                                            _ => {}
                                        }
                                    }
                                    FtlCommand::Dot => {
                                        writer.write(b"200. Use UDP port 65535\n").await.unwrap();
                                    }
                                    FtlCommand::Ping { channel_id: _ } => {
                                        writer.write(b"201\n").await.unwrap();
                                    }
                                    FtlCommand::Disconnect => {
                                        dbg!("disconnect");
                                    }
                                }
                            } else {
                                eprintln!("Failed to parse payload: {:?}", &buf);
                            }
                        }

                        buf.clear();
                        continue;
                    }

                    if byte == b'\r' {
                        continue;
                    }

                    buf.push(byte);
                } else {
                    eprintln!("Failed to read byte.");
                }
            }
        });
    }

    Ok(())
}

use rand::distributions::{Alphanumeric, Uniform};
use rand::{thread_rng, Rng};
use hex::{decode, encode};

fn generate_hmac() -> String {
    let dist = Uniform::new(0x00, 0xFF);
    let mut hmac_payload: Vec<u8> = Vec::new();
    let mut rng = thread_rng();
    for _ in 0..128 {
        hmac_payload.push(rng.sample(dist));
    }
    encode(hmac_payload.as_slice())
}
