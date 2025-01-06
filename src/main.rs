use colored::Colorize;
use rustyline::config::Configurer;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use simple_logger::SimpleLogger;
use warehousebot::bot::rest::RestBot;
use warehousebot::cli::{Cli, CliError};

fn main() -> rustyline::Result<()> {
    SimpleLogger::new()
        .without_timestamps()
        .with_module_level("rustyline", log::LevelFilter::Error)
        .with_module_level("reqwest", log::LevelFilter::Info)
        .with_module_level("warehousebot::bot::rest", log::LevelFilter::Info)
        .with_module_level("warehousebot::warehouse", log::LevelFilter::Debug)
        .with_module_level("warehousebot", log::LevelFilter::Debug)
        .init()
        .expect("Logger not initialized!");

    let mut rl = DefaultEditor::new()?;
    rl.set_edit_mode(rustyline::EditMode::Emacs);

    #[cfg(feature = "with-file-history")]
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }
    print!("\x1B[2J\x1B[1;1H");
    print!(
        r#"
============= Warehouse CLI ===============
Copyright (c) 1972 - Warehouse Control Ltd.
===========================================
"#
    );

    let executor = RestBot::default();
    let mut cli = Cli::new(executor);

    match cli.init_state() {
        Ok(_) => {}
        Err(CliError::CommandFailed(msg)) => {
            println!("{}{}", "Command failed : ".red(), msg);
        }
        _ => {
            panic!("Initialization failed!")
        }
    }

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                if line.is_empty() {
                    continue;
                }

                match cli.dispatch_command_for_string(&line) {
                    Ok(res) => {
                        println!("{}", res);
                        // only add successful commands to history
                        rl.add_history_entry(line.as_str())?;
                    }
                    Err(CliError::CommandFailed(msg)) => {
                        println!("{}{}", "Command failed : ".red(), msg);
                    }
                    Err(CliError::CommandNotImplemented) => {
                        println!("{}", "Command not yet implemented".yellow());
                    }
                    Err(CliError::CommandUnknown) => {
                        println!("{}", "Unknown command".yellow());
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    #[cfg(feature = "with-file-history")]
    rl.save_history("history.txt")?;
    Ok(())
}
