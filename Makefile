test:
	rustc -g --test parser.rs && ./parser

build:
	rustc irc.rs
