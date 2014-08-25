// Rust irc client
#![feature(macro_rules)]

extern crate debug;
use std::io::net::tcp::TcpStream;
use std::io::stdio::{stdin, stdout};
use std::io::{Buffer, BufferedReader};

mod Parser;

fn main () {
    let stream = TcpStream::connect("irc.freenode.net", 6667).unwrap();
    let stream_a = stream.clone();
    let mut stream = BufferedReader::new(stream);
    let mut parser = Parser::Parser::new(&mut stream);

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

    loop {
        for c in parser {
            match c {
                (Parser::Message, msg) => { println!("{}", msg); }
                _ => { }
            }
        }
    }
}
