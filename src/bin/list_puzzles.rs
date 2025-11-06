use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Deserialize, Serialize)]
struct Puzzle {
    address: String,
    amount: String,
    hash: String,
    hint: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    secret: Option<String>,
}

fn main() -> Result<()> {
    println!("🎯 PUZZLES DISPONÍVEIS");
    println!("=======================");
    println!();

    let mut puzzles_found = 0;

    // Lista todos os arquivos no diretório atual
    let entries = fs::read_dir(".")
        .context("Erro ao ler diretório atual")?;

    // Filtra e processa apenas arquivos puzzle_*.json
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let filename = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        if filename.starts_with("puzzle_") && filename.ends_with(".json") {
            puzzles_found += 1;

            // Lê o conteúdo do arquivo
            let content = fs::read_to_string(&path)
                .context(format!("Erro ao ler arquivo: {:?}", path))?;

            // Parseia o JSON
            let puzzle: Puzzle = serde_json::from_str(&content)
                .context(format!("Erro ao parsear JSON de: {:?}", path))?;

            // Exibe informações do puzzle
            println!("📦 Puzzle #{}", puzzles_found);
            println!("   Arquivo: {}", filename);
            println!("   📍 Endereço: {}", puzzle.address);
            println!("   💰 Prêmio: {} L-BTC", puzzle.amount);
            println!("   🔐 Hash: {}", puzzle.hash);
            println!("   💡 Dica: {}", puzzle.hint);

            // Se o secret estiver visível (para debug/desenvolvimento)
            if let Some(secret) = &puzzle.secret {
                println!("   ⚠️  Secret (DEV ONLY): {}", secret);
            }

            println!();
        }
    }

    if puzzles_found == 0 {
        println!("❌ Nenhum puzzle encontrado.");
        println!();
        println!("💡 Para criar um puzzle, use:");
        println!("   cargo run --bin create-puzzle -- \"senha\" 0.1");
    } else {
        println!("========================");
        println!("📊 Total de puzzles: {}", puzzles_found);
    }

    Ok(())
}