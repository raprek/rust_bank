use std::{
    io::{BufRead, BufReader, BufWriter, LineWriter, Read, Write},
    net::TcpStream,
};

fn main() {
    let mut stream = TcpStream::connect("127.0.0.1:1234").unwrap();
    {
        let mut buf_w = LineWriter::new(&mut stream);
        buf_w.write(b"123\n").unwrap();
        buf_w.write(b"\n").unwrap();
        buf_w.flush();
    }

    let mut b_reader = BufReader::new(&mut stream);
    let mut buf = String::new();
    b_reader.read_line(&mut buf);

    println!("msg from server {buf}")
}
