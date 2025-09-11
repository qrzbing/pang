use std::{fs, path::PathBuf};

use anyhow::Result;

use pang_derive::kaitai::convert;

fn main() -> Result<()> {
    // In a real app, you'd use a CLI parsing library like `clap`
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <input.ksy> <output.rs>", args[0]);
        std::process::exit(1);
    }

    let input_path = PathBuf::from(&args[1]);
    let output_path = PathBuf::from(&args[2]);

    let ksy_content = fs::read_to_string(&input_path)?;

    let rust_code = convert(&ksy_content)?;

    fs::write(&output_path, rust_code)?;

    println!(
        "Successfully converted {} to {}",
        input_path.display(),
        output_path.display()
    );

    Ok(())
}
