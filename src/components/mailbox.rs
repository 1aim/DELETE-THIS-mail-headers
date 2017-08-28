use ascii::AsciiChar;

use error::*;
use codec::{ MailEncodable, MailEncoder };

use super::Phrase;
use super::Email;

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct Mailbox {
    pub display_name: Option<Phrase>,
    pub email: Email
}


impl From<Email> for Mailbox {

    fn from( email: Email ) -> Self {
        Mailbox {
            email,
            display_name: None,
        }
    }
}


impl<E> MailEncodable<E> for Mailbox where E: MailEncoder {

    fn encode(&self, encoder: &mut E) -> Result<()> {
        if let Some( display_name ) = self.display_name.as_ref() {
            display_name.encode( encoder )?;
            encoder.write_fws();
        }
        //for now this always uses the "<user@do.main>" form even if no display-name is given
        encoder.write_char( AsciiChar::LessThan );
        self.email.encode( encoder )?;
        encoder.write_char( AsciiChar::GreaterThan );
        Ok( () )
    }
}


#[cfg(test)]
mod test {
    use data::FromInput;
    use components::{ Email, Phrase };
    use codec::test_utils::*;
    use super::*;

    ec_test!{ email_only, {
        Email::from_input( "affen@haus" )
            .map(Mailbox::from)
    } => ascii => [
        LinePart( "<" ),
        OptFWS,
        LinePart( "affen" ),
        OptFWS,
        LinePart( "@" ),
        OptFWS,
        LinePart( "haus" ),
        OptFWS,
        LinePart( ">" )
    ]}

    ec_test!{ with_display_text, { Some(
        Mailbox {
            display_name: Some( Phrase::from_input( "ay ya" ).unwrap() ),
            email: Email::from_input( "affen@haus" ).unwrap(),
        }
    ) } => ascii => [
        LinePart( "ay" ),
        FWS,
        LinePart( "ya" ),
        FWS,
        LinePart( "<" ),
        OptFWS,
        LinePart( "affen" ),
        OptFWS,
        LinePart( "@" ),
        OptFWS,
        LinePart( "haus" ),
        OptFWS,
        LinePart( ">" )
    ]}
}

