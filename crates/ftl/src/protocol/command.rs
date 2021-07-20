use std::str::FromStr;

use super::FtlError;

#[derive(Debug)]
pub enum FtlCommand {
    HMAC,
    Connect {
        channel_id: String,
        hashed_hmac_payload: String,
    },
    Dot,
    Attribute {
        key: String,
        value: String,
    },
    Ping {
        channel_id: String,
    },
    Disconnect,
}

impl FromStr for FtlCommand {
    type Err = FtlError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        dbg!(s);
        match s {
            "HMAC" => Ok(FtlCommand::HMAC),
            "." => Ok(FtlCommand::Dot),
            "DISCONNECT" => Ok(FtlCommand::Disconnect),
            s => {
                if &s[..4] == "PING" {
                    Ok(FtlCommand::Ping {
                        channel_id: s[5..].to_string(),
                    })
                } else if &s[..7] == "CONNECT" {
                    let parts = &mut s[8..].split(" ");

                    Ok(FtlCommand::Connect {
                        channel_id: parts
                            .next()
                            .ok_or_else(|| FtlError::MissingPart)?
                            .to_string(),
                        hashed_hmac_payload: parts
                            .next()
                            .ok_or_else(|| FtlError::MissingPart)?
                            .to_string(),
                    })
                } else {
                    if s.contains(':') {
                        let mut parts = s.split(':').map(|v| v.trim());

                        return Ok(FtlCommand::Attribute {
                            key: parts
                                .next()
                                .ok_or_else(|| FtlError::MissingPart)?
                                .to_string(),
                            value: parts
                                .next()
                                .ok_or_else(|| FtlError::MissingPart)?
                                .to_string(),
                        });
                    }

                    unimplemented!()
                }
            }
        }
    }
}
