#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod protocol;

#[cfg(feature = "server")]
#[cfg_attr(docsrs, doc(cfg(feature = "server")))]
pub mod server;

#[cfg(feature = "util")]
#[cfg_attr(docsrs, doc(cfg(feature = "util")))]
pub mod util;
