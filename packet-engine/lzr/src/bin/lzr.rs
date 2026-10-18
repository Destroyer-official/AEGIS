use std::env;
use std::process;

use aegis_lzr::signature_store::{SemVer, SignatureStore};

fn main() {
    // Skip "signatures" subcommand if present (lzr signatures <cmd>)
    let args: Vec<String> = {
        let raw: Vec<String> = env::args().skip(1).collect();
        if raw.first().is_some_and(|a| a == "signatures") {
            raw.into_iter().skip(1).collect()
        } else {
            raw
        }
    };

    if args.is_empty() {
        eprintln!("Usage: lzr signatures <command> [args]");
        eprintln!("Commands: init, push, pull, rollback, list, current, export");
        process::exit(1);
    }

    let store_path = env::var("LZR_SIGNATURES_DIR").unwrap_or_else(|_| {
        let home = env::var("USERPROFILE")
            .or_else(|_| env::var("HOME"))
            .unwrap_or_default();
        format!("{}/.lzr/signatures", home)
    });

    let store = SignatureStore::new(&store_path);

    let result = match args[0].as_str() {
        "init" => cmd_init(&store),
        "push" => cmd_push(&store, &args[1..]),
        "pull" => cmd_pull(&store),
        "rollback" => cmd_rollback(&store, &args[1..]),
        "list" | "ls" => cmd_list(&store),
        "current" => cmd_current(&store),
        "export" => cmd_export(&store),
        "path" => {
            println!("{}", store.path().display());
            Ok(())
        }
        "git-available" => {
            if aegis_lzr::signature_store::is_git_available() {
                println!("yes");
                Ok(())
            } else {
                println!("no");
                process::exit(1);
            }
        }
        other => {
            eprintln!("Unknown command: {}", other);
            eprintln!(
                "Commands: init, push, pull, rollback, list, current, export, path, git-available"
            );
            process::exit(1);
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn cmd_init(store: &SignatureStore) -> Result<(), Box<dyn std::error::Error>> {
    store.init()?;
    println!("Initialized signature store at {}", store.path().display());
    Ok(())
}

fn cmd_push(store: &SignatureStore, args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let mut version = None;
    let mut description = String::new();
    let mut author = String::new();
    let mut csv_path = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--version" | "-v" => {
                i += 1;
                version = Some(SemVer::parse(&args[i])?);
            }
            "--desc" | "-d" | "--description" => {
                i += 1;
                description = args[i].clone();
            }
            "--author" | "-a" => {
                i += 1;
                author = args[i].clone();
            }
            "--file" | "-f" => {
                i += 1;
                csv_path = Some(args[i].clone());
            }
            other => {
                csv_path = Some(other.to_string());
            }
        }
        i += 1;
    }

    let version = version.ok_or("Usage: lzr signatures push --version v1.0.0 [--desc ...] [--author ...] [--file signatures.csv]")?;
    if author.is_empty() {
        author = env::var("USER").unwrap_or_else(|_| "unknown".to_string());
    }
    if description.is_empty() {
        description = format!("Push {}", version);
    }

    let csv_data = if let Some(path) = csv_path {
        std::fs::read_to_string(&path)?
    } else {
        // Try stdin
        let mut buf = String::new();
        std::io::Read::read_to_string(&mut std::io::stdin(), &mut buf)?;
        buf
    };

    let ver = store.push(&csv_data, version, &description, &author)?;
    println!(
        "Pushed {} — {} signatures, commit {}",
        ver.version,
        ver.signature_count,
        &ver.commit_hash[..8.min(ver.commit_hash.len())]
    );
    Ok(())
}

fn cmd_pull(store: &SignatureStore) -> Result<(), Box<dyn std::error::Error>> {
    let csv = store.pull()?;
    let lines = csv.lines().count();
    println!(
        "Pulled signatures ({} lines) from {}",
        lines,
        store.path().display()
    );
    Ok(())
}

fn cmd_rollback(store: &SignatureStore, args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.is_empty() {
        eprintln!("Usage: lzr signatures rollback <version>");
        eprintln!("Use `lzr signatures list` to see available versions.");
        process::exit(1);
    }
    let version = SemVer::parse(&args[0])?;
    let csv = store.rollback(&version)?;
    let lines = csv.lines().count();
    println!("Rolled back to {} ({} lines)", version, lines);
    Ok(())
}

fn cmd_list(store: &SignatureStore) -> Result<(), Box<dyn std::error::Error>> {
    let versions = store.list()?;
    if versions.is_empty() {
        println!("No versions found. Use `lzr signatures push` to create one.");
        return Ok(());
    }
    println!(
        "{:<12} {:<20} {:<10} DESCRIPTION",
        "VERSION", "TIMESTAMP", "SIGS"
    );
    println!("{}", "-".repeat(80));
    for v in &versions {
        println!(
            "{:<12} {:<20} {:<10} {}",
            v.version, v.timestamp, v.signature_count, v.description
        );
    }
    Ok(())
}

fn cmd_current(store: &SignatureStore) -> Result<(), Box<dyn std::error::Error>> {
    let current = store.current()?;
    println!("Version:     {}", current.version);
    println!("Timestamp:   {}", current.timestamp);
    println!("Author:      {}", current.author);
    println!("Description: {}", current.description);
    println!(
        "Commit:      {}",
        &current.commit_hash[..8.min(current.commit_hash.len())]
    );
    println!("Signatures:  {}", current.signature_count);
    Ok(())
}

fn cmd_export(store: &SignatureStore) -> Result<(), Box<dyn std::error::Error>> {
    let csv = store.load_signatures()?;
    print!("{}", csv);
    Ok(())
}
