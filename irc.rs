// Rust irc client
extern crate debug;
use std::io::net::tcp::TcpStream;
use std::io::stdio::{stdin, stdout};

fn main () {
    let mut stream = TcpStream::connect("irc.freenode.net", 6667).unwrap();
    let stream_a = stream.clone();

    let mut irc = Irc::new(&mut stream);

    spawn(proc() {
        let mut stream = stream_a;

        stream.write_line("USER rust-irc 0 * :Rust irc");
        stream.write_line("NICK rust-irc");

        loop {
            match stdin().read_line() {
                Ok(str) => { stream.write_str(str.as_slice()); },
                Err(e) => { stdout().write_str(e.desc); }
            }
        }
    });

    for msg in irc.message_stream() {
        println!("{}", msg.content);
    }
}

struct Message {
    content: String
}

struct MessageIterator<'a, T> {
    stream: &'a mut T
}

impl<'a, T: Reader> Iterator<Message> for MessageIterator<'a, T> {
    fn next (&mut self) -> Option<Message> {
        let mut msg = Message { content: String::new() };

        loop {
            let byte = self.stream.read_byte().unwrap() as char;

            match byte {
                '\n' => { break }
                _ => { msg.content.push_char(byte) }
            }
        }

        Some(msg)
    }
}

struct Irc<'a, T> {
    stream: &'a mut T
}

impl<'a, T: Reader> Irc<'a, T> {
    fn new (stream: &'a mut T) -> Irc<'a, T> {
        Irc { stream: stream }
    }

    fn message_stream<'a>(&'a mut self) -> MessageIterator<'a, T> {
        MessageIterator { stream: self.stream }
    }
}

pub struct Parser {
    space_found: bool,
    is_message: bool
}

impl Parser {
    fn new () -> Parser {
        Parser {
            space_found: false,
            is_message: false
        }
    }

    fn parse (&mut self, letter: &char) {
        match *letter {
            ' ' => {
                self.space_found = true;
                if self.is_message {
                    stdout().write_char(*letter);
                }
            }

            ':' => {
                if self.space_found {
                    self.is_message = true;
                }
                if self.is_message {
                    stdout().write_char(*letter);
                }
            }

            '\n' => {
                if self.is_message {
                    stdout().write_char(*letter);
                }
                self.space_found = false;
                self.is_message = false;
            }

            _ => {
                if self.is_message {
                    stdout().write_char(*letter);
                }
            }

        }
    }
}

// Message format in Augmented BNF.
//
// message    =  [ ":" prefix SPACE ] command [ params ] crlf
//
// prefix     =  servername / ( nickname [ [ "!" user ] "@" host ] )
// command    =  1*letter / 3digit
// params     =  *14( SPACE middle ) [ SPACE ":" trailing ]
//            =/ 14( SPACE middle ) [ SPACE [ ":" ] trailing ]
//
// nospcrlfcl =  %x01-09 / %x0B-0C / %x0E-1F / %x21-39 / %x3B-FF
//            ; any octet except NUL, CR, LF, " " and ":"
// middle     =  nospcrlfcl *( ":" / nospcrlfcl )
// trailing   =  *( ":" / " " / nospcrlfcl )
//
// SPACE      =  %x20        ; space character
// crlf       =  %x0D %x0A   ; "carriage return" "linefeed"
