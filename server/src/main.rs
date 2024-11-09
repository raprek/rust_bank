use core::str;
use std::{
    io::{BufRead, BufReader, Read, Write},
    net::TcpListener,
};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:1234").unwrap();
    loop {
        let (mut stream, addr) = listener.accept().unwrap();
        println!("New connection add: {addr}");
        let buf_reader = BufReader::new(&mut stream);
        let buf = String::new();
        let res: String = buf_reader
            .lines()
            .map(|result| result.unwrap())
            .take_while(|line| !line.is_empty())
            .collect();

        // stream.read_to_string(&mut buf).unwrap();
        println!("New msg: {:?}", res);
        stream.write_all(b"echo");
        drop(stream)
    }
}
