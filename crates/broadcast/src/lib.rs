#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "rtc")]
#[cfg_attr(docsrs, doc(cfg(feature = "rtc")))]
pub mod rtc;

#[cfg(feature = "ingest")]
#[cfg_attr(docsrs, doc(cfg(feature = "ingest")))]
pub mod ingest;

#[cfg(feature = "signaling")]
#[cfg_attr(docsrs, doc(cfg(feature = "signaling")))]
pub mod signaling;
