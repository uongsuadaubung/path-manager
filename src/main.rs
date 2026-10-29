#[macro_use]
mod compat;
mod language;
mod logic;
mod tui;

use compat::anyhow::Result;
use language::text::*;
use logic::PathScope;

fn print_help() {
    println!(
        r#"PATH Manager (path-mgr) v{version}
Windows Environment PATH Editor (TUI & CLI)

USAGE:
    path-mgr [COMMAND] [OPTIONS]

WITHOUT COMMAND:
    Launches the interactive Terminal User Interface (TUI).

COMMANDS:
    list                     List all paths in current scope
    add <PATH>               Add a new path entry
    remove <INDEX>           Remove a path entry by its 1-based index
    set <INDEX> <PATH>       Update an existing path entry
    dedupe                   Remove duplicate and shadowed entries
    backup [-f FILE]         Backup current PATH list to file (default: path_backup.txt)
    restore [-f FILE] [-o]   Restore or merge PATH list from file (-o to overwrite)
    help, --help, -h         Print this help message
    --version, -v            Print version information

OPTIONS:
    --system, -s             Operate on System (Machine) PATH instead of User PATH
"#,
        version = env!("CARGO_PKG_VERSION")
    );
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    // If no arguments passed, launch the interactive TUI
    if args.len() <= 1 {
        return tui::run();
    }

    // Determine scope
    let has_system_flag = args.iter().any(|a| a == "--system" || a == "-s");
    let scope = if has_system_flag {
        PathScope::System
    } else {
        PathScope::User
    };

    // Filter out flags to get the positional command and arguments
    let mut positional_args = Vec::new();
    let mut file_override = false;
    let mut file_path = "path_backup.txt".to_string();

    let mut i = 1;
    while i < args.len() {
        let arg = &args[i];
        if arg == "--system" || arg == "-s" {
            i += 1;
        } else if arg == "-o" || arg == "--override" {
            file_override = true;
            i += 1;
        } else if arg == "-f" || arg == "--file" {
            if i + 1 < args.len() {
                file_path = args[i + 1].clone();
                i += 2;
            } else {
                i += 1;
            }
        } else if arg.starts_with("-f=") {
            file_path = arg[3..].to_string();
            i += 1;
        } else if arg.starts_with("--file=") {
            file_path = arg[7..].to_string();
            i += 1;
        } else {
            positional_args.push(arg.clone());
            i += 1;
        }
    }

    if positional_args.is_empty() {
        return tui::run();
    }

    let cmd = positional_args[0].to_lowercase();
    match cmd.as_str() {
        "list" => {
            let paths = logic::read_current_paths(scope)?;
            println!("\n{} ({} PATH):\n", CLI_CURRENT_PATHS, scope.name());
            for (idx, p) in paths.iter().enumerate() {
                println!("{:>2}. {}", idx + 1, p);
            }
            println!("\n{}{}", CLI_TOTAL_PATHS, paths.len());
        }
        "add" => {
            if positional_args.len() < 2 {
                eprintln!("Error: Missing path argument. Usage: path-mgr add <PATH>");
                std::process::exit(1);
            }
            let new_path = positional_args[1].clone();
            match logic::add_path(scope, new_path.clone()) {
                Ok(true) => println!("Added successfully: {}", new_path),
                Ok(false) => println!("{}", CLI_PATH_EXISTS),
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "remove" => {
            if positional_args.len() < 2 {
                eprintln!("Error: Missing index argument. Usage: path-mgr remove <INDEX>");
                std::process::exit(1);
            }
            let index: usize = positional_args[1].parse().map_err(|_| {
                compat::anyhow::anyhow!("Invalid index: '{}'", positional_args[1])
            })?;
            logic::remove_path(scope, index)?;
            println!("Successfully removed path at index #{}", index);
        }
        "set" => {
            if positional_args.len() < 3 {
                eprintln!("Error: Missing arguments. Usage: path-mgr set <INDEX> <NEW_PATH>");
                std::process::exit(1);
            }
            let index: usize = positional_args[1].parse().map_err(|_| {
                compat::anyhow::anyhow!("Invalid index: '{}'", positional_args[1])
            })?;
            let new_path = positional_args[2].clone();
            let old = logic::set_path(scope, index, new_path.clone())?;
            println!("{}", CLI_UPDATED_SUCCESS);
            println!("{}{}", CLI_FROM, old);
            println!("{}{}", CLI_TO, new_path);
        }
        "dedupe" => {
            let count = logic::dedupe_paths(scope)?;
            println!("Cleaned {} duplicate/redundant path(s).", count);
        }
        "backup" => {
            let paths = logic::read_current_paths(scope)?;
            let content = paths.join("\n");
            std::fs::write(&file_path, content)?;
            println!("Backup saved successfully to: {}", file_path);
        }
        "restore" => {
            let content = std::fs::read_to_string(&file_path)?;
            let imported: Vec<String> = content
                .lines()
                .filter(|s| !s.trim().is_empty())
                .map(|s| s.trim().to_string())
                .collect();

            if file_override {
                logic::write_paths(scope, imported)?;
                println!("Successfully restored and replaced PATH from: {}", file_path);
            } else {
                let count = logic::merge_paths(scope, imported)?;
                println!("Merged {} new path(s) from: {}", count, file_path);
            }
        }
        "help" | "--help" | "-h" => {
            print_help();
        }
        "--version" | "-v" => {
            println!("path-mgr v{}", env!("CARGO_PKG_VERSION"));
        }
        other => {
            eprintln!("Unknown command: '{}'. Run 'path-mgr --help' for usage.", other);
            std::process::exit(1);
        }
    }

    Ok(())
}
