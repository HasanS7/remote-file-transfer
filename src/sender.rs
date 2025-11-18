use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::env;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        println!("Usage: client <ip:port> <filepath>");
        return Ok(());
    }

    let addr = &args[1];
    let filepath = &args[2];

    println!("Connecting to {}...", addr);
    let mut socket = TcpStream::connect(addr)?;
    println!("Connection successful.");

    send_path(&mut socket, filepath)?;

    // Optional: send "end-of-transfer" marker
    socket.write_all(&0u16.to_be_bytes())?;
    println!("All files sent.");

    Ok(())
}

fn send_path(socket: &mut TcpStream, filepath: &str) -> io::Result<()> {
    let path = Path::new(filepath);

    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            send_path(socket, entry.path().to_str().unwrap())?;
        }
        return Ok(());
    }

    send_file(socket, filepath)
}

fn send_file(socket: &mut TcpStream, filepath: &str) -> io::Result<()> {
    let mut file = File::open(filepath)?;
    let filename = Path::new(filepath)
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string();

    println!("Sending file: {}", filename);

    // Send filename length (u16)
    let name_bytes = filename.as_bytes();
    let name_len = name_bytes.len() as u16;
    socket.write_all(&name_len.to_be_bytes())?;

    // Send filename
    socket.write_all(name_bytes)?;

    // Send file size
    let file_size = file.metadata()?.len();
    socket.write_all(&file_size.to_be_bytes())?;

    // Send file contents (exact number of bytes)
    let mut remaining = file_size;
    let mut buffer = [0u8; 8192];

    while remaining > 0 {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break; // EOF
        }

        socket.write_all(&buffer[..n])?;
        remaining -= n as u64;
    }

    println!("Finished sending {}", filename);
    Ok(())
}
