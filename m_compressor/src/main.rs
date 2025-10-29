use std::io::Read;

mod m_compressor;

fn main() {
    let mut file_path: String = String::new();

    match std::io::stdin().read_to_string(&mut file_path) {
        Ok(_) => {
            let trimmed_path = String::from(file_path.trim());

            if let Ok(m_comp) = m_compressor::MCompressor::new(&trimmed_path) {
                println!(
                    "The compressed file will be at: {:?}.",
                    m_comp.get_out_file_path()
                );
            } else {
                println!("Error: Could not open or process the file path!");
            }
        }
        Err(err) => {
            println!("Error reading input: {}", err);
            return;
        }
    }
}
