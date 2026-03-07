use bm_syntax::{Highlighter, Theme};
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("usage: bm-syntax highlight --buffer <text>");
        std::process::exit(1);
    }

    match args[1].as_str() {
        "highlight" => {
            let buffer = get_arg(&args, "--buffer").unwrap_or_default();
            if buffer.is_empty() {
                return;
            }

            let theme = match std::env::var("BM_SYNTAX_THEME") {
                Ok(path) => Theme::load(&PathBuf::from(path)).unwrap_or_default(),
                Err(_) => Theme::default(),
            };

            let mut highlighter = Highlighter::new(theme);
            print!("{}", highlighter.highlight_for_zsh(&buffer));
        }
        _ => {
            eprintln!("unknown command: {}", args[1]);
            std::process::exit(1);
        }
    }
}

fn get_arg(args: &[String], flag: &str) -> Option<String> {
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1))
        .cloned()
}
