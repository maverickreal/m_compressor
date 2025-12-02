use std::io::Read;

mod constants;
mod m_compressor;
mod utils;

fn main() {
    let mut file_path: String = String::new();

    match std::io::stdin().read_to_string(&mut file_path) {
        Ok(_) => {
            let trimmed_path = String::from(file_path.trim());
            let m_comp = m_compressor::MCompressor::new(&trimmed_path);

            if m_comp.compress().is_ok() {
                println!(
                    "The compressed file will be at: {:?}.",
                    m_comp.get_out_file_path()
                );
            } else {
                println!("Compression failed.");
            }
        }
        Err(err) => {
            println!("Error reading input: {err}");
        }
    }
}
