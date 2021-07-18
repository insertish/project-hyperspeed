import Head from 'next/head'
import { useEffect, useState } from 'react'
import styles from '../styles/Home.module.scss'

import { Device } from 'mediasoup-client';
import { Transport } from 'mediasoup-client/lib/Transport';

export default function Home() {
	const [src, setSrc] = useState<MediaProvider | null>(null);

	useEffect(() => {
		if (typeof window === 'undefined') return;
		const ws = new WebSocket('ws://localhost:9050');
		const device = new Device();

		ws.onopen = () => {
			console.info('Asking to begin Init');
			ws.send(JSON.stringify({ type: 'Begin' }));
		}

		let consumerTransport: Transport | undefined;
		let success: Function | undefined;
		let receiveMediaStream: MediaStream | undefined;

		ws.onmessage = async e => {
			if (typeof e.data === 'string') {
				const data = JSON.parse(e.data);
				console.log('Websocket data:', data);
				switch (data.type) {
					case 'Init': {
						console.info('Server sent us Init');

						await device.load({
							routerRtpCapabilities: data.router_rtp_capabilities
						});

						console.info('Loaded mediasoup device.');
						
						ws.send(JSON.stringify({ type: 'Init', rtp_capabilities: device.rtpCapabilities }));

						consumerTransport = device.createRecvTransport(
							data.transport
						);

						consumerTransport.on('connect', ({ dtlsParameters: dtls_parameters }, s) => {
							ws.send(JSON.stringify({ type: 'Connect', dtls_parameters }));
							success = s;
						});

						console.info('Created consumer transport.');

						ws.send(JSON.stringify({ type: 'Consume' }));

						/*for (const producer_id of data.producers) {
							ws.send(JSON.stringify({ type: 'Consume', producer_id }));
						}*/

						break;
					}
					case 'Consuming': {
						if (!consumerTransport) return;

						for (const entry of data.consume) {
							console.log(entry);
							const consumer = await consumerTransport.consume(entry);

							ws.send(JSON.stringify({ type: 'Resume', id: consumer.id }));

							if (receiveMediaStream)
							{
								receiveMediaStream.addTrack(consumer.track);
								setSrc(receiveMediaStream);
							}
							else
							{
								receiveMediaStream = new MediaStream([ consumer.track ]);
								setSrc(receiveMediaStream);
							}
						}

						break;
					}
					case 'Connected': {
						success?.();
						break;
					}
					default: {
						console.info('Unhandled packet.', data);
					}
				}
			}
		}
	}, [ ]);

	return (
		<div className={styles.container}>
			<Head>
				<title>Project Hyperspeed</title>
			</Head>
			
			<main className={styles.main}>
				<h1 className={styles.title}>
					Project Hyperspeed
				</h1>
			</main>

			<video controls muted onLoadedMetadata={e => e.currentTarget.play()} ref={el => {
				if (el) el.srcObject = src
			}} />
		</div>
	)
}
	