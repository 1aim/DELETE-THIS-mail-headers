use mime::error::{Error as ParserError};

quick_error! {
    #[derive(Debug)]
    pub enum HeaderValidationError {
        MultiMailboxFromWithoutSender {
            description("From field contained multiple addresses but no Sender field was set")
        }
        ResentDateFieldMissing {
            description("each resent block must have a resent-date field")
        }
        MultiMailboxResentFromWithoutResentSender {
            description("Resent-From field in resent block without a Resent-Sender field")
        }
    }
}


macro_rules! bail_header {
    ($ce:expr) => ({
        use $crate::error::HeaderValidationError;
        use $crate::core::error::{ErrorKind, ResultExt};
        let err: HeaderValidationError = $ce;
        return Err(err).chain_err(||ErrorKind::HeaderValidationFailure)
    });
}


quick_error! {
    #[derive(Debug)]
    pub enum ComponentError {


//        InvalidToken(got: String) {
//            description("given input was not a valid token (syntax)")
//            display("expected valid token (syntax) got: {:?}", got)
//        }

        InvalidContentDisposition(got: String) {
            description(
                "Content-Disposition can either be \"inline\" or \"attachment\""
            )
            display("expected \"inline\" or \"attachment\" got {:?}", got)
        }

        InvalidWord(got: String) {
            description("the given input word can not be encoded in given context")
            display("expected word valid in context, got {:?}", got)
        }

        InvalidDomainName(got: String) {
            description("given input is not a valid domain name")
            display("expected a valid domain name, got: {:?}", got)
        }

        InvalidLocalPart(got: String) {
            description("the local part (likely of an email) is invalid")
            display("expected a valid local part, got: {:?}", got)
        }

        InvalidEmail(got: String) {
            description("given input is not a valid Email")
            display("expected a valid Email, got: {:?}", got)
        }

        InvalidMessageId(got: String) {
            description("given input is not a valid MessageId")
            display("expected a valid MessageId, got: {:?}", got)
        }

        InvalidMediaTypeParts(error: ParserError) {
            description("constructing media type failed")
            display("constructing media type failed: {:?}", error)
        }

//        InvalidMimeRq(got: String) {
//            description(concat!(
//                "invalid mime, through could be valid with ",
//                "requoting/encoding parameter sections which is not supported"))
//            display(concat!(
//                "invalid mime, through could be valid with ",
//                "requoting/encoding parameter sections which is not supported",
//                ": {:?}"), got)
//        }

        MailboxListSize0 {
            description("a mailbox list consist of at last one phrase, not 0")
        }

        PhraseListSize0 {
            description("a phrase list consist of at last one phrase, not 0")
        }

        EmptyPhrase {
            description("a phrase consist of at last one word, neither empty nor only wsp are allowed")
        }

        NeedAtLastOneVCHAR(got: String) {
            description("given input did contain 0 VCHAR's but at last 1 was required")
            display("need at last one VCHAR in input got: {:?}", got)
        }

        ParsingMediaTypeFailed(err: ParserError) {
            description("parsing media type failed")
            display("parsing media type failed: {}", err)
        }

//        MimeSectionOverflow {
//            description("can not process a mime parameter split into more than 256 sections")
//        }

    }
}


macro_rules! bail {
    ($ce:expr) => ({
        use $crate::error::ComponentError;
        use $crate::core::error::{ErrorKind, ResultExt};
        let err: ComponentError = $ce;
        return Err(err).chain_err(||ErrorKind::HeaderComponentEncodingFailure)
    });
}


macro_rules! error {
    ($ce:expr) => ({
        use $crate::error::ComponentError;
        use $crate::core::error::{Error, ErrorKind};
        let err: ComponentError = $ce;
        Error::with_chain(err, ErrorKind::HeaderComponentEncodingFailure)
    });
}