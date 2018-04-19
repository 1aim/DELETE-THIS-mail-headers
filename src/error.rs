use std::fmt::{self, Display};

use failure::{Fail, Context, Error as FError, Backtrace};

use ::name::HeaderName;

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

#[derive(Debug, Fail)]
pub enum HeaderInsertionError {
    #[fail(display = "inserting header failed: {}", _0)]
    Type(HeaderTypeError),

    #[fail(display = "inserting header failed: {}", _0)]
    Component(ComponentCreationError),
}

impl From<HeaderTypeError> for HeaderInsertionError {
    fn from(inner: HeaderTypeError) -> Self {
        HeaderInsertionError::Type(inner)
    }
}

impl From<ComponentCreationError> for HeaderInsertionError {
    fn from(inner: ComponentCreationError) -> Self {
        HeaderInsertionError::Component(inner)
    }
}


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

#[derive(Copy, Clone, Debug, Fail, PartialEq, Eq, Hash)]
pub enum BuildInValidationError {
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

}

macro_rules! header_validation_bail {
    (kind: $($tt:tt)*) => ({
        let build_in = $crate::error::BuildInValidationError::$($tt)*;
        return Err(HeaderValidationError::BuildIn(::failure::Context::new(build_in)));
    });
}


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

    fn cause(&self) -> Option<&Fail> {
        match *self {
            ChainTail::Backtrace(_) => None,
            ChainTail::Error(ref error) => Some(error.cause())
        }
    }
}

#[derive(Debug)]
pub struct ComponentCreationError {
    component: &'static str,
    backtrace: ChainTail,
    str_context: Option<String>
}

impl ComponentCreationError {

    pub fn from_parent<P>(parent: P, component: &'static str) -> Self
        where P: Into<FError>
    {
        ComponentCreationError {
            component,
            backtrace: ChainTail::Error(parent.into()),
            str_context: None
        }
    }

    pub fn new(component: &'static str) -> Self {
        ComponentCreationError {
            component,
            backtrace: ChainTail::Backtrace(Backtrace::new()),
            str_context: None
        }
    }

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
        self.backtrace.cause()
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
