# TodoDB

A terminal-based todo application built with Rust, featuring hierarchical task organization, SQLite backend, and $EDITOR integration for seamless markdown editing.

## Demo

![Demo](https://github.com/bac2qh/tododb/raw/main/demo.gif)

## Features

- **Terminal UI**: Clean, interactive interface using ratatui with scrollbars
- **Tree Structure**: Hierarchical todo organization with move functionality
- **Vim-Style Navigation**: Half-page scrolling with Ctrl+d/Ctrl+u, ID-based goto
- **$EDITOR Integration**: Rich markdown editing with your preferred editor
- **SQLite Database**: Persistent storage with WAL mode
- **Advanced Search**: Real-time search with regex support, ID modulo navigation
- **Visual Feedback**: Scrollbars, live highlighting, and Catppuccin Frappe colors
- **Hidden Todo Management**: Toggle visibility and hide individual todos

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
- **j/k** or **↑/↓**: Navigate todos one by one
- **Ctrl+d/Ctrl+u**: Half-page scroll down/up (vim-style)
- **h/l** or **←/→**: Navigate hierarchy levels
- **Enter**: View/edit todo in your $EDITOR

### Todo Management
- **n**: Create new todo
- **m**: Move todo (tree view only) - select new parent with j/k, Enter to confirm
- **Space**: Toggle completion status
- **d**: Delete selected todo
- **c**: Show/hide completed todos
- **h**: Toggle hidden status of selected todo
- **H**: Toggle showing/hiding all hidden todos

### Tree & Search
- **t**: Expand/collapse tree nodes
- **f**: Search all todos (flat view)
- **/**: Search in tree view (live highlighting)
- **g**: Goto ID mode - type digits to jump to todos by ID % 100
- **n/N**: Navigate search/goto matches (next/previous)

### Help & System
- **a**: Show/hide help page
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

## ID Goto Navigation

Quickly jump to todos using ID modulo 100 (press **g** in tree view):
- **Type digits** to find todos whose ID % 100 matches your input
- **Yellow highlighting** shows all matching todos
- **Underlined** indicates the current match
- **n/N** to cycle through multiple matches
- **Perfect for large todo collections** - remember short 2-digit numbers instead of long IDs
- **Real-time filtering** as you type
- **Press Enter** to edit the selected todo, **Esc** to cancel

## Visual Features

**Scrollbars**: Visual position indicators appear automatically on all list views
- **Right-side scrollbars** with up/down arrows (↑/↓)
- **Real-time position tracking** as you navigate
- **Shows your location** in long todo lists
- **Minimal design** that doesn't interfere with content

**Live Search Highlighting**:
- **Tree search (/)**: Live yellow highlighting of matches
- **ID goto (g)**: Yellow highlighting with underlined current match
- **Vim-style n/N navigation** through search results

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
- **Tree structure** for hierarchical organization with move operations
- **$EDITOR integration** for seamless markdown editing
- **Terminal UI** with proper suspend/resume handling
- **Scrollbar widgets** for visual position feedback
- **Modal navigation** with vim-style keybindings
- **Real-time search** with regex and ID-based filtering
- **State management** for complex UI modes and transitions

## Contributing

1. Fork the repository
2. Create a feature branch
3. Submit a pull request

All pull requests require approval before merging to main.

## License

MIT License - see [LICENSE](LICENSE) file for details.