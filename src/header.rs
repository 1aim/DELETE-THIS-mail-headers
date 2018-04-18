use ::name::HeaderName;

//NOTE: this is a circular dependency between Header/HeaderMap
// but putting up e.g. a GenericHeaderMap trait/interface is
// not worth the work at all
use ::map::HeaderMap;

pub trait Header {

    /// true if the header can appear at most once
    const MAX_COUNT_EQ_1: bool;

    /// the component representing the header-field, e.g. `Unstructured` for `Subject`
    type Component;

    //FIXME[rust/const fn]: make this a associated constant
    fn name() -> HeaderName;

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
    const CONTEXTUAL_VALIDATOR: Option<
        fn(&HeaderMap)-> Result<(), ::error::HeaderValidationError>
    >;
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