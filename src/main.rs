mod indexer;
mod db;
mod llm;

use clap::{Parser, Subcommand};
use walkdir::WalkDir;
use std::fs;
use anyhow::Result;
use rayon::prelude::*;
use indexer::{get_language_for_file, extract_symbols};
use db::IndexDb;
use llm::explain_code;

#[derive(Parser)]
#[command(name = "repomind")]
#[command(about = "Codebase navigator", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Scan {
        #[arg(default_value = ".")]
        path: String,
    },
    Index {
        #[arg(default_value = ".")]
        path: String,
    },
    Find {
        name: String,
    },
    Explain {
        name: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let db = IndexDb::new("repomind.db")?;

    match cli.command {
        Commands::Scan { path } => {
            for entry in WalkDir::new(&path).into_iter().filter_map(|e| e.ok()) {
                if entry.file_type().is_file() {
                    println!("{}", entry.path().display());
                }
            }
        }
        Commands::Index { path } => {
            db.clear_all()?;

            let entries: Vec<_> = WalkDir::new(&path)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
                .collect();

            // Параллельная обработка: каждый поток возвращает Vec своих символов
            let symbol_batches: Vec<Vec<(String, String, String, u32)>> = entries
                .par_iter()
                .map(|entry| {
                    let path = entry.path();
                    let mut batch = Vec::new();
                    if let Some(lang) = get_language_for_file(path) {
                        if let Ok(content) = fs::read_to_string(path) {
                            let symbols = extract_symbols(&content, lang);
                            let path_str = path.to_string_lossy().to_string();
                            for (name, typ, line) in symbols {
                                batch.push((name, typ, path_str.clone(), line as u32));
                            }
                        }
                    }
                    batch
                })
                .collect();

            // Объединяем все векторы и вставляем в БД
            for batch in symbol_batches {
                for (name, typ, file_path, line) in batch {
                    db.insert_symbol(&name, &typ, &file_path, line)?;
                    println!("Indexed: {} {} at {}:{}", typ, name, file_path, line);
                }
            }
            println!("Indexing complete (parallel).");
        }
        Commands::Find { name } => {
            let results = db.find_symbol(&name)?;
            if results.is_empty() {
                println!("Symbol '{}' not found", name);
            } else {
                for (typ, file_path, line) in results {
                    println!("{}: {} at {}:{}", typ, name, file_path, line);
                }
            }
        }
        Commands::Explain { name } => {
            let results = db.find_symbol(&name)?;
            if results.is_empty() {
                println!("Symbol '{}' not found", name);
                return Ok(());
            }
            let (typ, file_path, line) = &results[0];
            let content = fs::read_to_string(file_path)?;
            let lines: Vec<&str> = content.lines().collect();
            let line_idx = (*line as usize).saturating_sub(1);
            if line_idx >= lines.len() {
                println!("Could not extract code snippet (line out of range)");
                return Ok(());
            }
            let start = line_idx.saturating_sub(2);
            let end = (line_idx + 8).min(lines.len());
            let snippet = lines[start..end].join("\n");
            println!("Explaining {} {} from {}:{}", typ, name, file_path, line);
            println!("Code snippet:\n{}\n", snippet);
            let ollama_url = std::env::var("OLLAMA_URL").unwrap_or_else(|_| "http://localhost:11434".to_string());
            let model = std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "llama3.2:1b".to_string());
            match explain_code(&snippet, &model, &ollama_url) {
                Ok(explanation) => println!("Explanation:\n{}", explanation),
                Err(e) => println!("Error calling Ollama: {}\nMake sure Ollama is running (ollama serve) and model is pulled.", e),
            }
        }
    }
    Ok(())
}