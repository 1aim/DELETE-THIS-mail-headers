//! mail-common does not ship with any predefined headers and components
//! except `RawUnstructured`, `TransferEncoding` and `DateTime`

use soft_ascii_string::SoftAsciiStr;

use common::{HeaderTryFrom, HeaderTryInto};
use common::data::Input;
use common::error::Result;
use common::grammar::is_vchar;
use common::codec::{EncodeHandle, EncodableInHeader};

use error::ComponentError::InvalidRawUnstructured;

/// A unstructured header field implementation which validates the given input
/// but does not encode any utf8 even if it would have been necessary (it will
/// error in that case) nor does it support breaking longer lines in multiple
/// ones (no FWS marked for the encoder)
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct RawUnstructured {
    text: Input
}

impl RawUnstructured {
    pub fn as_str(&self) -> &str {
        self.text.as_str()
    }
}

impl<T> From<T> for RawUnstructured
    where Input: From<T>
{
    fn from(val: T) -> Self {
        RawUnstructured { text: val.into() }
    }
}

impl<T> HeaderTryFrom<T> for RawUnstructured
    where T: HeaderTryInto<Input>
{
    fn try_from(val: T) -> Result<Self> {
        let input: Input = val.try_into()?;
        Ok( input.into() )
    }
}

impl Into<Input> for RawUnstructured {
    fn into(self) -> Input {
        self.text
    }
}

impl Into<String> for RawUnstructured {
    fn into(self) -> String {
        self.text.into()
    }
}

impl AsRef<str> for RawUnstructured {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl EncodableInHeader for RawUnstructured {
    fn encode(&self, handle: &mut EncodeHandle) -> Result<()> {
        let mail_type = handle.mail_type();

        if !self.text.chars().all(|ch| is_vchar(ch, mail_type)) {
            let input = self.text.as_str().to_owned();
            bail!(InvalidRawUnstructured(input, mail_type))
        }

        if handle.mail_type().is_internationalized() {
            handle.write_utf8(self.text.as_str())
        } else {
            handle.write_str(SoftAsciiStr::from_str_unchecked(self.text.as_str()))
        }
    }

    fn boxed_clone(&self) -> Box<EncodableInHeader> {
        Box::new(self.clone())
    }
}


