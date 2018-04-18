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
    #[fail(display = "inserting header failed: {}", inner)]
    Type { inner: HeaderTypeError },

    #[fail(display = "inserting header failed: {}", inner)]
    Component { inner: ComponentCreationError },
}

impl From<HeaderTypeError> for HeaderInsertionError {
    fn from(inner: HeaderTypeError) -> Self {
        HeaderInsertionError::Type { inner }
    }
}

impl From<ComponentCreationError> for HeaderInsertionError {
    fn from(inner: ComponentCreationError) -> Self {
        HeaderInsertionError::Component { inner }
    }
}


#[derive(Debug, Fail)]
pub enum HeaderValidationError {
    #[fail(display = "{}", _0)]
    BuildIn(Context<BuildInValidationError>),
    #[fail(display = "{}", _0)]
    Custom(FError)
}

#[derive(Copy, Clone, Debug, Fail, PartialEq, Eq, Hash)]
pub enum BuildInValidationError {
    #[fail(display="From field contained multiple addresses but no Sender field was set")]
    MultiMailboxFromWithoutSender,

    #[fail(display="each resent block must have a resent-date field")]
    ResentDateFieldMissing,

    #[fail(display="Resent-From field in resent block without a Resent-Sender field")]
    MultiMailboxResentFromWithoutResentSender,
}

macro_rules! header_validation_bail {
    (kind: $($tt:tt)*) => ({
        let build_in = $crate::error::BuildInValidationError::$($tt)*;
        return Err(HeaderValidationError::BuildIn(::failure::Context::new(build_in)));
    });
}


#[derive(Debug)]
enum Chain {
    Backtrace(Backtrace),
    Error(FError)
}

impl Chain {

    fn backtrace(&self) -> &Backtrace {
        match *self {
            Chain::Backtrace(ref trace) => trace,
            Chain::Error(ref error) => error.backtrace()
        }
    }

    fn cause(&self) -> Option<&Fail> {
        match *self {
            Chain::Backtrace(_) => None,
            Chain::Error(ref error) => Some(error.cause())
        }
    }
}

#[derive(Debug)]
pub struct ComponentCreationError {
    component: &'static str,
    backtrace: Chain,
    str_context: Option<String>
}

impl ComponentCreationError {

    pub fn from_parent<P>(parent: P, component: &'static str) -> Self
        where P: Into<FError>
    {
        ComponentCreationError {
            component,
            backtrace: Chain::Error(parent.into()),
            str_context: None
        }
    }

    pub fn new(component: &'static str) -> Self {
        ComponentCreationError {
            component,
            backtrace: Chain::Backtrace(Backtrace::new()),
            str_context: None
        }
    }

    pub fn new_with_str<I>(component: &'static str, str_context: I) -> Self
        where I: Into<String>
    {
        ComponentCreationError {
            component,
            backtrace: Chain::Backtrace(Backtrace::new()),
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
// #[derive(Clone, Debug, Fail, PartialEq, Eq, Hash)]
// pub enum ComponentError {
//     #[fail(display = "expected raw unstructured string got: {:?}", got)]
//     InvalidRawUnstructured { got: String },

//     #[fail(display = "expected \"inline\" or \"attachment\" got {:?}", got)]
//     InvalidContentDisposition { got: String },

//     #[fail(display = "expected word valid in context, got {:?}", got)]
//     InvalidWord { got: String },

//     #[fail(display = "expected a valid domain name, got: {:?}", got )]
//     InvalidDomainName { got: String },

//     #[fail(display = "expected a valid local part, got: {:?}", got)]
//     InvalidLocalPart { got: String },

//     #[fail(display = "expected a valid Email, got: {:?}", got)]
//     InvalidEmail { got: String },

//     #[fail(display = "expected a valid MessageId, got: {:?}", got)]
//     InvalidMessageId { got: String },

//     #[fail(display = "constructing media type failed: {:?}", error)]
//     InvalidMediaTypeParts { error: ParserError },

//     #[fail(display = "a mailbox list consist of at last one phrase, not 0")]
//     MailboxListSize0,

//     #[fail(display = "a phrase list consist of at last one phrase, not 0")]
//     PhraseListSize0,

//     #[fail(display = "a phrase consist of at last one word, neither empty nor only wsp are allowed")]
//     EmptyPhrase,

//     #[fail(display = "need at last one VCHAR in input got: {:?}", got)]
//     NeedAtLastOneVCHAR { got: String },

//     #[fail(display = "parsing media type failed: {}", error)]
//     ParsingMediaTypeFailed { error: ParserError }
// }

// impl ComponentError {

//     fn encoding_error_kind(&self) -> EncodingErrorKind {
//         use self::ComponentError::*;
//         match *self {
//             // all of this could be replaced with Malformed { place: "xxxx" }
//             InvalidRawUnstructured { .. } => T,
//             InvalidContentDisposition { .. } => T,
//             InvalidWord { .. } => T,
//             InvalidDomainName { .. } => T,
//             InvalidLocalPart { .. } => T,
//             InvalidLocalPart { .. } => T,
//             InvalidEmail { .. } => T,
//             InvalidMessageId { .. } => T,
//             InvalidMediaTypeParts { .. } => T,

//             // all of this are Size0 Errors in the error chain
//             MailboxListSize0 => T,
//             PhraseListSize0 => T,
//             EmptyPhrase => T,

//             // this is a PartitionError in the chain
//             NeedAtLastOneVCHAR { .. } => T,
//             // this is a  mime::ParsingError in the chain
//             ParsingMediaTypeFailed { .. } => T,
//         }
//     }
// }

// impl Into<EncodingError> for ComponentError {
//     fn into(self) -> EncodingError {

//     }
// }

// // macro_rules! bail {
// //     ($ce:expr) => ({
// //         use $crate::error::ComponentError;
// //         use $crate::__common::error::{ErrorKind, ResultExt};
// //         let err: ComponentError = $ce;
// //         return Err(err).chain_err(||ErrorKind::HeaderComponentEncodingFailure)
// //     });
// // }


// // macro_rules! error {
// //     ($ce:expr) => ({
// //         use $crate::error::ComponentError;
// //         use $crate::__common::error::{Error, ErrorKind};
// //         let err: ComponentError = $ce;
// //         Error::with_chain(err, ErrorKind::HeaderComponentEncodingFailure)
// //     });
// // }