---
sidebar_position: 3
---

# Establishing a connection

> Protocol Version: `0.9`

This page describes the steps required to negotiate with a FTL control server and start sending data. You may want to refer to the protocol documentation when going through this page, particularly [commands](/ftl/protocol#ftl-commands-client---server) and [responses](/ftl/protocol#ftl-responses-server---client).

### 0. Prerequisites

The client must have a stream key ready for authentication.

Stream keys are in the format of `{channel ID}-{shared key}`, e.g. `77-ieDQxSZ7q58EEeLTvja4QKKGzndwUkVQ`.

### 1. Authentication

The client should open a TCP socket to the remote control server, e.g. `ingest.server.ftl:8084`.

Next, we need to request a HMAC payload from the server (the server can provide virtually random data here as long as the data is kept around to then verify the client hash).

```
Client: HMAC\n
Server: 200 aaaa[snip]\n
```

The client should now parse the HMAC hex payload into a binary blob then hash it with the shared key that is in the second part of the streaming token.
The client can now tell the server that it wants to connect:

```
Client: CONNECT 77 $aaaa[snip]\n
Server: 200\n
```

The client should now send all the required attributes to form an [FTL handshake](/ftl/protocol#ftl-handshake), then send a Dot command when finished. The server will respond with a media ingest port.

```
Client: ProtocolVersion: 0.9\n
     C: VendorName: OBS Studio\n
     C: [.. additional attributes ..]
     C: .\n
Server: 200. Use UDP port 65534\n
```

### 2. Sending media

The client can now start sending RTP packets over UDP to the port provided by the server.

### 3. Maintaing the connection

The client should ping the server every 5 seconds to prevent being disconnected.

```
Client: PING 77\n
Server: 201\n
```

To stop the stream, the client should gracefully disconnect:

```
Client: DISCONNECT\n
```

## Full Example

```
Client: HMAC\n
Server: 200 5e0c41f532c44e01b06cdb3ca5d8dc699d3a031e716a63a137a6899d6bc7832b77b591e8a03e9f14e20bbccc1b0b674450a45b275461857efda6434d64993253dd534220c45f197c6dad61bdc0bae12fd1442e22939e650731e4ee51d03632a108b5f50831ca6f239876f348123b6d15bf31a4882ef75b4a57dfa8273f05432a\n
Client: CONNECT 77 $319f678a5871a2197fac50b314fd62435904aaabddaf87ec65f9c05351425d95f06d9e525c40ca9d344e4b22bdafdf64769a431464fabd9fac86cef820e5c0a1\n
Server: 200\n
Client: ProtocolVersion: 0.9\n
     C: VendorName: OBS Studio\n
     C: VendorVersion: 27.0.1-1\n
     C: Video: true\n
     C: VideoCodec: H264\n
     C: VideoHeight: 720\n
     C: VideoWidth: 1280\n
     C: VideoPayloadType: 96\n
     C: VideoIngestSSRC: 78\n
     C: Audio: true\n
     C: AudioCodec: OPUS\n
     C: AudioPayloadType: 97\n
     C: AudioIngestSSRC: 77\n
     C: .\n
Server: 200. Use UDP port 65534\n

Client: PING 77\n
Server: 201\n

Client: DISCONNECT\n
```
