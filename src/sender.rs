use std::fs::File;
use std::io::{self, Write};
use std::net::TcpStream;
use std::env;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        println!("Usage: client <ip:port> <filepath>");
        return Ok(());
    }

    let addr = &args[1];
    let filepath = &args[2];

    let mut file = File::open(filepath)?;
    let filename = filepath
        .split('\\')
        .last()
        .unwrap()
        .to_string();

    println!("Connecting to {}...", addr);
    let mut socket = TcpStream::connect(addr)?;
    println!("Connected. Sending {}...", filename);

    // Send filename length (2 bytes big-endian)
    let name_bytes = filename.as_bytes();
    let name_len = name_bytes.len() as u16;
    socket.write_all(&name_len.to_be_bytes())?;

    // Send filename
    socket.write_all(name_bytes)?;

    // Send file size
    let file_size: u64 = file.metadata()?.len() as u64;
    socket.write_all(&file_size.to_be_bytes())?;

    // Send file contents
    let bytes_copied = io::copy(&mut file, &mut socket)?;
    println!("Done. Sent {} bytes.", bytes_copied);

    Ok(())
    // Order of data sent:
    // File name length bytes
    // File name bytes
    // File size bytes
    // File content bytes
}
