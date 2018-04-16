use std::ops::Deref;

use soft_ascii_string::SoftAsciiChar;

use mime::spec::{MimeSpec, Ascii, Internationalized, Modern};
use quoted_string::quote_if_needed;

use common::error::Result;
use common::grammar::{
    is_ascii,
    is_atext,
    is_dtext,
    is_ws,
};
use common::MailType;
use common::codec::{EncodeHandle, EncodableInHeader };
use common::codec::idna;
use common::utils::{HeaderTryInto, HeaderTryFrom};
use common::data::{Input, SimpleItem, InnerUtf8 };
use common::codec::quoted_string::UnquotedDotAtomTextValidator;

use error::ComponentError::{InvalidDomainName, InvalidEmail, InvalidLocalPart};

/// an email of the form `local-part@domain`
/// corresponds to RFC5322 addr-spec, so `<`, `>` padding is _not_
/// part of this Email type (but of the Mailbox type instead)
#[derive(Debug,  Clone, Hash, PartialEq, Eq)]
pub struct Email {
    pub local_part: LocalPart,
    pub domain: Domain
}


#[derive(Debug,  Clone, Hash, PartialEq, Eq)]
pub struct LocalPart( Input );

#[derive(Debug,  Clone, Hash, PartialEq, Eq)]
pub struct Domain( SimpleItem );

impl Email {
    pub fn new<T: HeaderTryInto<Input>>(email: T) -> Result<Self> {
        let email = email.try_into()?.into_shared();
        match email {
            Input( InnerUtf8::Owned( .. ) ) => unreachable!(),
            Input( InnerUtf8::Shared( shared ) ) => {
                //1. ownify Input
                //2. get 2 sub shares split befor/after @
                let index = shared.find( "@" )
                    .ok_or_else( || { error!(InvalidEmail(shared.to_string())) })?;

                let left = shared.clone().map( |all| &all[..index] );
                let local_part = LocalPart::try_from( Input( InnerUtf8::Shared( left ) ) )?;
                //index+1 is ok as '@'.utf8_len() == 1
                let right = shared.map( |all| &all[index+1..] );
                let domain = Domain::try_from( Input( InnerUtf8::Shared( right ) ) )?;
                Ok( Email { local_part, domain } )
            }
        }
    }
}

impl<'a> HeaderTryFrom<&'a str> for Email {
    fn try_from( email: &str ) -> Result<Self> {
        Email::new(email)
    }
}

impl HeaderTryFrom<String> for Email {
    fn try_from( email: String ) -> Result<Self> {
        Email::new(email)
    }
}

impl HeaderTryFrom<Input> for Email {
    fn try_from( email: Input ) -> Result<Self> {
        Email::new(email)
    }
}


impl EncodableInHeader for  Email {

    fn encode(&self, handle: &mut EncodeHandle) -> Result<()> {
        self.local_part.encode( handle )?;
        handle.write_char( SoftAsciiChar::from_char_unchecked('@') )?;
        self.domain.encode( handle )?;
        Ok( () )
    }

    fn boxed_clone(&self) -> Box<EncodableInHeader> {
        Box::new(self.clone())
    }
}

impl<T> HeaderTryFrom<T> for LocalPart
    where T: HeaderTryInto<Input>
{

    fn try_from( input: T ) -> Result<Self> {
        Ok( LocalPart( input.try_into()? ) )
    }

}

impl EncodableInHeader for LocalPart {

    fn encode(&self, handle: &mut EncodeHandle) -> Result<()> {
        let input: &str = &*self.0;
        let mail_type = handle.mail_type();

        let mut validator = UnquotedDotAtomTextValidator::new(mail_type);

        let res = if mail_type.is_internationalized() {
            quote_if_needed::<MimeSpec<Internationalized, Modern>, _>(input, &mut validator)
        } else {
            quote_if_needed::<MimeSpec<Ascii, Modern>, _>(input, &mut validator)
        }.map_err(|_qs_err| error!(InvalidLocalPart(input.into())))?;


        handle.mark_fws_pos();
        // if mail_type == Ascii quote_if_needed already made sure it's ascii
        // it also made sure it is valid as it is either `dot-atom-text` or `quoted-string`
        handle.write_str_unchecked(&*res)?;
        handle.mark_fws_pos();
        Ok( () )
    }

    fn boxed_clone(&self) -> Box<EncodableInHeader> {
        Box::new(self.clone())
    }
}

impl Deref for LocalPart {
    type Target = Input;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}



impl<T> HeaderTryFrom<T> for Domain
    where T: HeaderTryInto<Input>
{
    fn try_from( input: T ) -> Result<Self> {
        let input = input.try_into()?;
        let item =
            match Domain::check_domain( input.as_str() )? {
                MailType::Ascii | MailType::Mime8BitEnabled => {
                    SimpleItem::Ascii( input.into_ascii_item_unchecked() )
                },
                MailType::Internationalized => {
                    SimpleItem::from_utf8_input( input )
                }
            };

        Ok( Domain( item ) )
    }
}

impl Domain {
    //SAFETY:
    //  the function is only allowed to return MailType::Ascii
    //  if the domain is actually ascii
    fn check_domain( domain: &str ) -> Result<MailType> {
        let mut ascii = true;
        if domain.starts_with("[") && domain.ends_with("]") {
            //check domain-literal
            //for now the support of domain literals is limited i.e:
            //  1. no contained line
            //  2. no leading/trailing CFWS before/after the "["/"]"
            for char in domain.chars() {
                if ascii { ascii = is_ascii( char ) }
                if !( is_dtext( char, MailType::Internationalized) || is_ws( char ) ) {
                    bail!(InvalidDomainName(domain.to_owned()));
                }
            }
        } else {
            //check dot-atom-text
            // when supported Comments will be supported through the type system,
            // not stringly typing
            let mut dot_alowed = false;
            for char in domain.chars() {
                if ascii { ascii = is_ascii( char ) }
                if char == '.' && dot_alowed {
                    dot_alowed = false;
                } else if !is_atext( char, MailType::Internationalized ) {
                    bail!(InvalidDomainName(domain.to_owned()));
                } else {
                    dot_alowed = true;
                }
            }
        }
        Ok( if ascii {
            MailType::Ascii
        } else {
            MailType::Internationalized
        } )
    }
}

impl EncodableInHeader for  Domain {

    fn encode(&self, handle: &mut EncodeHandle) -> Result<()> {
        handle.mark_fws_pos();
        match self.0 {
            SimpleItem::Ascii( ref ascii ) => {
                handle.write_str( ascii )?;
            },
            SimpleItem::Utf8( ref utf8 ) => {
                handle.write_if_utf8(utf8)
                    .handle_condition_failure(|handle| {
                        handle.write_str( &*idna::puny_code_domain( utf8 )? )
                    })?;
            }
        }
        handle.mark_fws_pos();
        Ok( () )
    }

    fn boxed_clone(&self) -> Box<EncodableInHeader> {
        Box::new(self.clone())
    }
}

impl Deref for Domain {
    type Target = SimpleItem;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}



#[cfg(test)]
mod test {
    use common::codec::{ Encoder, VecBodyBuf};
    use super::*;

    #[test]
    fn email_try_from() {
        let email = Email::try_from( "abc@de.fg" ).unwrap();
        assert_eq!(
            Email {
                local_part: LocalPart::try_from( "abc" ).unwrap(),
                domain: Domain::try_from( "de.fg" ).unwrap()
            },
            email
        )
    }

    ec_test!{ local_part_simple, {
        LocalPart::try_from(  "hans" )?
    } => ascii => [
        MarkFWS,
        Text "hans",
        MarkFWS
    ]}

    //fails tries to write utf8
    ec_test!{ local_part_quoted, {
        LocalPart::try_from(  "ha ns" )?
    } => ascii => [
        MarkFWS,
        Text "\"ha ns\"",
        MarkFWS
    ]}


    ec_test!{ local_part_utf8, {
        LocalPart::try_from( "Jörn" )?
    } => utf8 => [
        MarkFWS,
        Text "Jörn",
        MarkFWS
    ]}

    #[test]
    fn local_part_utf8_on_ascii() {
        let mut encoder = Encoder::<VecBodyBuf>::new( MailType::Ascii );
        let mut handle = encoder.encode_handle();
        let local = LocalPart::try_from( "Jörn" ).unwrap();
        assert_err!(local.encode( &mut handle ));
        handle.undo_header();
    }

    ec_test!{ domain, {
        Domain::try_from( "bad.at.domain" )?
    } => ascii => [
        MarkFWS,
        Text "bad.at.domain",
        MarkFWS
    ]}

    ec_test!{ domain_international, {
        Domain::try_from( "dömain" )?
    } => utf8 => [
        MarkFWS,
        Text "dömain",
        MarkFWS
    ]}


    ec_test!{ domain_encoded, {
        Domain::try_from( "dat.ü.dü" )?
    } => ascii => [
        MarkFWS,
        Text "dat.xn--tda.xn--d-eha",
        MarkFWS
    ]}


    ec_test!{ email_simple, {
        Email::try_from( "simple@and.ascii" )?
    } => ascii => [
        MarkFWS,
        Text "simple",
        MarkFWS,
        Text "@",
        MarkFWS,
        Text "and.ascii",
        MarkFWS
    ]}

    #[test]
    fn local_part_as_str() {
        let lp = LocalPart::try_from("hello").unwrap();
        assert_eq!(lp.as_str(), "hello")
    }

    #[test]
    fn domain_as_str() {
        let domain = Domain::try_from("hello").unwrap();
        assert_eq!(domain.as_str(), "hello")
    }
}