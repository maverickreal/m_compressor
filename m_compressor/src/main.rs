use std::io::Read;

mod m_compressor;

fn main() {
    let mut file_path: String = String::new();

    match std::io::stdin().read_to_string(&mut file_path) {
        Ok(_) => {
            if let Ok(m_comp) = m_compressor::MCompressor::new(file_path) {
                println!(
                    "The compressed file is at: {}.",
                    m_comp.get_compressed_file_path()
                );
            } else {
                print!("There was an error compressing the file!");
            }
        }
        Err(err) => {
            println!("Error reading input: {}", err);
            return;
        }
    }
}
