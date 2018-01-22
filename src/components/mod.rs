mod utils;

//reexport components from core
//TODO check if they should be moved from core to this crate
pub use core::utils::DateTime;
pub use core::codec::transfer_encoding::TransferEncoding;

//reexport our components
mod email;
pub use self::email::{ Email, Domain, LocalPart };

mod mailbox;
pub use self::mailbox::{Mailbox, NoDisplayName};

mod mailbox_list;
pub use self::mailbox_list::{MailboxList, OptMailboxList };

mod unstructured;
pub use self::unstructured::Unstructured;

mod message_id;
pub use self::message_id::{ MessageID, MessageIDList };

pub type ContentID = MessageID;
pub type ContentIDList = MessageIDList;

mod cfws;
pub use self::cfws::{ CFWS, FWS };

mod media_type;
pub use self::media_type::*;

mod path;
pub use self::path::Path;

mod received_token;
pub use self::received_token::ReceivedToken;

pub mod word;
pub use self::word::Word;

mod phrase;
pub use self::phrase::Phrase;

mod phrase_list;
pub use self::phrase_list::PhraseList;

mod disposition;
pub use self::disposition::*;
