# TodoDB

A terminal-based todo application built with Rust, featuring a tree-structured task organization, SQLite database backend, and integrated markdown editing with clickable links.

## Demo

<video width="600" controls>
  <source src="https://github.com/bac2qh/tododb/raw/main/demo.mov" type="video/mp4">
</video>

## Features

- **Terminal UI**: Clean, interactive interface using ratatui
- **Glow Integration**: View and edit todos in full markdown with clickable hyperlinks
- **Tree Structure**: Organize todos in a hierarchical tree format  
- **SQLite Database**: Persistent storage with WAL mode for performance
- **Search Functionality**: Real-time search with regex support
- **Color Themes**: Catppuccin Frappe color scheme
- **Persistent Files**: Markdown files saved in `markdowns/` directory
- **Test Mode**: Built-in functionality testing

## Installation

### Prerequisites

- Rust (latest stable version)
- Cargo  
- **glow** (for markdown viewing) - install via `brew install glow` or see [glow installation](https://github.com/charmbracelet/glow#installation)

### Install from Source

```bash
git clone https://github.com/bac2qh/tododb.git
cd tododb
cargo install --path .
```

This installs `tododb` to your system PATH, making it available from anywhere.

### Build from Source (Development)

```bash
git clone https://github.com/bac2qh/tododb.git
cd tododb  
cargo build --release
```

## Usage

### Basic Usage

```bash
# Run TodoDB (after installation)
tododb

# Or run with cargo (development)
cargo run
```

The database is automatically created at `~/.local/share/tododb/todos.db`.

### Key Bindings

- **Enter**: View/edit todo in glow (full markdown with clickable links)
- **Space**: Toggle todo completion status
- **n**: Create new todo
- **c**: Show/hide completed todos
- **d**: Delete selected todo
- **f**: Search all todos (flat view)
- **/**: Search in tree view
- **t**: Expand/collapse tree nodes
- **j/k** or **↑/↓**: Navigate todos
- **h/l** or **←/→**: Navigate hierarchy levels
- **q**: Quit application

### Markdown Support

TodoDB integrates with **glow** to provide rich markdown viewing and editing:

- **Clickable hyperlinks**: Links in todo descriptions are fully clickable
- **Rich formatting**: Full markdown rendering with syntax highlighting
- **Edit in place**: Changes made in glow automatically sync back to database
- **Persistent files**: Markdown files are saved in `markdowns/` directory as `{id}_{title}.md`

### Demo Mode

To quickly populate your database with realistic sample data for demonstration purposes:

```bash
# Create comprehensive demo data
tododb --demo

# Or with cargo (development)
cargo run -- --demo
```

This creates a rich dataset including:
- **3 Major Projects:** Web development, mobile app, and DevOps infrastructure
- **Hierarchical Task Structure:** Projects with subtasks and detailed descriptions
- **Markdown Content:** Rich formatting with links, code blocks, and documentation
- **Mixed Completion Status:** Both completed and pending tasks to showcase all features
- **Personal Development:** Learning goals, health routines, and financial planning
- **Professional Content:** Real-world scenarios for software development workflows

Perfect for:
- **Screen recordings** and app demonstrations
- **Feature showcasing** during presentations  
- **Testing** the full application workflow
- **Learning** how to structure complex projects

### Test Modes

```bash
# Run functionality tests
tododb --test

# Run tree functionality tests
tododb --tree-test

# Or with cargo (development)
cargo run -- --test
cargo run -- --tree-test
```

## Dependencies

- **rusqlite**: SQLite database interface with bundled SQLite and chrono support
- **ratatui**: Terminal user interface library
- **crossterm**: Cross-platform terminal manipulation
- **chrono**: Date and time handling
- **anyhow**: Error handling
- **regex**: Regular expression support
- **tokio**: Async runtime (full features)
- **serde**: Serialization framework

## Architecture

The application is structured into several modules:

- `main.rs`: Entry point and UI initialization
- `database.rs`: SQLite database operations and schema management
- `ui.rs`: Terminal user interface implementation with glow integration
- `tree.rs`: Tree data structure for hierarchical todo organization
- `markdown.rs`: Markdown rendering and syntax highlighting
- `colors.rs`: Color scheme definitions (Catppuccin Frappe)
- `test.rs`: Functionality testing suite
- `tree_test.rs`: Tree-specific testing

## Database

The application uses SQLite with WAL (Write-Ahead Logging) mode for:
- Better performance for concurrent operations
- Crash safety  
- Reduced locking contention

Database location: `~/.local/share/tododb/todos.db`

## Glow Integration

TodoDB integrates seamlessly with [glow](https://github.com/charmbracelet/glow) for enhanced markdown viewing and editing:

### How it Works
1. Press **Enter** on any todo to open it in glow
2. TodoDB suspends its interface and launches glow in TUI mode
3. Edit your todo content with full markdown support and clickable links
4. Exit glow (press `q`) to return to TodoDB
5. Changes are automatically synchronized back to the database

### Markdown Files
- Stored in `markdowns/` directory with format: `{id}_{title}.md`
- Contains structured sections: Title, Description, and Metadata
- Persistent files allow external editing and version control
- Bidirectional sync ensures database stays updated
- **Cleanup**: Remove old files with `rm markdowns/*.md` when needed (keep the folder)

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

All pull requests require approval before merging to main.

## License

This project is open source. Please check the repository for license details.