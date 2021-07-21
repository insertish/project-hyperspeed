---
sidebar_position: 2
---

# FTL Protocol

> Protocol Version: `0.9`

There are two parts to FTL, one tells clients how to send data and the other accepts said data.

This page will exclusively focus on describing what is sent between client and server, for an explanation on how FTL is established, see the article on [establishing a connection](/docs/ftl/establishing).

## Control Protocol ("Charon")

To authenticate and setup a new FTL stream, clients must first connect to a TCP ingest control server.

:::note

The ingest control server always listens on port `8084`.

FTL clients expect it to be present on this port. In fact, there is no way to specify a different port.

:::

Communication is done through a text protocol where each line (bi-directionally) represents a command or response. This is similar in nature to the SMTP protocol, for example:

```yaml
Client: CONNECT 123 abcdef\n
     C: ProtocolVersion: 0.9\n
     C: VendorName: My Streaming Software\n
     C: .\n
Server: 200.
```

:::warning

Some implementations such as [microsoft/ftl-sdk](https://github.com/microsoft/ftl-sdk/blob/d0c8469f66806b5ea738d607f7d2b000af8b1129/libftl/ftl_helpers.c#L30) and the FTL output in OBS send `\r\n\r\n` instead of `\n` as a command separator.

The reasoning behind this is to prevent packets being lost due to "certain firewalls / anti-malware systems", which otherwise block the packet if they aren't sent as double Windows newlines. According to the source above, the protocol is incorrectly determined as HTTP.

As a client, always send `\r\n\r\n` instead of `\n`.

As a server, it is safe to interpret incoming commands as being separated by a newline, but you should take care to:
- Ignore carriage returns.
- Ignore empty commands (empty lines).

:::

### FTL Commands (Client -> Server)

FTL commands do not have a set structure to them so you do have to guess a bit when interpreting data.

The formats below are given as [Rust format strings](https://doc.rust-lang.org/std/fmt/). The table is also ordered in order of use.

 Command | Description | Format | First Parameter | Second Parameter 
:-------:|:------------|:------:|:---------------:|:----------------:
 HMAC | Request a HMAC payload from the server. | `HMAC`
 Connect | Authenticate with the server providing the channel ID and **hex encoded hashed HMAC payload** (prefix: $). | `CONNECT {} ${}` | Channel ID<br/> [String][String] | Hashed data<br/> [String][String]
 Attribute | Provide an attribute for the [FTL handshake](#ftl-handshake). | `{}: {}` [^1] | Key<br/> [String][String] | Value<br/> [String][String]
 Dot | Complete the [FTL handshake](#ftl-handshake) and tell the server we want to start sending media. | `.`
 Ping | Let the server know we're still sending data and ensure the other end is alive too. This includes the channel ID as a parameter, although servers may choose to omit it. | `PING {}` | Channel ID<br/> [String][String]
 Disconnect | Tell the server we are finished. | `DISCONNECT`

### FTL Handshake

There are a number of attributes sent over by the client which are used to determine capabilities and construct A/V streams. These are sent by the client after successful authentication.

 Key | Value | Description | Example
:----|:-----:|:------------|:--------
`ProtocolVersion` | [isize][isize].[isize][isize] | Two unsigned integers separated by a period, representing major and minor parts of the protocol version. | `0.9`
`VendorName` | [String][String] | Name of streaming software in use. | `OBS Studio`
`VendorVersion` | [String][String] | Version of streaming software in use. | `27.0.1`
`Video` | [bool][bool] | Whether video is enabled for this stream. | `true`
`VideoCodec` | [String][String] | Video codec in use. | `H264` / `VP8`
`VideoHeight` | [isize][isize] | Height of video stream. | `1920`
`VideoWidth` | [isize][isize] | Width of video stream. | `1080`
`VideoPayloadType` | [isize][isize] | RTP payload type, currently a constant value. | `96`
`VideoIngestSSRC` | [isize][isize] | RTP video synchronization source.<br/> Currently determined as `channel_id + 1`. | `78`
`Audio` | [bool][bool] | Whether audio is enabled for this stream. | `true`
`AudioCodec` | [String][String] | Audio codec in use. | `OPUS`
`AudioPayloadType` | [isize][isize] | RTP payload type, currently a constant value. | `97`
`AudioIngestSSRC` | [isize][isize] | RTP audio synchronization source.<br/> Currently determined as the value of `channel_id`. | `78`

:::note

Some of these fields are not strictly necessary, although the client is expected to send this data anyways.

The Hyperspeed FTL implementation ensures that:
- A valid protocol version is given.
- A vendor name / version is optionally given.
- If video is enabled, all relevant fields are present.
- If audio is enabled, all relevant fields are present.

:::

### FTL Responses (Server -> Client)

Responses are done in the form of a status code and body in one line. For example, `200 DATA\n`.

 Event | Code | Description | Example
:------|:----:|:------------|:--------
HMAC | 200 | This response contains the server's HMAC payload. This should be a 256-character hex-encoded string, you can encode 128-bits of random data and use it here. | `200 abcdef1234\n` [^3]
Ok | 200 | Sent when `Connect` has succeeded. | `200\n`
Connect | 200 | Tells the client where to send RTP data. | `200. Use UDP port 65535\n` [^2]
Ping | 201 | Keeps the client's connection alive. | `201\n`

### Error Responses

There are a number of documented error responses, [found here](https://github.com/microsoft/ftl-sdk/blob/master/libftl/ftl_private.h#L365), which are listed below. Each of the following (to the best of my knowledge) can be sent as a single status code on a new line, for example, `400\n`.

 Error | Code | Description
:------|:----:|:------------
Bad Request | 400 | The handshake was not formatted correctly.
Unauthorised | 401 | This channel ID is not authorised to stream.
Old Version | 402 | This FTL protocol version is no longer supported.
Audio SSRC collision | 403 | Audio SSRC value collides with another.
Video SSRC collision | 404 | Video SSRC value collides with another.
Invalid Stream Key | 405 | Corresponding channel does not match this key.
Channel In Use | 406 | Channel ID successfully authenticated but it is already actively streaming.
Region Unsupported | 407 | Streaming from this country or region is not authorised by local governments.
No Media Timeout | 408 | Media ingest server has not received any data and has disconnected the client.
Game Blocked | 409 | The game the user account is set to cannot be streamed.
Server Terminate | 410 | The server has terminated the stream.
Internal Server Error | 500 | The server has hit an internal error.
Internal Memory Error | 900 | Internal server error relating to memory.
Internal Command Error | 901 | Internal server error relating to failing procedures.
Internal Socket Closed | 902 | The socket has unexpectedly closed on the server.
Internal Socket Timeout | 903 | The server has not received any data and has disconnected the client.

[String]: https://doc.rust-lang.org/std/string/struct.String.html
[isize]: https://doc.rust-lang.org/std/string/struct.String.html
[bool]: https://doc.rust-lang.org/std/primitive.bool.html

## Media Protocol ("Styx")

Once the handshake with the control server is complete, the client sends RTP packets (containing A/V streams) over UDP to the given port. The protocol for the media connection is similar to the RTP protocol but FTL ignores certain limitations.

### RTP packets

Each RTP packet consists of a header followed by payload data (we won't go into serialisation details here, for deserialisation in Rust use the [rtp crate](https://docs.rs/rtp/0.3.0/rtp/packet/struct.Packet.html#method.unmarshal)). The [RTP header](https://datatracker.ietf.org/doc/html/rfc3550#section-5.1) consists of the following fields:

```rust
struct Header {
    version: u8
    padding: bool
    extension: bool
    marker: bool
    payload_type: u8
    sequence_number: u16
    timestamp: u32
    ssrc: u32
    csrc: Vec<u32>
    extension_profile: u16
    extensions: Vec<Extension>
}
```

The fields we are particularly interested in are `payload_type` and `ssrc`, when processing incoming packets we use the payload type to determine where the packet should be routed (more on this below) and we could also verify the SSRC to ensure it matches what the client previously gave us. Alternatively, the server could opt to route packets depending on the SSRC instead of type but as the type is a constant it's easier to follow.

### Differences from RTP protocol

The payload type field specified in the RTP header specifies the type of payload (or otherwise the type of media being sent, video or audio), the RTP specification forbids multiplexing different types of media in the same stream and instead recommends a unique port is used for each stream. FTL instead sends both V/A data on the same port and separates them by the use of the payload type field. (96 for video, 97 for audio)

:::warning

While the payload type field is constant in the specification, a client still has the ability to theoretically negotiate a different payload type for each stream, you should use the payload type provided in the handshake for handling incoming data.

:::

[^1]: Notice the space separation between the colon and value, you should trim both key and value when parsing this data.
[^3]: See https://github.com/microsoft/ftl-sdk/blob/master/libftl/ftl_helpers.c#L112 for details on how this is parsed.
[^2]: The client (microsoft/ftl-sdk) expects this exact format, see https://github.com/microsoft/ftl-sdk/blob/master/libftl/ftl_helpers.c#L48.
