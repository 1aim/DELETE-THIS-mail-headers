

use components;
use self::validators::{
    from as validator_from,
    resent_any as validator_resent_any
};

def_headers! {
    test_name: validate_header_names,
    scope: components,
    //RFC 5322:
    1 Date,                    unchecked { "Date"          },  DateTime,       None,
    1 From,                    unchecked { "From"          },  MailboxList,    validator_from,
    1 Sender,                  unchecked { "Sender"        },  Mailbox,        None,
    1 ReplyTo,                 unchecked { "Reply-To"      },  MailboxList,    None,
    1 To,                      unchecked { "To"            },  MailboxList,    None,
    1 Cc,                      unchecked { "Cc"            },  MailboxList,    None,
    1 Bcc,                     unchecked { "Bcc"           },  MailboxList,    None,
    1 MessageId,               unchecked { "Message-Id"    },  MessageID,      None,
    1 InReplyTo,               unchecked { "In-Reply-To"   },  MessageIDList,  None,
    1 References,              unchecked { "References"    },  MessageIDList,  None,
    1 Subject,                 unchecked { "Subject"       },  Unstructured,   None,
    + Comments,                unchecked { "Comments"      },  Unstructured,   None,
    + Keywords,                unchecked { "Keywords"      },  PhraseList,     None,
    + ResentDate,              unchecked { "Resent-Date"   },  DateTime,       validator_resent_any,
    + ResentFrom,              unchecked { "Resent-From"   },  MailboxList,    validator_resent_any,
    + ResentSender,            unchecked { "Resent-Sender" },  Mailbox,        validator_resent_any,
    + ResentTo,                unchecked { "Resent-To"     },  MailboxList,    validator_resent_any,
    + ResentCc,                unchecked { "Resent-Cc"     },  MailboxList,    validator_resent_any,
    + ResentBcc,               unchecked { "Resent-Bcc"    },  OptMailboxList, validator_resent_any,
    + ResentMsgId,             unchecked { "Resent-Msg-Id" },  MessageID,      validator_resent_any,
    + ReturnPath,              unchecked { "Return-Path"   },  Path,           None,
    + Received,                unchecked { "Received"      },  ReceivedToken,  None,
    //RFC 2045:
    1 ContentType,             unchecked { "Content-Type"              }, MediaType,        None,
    1 ContentId,               unchecked { "Content-Id"                }, ContentID,        None,
    1 ContentTransferEncoding, unchecked { "Content-Transfer-Encoding" }, TransferEncoding, None,
    1 ContentDescription,      unchecked { "Content-Description"       }, Unstructured,     None,
    //RFC 2183:
    1 ContentDisposition,      unchecked { "Content-Disposition"       }, Disposition, None
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