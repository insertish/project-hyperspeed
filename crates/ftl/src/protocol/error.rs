use std::string::ToString;

#[derive(Debug)]
pub enum FtlError {
    IoError,
    AllocateError,
    RingError,
    DecodeError,
    MissingPart,

    InvalidStreamKey, // applies to channel id as well
    ChannelNotAuthorized,
    ChannelInUse,
    UnsupportedRegion,
    GameBlocked,
    
    InvalidProtocolVersion,
    UnsupportedProtocolVersion,
    MissingCodecInformation,
    UnimplementedCommand,
    
    Disconnect,
}

impl FtlError {
    pub fn is_err(&self) -> bool {
        match self {
            FtlError::Disconnect => false,
            _ => true
        }
    }
}

impl ToString for FtlError {
    fn to_string(&self) -> String {
        match self {
            FtlError::IoError => "500 Internal Server Error\n".to_string(),
            FtlError::AllocateError => "500 Internal Server Error\n".to_string(),
            FtlError::RingError => "400 HMAC Decode Error\n".to_string(),
            FtlError::DecodeError => "400 HMAC Decode Error\n".to_string(),
            FtlError::MissingPart => "400 Bad Request\n".to_string(),

            FtlError::InvalidStreamKey => "405 Invalid stream key\n".to_string(),
            FtlError::ChannelNotAuthorized => "401 Channel not authorized to stream\n".to_string(),
            FtlError::ChannelInUse => "406 Channel actively streaming\n".to_string(),
            FtlError::UnsupportedRegion => "407 Streaming from your region is not authorized\n".to_string(),
            FtlError::GameBlocked => "409 Channel is not allowed to stream set game\n".to_string(),

            FtlError::InvalidProtocolVersion => "400 Invalid Protocol Version\n".to_string(),
            FtlError::UnsupportedProtocolVersion => "402 Outdated FTL SDK version\n".to_string(),
            FtlError::MissingCodecInformation => "400 Missing Codec Information\n".to_string(),
            FtlError::UnimplementedCommand => "901 Invalid Command\n".to_string(),
            _ => "".to_string(),
        }
    }
}
