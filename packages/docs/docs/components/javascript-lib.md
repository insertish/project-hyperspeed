---
sidebar_position: 3
---

# hyperspeed.js

- Version: `0.0.1`
- License: **MIT**
- npm: [npmjs.com/packages/hyperspeed.js](https://www.npmjs.com/package/hyperspeed.js)

[hyperspeed.js](https://gitlab.insrt.uk/insert/project-hyperspeed/-/tree/master/packages/hyperspeed.js) is a library for connecting to Hyperspeed streams (interacts with the broadcast signaling server).

### Example Usage

```javascript
import { Client } from "hyperspeed.js";

const client = new Client({
    // Signaling server to connect to.
    signalingServer: 'ldn1.node.hyperspeed',

    // Whether to log internal events.
    // Default: false
    debug: false,

    // Whether the client should manage its own MediaStream.
    // Default: true
    manageStream: true,
});

client.watch(/* Channel ID */ '77');

client.on('trackCreated', mediaTrack => {
    // Do something with the mediaTrack
});

client.on('streamUpdated', mediaStream => {
    // Do something with the mediaStream
});
```

### Usage with DOM

The library integrates very easily with DOM APIs.

```html
<video muted controls />
```

```javascript
const el = document.querySelector('video');

el.onloadedmetadata = ev => {
    ev.target.play()
};

client.on('streamUpdated', mediaStream => {
    el.srcObject = mediaStream;
});
```

### Constructing your own MediaStream

In some cases, you may want to construct your own stream.

```javascript
const client = new Client({
    [..]
    manageStream: false,
});

let mediaStream;
client.on('trackCreated', mediaTrack => {
    if (mediaStream) {
        mediaStream.addTrack(mediaTrack);
    } else {
        mediaStream = new MediaStream([ mediaTrack ]);
    }

    // el.srcObject = mediaStream;
});
```
