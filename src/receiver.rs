use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::net::TcpListener;
use indicatif::{ProgressBar, ProgressStyle};

fn main() -> io::Result<()> {
    // Start the server
    let listener = TcpListener::bind("0.0.0.0:9000")?;
    println!("Server listening on port 9000...");

    loop {
        let (mut socket, addr) = listener.accept()?;
        println!("Connection from {}", addr);

        loop {
            // Read filename length (2 bytes)
            let mut name_len_buf = [0u8; 2];
            if let Err(_) = socket.read_exact(&mut name_len_buf) {
                println!("Client disconnected.");
                break;
            }
            let name_len = u16::from_be_bytes(name_len_buf);

            // End-of-transfer marker
            if name_len == 0 {
                println!("All files received.");
                break;
            }

            // Read filename
            let mut name_buf = vec![0u8; name_len as usize];
            socket.read_exact(&mut name_buf)?;
            let filename = String::from_utf8_lossy(&name_buf).to_string();

            println!("Receiving file: {}", filename);

            // Read file size (8 bytes)
            let mut size_buf = [0u8; 8];
            socket.read_exact(&mut size_buf)?;
            let file_size = u64::from_be_bytes(size_buf);

            // Ensure output directory exists
            fs::create_dir_all("./received_files")?;

            // Create output file
            let mut outfile = File::create(format!("./received_files/{}", filename))?;

            // Setup progress bar
            let bar = ProgressBar::new(file_size);
            bar.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                    .unwrap()
                    .progress_chars("#>-"),
            );
            let mut writer = bar.wrap_write(&mut outfile);

            // Read exactly file_size bytes
            let mut remaining = file_size;
            let mut buffer = [0u8; 8192];

            while remaining > 0 {
                let to_read = remaining.min(buffer.len() as u64) as usize;
                let n = socket.read(&mut buffer[..to_read])?;

                if n == 0 {
                    return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Sender closed early"));
                }

                writer.write_all(&buffer[..n])?;
                remaining -= n as u64;
            }

            writer.flush()?;
            bar.finish_with_message("Done");
            println!("Finished receiving {}", filename);
        }
    }
}
