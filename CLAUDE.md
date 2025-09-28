# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

TodoDB is a terminal-based hierarchical todo application built with Rust. It features a TUI interface using ratatui, SQLite database with WAL mode, and $EDITOR integration for markdown editing.

## Development Commands

### Building and Running
```bash
cargo build                    # Build the project
cargo run                      # Run the application
cargo install --path .         # Install locally
```

### Testing and Development
```bash
cargo run -- --test           # Run functionality tests
cargo run -- --tree-test      # Run tree functionality tests
cargo run -- --demo           # Run with demo data (uses demo_todos.db)
cargo check                    # Quick compile check
cargo clippy                   # Lint with Clippy
```

### Dependencies
Core dependencies in use:
- **ratatui** (0.29): Terminal UI framework
- **crossterm** (0.28): Cross-platform terminal manipulation
- **rusqlite** (0.32): SQLite database interface with chrono support
- **chrono** (0.4): Date/time handling
- **anyhow** (1.0): Error handling
- **regex** (1.0): Pattern matching for search functionality

### Running with Custom Database
```bash
cargo run -- path/to/custom.db    # Use custom database path
```

## Architecture Overview

### Core Components

- **src/main.rs**: Entry point with command-line argument handling and terminal UI initialization
- **src/database.rs**: SQLite database layer with WAL mode, CRUD operations for todos
- **src/ui.rs**: Main UI application state and event handling using ratatui
- **src/tree.rs**: Hierarchical tree management for todo organization and rendering
- **src/markdown.rs**: Markdown rendering with pulldown-cmark, supports syntax highlighting
- **src/colors.rs**: Catppuccin Frappe color theme definitions
- **src/demo_data.rs**: Demo data generation for testing

### Key Data Structures

- **Todo**: Core todo item with id, title, description, timestamps, parent relationship
- **NewTodo**: For creating new todos
- **TreeNode**: Hierarchical tree representation with expansion states
- **App**: Main application state managing UI modes, lists, and database interactions

### Application Modes

The app operates in several modes defined in `AppMode` enum:
- `List`: Normal flat todo list view
- `Tree`: Hierarchical tree view with expand/collapse
- `Edit/Create`: Todo editing/creation forms
- `Search`: Various search modes (ListFind, TreeSearch, ParentSearch)
- `Move`: Todo reorganization mode

### Database Schema

SQLite database with WAL mode enabled:
- Single `todos` table with hierarchical parent_id relationships
- Timestamps using chrono DateTime<Utc>
- Regular checkpointing for data safety

### Editor Integration

- Uses $EDITOR environment variable (fallback chain: $VISUAL → vim → nano → vi)
- Creates temporary markdown files in `markdowns/` directory
- Format: `{id}_{title}.md`
- Automatically syncs changes back to database on editor exit

### Tree Management

The `TodoTreeManager` handles:
- Building tree structure from flat todo list
- Rendering tree with proper indentation and expansion indicators
- Managing expansion states persistently
- Move operations with circular dependency prevention

### Terminal UI Architecture

- Built with ratatui and crossterm
- Proper terminal suspend/resume for editor integration
- Event-driven architecture with keyboard input handling
- Multiple list states for different views (normal, tree, completed)

## Key Bindings

Understanding the UI interactions for testing and development:

### Navigation & Selection
- **j/k** or **↑/↓**: Navigate todos
- **h/l** or **←/→**: Navigate hierarchy levels
- **Enter**: View/edit todo in $EDITOR

### Todo Management
- **n**: Create new todo
- **m**: Move todo (tree view only)
- **Space**: Toggle completion status
- **d**: Delete selected todo
- **c**: Show/hide completed todos

### Modes & Search
- **t**: Expand/collapse tree nodes
- **f**: Search all todos (flat view)
- **/**: Search in tree view (live highlighting)
- **q**: Quit application
- **Esc**: Cancel current operation

## Development Notes

- Database path: Default `~/.local/share/tododb/todos.db`
- Demo mode uses `demo_todos.db` in project root
- WAL mode enabled for concurrent access safety
- Markdown rendering supports tables, strikethrough, task lists, footnotes
- Tree search supports live highlighting with vim-like n/N navigation
- Color scheme: Catppuccin Frappe throughout
- Editor preference: Helix recommended (supports `gf` to open URLs from markdown)

## File Organization

- All Rust source in `src/`
- No external configuration files
- Dependencies managed via `Cargo.toml`
- Demo assets: `demo.gif`, `demo.mp4`