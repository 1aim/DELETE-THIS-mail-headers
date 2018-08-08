//! module contains the (new) errors emitted by this crate
use std::fmt::{self, Display};

use failure::{Fail, Context, Error as FError, Backtrace};

use ::name::HeaderName;

/// This error can occur if different implementations for the
/// same header (e.g. `Subject`) where used in the same `HeaderMap`.
#[derive(Debug, Fail)]
#[fail(display = "cast error caused by mixing different header implementations for {}", header_name)]
pub struct HeaderTypeError {
    header_name: HeaderName,
    backtrace: Backtrace
}

impl HeaderTypeError {
    pub fn new(name: HeaderName) -> Self {
        HeaderTypeError {
            header_name: name,
            backtrace: Backtrace::new()
        }
    }

    pub fn new_with_backtrace(name: HeaderName, backtrace: Backtrace) -> Self {
        HeaderTypeError {
            header_name: name,
            backtrace
        }
    }
}

/// A validator specified in a header definition failed.
///
/// Common validators are e.g. to make sure that if a
/// From header has multiple mailboxes that there is
/// a Sender header etc.
#[derive(Debug, Fail)]
pub enum HeaderValidationError {
    #[fail(display = "{}", _0)]
    BuildIn(Context<BuildInValidationError>),
    #[fail(display = "{}", _0)]
    Custom(FError)
}

impl From<BuildInValidationError> for HeaderValidationError {
    fn from(err: BuildInValidationError) -> Self {
        HeaderValidationError::BuildIn(Context::new(err))
    }
}

impl From<Context<BuildInValidationError>> for HeaderValidationError {
    fn from(err: Context<BuildInValidationError>) -> Self {
        HeaderValidationError::BuildIn(err)
    }
}

/// The build-in error variants (error kinds) which can be returned
/// when running a header map validator.
#[derive(Copy, Clone, Debug, Fail, PartialEq, Eq, Hash)]
pub enum BuildInValidationError {

    #[fail(display = "{} header field can appear at most one time in a header map", header_name)]
    MoreThenOne{ header_name: &'static str },

    #[fail(display = "From field contained multiple addresses but no Sender field was set")]
    MultiMailboxFromWithoutSender,

    #[fail(display = "each resent block must have a resent-date field")]
    ResentDateFieldMissing,

    #[fail(display = "Resent-From field in resent block without a Resent-Sender field")]
    MultiMailboxResentFromWithoutResentSender,

    #[fail(display = "From field missing")]
    NoFrom,

    // theoretically content type is optional practically it's recommended even
    // for plain text mails, to indicate that they are indeed plain text mails
    #[fail(display = "Content-Type field missing")]
    NoContentType,

    #[fail(display = "Content-Type header misses boundary parameter in multipart body")]
    NoMultipartBoundary,

    #[fail(display = "multipart bodies need to contain at last one part (/sub-body)")]
    EmptyMultipartBody,

    /// Indicates the `To` header is missing
    ///
    /// While rfc5322 does not require a `To` header
    /// field, it's a sane choice to reject mails without
    /// it.
    ///
    /// This error is _not_ used by the general validation,
    /// provided with this crate, but can be used e.g. by
    /// external libraries which do generate mails.
    #[fail(display = "missing To header field")]
    NoTo,

}

macro_rules! header_validation_bail {
    (kind: $($tt:tt)*) => ({
        let build_in = $crate::error::BuildInValidationError::$($tt)*;
        return Err(HeaderValidationError::BuildIn(::failure::Context::new(build_in)));
    });
}


/// Helper type which is either a `Backtrace` or an full `failure::Error`.
///
/// This can be used to either just contain a backtrace into an custom
/// error or to chain it in front of another error without adding another
/// backtrace, depending on the creating context.
#[derive(Debug)]
pub enum ChainTail {
    Backtrace(Backtrace),
    Error(FError)
}

impl ChainTail {

    fn backtrace(&self) -> &Backtrace {
        match *self {
            ChainTail::Backtrace(ref trace) => trace,
            ChainTail::Error(ref error) => error.backtrace()
        }
    }

    fn as_fail(&self) -> Option<&Fail> {
        match *self {
            ChainTail::Backtrace(_) => None,
            ChainTail::Error(ref error) => Some(error.as_fail())
        }
    }
}

/// Creating a (header field) component from the given data failed
///
/// A good example converting a string to a mailbox by parsing it,
/// or more concretely failing to do so because it's not a valid
/// mail address.
#[derive(Debug)]
pub struct ComponentCreationError {
    component: &'static str,
    backtrace: ChainTail,
    str_context: Option<String>
}

impl ComponentCreationError {

    /// create a new `ComponentCreationError` based on a different error and the name of the component
    ///
    /// The name is normally the type name, for example `Email`, `Mailbox` etc.
    pub fn from_parent<P>(parent: P, component: &'static str) -> Self
        where P: Into<FError>
    {
        ComponentCreationError {
            component,
            backtrace: ChainTail::Error(parent.into()),
            str_context: None
        }
    }

    /// creates a new `ComponentCreationError` based on the components name
    ///
    /// The name is normally the type name, for example `Email`, `Mailbox` etc.
    pub fn new(component: &'static str) -> Self {
        ComponentCreationError {
            component,
            backtrace: ChainTail::Backtrace(Backtrace::new()),
            str_context: None
        }
    }

    /// creates a new `ComponentCreationError` based on the components name with a str_context
    ///
    /// The name is normally the type name, for example `Email`, `Mailbox` etc.
    ///
    /// The `str_context` is a snipped of text which can help a human to identify the
    /// invalid parts, e.g. for parsing a email it could be the invalid email address.
    pub fn new_with_str<I>(component: &'static str, str_context: I) -> Self
        where I: Into<String>
    {
        ComponentCreationError {
            component,
            backtrace: ChainTail::Backtrace(Backtrace::new()),
            str_context: Some(str_context.into())
        }
    }

    pub fn str_context(&self) -> Option<&str> {
        self.str_context.as_ref().map(|s|&**s)
    }

    pub fn set_str_context<I>(&mut self, ctx: I)
        where I: Into<String>
    {
        self.str_context = Some(ctx.into());
    }

    pub fn with_str_context<I>(mut self, ctx: I) -> Self
        where I: Into<String>
    {
        self.set_str_context(ctx);
        self
    }
}

impl Fail for ComponentCreationError {
    fn cause(&self) -> Option<&Fail> {
        self.backtrace.as_fail()
    }
    fn backtrace(&self) -> Option<&Backtrace> {
        Some(self.backtrace.backtrace())
    }
}

impl Display for ComponentCreationError {
    fn fmt(&self, fter: &mut fmt::Formatter) -> fmt::Result {
        write!(fter, "creating component {} failed", self.component)
    }
}
