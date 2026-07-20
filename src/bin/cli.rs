use clap::Parser;
use sailii::config::Config;
use sailii::perform_cracking;

#[derive(Parser)]
#[command(name = "sailii-cli", about = "Automatically decrypt encryptions without knowing the key or cipher")]
struct Cli {
    text: Option<String>,

    #[arg(short = 'f', long = "file")]
    file: Option<String>,

    #[arg(short = 't', long = "timeout", default_value = "10")]
    timeout: u64,

    #[arg(short = 'v', long = "verbose")]
    verbose: bool,

    #[arg(long = "max-depth", default_value = "20")]
    max_depth: usize,

    #[arg(long = "key")]
    key: Option<String>,

    #[arg(long = "key-file")]
    key_file: Option<String>,
}

fn main() {
    let cli = Cli::parse();

    let input = if let Some(text) = cli.text {
        text
    } else if let Some(file) = cli.file {
        std::fs::read_to_string(file).unwrap_or_else(|e| {
            eprintln!("Error reading file: {}", e);
            std::process::exit(1);
        })
    } else {
        eprintln!("Please provide text to decode or use --file");
        std::process::exit(1);
    };

    let mut keys = Vec::new();
    if let Some(k) = cli.key {
        for part in k.split(',') {
            let trimmed = part.trim();
            if !trimmed.is_empty() {
                keys.push(trimmed.to_string());
            }
        }
    }
    if let Some(path) = cli.key_file {
        if let Ok(contents) = std::fs::read_to_string(&path) {
            for line in contents.lines() {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    keys.push(trimmed.to_string());
                }
            }
        }
    }

    let config = Config {
        timeout_secs: cli.timeout,
        verbose: cli.verbose,
        max_depth: cli.max_depth,
        keys,
        ..Default::default()
    };

    println!("[sailii] Analyzing: {}", &input[..input.len().min(80)]);
    if input.len() > 80 {
        println!("[sailii] ... ({} total chars)", input.len());
    }

    match perform_cracking(&input, config) {
        Some(result) if result.success => {
            println!("\n[+] Decoded successfully!");
            println!("    Decoder: {}", result.decoder);
            if let Some(key) = &result.key {
                println!("    Key: {}", key);
            }
            if let Some(texts) = &result.unencrypted_text {
                for text in texts {
                    println!("\n    Plaintext: {}", text);
                }
            }
        }
        _ => {
            println!("\n[-] Could not decode the input within the time limit.");
            println!("    Try increasing the timeout with --timeout <seconds>");
        }
    }
}
