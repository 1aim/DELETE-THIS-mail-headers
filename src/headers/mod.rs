

use components;
use self::validators::{
    from as validator_from,
    resent_any as validator_resent_any
};

def_headers! {
    test_name: validate_header_names,
    scope: components,
    /// (rfc5322)
    Date,            maxOne, unchecked { "Date"          },  DateTime,       None,
    /// (rfc5322)
    From,            maxOne, unchecked { "From"          },  MailboxList,    validator_from,
    /// (rfc5322)
    Sender,          maxOne, unchecked { "Sender"        },  Mailbox,        None,
    /// (rfc5322)
    ReplyTo,         maxOne, unchecked { "Reply-To"      },  MailboxList,    None,
    /// (rfc5322)
    To,              maxOne, unchecked { "To"            },  MailboxList,    None,
    /// (rfc5322)
    Cc,              maxOne, unchecked { "Cc"            },  MailboxList,    None,
    /// (rfc5322)
    Bcc,             maxOne, unchecked { "Bcc"           },  MailboxList,    None,
    /// (rfc5322)
    MessageId,       maxOne, unchecked { "Message-Id"    },  MessageID,      None,
    /// (rfc5322)
    InReplyTo,       maxOne, unchecked { "In-Reply-To"   },  MessageIDList,  None,
    /// (rfc5322)
    References,      maxOne, unchecked { "References"    },  MessageIDList,  None,
    /// (rfc5322)
    Subject,         maxOne, unchecked { "Subject"       },  Unstructured,   None,
    /// (rfc5322)
    Comments,     anyNumber, unchecked { "Comments"      },  Unstructured,   None,
    /// (rfc5322)
    Keywords,     anyNumber, unchecked { "Keywords"      },  PhraseList,     None,
    /// (rfc5322)
    ResentDate,   anyNumber, unchecked { "Resent-Date"   },  DateTime,       validator_resent_any,
    /// (rfc5322)
    ResentFrom,   anyNumber, unchecked { "Resent-From"   },  MailboxList,    validator_resent_any,
    /// (rfc5322)
    ResentSender, anyNumber, unchecked { "Resent-Sender" },  Mailbox,        validator_resent_any,
    /// (rfc5322)
    ResentTo,     anyNumber, unchecked { "Resent-To"     },  MailboxList,    validator_resent_any,
    /// (rfc5322)
    ResentCc,     anyNumber, unchecked { "Resent-Cc"     },  MailboxList,    validator_resent_any,
    /// (rfc5322)
    ResentBcc,    anyNumber, unchecked { "Resent-Bcc"    },  OptMailboxList, validator_resent_any,
    /// (rfc5322)
    ResentMsgId,  anyNumber, unchecked { "Resent-Msg-Id" },  MessageID,      validator_resent_any,
    /// (rfc5322)
    ReturnPath,   anyNumber, unchecked { "Return-Path"   },  Path,           None,
    /// (rfc5322)
    Received,     anyNumber, unchecked { "Received"      },  ReceivedToken,  None,

    /// (rfc2045)
    ContentType,     maxOne, unchecked { "Content-Type"              }, MediaType,        None,

    /// (rfc2045)
    ContentId,       maxOne, unchecked { "Content-Id"                }, ContentID,        None,

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
    ContentTransferEncoding, maxOne, unchecked { "Content-Transfer-Encoding" }, TransferEncoding, None,

    /// A description of the content of the body (rfc2045)
    ///
    /// This is mainly usefull for multipart body parts, e.g.
    /// to add an description to a inlined/attached image.
    ContentDescription,      maxOne, unchecked { "Content-Description"       }, Unstructured,     None,

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
    ContentDisposition, maxOne, unchecked { "Content-Disposition"       }, Disposition, None
}

mod validators {
    use std::collections::HashMap;

    use core::error::Result;
    use core::codec::EncodableInHeader;
    use core::headers::{ HeaderMap, Header, HeaderName };

    use error::HeaderValidationError::{
        MultiMailboxFromWithoutSender,
        ResentDateFieldMissing,
        MultiMailboxResentFromWithoutResentSender
    };

    use super::{ From, ResentFrom, Sender, ResentSender, ResentDate };


    pub fn from(map: &HeaderMap) -> Result<()> {
        // Note: we do not care about the quantity of From bodies,
        // nor "other" From bodies
        // (which do not use a MailboxList and we could
        //  therefore not cast to it,
        // whatever header put them in has also put in
        // this bit of validation )
        let needs_sender =
            map.get(From).map(|bodies|
                bodies.filter_map(|res| res.ok()).any(|list| list.len() > 1 )
            ).unwrap_or(false);

        if needs_sender && !map.contains(Sender) {
            //this is the wrong bail...
            bail_header!(MultiMailboxFromWithoutSender);
        }
        Ok(())
    }

    fn validate_resent_block<'a>(
            block: &HashMap<HeaderName, &'a EncodableInHeader>
    ) -> Result<()> {
        if !block.contains_key(&ResentDate::name()) {
            //this is the wrong bail...
            bail_header!(ResentDateFieldMissing);
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
            bail_header!(MultiMailboxResentFromWithoutResentSender)
        }
        Ok(())
    }

    pub fn resent_any(map: &HeaderMap) -> Result<()> {
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
    use core::headers::HeaderMap;
    use components::DateTime;
    use headers::{
        From, ResentFrom, ResentTo, ResentDate,
        Sender, ResentSender, Subject
    };

    #[test]
    fn from_validation_normal() {
        let mut map = HeaderMap::new();
        map.insert(From, [("Mr. Peté", "pete@nixmail.nixdomain")]).unwrap();
        map.insert(Subject, "Ok").unwrap();

        assert_ok!(map.use_contextual_validators());
    }
    #[test]
    fn from_validation_multi_err() {
        let mut map = HeaderMap::new();
        map.insert(From, (
            ("Mr. Peté", "nixperson@nixmail.nixdomain"),
            "a@b.c"
        )).unwrap();
        map.insert(Subject, "Ok").unwrap();

        assert_err!(map.use_contextual_validators());
    }

    #[test]
    fn from_validation_multi_ok() {
        let mut map = HeaderMap::new();
        map.insert(From, (
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