// Irc Message Parser
//
#![feature(macro_rules)]

pub struct Parser<I> {
    reader: Box<I>,
    state: State,
    ch: Option<char>
}

pub struct Message {
    pub prefix: String,
    pub command: String,
    pub params: String,
    pub body: String
}

#[deriving(PartialEq, Show, Clone)]
pub enum State {
    Start,
    Prefix,
    Command,
    Params,
    Body
}

pub type Token = (State, String);

impl<I: Buffer> Parser<I> {
    pub fn new(rdr: Box<I>) -> Parser<I> {
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

        (Body, msg)
    }

    pub fn messages<'a>(&'a mut self) -> MessageIterator<'a, I> {
        MessageIterator { parser: self }
    }
}

impl<I: Buffer> Iterator<Token> for Parser<I> {
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
                    Prefix | Start | Body=> { self.parse_command(c.is_digit()) }
                    _ => { self.parse_params() }
                }
            }

            _ => (Start, String::new())

        };

        self.state = token.ref0().clone(); // TODO: Why do I have to clone?

        Some(token)
    }
}

struct MessageIterator<'a, I> {
    parser: &'a mut Parser<I>
}

impl<'a, I: Buffer> Iterator<Message> for MessageIterator<'a, I> {
    fn next(&mut self) -> Option<Message> {
        let mut msg = Message {
            prefix: String::new(),
            command: String::new(),
            params: String::new(),
            body: String::new()
        };

        let mut token = self.parser.next();

        while token.is_some() {
            let (c, s) = token.unwrap();
            match c {
                Prefix => { msg.prefix = s }
                Command => { msg.command = s }
                Params => { msg.params = s }
                Body => { msg.body = s; break; }
                _ => { break; }
            }

            token = self.parser.next();
        }

        Some(msg)
    }
}

#[cfg(test)]
mod test {

    use super::{Parser, Message, Prefix, Command, Params,Body};
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

        let mut buf = box BufReader::new(example.as_bytes());
        let mut parser = Parser::new(buf);

        test_token!(parser.next(), Prefix, "Angel!wings@irc.org");
        test_token!(parser.next(), Command, "PRIVMSG");
        test_token!(parser.next(), Params, "Wiz");
        test_token!(parser.next(), Body, "Are you receiving this message ?");
        test_token!(parser.next(), Command, "PING");
        test_token!(parser.next(), Body, "irc.funet.fi");
    }

    #[test]
    fn test_by_message () {
        let example = ":Angel PRIVMSG Wiz :Hello message ?\r\n";
        let mut buf = box BufReader::new(example.as_bytes());
        let mut parser = Parser::new(buf);
        let msg = parser.messages().next().unwrap();

        assert_eq!(msg.prefix, "Angel".to_string());
        assert_eq!(msg.command, "PRIVMSG".to_string());
        assert_eq!(msg.params, "Wiz".to_string());
        assert_eq!(msg.body, "Hello message ?".to_string());
    }
}
