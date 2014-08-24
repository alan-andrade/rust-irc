// Irc Message Parser
//
#![feature(macro_rules)]

pub struct Parser<'i, I> {
    reader: &'i mut I,
    state: ParserState,
    ch: Option<char>
}

#[deriving(PartialEq, Show, Clone)]
pub enum ParserState {
    Start,
    Prefix,
    Command,
    Params,
    Message
}

pub type Token = (ParserState, String);

impl<'i, I: Buffer> Parser<'i, I> {
    pub fn new(rdr: &'i mut I) -> Parser<'i, I> {
        Parser {
            reader: rdr,
            ch: Some('\x00'),
            state: Start
        }
    }

    fn bump(&mut self) {
        self.ch = self.reader.read_char().ok();
    }

    fn is_ch(&mut self, ch: char) -> bool {
        self.ch == Some(ch)
    }

    fn ch_or_null(&mut self) -> char {
        self.ch.unwrap_or('\x00')
    }

    fn eof(&mut self) -> bool {
        self.ch.is_none()
    }

    fn parse_prefix(&mut self) -> Token {
        let mut prefix = String::new();

        loop {
            self.bump();
            if self.is_ch(' ') || self.eof() { break; }
            prefix.push_char(self.ch_or_null());
        }

        (Prefix, prefix)
    }

    fn parse_command(&mut self, numeric: bool) -> Token {
        let mut command = String::new();

        if numeric {
            for _ in range(1u, 4u) {
                command.push_char(self.ch_or_null());
                self.bump();
            }
        } else {
            loop {
                if self.is_ch(' ') { break; }
                command.push_char(self.ch_or_null());
                self.bump();
            }
        }

        (Command, command)
    }

    fn parse_params(&mut self) -> Token {
        let mut params = String::new();

        loop {
            if self.is_ch(' ') { break; }
            params.push_char(self.ch_or_null());
            self.bump();
        }

        (Params, params)
    }

    fn parse_message(&mut self) -> Token {
        let mut msg = String::new();

        loop {
            self.bump(); // Skip the : on the first iteration
            if self.is_ch('\r') { continue; }
            if self.is_ch('\n') { break; }
            msg.push_char(self.ch_or_null());
        }

        (Message, msg)
    }
}

impl<'i, I: Buffer> Iterator<Token> for Parser<'i, I> {
    fn next(&mut self) -> Option<Token> {
        self.bump();

        let token = match self.ch_or_null() {
            ':' => {
                match self.state {
                    Start => { self.parse_prefix() }
                    Params => { self.parse_message() }
                    Command => { self.parse_message() }
                    _ => (Start, String::new())
                }
            }

            c @ 'A'..'Z' |
            c @ 'a'..'z' |
            c @ '0'..'9' => {
                match self.state {
                    Prefix | Start | Message => { self.parse_command(c.is_digit()) }
                    _ => { self.parse_params() }
                }
            }

            _ => (Start, String::new())

        };

        self.state = token.ref0().clone();

        Some(token)
    }
}

#[cfg(test)]
mod test {

    use super::Parser;
    use super::{Prefix, Command, Params, Message};
    use std::io::BufReader;

    #[test]
    fn test_commands() {
        macro_rules! test_token(
            ($next: expr, $state: ident, $string: expr) => (
                assert_eq!($next, Some(($state, $string.to_string())));
            )
        )

        let mut example = String::new();
        example.push_str(":Angel!wings@irc.org PRIVMSG Wiz :Are you receiving this message ?\r\n");
        example.push_str("PING :irc.funet.fi\r\n");

        let mut buf = BufReader::new(example.as_bytes());
        let mut parser = Parser::new(&mut buf);

        test_token!(parser.next(), Prefix, "Angel!wings@irc.org");
        test_token!(parser.next(), Command, "PRIVMSG");
        test_token!(parser.next(), Params, "Wiz");
        test_token!(parser.next(), Message, "Are you receiving this message ?");
        test_token!(parser.next(), Command, "PING");
        test_token!(parser.next(), Message, "irc.funet.fi");
    }
}
