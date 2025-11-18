use std::fs::File;
use std::io::{self, Read};
use std::net::TcpListener;
use indicatif::{ProgressBar, ProgressStyle};

fn main() -> io::Result<()> {
    // Start the server on port 9000
    let listener = TcpListener::bind("0.0.0.0:9000")?;
    println!("Server listening on port 9000...");

    loop {
        let (mut socket, addr) = listener.accept()?;
        println!("Connection from {}", addr);

        // First read filename length
        let mut name_len_buf = [0u8; 2];
        socket.read_exact(&mut name_len_buf)?;
        let name_len = u16::from_be_bytes(name_len_buf);

        // Read filename
        let mut name_buf = vec![0u8; name_len as usize];
        socket.read_exact(&mut name_buf)?;
        let file = String::from_utf8_lossy(&name_buf);
        let filename = file.split('/').last().unwrap();
        println!("Receiving file: {}", filename);

        // Read file size
        let mut size_buf = [0u8; 8];
        socket.read_exact(&mut size_buf)?;
        let file_size = u64::from_be_bytes(size_buf);

        // Create output file
        const OUTPUT_DIR: &str = "./received_files/";
        let mut outfile = File::create(OUTPUT_DIR.to_string() + &filename.to_string())?;

        // Setup progress bar
        let bar = ProgressBar::new(file_size);
        let style = ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .expect("invalid progress template")
            .progress_chars("#>-");
        bar.set_style(style);
        let mut writer = bar.wrap_write(&mut outfile);

        // Stream the data directly
        let bytes_copied = io::copy(&mut socket, &mut writer)?;
        println!("Done. Received {} bytes.", bytes_copied);

        println!("Server listening on port 9000...");

    }
    // Order of data received:
    // File name length bytes
    // File name bytes
    // File size bytes
    // File content bytes
}
