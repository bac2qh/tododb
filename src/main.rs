mod database;
mod ui;
mod test;
mod tree;
mod tree_test;
mod colors;
mod demo_data;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use database::Database;
use demo_data::DemoDataGenerator;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{env, io, path::PathBuf};
use ui::App;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    
    
    // Check for test mode
    if args.len() > 1 && args[1] == "--test" {
        return test::test_functionality();
    }
    
    // Check for tree test mode
    if args.len() > 1 && args[1] == "--tree-test" {
        return tree_test::test_tree_functionality();
    }
    
    // Check for demo mode - handle both "--demo" and "<db_path> --demo"
    let has_demo_flag = (args.len() > 1 && args[1] == "--demo") || 
                        (args.len() > 2 && args[2] == "--demo");
    
    if has_demo_flag {
        let demo_db_path = get_demo_db_path()?;
        let database = Database::new(&demo_db_path)?;
        let generator = DemoDataGenerator::new(database);
        return generator.populate_demo_data();
    }
    
    let db_path = get_db_path(&args)?;

    let database = Database::new(&db_path)?;
    
    // Try to initialize terminal UI, fallback to test mode if it fails
    match try_run_ui(database) {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("Failed to initialize terminal UI: {}", e);
            eprintln!("Running in test mode instead...\n");
            let _database = Database::new(&db_path)?;
            test::test_functionality()
        }
    }
}

fn try_run_ui(database: Database) -> anyhow::Result<()> {
    let mut app = App::new(database)?;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal, &mut app);

    // Ensure data is written to disk before exit
    let _ = app.database.checkpoint_and_close();

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

fn run_app<B: ratatui::backend::Backend + std::io::Write>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> anyhow::Result<()> {
    loop {
        // Check if editor should be launched
        if let Some(todo) = app.editor_pending.take() {
            if let Err(e) = app.launch_editor(&todo, terminal) {
                app.error_message = Some(format!("Editor error: {}", e));
            }
        }
        
        terminal.draw(|f| app.draw(f))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                app.handle_key_event(key.code)?;
                if app.should_quit {
                    break;
                }
            }
        }
    }
    Ok(())
}

fn get_demo_db_path() -> anyhow::Result<String> {
    // Always use demo_todos.db in the current directory for demo mode
    Ok("demo_todos.db".to_string())
}

fn get_db_path(args: &[String]) -> anyhow::Result<String> {
    let db_path = if args.len() > 1 && !args[1].starts_with("--") {
        // Custom database path provided (not a flag)
        args[1].clone()
    } else {
        // Use default database path in ~/.local/share/tododb/
        let mut home_path = PathBuf::from(env::var("HOME").unwrap_or_else(|_| ".".to_string()));
        home_path.push(".local");
        home_path.push("share");
        home_path.push("tododb");
        
        // Create directory if it doesn't exist
        std::fs::create_dir_all(&home_path)?;
        
        home_path.push("todos.db");
        home_path.to_string_lossy().to_string()
    };
    
    Ok(db_path)
}
