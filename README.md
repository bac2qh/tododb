# TodoDB

A terminal-based todo application built with Rust, featuring hierarchical task organization, SQLite backend, and $EDITOR integration for seamless markdown editing.

## Demo

![Demo](https://github.com/bac2qh/tododb/raw/main/demo.gif)

## Features

- **Terminal UI**: Clean, interactive interface using ratatui
- **Tree Structure**: Hierarchical todo organization with move functionality
- **$EDITOR Integration**: Rich markdown editing with your preferred editor
- **SQLite Database**: Persistent storage with WAL mode
- **Search**: Real-time search with regex support
- **Color Themes**: Catppuccin Frappe color scheme

## Quick Start

### Prerequisites
- **Rust** (latest stable)
- **Text Editor**: We recommend [Helix](https://helix-editor.com/) for the best experience:
  ```bash
  # Install Helix
  brew install helix  # macOS
  # or follow: https://docs.helix-editor.com/install.html
  
  # Set as default editor
  export EDITOR=hx
  ```
  
  **Why Helix?** When viewing markdown todos, use `gf` (goto file) to open URLs directly in your browser!
  
  Other editors work too - TodoDB uses `$EDITOR` → `$VISUAL` → `vim` → `nano` → `vi`

### Installation
```bash
git clone https://github.com/bac2qh/tododb.git
cd tododb
cargo install --path .
```

### Usage
```bash
tododb                    # Run the app
tododb --demo            # Try with demo data (separate DB)
tododb --test            # Run functionality tests
```

Database location: `~/.local/share/tododb/todos.db`

## Key Bindings

### Navigation & Selection
- **j/k** or **↑/↓**: Navigate todos
- **h/l** or **←/→**: Navigate hierarchy levels
- **Enter**: View/edit todo in your $EDITOR

### Todo Management
- **n**: Create new todo
- **m**: Move todo (tree view only) - select new parent with j/k, Enter to confirm
- **Space**: Toggle completion status
- **d**: Delete selected todo
- **c**: Show/hide completed todos

### Tree & Search
- **t**: Expand/collapse tree nodes
- **f**: Search all todos (flat view)
- **/**: Search in tree view (live highlighting)

### Other
- **q**: Quit application
- **Esc**: Cancel current operation

## Move Functionality

Press **m** on any todo in tree view to reorganize your tasks:
- **Current parent automatically highlighted**
- **Green highlighting** shows valid parent candidates
- **Yellow highlighting** shows the todo being moved
- **j/k** to navigate between valid parents
- **Enter** to confirm move, **Esc** to cancel
- **Prevents circular dependencies** automatically

## Editor Integration

TodoDB seamlessly integrates with your preferred text editor:

- Press **Enter** on any todo to open in your `$EDITOR`
- Full markdown support with syntax highlighting
- Changes automatically sync back to database when you save and exit
- Temporary files created in `markdowns/` directory as `{id}_{title}.md`
- **Pro tip**: Use Helix editor and press `gf` on URLs to open them in your browser!

**Note**: Only title and description can be edited through markdown. Metadata like completion status, parent relationships, and creation dates must be managed through the TUI interface.

## Demo Mode

Create sample data for testing (uses separate `demo_todos.db`):
```bash
tododb --demo
```

Includes hierarchical projects, markdown content, and mixed completion status.

## Development

**Built with:**
- **Rust** + **ratatui** for the terminal interface
- **SQLite** with WAL mode for data storage
- **Crossterm** for terminal handling
- Developed and tested using **Ghostty** terminal emulator

**Architecture:**
- **SQLite** with WAL mode for performance and safety
- **Tree structure** for hierarchical organization  
- **$EDITOR integration** for seamless markdown editing
- **Terminal UI** with proper suspend/resume handling

## Contributing

1. Fork the repository
2. Create a feature branch
3. Submit a pull request

All pull requests require approval before merging to main.

## License

MIT License - see [LICENSE](LICENSE) file for details.