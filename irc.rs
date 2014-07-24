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
        println!("{}", msg.prefix);
    }
}

struct Message {
    content: String,
    prefix: String
}

impl Message {
    fn new () -> Message {
        Message {
            content: String::new(),
            prefix: String::new()
        }
    }
}

struct MessageIterator<'a, T> {
    stream: &'a mut T
}

enum State {
    Start,
    Command
}

struct Parser<'a, I> {
    current_state: State,
    iter: &'a mut I
}

impl<'a, I: Iterator<char>> Parser<'a, I> {
    fn new(iter: &'a mut I) -> Parser<'a, I> {
        Parser {
            current_state: Start,
            iter: iter
        }
    }

    fn parse (&self, c: char) {
        match self.current_state {
            Start => {
                match c {
                    'a'..'z' | 'A'..'Z' => { }
                    '0'..'9' => { }
                    _ => {}
                }
            }
            Command => {}
        }
    }
}

#[test]
fn test_state_machine () {
    let msg = "COMMAND\n";
    let mut chars = msg.chars();
    let parser = Parser::new(&mut chars);
}

// Message format in Augmented BNF.
//
// command crlf
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

impl<'a, T: Reader> Iterator<Message> for MessageIterator<'a, T> {
    fn next (&mut self) -> Option<Message> {
        let mut msg = Message::new();
        let mut is_prefix = false;

        loop {
            let byte = self.stream.read_byte().unwrap() as char;
            // Fixme. figure out a better way and stop using unwrap

            match byte {
                '\n' => { break }
                ':' => {
                    msg.prefix.push_char(byte);
                    is_prefix = !is_prefix;
                    continue;
                }
                _ => {
                    if is_prefix {
                        msg.prefix.push_char(byte);
                    } else {
                        msg.content.push_char(byte);
                    }
                }
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
