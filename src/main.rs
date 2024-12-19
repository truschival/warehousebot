use rustyline::config::Configurer;
use rustyline::error::ReadlineError;
use rustyline::{Cmd, DefaultEditor, KeyEvent};
use simple_logger::SimpleLogger;

use warehousebot::botcommands;
use warehousebot::botcommands::Command;

fn main() -> rustyline::Result<()> {
    SimpleLogger::new()
        .without_timestamps()
        .with_module_level("rustyline", log::LevelFilter::Error)
        .init()
        .expect("Logger not initialized!");

    let mut rl = DefaultEditor::new()?;
    rl.set_edit_mode(rustyline::EditMode::Emacs);
    rl.bind_sequence(KeyEvent::ctrl('s'), Cmd::HistorySearchForward);
    rl.bind_sequence(KeyEvent::ctrl('r'), Cmd::HistorySearchBackward);
    #[cfg(feature = "with-file-history")]
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                if line.is_empty() {
                    continue;
                }

                rl.add_history_entry(line.as_str())?;
                println!("Line: {}", line);

                match botcommands::command_for_string(&line) {
                    Ok(cmd) => {
                        println!("ok, executing {:?}", cmd.info());
                        cmd.execute()
                    }
                    Err(e) => {
                        println!("Bad Error {:?}", e);
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
