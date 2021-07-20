---
slug: /ftl
sidebar_position: 1
---

# Overview

FTL (Faster Than Light) is a low-latency streaming protocol developed by [Mixer](https://en.wikipedia.org/wiki/Mixer_(service)) (formerly Beam) which offers sub-second latency screen-to-screen (data leaving streamer to reaching viewer's browser).

<p align="center">
<img src="/img/ftl_test.png" width="840px" />

To the right is a source window being streamed and to the left is said stream.
The measured delay here is `650ms`.
</p>

## Explaining FTL's low latency.

*This section uses a considerable amount of information from [Hayden McAfee's FTL engineering notes](https://hayden.fyi/posts/2020-08-03-Faster-Than-Light-protocol-engineering-notes.html).*

There are several key differences between FTL and other typical protocols which allow FTL to achieve sub-second latency:

### TCP vs. UDP Transport

Most streaming services, such as Twitch, use a combination of [RTMP](https://en.wikipedia.org/wiki/Real-Time_Messaging_Protocol) for ingest and [HLS](https://en.wikipedia.org/wiki/HTTP_Live_Streaming) for viewing streams, both of these are TCP protocols which can add a significant amount of latency as each packet needs to be received in the correct order and data needs to be reassembled.

FTL uses [the RTP protocol](https://en.wikipedia.org/wiki/Real-time_Transport_Protocol) for ingest and [WebRTC](https://en.wikipedia.org/wiki/WebRTC) for viewing streams, these are both UDP protocols which means it doesn't matter when, how or in which order packets arrive, if they even arrive at all, meaning that data can be immediately used as it is received.

This greatly reduces latency but makes it more susceptible to network jitter and other interference.

### HLS Chunking

The HLS protocol works by dividing the stream up into chunks that are downloaded over HTTP and then stitched together on the client. This adds additional latency.

## Should I use FTL?

Mixer is dead and the FTL protocol is essentially frozen in time, OBS also plans to [remove ability to stream using FTL](https://github.com/obsproject/obs-studio/discussions/4021), despite this, it is a relatively simple protocol to implement and lives up to its promises. The next best choice would be to work with OBS directly through WebRTC, but as of right now that is still in early days.

If you want a simple streaming protocol, FTL is perfect. If you build anything with FTL you'll likely be using WebRTC somewhere along the way, so migration should be fairly painless.
