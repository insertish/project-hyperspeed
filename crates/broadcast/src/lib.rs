#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "ingest")]
#[cfg_attr(docsrs, doc(cfg(feature = "ingest")))]
#[macro_use]
extern crate lazy_static;

#[cfg(feature = "rtc")]
#[cfg_attr(docsrs, doc(cfg(feature = "rtc")))]
pub mod rtc;

#[cfg(feature = "signaling")]
#[cfg_attr(docsrs, doc(cfg(feature = "signaling")))]
pub mod signaling;
