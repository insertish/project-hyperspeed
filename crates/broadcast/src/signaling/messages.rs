use mediasoup::rtp_parameters::{MediaKind, RtpCapabilities, RtpCapabilitiesFinalized, RtpParameters};
use mediasoup::data_structures::{DtlsParameters, IceCandidate, IceParameters};
use mediasoup::transport::TransportId;
use mediasoup::consumer::ConsumerId;
use mediasoup::producer::ProducerId;

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransportOptions {
    pub id: TransportId,
    pub dtls_parameters: DtlsParameters,
    pub ice_candidates: Vec<IceCandidate>,
    pub ice_parameters: IceParameters,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerboundMessage {
    Begin {
        channel_id: String
    },
    Init {
        rtp_capabilities: RtpCapabilities
    },
    Connect {
        dtls_parameters: DtlsParameters
    },
    Consume,
    Resume {
        id: ConsumerId
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Consume {
    pub id: ConsumerId,
    pub producer_id: ProducerId,
    pub kind: MediaKind,
    pub rtp_parameters: RtpParameters
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientboundMessage {
    Init {
        producers: Vec<ProducerId>,
        transport: TransportOptions,
        router_rtp_capabilities: RtpCapabilitiesFinalized
    },
    Connected,
    #[serde(rename_all = "camelCase")]
    Consuming {
        consume: Vec<Consume>
    }
}
