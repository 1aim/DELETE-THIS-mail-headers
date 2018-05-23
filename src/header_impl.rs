

use components;
use self::validators::{
    from as validator_from,
    resent_any as validator_resent_any
};

def_headers! {
    test_name: validate_header_names,
    scope: components,
    /// (rfc5322)
    Date,         unchecked { "Date"          },  DateTime,       maxOne,
    /// (rfc5322)
    _From,        unchecked { "From"          },  MailboxList,    validator_from,
    /// (rfc5322)
    Sender,       unchecked { "Sender"        },  Mailbox,        maxOne,
    /// (rfc5322)
    ReplyTo,      unchecked { "Reply-To"      },  MailboxList,    maxOne,
    /// (rfc5322)
    _To,          unchecked { "To"            },  MailboxList,    maxOne,
    /// (rfc5322)
    Cc,           unchecked { "Cc"            },  MailboxList,    maxOne,
    /// (rfc5322)
    Bcc,          unchecked { "Bcc"           },  MailboxList,    maxOne,
    /// (rfc5322)
    MessageId,    unchecked { "Message-Id"    },  MessageId,      maxOne,
    /// (rfc5322)
    InReplyTo,    unchecked { "In-Reply-To"   },  MessageIdList,  maxOne,
    /// (rfc5322)
    References,   unchecked { "References"    },  MessageIdList,  maxOne,
    /// (rfc5322)
    Subject,      unchecked { "Subject"       },  Unstructured,   maxOne,
    /// (rfc5322)
    Comments,     unchecked { "Comments"      },  Unstructured,   None,
    /// (rfc5322)
    Keywords,     unchecked { "Keywords"      },  PhraseList,     None,
    /// (rfc5322)
    ResentDate,   unchecked { "Resent-Date"   },  DateTime,       validator_resent_any,
    /// (rfc5322)
    ResentFrom,   unchecked { "Resent-From"   },  MailboxList,    validator_resent_any,
    /// (rfc5322)
    ResentSender, unchecked { "Resent-Sender" },  Mailbox,        validator_resent_any,
    /// (rfc5322)
    ResentTo,     unchecked { "Resent-To"     },  MailboxList,    validator_resent_any,
    /// (rfc5322)
    ResentCc,     unchecked { "Resent-Cc"     },  MailboxList,    validator_resent_any,
    /// (rfc5322)
    ResentBcc,    unchecked { "Resent-Bcc"    },  OptMailboxList, validator_resent_any,
    /// (rfc5322)
    ResentMsgId,  unchecked { "Resent-Msg-Id" },  MessageId,      validator_resent_any,
    /// (rfc5322)
    ReturnPath,   unchecked { "Return-Path"   },  Path,           None,
    /// (rfc5322)
    Received,     unchecked { "Received"      },  ReceivedToken,  None,

    /// (rfc2045)
    ContentType,  unchecked { "Content-Type"  }, MediaType,        maxOne,

    /// (rfc2045)
    ContentId,    unchecked { "Content-Id"    }, ContentId,        maxOne,

    /// The transfer encoding used to (transfer) encode the body (rfc2045)
    ///
    /// This should either be:
    ///
    /// - `7bit`: Us-ascii only text, default value if header filed is not present
    /// - `quoted-printable`: Data encoded with quoted-printable encoding).
    /// - `base64`: Data encoded with base64 encoding.
    ///
    /// Through other defined values include:
    ///
    /// - `8bit`: Data which is not encoded but still considers lines and line length,
    ///           i.e. has no more then 998 bytes between two CRLF (or the start/end of data).
    ///           Bodies of this kind can still be send if the server supports the 8bit
    ///           mime extension.
    ///
    /// - `binary`: Data which is not encoded and can be any kind of arbitrary binary data.
    ///             To send binary bodies the `CHUNKING` smpt extension (rfc3030) needs to be
    ///             supported using BDATA instead of DATA to send the content. Note that the
    ///             extension does not fix the potential but rare problem of accendentall
    ///             multipart boundary collisions.
    ///
    ///
    /// Nevertheless this encodings are mainly meant to be used for defining the
    /// domain of data in a system before it is encoded.
    ContentTransferEncoding, unchecked { "Content-Transfer-Encoding" }, TransferEncoding, maxOne,

    /// A description of the content of the body (rfc2045)
    ///
    /// This is mainly usefull for multipart body parts, e.g.
    /// to add an description to a inlined/attached image.
    ContentDescription,   unchecked { "Content-Description"       }, Unstructured,     maxOne,

    /// Defines the disposition of a multipart part it is used on (rfc2183)
    ///
    /// This is meant to be used as a header for a multipart body part, which
    /// was created from a resource, mainly a file.
    ///
    /// Examples are attachments like images, etc.
    ///
    /// Possible Dispositions are:
    /// - Inline
    /// - Attachment
    ///
    /// Additional it is used to provide following information as parameters:
    /// - `filename`: the file name associated with the resource this body is based on
    /// - `creation-date`: when the resource this body is based on was created
    /// - `modification-date`: when the resource this body is based on was last modified
    /// - `read-date`: when the resource this body is based on was read (to create the body)
    /// - `size`: the size this resource should have, note that `Content-Size` is NOT a mail
    ///           related header but specific to http.
    ContentDisposition, unchecked { "Content-Disposition"       }, Disposition, maxOne
}

mod validators {
    use std::collections::HashMap;

    use common::encoder::EncodableInHeader;
    use ::{ HeaderMap, Header, HeaderName };
    use ::error::HeaderValidationError;

    use super::{ _From, ResentFrom, Sender, ResentSender, ResentDate };


    pub fn from(map: &HeaderMap) -> Result<(), HeaderValidationError> {
        // Note: we do not care about the quantity of From bodies,
        // nor "other" From bodies
        // (which do not use a MailboxList and we could
        //  therefore not cast to it,
        // whatever header put them in has also put in
        // this bit of validation )
        let needs_sender =
            map.get(_From).map(|bodies|
                bodies.filter_map(|res| res.ok()).any(|list| list.len() > 1 )
            ).unwrap_or(false);

        if needs_sender && !map.contains(Sender) {
            //this is the wrong bail...
            header_validation_bail!(kind: MultiMailboxFromWithoutSender);
        }
        Ok(())
    }

    fn validate_resent_block<'a>(
            block: &HashMap<HeaderName, &'a EncodableInHeader>
    ) -> Result<(), HeaderValidationError> {
        if !block.contains_key(&ResentDate::name()) {
            //this is the wrong bail...
            header_validation_bail!(kind: ResentDateFieldMissing);
        }
        let needs_sender =
            //no Resend-From? => no problem
            block.get(&ResentFrom::name())
                //can't cast? => not my problem/responsibility
                .and_then(|tobj| tobj.downcast_ref::<<ResentFrom as Header>::Component>())
                .map(|list| list.len() > 1)
                .unwrap_or(false);

        if needs_sender && !block.contains_key(&ResentSender::name()) {
            //this is the wrong bail...
            header_validation_bail!(kind: MultiMailboxResentFromWithoutResentSender)
        }
        Ok(())
    }

    pub fn resent_any(map: &HeaderMap) -> Result<(), HeaderValidationError> {
        let resents = map
            .iter()
            .filter(|&(name, _)| name.as_str().starts_with("Resent-"));

        let mut block = HashMap::new();
        for (name, content) in resents {
            if block.contains_key(&name) {
                validate_resent_block(&block)?;
                //create new block
                block = HashMap::new();
            }
            block.insert(name, content);
        }
        validate_resent_block(&block)
    }
}

#[cfg(test)]
mod test {
    use components::DateTime;
    use ::{
        HeaderMap,
        _From, ResentFrom, ResentTo, ResentDate,
        Sender, ResentSender, Subject
    };

    #[test]
    fn from_validation_normal() {
        let mut map = HeaderMap::new();
        map.insert(_From, [("Mr. Peté", "pete@nixmail.nixdomain")]).unwrap();
        map.insert(Subject, "Ok").unwrap();

        assert_ok!(map.use_contextual_validators());
    }
    #[test]
    fn from_validation_multi_err() {
        let mut map = HeaderMap::new();
        map.insert(_From, (
            ("Mr. Peté", "nixperson@nixmail.nixdomain"),
            "a@b.c"
        )).unwrap();
        map.insert(Subject, "Ok").unwrap();

        assert_err!(map.use_contextual_validators());
    }

    #[test]
    fn from_validation_multi_ok() {
        let mut map = HeaderMap::new();
        map.insert(_From, (
            ("Mr. Peté", "nixperson@nixmail.nixdomain"),
            "a@b.c"
        )).unwrap();
        map.insert(Sender, "abx@d.e").unwrap();
        map.insert(Subject, "Ok").unwrap();

        assert_ok!(map.use_contextual_validators());
    }

    #[test]
    fn resent_no_date_err() {
        let mut map = HeaderMap::new();
        map.insert(ResentFrom,["a@b.c"]).unwrap();
        assert_err!(map.use_contextual_validators());
    }

    #[test]
    fn resent_with_date() {
        let mut map = HeaderMap::new();
        map.insert(ResentFrom,["a@b.c"]).unwrap();
        map.insert(ResentDate, DateTime::now()).unwrap();
        assert_ok!(map.use_contextual_validators());
    }

    #[test]
    fn resent_no_date_err_second_block() {
        let mut map = HeaderMap::new();
        map.insert(ResentDate, DateTime::now()).unwrap();
        map.insert(ResentFrom,["a@b.c"]).unwrap();
        map.insert(ResentTo, ["e@f.d"]).unwrap();
        map.insert(ResentFrom, ["ee@ee.e"]).unwrap();

        assert_err!(map.use_contextual_validators());
    }

    #[test]
    fn resent_with_date_second_block() {
        let mut map = HeaderMap::new();
        map.insert(ResentDate, DateTime::now()).unwrap();
        map.insert(ResentFrom,["a@b.c"]).unwrap();
        map.insert(ResentTo, ["e@f.d"]).unwrap();
        map.insert(ResentFrom, ["ee@ee.e"]).unwrap();
        map.insert(ResentDate, DateTime::now()).unwrap();

        assert_ok!(map.use_contextual_validators());
    }

    #[test]
    fn resent_multi_mailbox_from_no_sender() {
        let mut map = HeaderMap::new();
        map.insert(ResentDate, DateTime::now()).unwrap();
        map.insert(ResentFrom, ["a@b.c","e@c.d"]).unwrap();

        assert_err!(map.use_contextual_validators());
    }

    #[test]
    fn resent_multi_mailbox_from_with_sender() {
        let mut map = HeaderMap::new();
        map.insert(ResentDate, DateTime::now()).unwrap();
        map.insert(ResentFrom, ["a@b.c","e@c.d"]).unwrap();
        map.insert(ResentSender, "a@b.c").unwrap();

        assert_ok!(map.use_contextual_validators());
    }
}