use ::name::HeaderName;

//NOTE: this is a circular dependency between Header/HeaderMap
// but putting up e.g. a GenericHeaderMap trait/interface is
// not worth the work at all
use ::map::HeaderMapValidator;

/// Trait representing a mail header.
///
/// **This is not meant to be implemented by hand.***
/// Use the `def_headers` macro instead.
///
pub trait Header {

    /// the component representing the header-field, e.g. `Unstructured` for `Subject`
    type Component;

    //FIXME[rust/const fn]: make this a associated constant
    /// a method returning the header name
    ///
    /// # Note:
    /// Once `const fn` is stable this will be changed to
    /// a associated constant.
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
    const VALIDATOR: Option<HeaderMapValidator>;
}


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