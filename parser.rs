// Irc Message Parser
//

struct Parser<'i, I> {
    chars: &'i mut I,
    ch: Option<char>,
    state: ParserState
}

#[deriving(PartialEq, Show)]
enum ParserState {
    Start,
    Prefix,
    Command,
    NumericCommand,
    Params,
    Message
}

impl<'i, I: Iterator<char>> Parser<'i, I> {
    fn new(chars: &'i mut I) -> Parser<'i, I> {
        Parser {
            chars: chars,
            ch: Some('\x00'),
            state: Start
        }
    }

    fn bump(&mut self) {
        self.ch = self.chars.next()
    }

    fn is_ch(&mut self, ch: char) -> bool {
        self.ch == Some(ch)
    }

    fn ch_or_null(&mut self) -> char {
        self.ch.unwrap_or('\x00')
    }

    fn eof(&mut self) -> bool {
        self.ch == None
    }

    fn parse_prefix(&mut self) -> Option<String> {
        let mut prefix = String::new();

        loop {
            self.bump();
            if self.is_ch(' ') || self.eof() { break; }
            prefix.push_char(self.ch.unwrap());
        }

        Some(prefix)
    }

    fn parse_command(&mut self) -> Option<String> {
        let mut command = String::new();

        match self.state {
            NumericCommand => {
                for _ in range(1u, 4u) {
                    command.push_char(self.ch.unwrap());
                    self.bump();
                }
            }

            Command => {
                loop {
                    if self.is_ch(' ') { break; }
                    command.push_char(self.ch.unwrap());
                    self.bump();
                }
            }

            _ => { return None }
        }

        Some(command)
    }

    fn parse_params(&mut self) -> Option<String> {
        let mut params = String::new();

        loop {
            if self.is_ch(' ') { break; }
            params.push_char(self.ch.unwrap());
            self.bump();
        }

        Some(params)
    }

    fn parse_message(&mut self) -> Option<String> {
        let mut msg = String::new();

        self.bump(); // Skip the :

        loop {
            if self.is_ch('\n') { break; }
            msg.push_char(self.ch_or_null());
            self.bump();
        }

        Some(msg)
    }
}

impl<'i, I: Iterator<char>> Iterator<String> for Parser<'i, I> {
    fn next(&mut self) -> Option<String> {
        self.ch = self.chars.next();

        match self.ch_or_null() {
            ':' => {
                match self.state {
                    Start => {
                        self.state = Prefix;
                        self.parse_prefix()
                    }

                    Params => {
                        self.state = Message;
                        self.parse_message()
                    }

                    _ =>  None
                }
            }

            c @ 'A'..'Z' |
            c @ 'a'..'z' |
            c @ '0'..'9' => {
                if self.state == Prefix || self.state == Start {
                    self.state = if c.is_digit() { NumericCommand } else { Command };
                    self.parse_command()
                } else {
                    self.state = Params;
                    self.parse_params()
                }
            }

            _ => None
        }
    }
}

#[cfg(test)]
mod test {
    use super::Parser;

    #[test]
    fn test_commands() {
        let mut example = String::new();
        example.push_str(":Angel!wings@irc.org PRIVMSG Wiz :Are you receiving this message ?\n");
        example.push_str("PING :irc.funet.fi\n");
        let mut chars = example.as_slice().clone().chars();
        let mut parser = Parser::new(&mut chars);

        assert_eq!(parser.next(), Some("Angel!wings@irc.org".to_string()));
        assert_eq!(parser.next(), Some("PRIVMSG".to_string()));
        assert_eq!(parser.next(), Some("Wiz".to_string()));
        assert_eq!(parser.next(), Some("Are you receiving this message ?".to_string()));
        assert_eq!(parser.next(), Some("PING".to_string()));
        assert_eq!(parser.next(), Some("irc.funet.fi".to_string()));
        assert_eq!(parser.next(), None);
    }
}
