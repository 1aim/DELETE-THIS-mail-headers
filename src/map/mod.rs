//! Module containing the `HeaderMap`.
//!
//! It also contains some helper types like iterator types
//! for the HeaderMap etc.
use std::marker::PhantomData;
use std::iter::ExactSizeIterator;
use std::fmt::{self, Debug};
use std::collections::HashSet;
use std::cmp::PartialEq;
use std::hash::{Hash, Hasher};

use total_order_multi_map::{
    self,
    TotalOrderMultiMap,
    EntryValues
};

use common::encoder::EncodableInHeader;
use ::HeaderTryInto;
use ::error::{
    ComponentCreationError, HeaderTypeError,
    HeaderValidationError, BuildInValidationError
};

use super::{
    HeaderName,
    Header,
    HasHeaderName
};

mod into_iter;
pub use self::into_iter::*;

/// The type of an validator used to check more complex header contraints.
///
/// An example constraint would be if a `From` header field contains more than
/// one mailbox a `Sender` header field is required to be present.
pub type HeaderMapValidator = fn(&HeaderMap) -> Result<(), ::error::HeaderValidationError>;

//TODO extend example to use get,get_mut etc.
/// A header map is a collection representing a number
/// of mail headers in an specific order.
///
///
/// # Example
///
/// ```
/// # #[macro_use]
/// # extern crate mail_headers;
///
/// // just import all headers
/// use mail_headers::*;
/// use mail_headers::error::ComponentCreationError;
///
/// fn create_headers() -> Result<HeaderMap, ComponentCreationError> {
///     headers!{
///         // from and to can have multiple values
///         // until specialization is stable is array
///         // is necessary
///         _From: [("My Fancy Display Name", "theduck@example.com")],
///         _To: [ "unknown@example.com", ],
///         Subject: "Who are you?"
///     }
/// }
///
/// fn main() {
///     let headers = create_headers().unwrap();
///     assert_eq!(headers.len(), 3);
/// }
/// ```
///
/// # Note
///
/// A number of methods implemented on HeaderMap appear in two variations,
/// one which accepts a type hint (a normally zero sized struct implementing
/// Header) and on which just accepts the type and needs to be called with
/// the turbofish operator. The later one is prefixed by a `_` as the former
/// one is more nice to use, but in some situations, e.g. when wrapping
/// `HeaderMap` in custom code the only type accepting variations are more
/// useful.
///
/// ```rust,ignore
/// let _ = map.get(Subject);
/// //is equivalent to
/// let _ = map._get::<Subject>();
/// ```
///
#[derive(Clone)]
pub struct HeaderMap {
    validators: HashSet<ValidatorHashWrapper>,
    inner_map: TotalOrderMultiMap<HeaderName, Box<EncodableInHeader>>,
}

pub type Iter<'a> = total_order_multi_map::Iter<'a, HeaderName, Box<EncodableInHeader>>;
pub type IterMut<'a> = total_order_multi_map::IterMut<'a, HeaderName, Box<EncodableInHeader>>;

impl Debug for HeaderMap {
    fn fmt(&self, fter: &mut fmt::Formatter) -> fmt::Result {
        write!(fter, "HeaderMap {{ ")?;
        for (key, val_cont) in self.iter() {
            write!(fter, "{}: {:?},", key.as_str(), val_cont)?;
        }
        write!(fter, " }}")
    }
}

impl Default for HeaderMap {
    fn default() -> Self {
        HeaderMap {
            validators: Default::default(),
            inner_map: Default::default()
        }
    }
}

impl HeaderMap {

    /// create a new empty header map
    pub fn new() -> Self {
        Default::default()
    }

    /// returns the number of headers in this map
    pub fn len(&self) -> usize {
        self.inner_map.len()
    }

    /// clears the header map
    ///
    /// This removes all headers _and_ all validators
    pub fn clear(&mut self) {
        self.validators.clear();
        self.inner_map.clear();
    }

    /// call each unique contextual validator exactly once with this map as parameter
    ///
    /// If multiple Headers provide the same contextual validator (e.g. the resent headers)
    /// it's still only called once.
    pub fn use_contextual_validators(&self) -> Result<(), HeaderValidationError> {
        for validator in self.validators.iter() {
            (validator.as_func())(self)?;
        }
        Ok(())
    }

    /// returns true if the headermap contains a header with the same name
    pub fn contains<H: HasHeaderName>(&self, name: H ) -> bool {
        self.inner_map.contains_key(name.get_name())
    }

    /// returns the first header field ignoring any additional fields with the same name
    #[inline(always)]
    pub fn get_single<'a ,H>(&'a self, _type_hint: H)
        -> Option<Result<&'a H::Component, HeaderTypeError>>
        where H: Header,
              H::Component: EncodableInHeader
    {
        self._get_single::<H>()
    }

    /// returns the first header field ignoring any additional fields with the same name
    pub fn _get_single<'a ,H>(&'a self)
        -> Option<Result<&'a H::Component, HeaderTypeError>>
        where H: Header,
              H::Component: EncodableInHeader
    {
        self.get_untyped(H::name())
            .map( |mut bodies| {
                //UNWRAP_SAFE: we have at last one element
                let untyped = bodies.next().unwrap();
                untyped.downcast_ref::<H::Component>()
                    .ok_or_else(|| HeaderTypeError::new(H::name()))
            } )
    }

    /// Returns all header bodies for a given header name, without trying to cast them to a concrete type
    ///
    /// Accepts both `HeaderName` or a type implementing `Header`.
    ///
    #[inline]
    pub fn get_untyped<H: HasHeaderName>( &self, name: H ) -> Option<UntypedBodies> {
        self.inner_map.get( name.get_name() )
    }

    /// Returns all header bodies for a given header
    #[inline(always)]
    pub fn get<H>( &self, _type_hint: H) -> Option<TypedBodies<H>>
        where H: Header, H::Component: EncodableInHeader
    {
        self._get::<H>()
    }

    /// Returns all header bodies for a given header
    pub fn _get<H>( &self ) -> Option<TypedBodies<H>>
        where H: Header, H::Component: EncodableInHeader
    {
        self.get_untyped( H::name() )
            .map( |untyped| untyped.into() )
    }

    /// Inserts given header into the header map.
    ///
    /// Returns the count of headers with the given name after inserting
    /// this header
    ///
    /// # Error
    ///
    /// returns a error if `body` can not be converted into the
    /// right component type (which is specified through the headers
    /// associated `Component` type).
    ///
    /// # Note (_insert)
    ///
    /// This method does two thinks for better usability/ergonomic:
    ///
    /// 1. internally convert body to H::Component (which can fail)
    /// 2. accept a (normally zero-sized) type hint as first parameter
    ///
    /// This allows writing e.g. `.insert(Sender, address)`
    /// instead of `._insert::<Sender>(<Sender as Header>::Component::try_from(address))`.
    /// But for some use cases this is suboptimal, e.g. if you already have the right
    /// type and therefore insertion should not be able to fail or if you have `H` as
    /// type parameter but not as type hint. For this cases `_insert` exist, which doesn't
    /// do any conversion and accepts `H` only as generic type parameter.
    ///
    pub fn insert<H, C>( &mut self, _htype_hint: H, body: C ) -> Result<usize, ComponentCreationError>
        where H: Header,
              H::Component: EncodableInHeader,
              C: HeaderTryInto<H::Component>
    {
        Ok(self._insert::<H>(body.try_into()?))
    }

    /// Inserts given header into the header map.
    ///
    /// Returns the count of headers with the given name after inserting
    /// this header.
    pub fn _insert<H>(&mut self, component: H::Component) -> usize
        where H: Header,
              H::Component: EncodableInHeader
    {
        let obj: Box<EncodableInHeader> = Box::new(component);
        let name = H::name();
        let count = self.inner_map.insert(name, obj);
        if let Some(validator) = H::VALIDATOR {
            self.validators.insert(ValidatorHashWrapper(validator));
        }
        count
    }

    /// combines this header map with another header map
    ///
    /// All headers in other get inserted into this map
    /// in the order they where inserted into other.
    /// Additionally all validators in other get inserted
    /// into this map.
    pub fn combine(&mut self, other: HeaderMap )  -> &mut Self {
        self.validators.extend(other.validators);
        self.inner_map.extend(other.inner_map);
        self
    }

    /// remove all headers with the given header name
    ///
    /// returns true, if at last one header was removed
    pub fn remove_by_name<H: HasHeaderName>(&mut self, name: H ) -> bool {
        self.inner_map.remove_all(name.get_name())
    }

    /// iterate over all (header name, boxed body) pairs in this map
    pub fn iter(&self) -> Iter {
        self.inner_map.iter()
    }

}

/// Iterator over all boxed bodies for a given header name
pub type UntypedBodies<'a> = EntryValues<'a, EncodableInHeader>;


/// Iterator over all boxed bodies for a given header name with knows which type they should have
///
/// This iterator will automatically try to cast each header body of this
/// header to `H::Component`, i.e. the type this body _should_ have.
pub struct TypedBodies<'a, H>
    where H: Header,
          H::Component: EncodableInHeader
{
    inner: UntypedBodies<'a>,
    _marker: PhantomData<H>
}

impl<'a, H> From<UntypedBodies<'a>> for TypedBodies<'a, H>
    where H: Header,
          H::Component: EncodableInHeader
{
    fn from(untyped: UntypedBodies<'a>) -> Self {
        TypedBodies { inner: untyped, _marker: PhantomData }
    }
}

impl<'a, H> TypedBodies<'a, H>
    where H: Header,
          H::Component: EncodableInHeader
{
    fn new(inner: UntypedBodies<'a>) -> Self {
        TypedBodies {
            inner,
            _marker: PhantomData
        }
    }
}

impl<'a, H> Iterator for TypedBodies<'a, H>
    where H: Header,
          H::Component: EncodableInHeader
{
    type Item = Result<&'a H::Component, HeaderTypeError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
            .map( |tobj| {
                tobj.downcast_ref::<H::Component>()
                    .ok_or_else(|| HeaderTypeError::new(H::name()))
            } )
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'a, H> ExactSizeIterator for TypedBodies<'a, H>
    where H: Header,
          H::Component: EncodableInHeader
{
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<'a, H> Clone for TypedBodies<'a, H>
    where H: Header,
          H::Component: EncodableInHeader
{
    fn clone(&self) -> Self {
        TypedBodies::new(self.inner.clone())
    }
}

impl<'a, H> Debug for TypedBodies<'a, H>
    where H: Header,
          H::Component: EncodableInHeader
{
    fn fmt(&self, fter: &mut fmt::Formatter) -> fmt::Result {
        fter.debug_struct("TypedBodies")
            .field("inner", &self.inner)
            .finish()
    }
}

/// Create a header map from a list of header's with ther fields
///
/// # Example
///
/// ```
/// # #[macro_use]
/// # extern crate mail_headers;
/// # use mail_headers::*;
/// # use mail_headers::error::ComponentCreationError;
/// # fn main() { (|| -> Result<(), ComponentCreationError> {
/// let map = headers! {
///     _From: ["bobo@nana.test"],
///     Subject: "hy there"
/// }?;
/// # Ok(()) })(); }
/// ```
#[macro_export]
macro_rules! headers {
    ($($header:ty : $val:expr),*) => ({
        //FIXME[rust/catch block] use catch block once available
        (|| -> Result<$crate::HeaderMap, $crate::error::ComponentCreationError>
        {
            let mut map = $crate::HeaderMap::new();
            $(
                let component: <$header as $crate::Header>::Component = $crate::HeaderTryFrom::try_from($val)?;
                map._insert::<$header>(component);
            )*
            Ok(map)
        })()
    });
}

/// HeaderMapValidator is just a function pointer,
/// but it does not implement Hash so we wrap it
/// and implement Hash on it. Note that some function
/// pointers implement Hash/Eq and other doesn't,
/// which is caused by some limitations with wildcard
/// implementations
#[derive(Copy, Clone)]
struct ValidatorHashWrapper(HeaderMapValidator);

impl ValidatorHashWrapper {

    fn as_func(&self) -> HeaderMapValidator {
        self.0
    }

    fn identity_repr(&self) -> usize {
        self.0 as usize
    }
}

impl PartialEq<Self> for ValidatorHashWrapper {
    fn eq(&self, other: &Self) -> bool {
        self.identity_repr() == other.identity_repr()
    }
}

impl Eq for ValidatorHashWrapper {}

impl Debug for ValidatorHashWrapper {
    fn fmt(&self, fter: &mut fmt::Formatter) -> fmt::Result {
        write!(fter, "ValidatorHashWrapper(0x{:x})", self.identity_repr())
    }
}

impl Hash for ValidatorHashWrapper {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_usize(self.identity_repr())
    }
}


pub fn check_header_count_max_one(name: HeaderName, map: &HeaderMap)
    -> Result<(), HeaderValidationError>
{
    let valid = map
        .get_untyped(name)
        .map(|bodies| bodies.len() <= 1)
        .unwrap_or(true);

    if valid {
        Ok(())
    } else {
        Err(HeaderValidationError::from(
            BuildInValidationError::MoreThenOne {
                header_name: name.as_str()
            }
        ))
    }
}

#[cfg(test)]
mod test {
    use failure::Context;
    use soft_ascii_string::SoftAsciiStr;

    use common::error::{EncodingError, EncodingErrorKind};
    use common::encoder::{EncodableInHeader, EncodingWriter};

    use ::HeaderTryFrom;
    use ::error::{ComponentCreationError, HeaderValidationError};
    use ::components::RawUnstructured;

    use super::*;

    use self::good_headers::*;
    use self::bad_headers::{
        Subject as BadSubject,
        Comments as BadComments
    };

    #[derive(Debug, Clone, Eq, PartialEq, Hash)]
    pub struct OtherComponent;

    impl HeaderTryFrom<()> for OtherComponent {
        fn try_from(_: ()) -> Result<OtherComponent, ComponentCreationError> {
            Ok(OtherComponent)
        }
    }
    impl EncodableInHeader for OtherComponent {
        fn encode(&self, _encoder:  &mut EncodingWriter) -> Result<(), EncodingError> {
            Err(EncodingError::from(
                    EncodingErrorKind::Other { kind: "encoding is not implemented" }))
        }

        fn boxed_clone(&self) -> Box<EncodableInHeader> {
            Box::new(self.clone())
        }
    }


    mod good_headers {
        use components;
        def_headers! {
            test_name: validate_header_names,
            scope: components,
            Subject, unchecked { "Subject" }, RawUnstructured, maxOne,
            Comments, unchecked { "Comments" }, RawUnstructured, None
        }
    }

    mod bad_headers {
        def_headers! {
            test_name: validate_header_names,
            scope: super,
            Subject, unchecked { "Subject" },  OtherComponent, maxOne,
            Comments, unchecked { "Comments" }, OtherComponent, None
        }
    }

    const TEXT_1: &str = "Random stuff XD";
    const TEXT_2: &str = "Having a log of fun, yes a log!";

    #[test]
    fn headers_macro() {
        let headers = headers! {
            Comments: TEXT_1,
            Subject: TEXT_2
        }.unwrap();


        let count = headers
            // all headers _could_ have multiple values, through neither
            // ContentType nor Subject do have multiple value
            .get(Comments)
            .expect("where did the header go?")
            .map( |h: Result<&RawUnstructured, HeaderTypeError>| {
                let v = h.expect( "the trait object to be downcastable to RawUnstructured" );
                assert_eq!(v.as_str(), TEXT_1);
            })
            .count();
        assert_eq!( 1, count );

        let count = headers
            .get(Subject)
            .expect( "content type header must be present" )
            .map( |h: Result<&RawUnstructured, HeaderTypeError>| {
                let val = h.expect( "the trait object to be downcastable to H::Component" );
                assert_eq!(val.as_str(), TEXT_2);
            })
            .count();
        assert_eq!( 1, count );
    }

    #[test]
    fn get_single() {
        let headers = headers! {
            Subject: "abc"
        }.unwrap();

        assert_eq!(
            "abc",
            headers.get_single(Subject)
                .unwrap()//Some
                .unwrap()//Result
                .as_str()
        );
    }

    #[test]
    fn get_single_cast_error() {
        let headers = headers! {
            Subject: "abc"
        }.unwrap();

        let res = headers.get_single(BadSubject);
        assert_err!( res.expect("where did the header go?") );
    }

    #[test]
    fn get() {
        let headers = headers! {
            Subject: "abc",
            Comments: "1st",
            BadComments: ()
        }.unwrap();


        let mut res = headers.get(Comments)
            .unwrap();

        assert_eq!(res.size_hint(), (2, Some(2)));

        assert_eq!(
            "1st",
            assert_ok!(res.next().unwrap()).as_str()
        );

        assert_err!(res.next().unwrap());

        assert!( res.next().is_none() )

    }

    #[test]
    fn get_untyped() {
        let headers = headers! {
            Subject: "abc",
            Comments: "1st",
            BadComments: ()
        }.unwrap();


        let res = headers.get_untyped(Subject::name())
            .unwrap()
            .map(|entry| entry.downcast_ref::<RawUnstructured>().unwrap().as_str() )
            .collect::<Vec<_>>();

        assert_eq!(
            res.as_slice(),
            &[ "abc" ]
        );

        let mut res = headers.get_untyped(Comments::name()).unwrap();

        assert_eq!((2, Some(2)), res.size_hint());

        assert_eq!(
            res.next().unwrap().downcast_ref::<RawUnstructured>().unwrap().as_str(),
            "1st"
        );

        assert_eq!((1, Some(1)), res.size_hint());

        assert_eq!(
            res.next().unwrap().downcast_ref::<OtherComponent>().unwrap(),
            &OtherComponent
        );

        assert!(res.next().is_none());
    }

    #[test]
    fn fmt_debug() {
        let headers = headers! {
            Subject: "hy there"
        }.unwrap();

        let res = format!("{:?}", headers);
        assert_eq!(
            "HeaderMap { Subject: RawUnstructured { text: Input(Owned(\"hy there\")) }, }",
            res.as_str()
        );
    }

    #[test]
    fn combine_keeps_order() {
        let mut headers = headers! {
            XComment: "ab@c"
        }.unwrap();

        headers.combine( headers! {
            Subject: "hy there",
            Comments: "magic+spell"
        }.unwrap());

        assert_eq!(
            &[
                "X-Comment",
                "Subject",
                "Comments"
            ],
            headers.into_iter()
                .map(|(name, _val)| name.as_str())
                .collect::<Vec<_>>()
                .as_slice()
        );
    }


    #[test]
    fn remove_1() {
        let mut headers = headers!{
            Comments: "a",
            Subject: "b",
            Comments: "c",
            Comments: "d"
        }.unwrap();

        assert_eq!( false, headers.remove_by_name( XComment::name() ) );
        assert_eq!( true, headers.remove_by_name( Subject::name() ) );

        assert_eq!( 3, headers.iter().count() );

        let values = headers.get(Comments)
            .unwrap()
            .map(|comp| comp.unwrap().as_str() )
            .collect::<Vec<_>>();

        assert_eq!(
            &[ "a", "c", "d" ],
            values.as_slice()
        )
    }

    #[test]
    fn remove_2() {
        let mut headers = headers!{
            Comments: "a",
            Subject: "b",
            Comments: "c",
            Comments: "d"
        }.unwrap();

        assert_eq!( true, headers.remove_by_name( Comments::name() ) );
        assert_eq!( false, headers.remove_by_name( Comments::name() ) );

        assert_eq!( 1, headers.iter().count() );

        let values = headers.get(Subject)
            .unwrap()
            .map(|comp| comp.unwrap().as_str() )
            .collect::<Vec<_>>();

        assert_eq!(
            &[ "b" ],
            values.as_slice()
        );
    }

    struct XComment;
    impl Header for XComment {
        type Component = RawUnstructured;

        fn name() -> HeaderName {
            HeaderName::new(SoftAsciiStr::from_unchecked("X-Comment")).unwrap()
        }

        const VALIDATOR: Option<
            fn(&HeaderMap)-> Result<(), HeaderValidationError>
        > = Some(__validator);
    }

    //some stupid but simple validator
    fn __validator(map: &HeaderMap) -> Result<(), HeaderValidationError> {
        if map.get_untyped(Comments::name()).is_some() {
            return Err(HeaderValidationError::Custom(
                Context::new("can't have X-Comment and Comments in same mail")
                .into()
            ));
        }
        Ok(())
    }

    #[test]
    fn contains_works() {
        let map = headers! {
            Subject: "soso"
        }.unwrap();

        assert_eq!( true, map.contains(Subject::name()) );
        assert_eq!( true, map.contains(Subject) );
        assert_eq!( false, map.contains(Comments::name()) );
        assert_eq!( false, map.contains(Comments) );
    }

    #[test]
    fn use_validator_ok() {
        let map = headers! {
            XComment: "yay",
            Subject: "soso"
        }.unwrap();

        assert_ok!(map.use_contextual_validators());
    }

    #[test]
    fn use_validator_err() {
        let map = headers! {
            XComment: "yay",
            Comments: "oh no",
            Subject: "soso"
        }.unwrap();

        assert_err!(map.use_contextual_validators());
    }

    #[test]
    fn has_len() {
        let map = headers! {
            XComment: "yay",
            Comments: "oh no",
            Subject: "soso"
        }.unwrap();

        assert_eq!(3, map.len());
    }
}