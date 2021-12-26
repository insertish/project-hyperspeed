---
sidebar_position: 2
---

# Broadcast

- Version: `0.0.1`
- License: **AGPLv3**
- Crates.io: [crates.io/hyperspeed-broadcast](https://crates.io/hyperspeed-broadcast)
- Rust Documentation: [docs.rs/hyperspeed-broadcast](https://docs.rs/hyperspeed-broadcast)

[hyperspeed-broadcast](https://gitlab.insrt.uk/insert/project-hyperspeed/-/tree/master/crates/broadcast) is a media ingest server (currently only supports FTL media ingest, forwarding RTP packets to WebRTC clients), it also provides a signaling WebSocket server for establishing new client connections.

## Streaming to Ingest Server (OBS)

To stream from OBS Studio, you must have it compiled with FTL support (use [obs-studio-ftl on Arch Linux]()).

Find `services.json`:
- **Linux**: `~/.config/obs-studio/plugin_config/rtmp-services/services.json`
- **Windows**: `%appdata%\obs-studio\plugin_config\rtmp-services\services.json`

Edit your `services.json` config to include:

```json
{
    "name": "Hyperspeed Ingest",
    "common": true,
    "servers": [
        {
            "name": "Ingest 1",
            "url": "<your hostname>"
        }
    ],
    "recommended": {
        "keyint": 2,
        "output": "ftl_output",
        "max audio bitrate": 160,
        "max video bitrate": 8000,
        "profile": "main",
        "bframes": 0
    }
}
```
