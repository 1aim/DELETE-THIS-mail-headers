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
#[cfg(test)]
extern crate chrono;


#[macro_use]
pub mod error;
pub mod components;
