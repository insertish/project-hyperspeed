use std::string::ToString;

#[derive(Debug)]
pub enum FtlResponse {
    HMAC { hmac_payload: String },
    Success,
    Connect { udp_port: u16 },
    Pong,
}

impl ToString for FtlResponse {
    fn to_string(&self) -> String {
        match self {
            FtlResponse::HMAC { hmac_payload } => format!("200 {}\n", hmac_payload),
            FtlResponse::Success => "200\n".to_string(),
            FtlResponse::Connect { udp_port } => format!("200. Use UDP port {}\n", udp_port),
            FtlResponse::Pong => "201\n".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::protocol::FtlResponse;

    #[test]
    fn should_success() {
        let resp = FtlResponse::Success;
        assert_eq!(resp.to_string(), "200\n".to_string());
    }
}
