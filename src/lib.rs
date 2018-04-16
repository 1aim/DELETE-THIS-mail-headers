extern crate mime;
extern crate soft_ascii_string;
extern crate quoted_string;
#[macro_use]
extern crate nom;
#[macro_use]
extern crate quick_error;
extern crate chrono;
#[cfg_attr(test, macro_use)]
extern crate vec1;
extern crate total_order_multi_map;
//FIXME[rust/macros use private] remove pub re-export
#[cfg_attr(test, macro_use)]
pub extern crate mail_common as __common;

#[cfg(all(test, not(feature="traceing")))]
compile_error! { "testing needs feature `traceing` to be enabled" }

//TODO order modules
#[macro_use]
mod macros;
#[macro_use]
pub mod error;
pub mod components;
mod name;
#[macro_use]
pub mod map;
#[macro_use]
mod header_macro;
mod header_impl;

pub use self::header_macro::*;
pub use self::map::HeaderMap;
pub use self::name::*;
pub use self::header_impl::*;

// reexports for macros
#[doc(hidden)]
pub use soft_ascii_string::SoftAsciiStr as __SoftAsciiStr;
// I can not reexport a private think anymore, so I need to reexport the
// extern crate and then make the normal name available, too
use __common as common;


pub trait Header {

    /// true if the header can appear at most once
    const MAX_COUNT_EQ_1: bool;

    /// the component representing the header-field, e.g. `Unstructured` for `Subject`
    type Component;

    //FIXME[rust/const fn]: make this a associated constant
    fn name() -> HeaderName;

    //NOTE: this is a circular dependency between Header/HeaderMap
    // but putting up e.g. a GenericHeaderMap trait/interface is
    // not worth the work at all
    /// A function which is meant to be called with a reference
    /// to the final header map before encoding the headers. It is
    /// meant to be used do some of the contextual validations,
    /// like e.g. a `From` header might return a function which
    /// checks if the `From` header has multiple mailboxes and
    /// if so checks if there is a `Sender` header
    ///
    /// Calling a contextual validator with a header map not
    /// containing a header which it is meant to validate
    /// should not cause an error. Only if the header is
    /// there and the component is of the expected type
    /// and it is invalid in the context
    /// an error should be returned.
    const CONTEXTUAL_VALIDATOR: Option<fn(&HeaderMap)-> Result<(), common::error::Error>>;
}

/// all headers defined with `def_headers!` where
/// `MAX_COUNT_EQ_1` is `true` do implement
/// `SingularHeaderMarker` which is required to use
/// the `HeaderMap::get_single` functionality.
pub trait SingularHeaderMarker {}

/// a utility trait allowing us to use type hint structs
/// in `HeaderMap::{contains, get_untyped}`
pub trait HasHeaderName {
    fn get_name(&self) -> HeaderName;
}

impl HasHeaderName for HeaderName {
    fn get_name(&self) -> HeaderName {
        *self
    }
}

impl<H> HasHeaderName for H
    where H: Header
{
    fn get_name(&self) -> HeaderName {
        H::name()
    }
}