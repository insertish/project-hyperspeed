import Head from 'next/head'
import { useEffect, useMemo, useState } from 'react'
import styles from '../styles/Home.module.scss'

import { Client } from 'hyperspeed.js';

export default function Home() {
	const [client, setClient] = useState<Client | undefined>();

	const [source, setSource] = useState<MediaProvider | null>(null);

	useEffect(() => {
		if (typeof window === 'undefined') return;

		let client = new Client({
			signalingServer: 'ws://insrt.uk.to:9050',
			debug: true
		});

		setClient(client);
		client.watch(prompt('enter channel id'));
	}, [ ]);

	useEffect(() => {
		if (!client) return;
		client.addListener('streamUpdated', setSource);
		return () => client.removeListener('streamUpdated', setSource);
	}, [ client ]);

	return (
		<div className={styles.container}>
			<Head>
				<title>aaa</title>
			</Head>
			
			{/*<main className={styles.main}>
				<h1 className={styles.title}>
					Project Hyperspeed
				</h1>
			</main>*/}

			<video
				muted
				controls
				onLoadedMetadata={e => e.currentTarget.play()}
				ref={el => {
					if (el) el.srcObject = source
				}} />
		</div>
	)
}
	