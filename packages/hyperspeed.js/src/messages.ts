import type { MediaKind, RtpCapabilities, RtpParameters } from "mediasoup-client/lib/RtpParameters";
import type { DtlsParameters, IceCandidate, IceParameters } from "mediasoup-client/lib/Transport";

export interface TransportOptions {
    id: string;
    dtlsParameters: DtlsParameters;
    iceCandidates: IceCandidate[];
    iceParameters: IceParameters
}

export interface Consume {
    id: string;
    producerId: string;
    kind: MediaKind;
    rtpParameters: RtpParameters;
}

export type ServerboundMessage = (
    {
        type: 'Begin',
        channel_id: string
    } | {
        type: 'Init',
        rtp_capabilities: RtpCapabilities
    } | {
        type: 'Connect',
        dtls_parameters: DtlsParameters,
    } | {
        type: 'Consume'
    } | {
        type: 'Resume',
        id: string
    }
)

export type ClientboundMessage = (
    {
        type: 'Init',
        producers: string[],
        transport: TransportOptions,
        router_rtp_capabilities: Required<RtpCapabilities>
    } | {
        type: 'Connected'
    } | {
        type: 'Consuming',
        consume: Consume[]
    }
)
