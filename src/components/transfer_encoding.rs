use soft_ascii_string::SoftAsciiStr;

use core::error::Result;
use core::codec::{ EncodeHandle, EncodableInHeader};

/// The TransferEnecoding header component mainly used by the ContentTransferEncodingHeader.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum TransferEncoding {
    _7Bit,
    _8Bit,
    Binary,
    QuotedPrintable,
    Base64
}

impl TransferEncoding {
    pub fn repr(&self ) -> &SoftAsciiStr {
        use self::TransferEncoding::*;
        match *self {
            _7Bit => SoftAsciiStr::from_str_unchecked("7bit"),
            _8Bit => SoftAsciiStr::from_str_unchecked("8bit"),
            Binary =>  SoftAsciiStr::from_str_unchecked("binary"),
            QuotedPrintable => SoftAsciiStr::from_str_unchecked("quoted-printable"),
            Base64 =>  SoftAsciiStr::from_str_unchecked("base64"),
        }
    }
}


impl EncodableInHeader for  TransferEncoding {

    fn encode(&self, handle: &mut EncodeHandle) -> Result<()> {
        handle.write_str( self.repr() )?;
        Ok( () )
    }

    fn boxed_clone(&self) -> Box<EncodableInHeader> {
        Box::new(self.clone())
    }
}


#[cfg(test)]
mod test {
    use super::TransferEncoding;

    ec_test! {_7bit, {
        TransferEncoding::_7Bit
    } => ascii => [
        Text "7bit"
    ]}

    ec_test! {_8bit, {
        TransferEncoding::_8Bit
    } => ascii => [
        Text "8bit"
    ]}

    ec_test!{binary, {
        TransferEncoding::Binary
    } => ascii => [
        Text "binary"
    ]}

    ec_test!{base64, {
        TransferEncoding::Base64
    } => ascii => [
        Text "base64"
    ]}

    ec_test!{quoted_printable, {
        TransferEncoding::QuotedPrintable
    } => ascii => [
        Text "quoted-printable"
    ]}
}


