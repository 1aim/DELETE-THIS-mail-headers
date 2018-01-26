#[macro_use]
extern crate mail_codec_core as core;

#[cfg_attr(test, macro_use)]
extern crate vec1;
extern crate mime;
extern crate soft_ascii_string;
extern crate quoted_string;
#[macro_use]
extern crate nom;
#[macro_use]
extern crate quick_error;
extern crate chrono;

#[cfg(all(not(feature="traceing"), test))]
compile_error! { "testing needs feature `traceing` to be enabled" }

#[macro_use]
mod macros;

#[macro_use]
pub mod error;
pub mod components;
pub mod headers;
pub use self::headers::*;