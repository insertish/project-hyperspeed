---
sidebar_position: 2
---

# FTL Protocol

> Protocol Version: `0.9`

There are two parts to FTL, one tells clients how to send data and the other accepts said data.

This page will exclusively focus on describing what is sent between client and server, for an explanation on how FTL is established, see the article on [establishing a connection](/docs/ftl/establishing).

## Control

To authenticate and setup a new FTL stream, clients must first connect to a TCP ingest control server.

:::note

The ingest control server always listens on port `8084`.

FTL clients expect it to be present on this port. In fact, there is no way to specify a different port.

:::

Communication is done through a text protocol where each line (bi-directionally) represents a command or response.

:::warning

As a server, it is safe to interpret incoming commands as being separated by a newline, but you should take care to:
- Ignore carriage returns.
- Ignore empty commands.

Some implementations such as [microsoft/ftl-sdk](https://github.com/microsoft/ftl-sdk/blob/d0c8469f66806b5ea738d607f7d2b000af8b1129/libftl/ftl_helpers.c#L30) and the FTL output in OBS send `\r\n\r\n` instead of `\n` as a command separator.

The reasoning behind this is to prevent packets being lost due to "certain firewalls / anti-malware systems", which otherwise block the packet if they aren't sent as double Windows newlines. According to the source above, the protocol is incorrectly determined as HTTP.

:::

### FTL Commands (Client -> Server)

FTL commands do not have a set structure to them so you do have to guess a bit when interpreting data.

The formats below are given as [Rust format strings](https://doc.rust-lang.org/std/fmt/). The table is also ordered in order of use.

 Command | Description | Format | First Parameter | Second Parameter 
:-------:|:------------|:------:|:---------------:|:----------------:
 HMAC | Request a HMAC payload from the server. | `HMAC`
 Connect | Authenticate with the server providing the channel ID and **hex encoded** stream key hash [^1] in that order. | `CONNECT {} {}` | [String][String] | [String][String]
 Attribute | Provide an attribute for the [FTL handshake](#ftl-handshake). | `{}: {}` [^2] | [String][String] | [String][String]
 Dot | Complete the [FTL handshake](#ftl-handshake) and tell the server we want to start sending media. | `.`
 Ping | Let the server know we're still sending data and ensure the other end is alive too. This includes the channel ID as a parameter, although servers may choose to omit it. | `PING {}` | [String][String]
 Disconnect | Tell the server we are finished. | `DISCONNECT`

#### FTL Handshake

There are a number of attributes sent over by the client which are used to determine capabilities and construct A/V streams.

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

### FTL Responses (Server -> Client)

Responses are done in the form of a status code and body in one line. For example, `200 DATA\n`.

 Event | Code | Description | Example
:------|:----:|:------------|:--------
 HMAC | 200 | This response contains the server's HMAC payload. | `200 abcdef1234\n`
 Ok | 200 | Sent when `Connect` has succeeded. | `200\n`
 Connect | 200 | Tells the client where to send RTP data. | `200. Use UDP port 65535\n`
 Ping | 201 | Keeps the client's connection alive. | `201\n`

TODO: Document https://github.com/microsoft/ftl-sdk/blob/master/libftl/ftl_private.h#L365

[String]: https://doc.rust-lang.org/std/string/struct.String.html
[isize]: https://doc.rust-lang.org/std/string/struct.String.html
[bool]: https://doc.rust-lang.org/std/primitive.bool.html

## Media

[^2]: The stream key hash appears to be prefixed with `$`, this should be omitted when decoding or you may hit an error.

[^1]: Notice the space separation between the colon and value, you should trim both key and value when parsing this data.
