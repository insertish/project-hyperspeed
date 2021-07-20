#[derive(Debug)]
pub enum FtlError {
    NoLabel,
    IoError,
    RingError,
    DecodeError,
    MissingPart,
    ExternalError,
    Unauthenticated,
    
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
