use std::str::FromStr;

use super::FtlError;

#[derive(Debug, PartialEq)]
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

                    Err(FtlError::UnimplementedCommand)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::protocol::FtlCommand;

    #[test]
    fn should_parse_hmac() {
        let command = FtlCommand::from_str("HMAC").unwrap();
        assert_eq!(command, FtlCommand::HMAC);
    }

    #[test]
    fn should_parse_dot() {
        let command = FtlCommand::from_str(".").unwrap();
        assert_eq!(command, FtlCommand::Dot);
    }

    #[test]
    fn should_parse_disconnect() {
        let command = FtlCommand::from_str("DISCONNECT").unwrap();
        assert_eq!(command, FtlCommand::Disconnect);
    }

    #[test]
    fn should_parse_ping() {
        let command = FtlCommand::from_str("PING channel_id").unwrap();
        assert_eq!(command, FtlCommand::Ping { channel_id: "channel_id".to_string() });
    }

    #[test]
    fn should_parse_connect() {
        let command = FtlCommand::from_str("CONNECT channel_id hmac").unwrap();
        assert_eq!(command, FtlCommand::Connect { channel_id: "channel_id".to_string(), hashed_hmac_payload: "hmac".to_string() });
    }

    #[test]
    fn should_parse_attribute() {
        let command = FtlCommand::from_str("ProtocolVersion: 0.9").unwrap();
        assert_eq!(command, FtlCommand::Attribute { key: "ProtocolVersion".to_string(), value: "0.9".to_string() });
    }

    #[test]
    fn should_fail_other() {
        let command = FtlCommand::from_str("i am invalid data");
        assert!(command.is_err())
    }

    #[test]
    fn doc_test() {
        use crate::protocol::FtlCommand;

        let command = FtlCommand::from_str(".").unwrap();
        assert_eq!(command, FtlCommand::Dot);

        let command = FtlCommand::from_str("PING 123").unwrap();
        assert_eq!(command, FtlCommand::Ping {
            channel_id: "123".to_string()
        });
    }
}
