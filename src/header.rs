use std::any::TypeId;
use std::fmt::{self, Debug};

use common::{
    error::EncodingError,
    encoder::{
        EncodableInHeader,
        EncodingWriter,
    }
};


use ::name::{HeaderName, HasHeaderName};
//NOTE: this is a circular dependency between Header/HeaderMap
// but putting up e.g. a GenericHeaderMap trait/interface is
// not worth the work at all
use ::map::HeaderMapValidator;

/// Trait representing a mail header.
///
/// **This is not meant to be implemented by hand.***
/// Use the `def_headers` macro instead.
///
pub trait Header: Clone + Default + 'static {

    /// the component representing the header-field, e.g. `Unstructured` for `Subject`
    type Component: EncodableInHeader + Clone;

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

impl<H> HasHeaderName for H
    where H: Header
{
    fn get_name(&self) -> HeaderName {
        H::name()
    }
}

#[derive(Clone)]
pub struct HeaderBody<H>
    where H: Header
{
    body: H::Component
}

impl<H> HeaderBody<H>
    where H: Header
{
    pub fn new(body: H::Component) -> HeaderBody<H> {
        HeaderBody { body }
    }
}

impl<H> Debug for HeaderBody<H>
    where H: Header
{
    fn fmt(&self, fter: &mut fmt::Formatter) -> fmt::Result {
        self.body.fmt(fter)
    }
}

/// Type alias for HeaderObjTrait's trait object.
pub type HeaderObj = dyn HeaderObjTrait;

pub trait HeaderObjTrait: Sync + Send + ::std::any::Any + Debug {
    fn name(&self) -> HeaderName;
    // fn is_max_one(&self) -> bool;
    fn validator(&self) -> Option<HeaderMapValidator>;
    fn encode(&self, encoder: &mut EncodingWriter) -> Result<(), EncodingError>;
    fn boxed_clone(&self) -> Box<HeaderObj>;

    #[doc(hidden)]
    fn type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }
}

impl<H> HeaderObjTrait for HeaderBody<H>
    where H: Header
{
    fn name(&self) -> HeaderName {
        H::name()
    }

    // fn is_max_one(&self) -> bool {
    //     H::MAX_ONE
    // }

    fn validator(&self) -> Option<HeaderMapValidator> {
        H::VALIDATOR
    }

    fn encode(&self, encoder: &mut EncodingWriter) -> Result<(), EncodingError> {
        self.body.encode(encoder)
    }

    fn boxed_clone(&self) -> Box<HeaderObj> {
        let cloned = self.clone();
        Box::new(cloned)
    }
}

impl<H> HasHeaderName for HeaderBody<H>
    where H: Header
{
    fn get_name(&self) -> HeaderName {
        H::name()
    }
}


impl HeaderObj {
    pub fn is<H>(&self) -> bool
        where H: Header
    {
        self.type_id() == TypeId::of::<HeaderBody<H>>()
    }

    pub fn downcast_ref<H>(&self) -> Option<&HeaderBody<H>>
        where H: Header
    {
        if self.is::<H>() {
            Some(unsafe { &*(self as *const _ as *const HeaderBody<H>) })
        } else {
            None
        }
    }

    pub fn downcast_mut<H>(&mut self) -> Option<&mut HeaderBody<H>>
        where H: Header
    {
        if self.is::<H>() {
            Some(unsafe { &mut *(self as *mut _ as *mut HeaderBody<H>) })
        } else {
            None
        }
    }
}

impl Clone for Box<HeaderObj> {
    fn clone(&self) -> Self {
        self.boxed_clone()
    }
}

impl HasHeaderName for HeaderObj {
    fn get_name(&self) -> HeaderName {
        self.name()
    }
}

pub trait HeaderObjTraitBoxExt: Sized {
    fn downcast<H>(self) -> Result<Box<HeaderBody<H>>, Self>
        where H: Header;
}

impl HeaderObjTraitBoxExt for Box<HeaderObjTrait> {

    fn downcast<H>(self) -> Result<Box<HeaderBody<H>>, Self>
        where H: Header
    {
        if HeaderObjTrait::is::<H>(&*self) {
            let ptr: *mut (HeaderObj) = Box::into_raw(self);
            Ok(unsafe { Box::from_raw(ptr as *mut HeaderBody<H>) })
        } else {
            Err(self)
        }
    }
}