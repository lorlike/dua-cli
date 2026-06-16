use anyhow::Result;
use clap::Parser;
use crossterm::event::{self, KeyCode};
use dua_cli::{App, Args, Command, aggregate, output_with_color};
use std::path::PathBuf;

fn main() -> Result<()> {
    let args = Args::parse();

    // Handle subcommands
    if let Some(command) = &args.command {
        match command {
            Command::Aggregate { input } => {
                run_aggregate(input.clone())?;
            }
            Command::Interactive { input } => {
                run_interactive(input.clone())?;
            }
            Command::Completions => {
                println!("Shell completions not yet implemented");
            }
            Command::Config => {
                println!("Config not yet implemented");
            }
        }
        return Ok(());
    }

    // Default behavior: if input paths provided, use them; otherwise use current directory
    let paths = if args.input.is_empty() {
        vec![PathBuf::from(".")]
    } else {
        args.input.clone()
    };

    run_aggregate(paths)
}

fn run_aggregate(paths: Vec<PathBuf>) -> Result<()> {
    // If paths are directories, show their contents
    let mut items_to_aggregate = Vec::new();
    for path in paths {
        if path.is_dir() {
            let entries = dua_cli::get_path_list(&path.to_string_lossy())?;
            items_to_aggregate.extend(entries);
        } else {
            items_to_aggregate.push(path);
        }
    }

    let items = aggregate(items_to_aggregate)?;
    output_with_color(items);
    Ok(())
}

fn run_interactive(paths: Vec<PathBuf>) -> Result<()> {
    let mut app = App::new(paths)?;

    ratatui::run(|terminal| -> Result<()> {
        loop {
            terminal.draw(|frame| app.render(frame))?;
            if let Some(key) = event::read()?.as_key_press_event() {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    KeyCode::Char('j') | KeyCode::Down => app.table_state.select_next(),
                    KeyCode::Char('k') | KeyCode::Up => app.table_state.select_previous(),
                    KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => {
                        app.navigate_into()?;
                    }
                    KeyCode::Char('h') | KeyCode::Left | KeyCode::Backspace => {
                        app.navigate_back()?;
                    }
                    KeyCode::Char(' ') => app.toggle_mark(),
                    KeyCode::Char('g') => app.table_state.select_first(),
                    KeyCode::Char('G') => app.table_state.select_last(),
                    _ => {}
                }
            }
        }
    })?;

    Ok(())
}
