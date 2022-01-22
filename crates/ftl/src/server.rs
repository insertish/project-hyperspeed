use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use async_std::{io, task};
use async_std::prelude::*;
use async_trait::async_trait;
use async_std::net::{TcpListener, TcpStream};

use log::{debug, error, info, trace};

use crate::protocol::{FtlCommand, FtlError, FtlHandshake, FtlHandshakeFinalised, FtlResponse};
use crate::util;

pub struct IngestClient {
    channel_id: Option<String>,
    hmac_payload: String,
    handshake: FtlHandshake,
    should_stop: Arc<AtomicBool>
}

#[async_trait]
pub trait IngestServer {
    async fn launch(&'static self, addr: String) -> Result<(), io::Error> {
        let listener = TcpListener::bind(addr).await?;

        while let Ok((stream, address)) = listener.accept().await {
            info!("Remote client connected: {}", address);

            task::spawn(async move {
                let (reader, writer) = &mut (&stream, &stream);

                // Common data needed by client / server.
                let mut client = IngestClient {
                    channel_id: None,
                    hmac_payload: util::generate_hmac(),
                    handshake: FtlHandshake::default(),
                    should_stop: Arc::new(AtomicBool::new(false)),
                };

                // Socket reader
                let mut reader = reader.bytes();
                let mut buffer = Vec::with_capacity(128);
                while let Some(byte) = reader.next().await {
                    if let Ok(byte) = byte {
                        match byte {
                            b'\n' => {
                                if buffer.len() > 0 {
                                    if let Ok(payload) = std::str::from_utf8(&buffer) {
                                        if let Ok(command) = FtlCommand::from_str(payload) {
                                            if let Err(error) = self.handler(&mut client, writer, command).await {
                                                if error.is_err() {
                                                    error!("Failed to execute FTL command. {:?}", error);
                                                }

                                                writer.write(
                                                    error
                                                        .to_string()
                                                        .as_bytes()
                                                )
                                                .await
                                                .ok();

                                                break;
                                            }
                                        } else {
                                            error!("Failed to deserialise FTL command. {}", payload);
                                        }
                                    } else {
                                        error!("Failed to convert buffer to UTF8 string.");
                                    }

                                    buffer.clear();
                                }
                            }
                            // Ignore carriage returns in our implementation.
                            b'\r' => continue,
                            byte => buffer.push(byte)
                        }
                    } else {
                        error!("Failed to read anymore bytes from client.");
                        break;
                    }
                }

                info!("Remote FTL client disconnected.");
                client.should_stop.store(true, Ordering::Relaxed);
                stream.shutdown(std::net::Shutdown::Both).ok();
            });
        }

        Ok(())
    }

    async fn handler(&self, client: &mut IngestClient, mut writer: &TcpStream, command: FtlCommand) -> Result<(), FtlError> {
        match command {
            FtlCommand::HMAC => {
                debug!("Client requested HMAC payload, sending response.");
                writer.write(
                    FtlResponse::HMAC {
                        hmac_payload: client.hmac_payload.clone()
                    }
                    .to_string()
                    .as_bytes()
                )
                .await
                .map_err(|_| FtlError::IoError)?;
    
                Ok(())
            }
            FtlCommand::Connect { channel_id, hashed_hmac_payload } => {
                debug!("Client is connecting, attempting to stream to {}.", &channel_id);
                let known_key = self.get_stream_key(&channel_id)
                    .await.map_err(|_| FtlError::InvalidStreamKey)?;
    
                // * Key starts with $, omit and decode.
                let client_hash = hex::decode(hashed_hmac_payload[1..].to_string())
                    .map_err(|_| FtlError::DecodeError)?;
                
                let key = ring::hmac::Key::new(ring::hmac::HMAC_SHA512, &known_key.as_bytes());
    
                ring::hmac::verify(
                    &key,
                    &hex::decode(client.hmac_payload.clone().into_bytes())
                        .map_err(|_| FtlError::DecodeError)?,
                    &client_hash.as_slice()
                ).map_err(|_| FtlError::RingError)?;

                debug!("Client was verified, ready to stream to {}.", &channel_id);
                client.channel_id = Some(channel_id);

                writer.write(
                    FtlResponse::Success
                        .to_string()
                        .as_bytes()
                )
                .await
                .map_err(|_| FtlError::IoError)?;

                Ok(())
            }
            FtlCommand::Attribute { key, value } => client.handshake.insert(key, value),
            FtlCommand::Dot => {
                if let Some(channel_id) = &client.channel_id {
                    let handshake = client.handshake.clone().finalise()?;
                    let udp_port = self.allocate_ingest(channel_id, handshake, client.should_stop.clone())
                        .await.map_err(|_| FtlError::AllocateError)?;
                    
                    debug!("Client is about to begin stream. Allocated port {}.", udp_port);
                    writer.write(
                        FtlResponse::Connect { udp_port }
                            .to_string()
                            .as_bytes()
                    )
                    .await
                    .map_err(|_| FtlError::IoError)?;

                    Ok(())
                } else {
                    Err(FtlError::InvalidStreamKey)
                }
            }
            FtlCommand::Ping { channel_id } => {
                trace!("Client sent ping. {}", &channel_id);
                writer.write(
                    FtlResponse::Pong
                        .to_string()
                        .as_bytes()
                )
                .await
                .map_err(|_| FtlError::IoError)?;

                Ok(())
            }
            FtlCommand::Disconnect => Err(FtlError::Disconnect)
        }
    }

    async fn get_stream_key(&self, channel_id: &str) -> Result<String, ()>;
    async fn allocate_ingest(&self, channel_id: &str, handshake: FtlHandshakeFinalised, should_stop: Arc<AtomicBool>) -> Result<u16, ()>;
}
