import { Device } from "mediasoup-client";
import { Transport } from "mediasoup-client/lib/Transport";
import { ClientboundMessage, ServerboundMessage } from "./messages";

import EventEmitter from 'events';

interface ClientEvents {
    trackCreated: (track: MediaStreamTrack) => void,
    streamUpdated: (stream: MediaStream) => void,
}

interface Options {
    signalingServer: string;

    debug?: boolean;
    manageStream?: boolean;
    autoReconnect?: boolean;
}

export class Client extends EventEmitter {
    device: Device;
    options: Options;

    private ws?: WebSocket;
    private consumerTransport?: Transport;
    private receiveMediaStream?: MediaStream;

    private _success?: Function;

    constructor(options: Options) {
        super();
        this.device = new Device();
        this.options = {
            manageStream: true,
            ...options
        };
    }

    reset() {
        delete this.ws;
        delete this.consumerTransport;
        delete this.receiveMediaStream;
    }

    send(data: ServerboundMessage) {
        if (!this.ws) throw "WebSocket does not exist.";
        this.ws.send(JSON.stringify(data));
    }

    watch(channel_id: string) {
        if (this.ws) throw "Client already active.";

        this.ws = new WebSocket(this.options.signalingServer);
        this.ws.onopen = () => {
            if (this.options.debug) console.info('Asking to begin Init');
            this.send({ type: 'Begin', channel_id });
        };

        if (this.options.autoReconnect) {
            this.ws.onclose = () => {
                if (this.options.debug) console.warn('Disconnected, attempting to reconnect...');
                this.reset();
                this.watch(channel_id);
            };
        }

		this.ws.onmessage = async e => {
			if (typeof e.data === 'string') {
				const data = JSON.parse(e.data) as ClientboundMessage;
				if (this.options.debug) console.debug('Websocket data:', data);

				switch (data.type) {
					case 'Init': {
						if (this.options.debug) console.info('Server sent us Init');

						await this.device.load({
							routerRtpCapabilities: data.router_rtp_capabilities
						});

						if (this.options.debug) console.info('Loaded mediasoup device.');
						
						this.send({
                            type: 'Init',
                            rtp_capabilities: this.device.rtpCapabilities
                        });

						this.consumerTransport = this.device.createRecvTransport(data.transport);

						this.consumerTransport.on('connect', ({ dtlsParameters: dtls_parameters }, success) => {
							this.send({
                                type: 'Connect',
                                dtls_parameters
                            });

							this._success = success;
						});

						if (this.options.debug) console.info('Created consumer transport.');

						this.send({
                            type: 'Consume'
                        });

						break;
					}
					case 'Consuming': {
						if (!this.consumerTransport) return;

						for (const entry of data.consume) {
							const consumer = await this.consumerTransport.consume(entry);
                            if (this.options.debug) console.info('Created new track of type:', consumer.kind);

							this.send({
                                type: 'Resume',
                                id: consumer.id
                            });

                            this.emit('trackCreated', consumer.track);

                            if (this.options.manageStream) {
                                if (this.receiveMediaStream) {
                                    this.receiveMediaStream.addTrack(consumer.track);
                                } else {
                                    this.receiveMediaStream = new MediaStream([ consumer.track ]);
                                }

                                this.emit('streamUpdated', this.receiveMediaStream);
                            }
						}

						break;
					}
					case 'Connected': {
						this._success?.();
						break;
					}
				}
			}
		};
    }
}
