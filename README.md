# TodoDB

A terminal-based todo application built with Rust, featuring hierarchical task organization, SQLite backend, and integrated markdown editing with clickable links.

## Demo

![Demo](https://github.com/bac2qh/tododb/raw/main/demo.gif)

## Features

- **Terminal UI**: Clean, interactive interface using ratatui
- **Tree Structure**: Hierarchical todo organization with move functionality
- **Glow Integration**: Rich markdown editing with clickable hyperlinks
- **SQLite Database**: Persistent storage with WAL mode
- **Search**: Real-time search with regex support
- **Color Themes**: Catppuccin Frappe color scheme

## Quick Start

### Prerequisites
- Rust (latest stable)
- **glow** - `brew install glow` or [install guide](https://github.com/charmbracelet/glow#installation)

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
- **Enter**: View/edit todo in glow markdown editor

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

## Markdown Integration

TodoDB integrates with [glow](https://github.com/charmbracelet/glow) for rich markdown editing:

- Press **Enter** on any todo to open in glow editor
- Full markdown support with clickable links
- Changes automatically sync back to database
- Files saved in `markdowns/` directory as `{id}_{title}.md`

## Demo Mode

Create sample data for testing (uses separate `demo_todos.db`):
```bash
tododb --demo
```

Includes hierarchical projects, markdown content, and mixed completion status.

## Architecture

- **SQLite** with WAL mode for performance and safety
- **Tree structure** for hierarchical organization
- **Markdown files** for rich content editing
- **Terminal UI** with ratatui and crossterm

## Contributing

1. Fork the repository
2. Create a feature branch
3. Submit a pull request

All pull requests require approval before merging to main.

## License

MIT License - see [LICENSE](LICENSE) file for details.