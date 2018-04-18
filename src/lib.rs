extern crate mime;
extern crate soft_ascii_string;
extern crate quoted_string;
#[macro_use]
extern crate failure;
extern crate owning_ref;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate nom;
extern crate chrono;
#[cfg_attr(test, macro_use)]
extern crate vec1;
extern crate total_order_multi_map;
//FIXME[rust/macros use private] remove pub re-export
#[cfg_attr(test, macro_use)]
pub extern crate mail_common as __common;

#[cfg(all(test, not(feature="traceing")))]
compile_error! { "testing needs feature `traceing` to be enabled" }

#[macro_use]
mod macros;
mod name;
#[macro_use]
pub mod error;
mod header;
mod convert;
pub mod data;
#[macro_use]
mod header_macro;
#[macro_use]
pub mod map;
pub mod components;
mod header_impl;

pub use self::name::*;
pub use self::header::*;
pub use self::convert::*;
pub use self::header_macro::*;
pub use self::map::HeaderMap;
pub use self::header_impl::*;

// reexports for macros
#[doc(hidden)]
pub use soft_ascii_string::SoftAsciiStr as __SoftAsciiStr;
// I can not reexport a private think anymore, so I need to reexport the
// extern crate and then make the normal name available, too
use __common as common;