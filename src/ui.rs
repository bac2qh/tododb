use crate::database::{Database, NewTodo, Todo};
use crate::tree::TodoTreeManager;
use crate::colors::CatppuccinFrappe;
use chrono::Local;
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap,
    },
    Frame,
};

#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    List,
    CompletedView,
    View,
    Edit,
    Create,
    ConfirmDelete,
    ListFind,
    TreeSearch,
    ParentSearch,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ListViewType {
    Normal,
    Tree,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CreateFieldFocus {
    Title,
    Parent,
    Description,
}

pub struct App {
    pub database: Database,
    pub incomplete_todos: Vec<Todo>,
    pub completed_todos: Vec<Todo>,
    pub tree_manager: TodoTreeManager,
    pub list_state: ListState,
    pub tree_list_state: ListState,
    pub completed_list_state: ListState,
    pub mode: AppMode,
    pub input: String,
    pub input_title: String,
    pub input_description: String,
    pub editing_id: Option<i64>,
    pub current_parent: Option<i64>,
    pub should_quit: bool,
    pub error_message: Option<String>,
    pub search_query: String,
    pub search_results: Vec<Todo>,
    pub search_list_state: ListState,
    pub search_matches: Vec<i64>,
    pub input_parent: String,
    pub selected_parent_id: Option<i64>,
    pub create_field_focus: CreateFieldFocus,
    pub use_tree_view: bool,
    pub glow_pending: Option<Todo>,
}

impl App {
    fn create_markdown_file(&self, todo: &Todo) -> Result<std::path::PathBuf, String> {
        use std::fs;
        use std::path::Path;
        
        // Create markdowns directory if it doesn't exist
        let markdowns_dir = Path::new("markdowns");
        if !markdowns_dir.exists() {
            fs::create_dir_all(markdowns_dir)
                .map_err(|e| format!("Failed to create markdowns directory: {}", e))?;
        }
        
        // Create sanitized filename from id and title
        let sanitized_title = todo.title
            .chars()
            .map(|c| if c.is_alphanumeric() || c == ' ' || c == '-' || c == '_' { c } else { '_' })
            .collect::<String>()
            .replace(' ', "_")
            .chars()
            .take(50) // Limit length
            .collect::<String>();
        
        let filename = format!("{}_{}.md", todo.id, sanitized_title);
        let file_path = markdowns_dir.join(&filename);
        
        // Format todo as markdown
        let markdown_content = format!(
            "# {}\n\n## Description\n{}\n\n## Metadata\n- **ID:** {}\n- **Status:** {}\n- **Created:** {}\n", 
            todo.title,
            if todo.description.trim().is_empty() { "(No description)" } else { &todo.description },
            todo.id,
            if todo.is_completed() { "✓ Completed" } else { "○ Incomplete" },
            todo.created_at.format("%Y-%m-%d %H:%M:%S")
        );
        
        // Write markdown file
        fs::write(&file_path, &markdown_content)
            .map_err(|e| format!("Failed to write markdown file: {}", e))?;
        
        Ok(file_path)
    }
    
    pub fn launch_glow_editor<B: ratatui::backend::Backend + std::io::Write>(&mut self, todo: &Todo, terminal: &mut ratatui::Terminal<B>) -> Result<(), String> {
        use std::process::Command;
        use crossterm::{
            execute,
            terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
            event::{DisableMouseCapture, EnableMouseCapture},
        };
        
        // Create the markdown file
        let file_path = self.create_markdown_file(todo)?;
        
        // Suspend TUI - restore terminal to normal mode
        disable_raw_mode()
            .map_err(|e| format!("Failed to disable raw mode: {}", e))?;
        
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        ).map_err(|e| format!("Failed to leave alternate screen: {}", e))?;
        
        terminal.show_cursor()
            .map_err(|e| format!("Failed to show cursor: {}", e))?;
        
        // Launch glow and WAIT for it to complete (foreground process)
        let status = Command::new("glow")
            .arg("--tui")  // Force TUI mode
            .arg(&file_path)
            .stdin(std::process::Stdio::inherit())
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .status()
            .map_err(|e| format!("Failed to launch glow (is it installed?): {}", e))?;
        
        // Restore TUI - re-enter alternate screen mode
        enable_raw_mode()
            .map_err(|e| format!("Failed to enable raw mode: {}", e))?;
        
        execute!(
            terminal.backend_mut(),
            EnterAlternateScreen,
            EnableMouseCapture
        ).map_err(|e| format!("Failed to enter alternate screen: {}", e))?;
        
        // Force a full redraw
        terminal.clear()
            .map_err(|e| format!("Failed to clear terminal: {}", e))?;
        
        if !status.success() {
            return Err("Glow exited with error".to_string());
        }
        
        // Read back the edited content and update database
        if let Ok(edited_content) = std::fs::read_to_string(&file_path) {
            if let Ok((new_title, new_description)) = self.parse_markdown(&edited_content) {
                if new_title != todo.title || new_description != todo.description {
                    if let Err(e) = self.database.update_todo(todo.id, new_title, new_description) {
                        return Err(format!("Failed to update todo: {}", e));
                    } else {
                        let _ = self.refresh_todos();
                    }
                }
            }
        }
        
        Ok(())
    }
    
    fn parse_markdown(&self, content: &str) -> Result<(String, String), String> {
        let lines: Vec<&str> = content.lines().collect();
        let mut title = String::new();
        let mut description = String::new();
        
        let mut in_description = false;
        
        for line in lines {
            if line.starts_with("# ") && title.is_empty() {
                title = line[2..].trim().to_string();
            } else if line.starts_with("## Description") {
                in_description = true;
            } else if line.starts_with("## Metadata") {
                // Stop collecting description when we hit the metadata section
                in_description = false;
            } else if in_description {
                // Collect all lines in the description section, including empty lines and headers
                if !description.is_empty() {
                    description.push('\n');
                }
                if line.trim() != "(No description)" {
                    description.push_str(line);
                } else {
                    // Don't add the "(No description)" placeholder
                    description.pop(); // Remove the newline we just added
                }
            }
        }
        
        Ok((title, description.trim().to_string()))
    }

    pub fn new(database: Database) -> anyhow::Result<Self> {
        let mut app = App {
            database,
            incomplete_todos: Vec::new(),
            completed_todos: Vec::new(),
            tree_manager: TodoTreeManager::new(),
            list_state: ListState::default(),
            tree_list_state: ListState::default(),
            completed_list_state: ListState::default(),
            mode: AppMode::List,
            input: String::new(),
            input_title: String::new(),
            input_description: String::new(),
            editing_id: None,
            current_parent: None,
            should_quit: false,
            error_message: None,
            search_query: String::new(),
            search_results: Vec::new(),
            search_list_state: ListState::default(),
            search_matches: Vec::new(),
            input_parent: String::new(),
            selected_parent_id: None,
            create_field_focus: CreateFieldFocus::Title,
            use_tree_view: true,
            glow_pending: None,
        };
        app.refresh_todos()?;
        if !app.incomplete_todos.is_empty() {
            app.list_state.select(Some(0));
        }
        Ok(app)
    }

    pub fn refresh_todos(&mut self) -> anyhow::Result<()> {
        self.incomplete_todos = self.database.get_incomplete_todos(self.current_parent)?;
        // Load ALL completed todos for the completed view (not just recent 5)
        self.completed_todos = self.get_all_completed_todos()?;
        
        // Rebuild tree view with all todos
        let all_todos = self.database.get_all_todos()?;
        self.tree_manager.rebuild_from_todos(all_todos);
        
        // Initialize tree selection if we have items
        if !self.tree_manager.get_rendered_lines().is_empty() && self.tree_list_state.selected().is_none() {
            self.tree_list_state.select(Some(0));
        }
        
        Ok(())
    }

    fn get_all_completed_todos(&self) -> anyhow::Result<Vec<Todo>> {
        // Use existing database method but without limit
        let all_todos = self.database.get_all_todos()?;
        let mut completed_todos: Vec<Todo> = all_todos
            .into_iter()
            .filter(|todo| todo.is_completed())
            .collect();
        
        // Sort by completion time (most recent first)
        completed_todos.sort_by(|a, b| {
            match (a.completed_at, b.completed_at) {
                (Some(a_time), Some(b_time)) => b_time.cmp(&a_time),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            }
        });
        
        Ok(completed_todos)
    }

    fn update_tree_search_matches(&mut self) -> anyhow::Result<()> {
        if self.search_query.is_empty() {
            self.search_matches.clear();
        } else {
            self.search_matches = self.database.search_todos(&self.search_query)?
                .into_iter()
                .map(|todo| todo.id)
                .collect();
        }
        Ok(())
    }

    pub fn update_search_results(&mut self) -> anyhow::Result<()> {
        self.search_results = self.database.search_todos(&self.search_query)?;
        // Reset selection when search results change
        if !self.search_results.is_empty() {
            self.search_list_state.select(Some(0));
        } else {
            self.search_list_state.select(None);
        }
        Ok(())
    }

    fn get_current_todos(&self) -> &Vec<Todo> {
        match self.mode {
            AppMode::CompletedView => &self.completed_todos,
            _ => &self.incomplete_todos,
        }
    }

    fn get_current_list_state(&self) -> &ListState {
        match self.mode {
            AppMode::CompletedView => &self.completed_list_state,
            _ if self.use_tree_view => &self.tree_list_state,
            _ => &self.list_state,
        }
    }

    fn get_current_list_state_mut(&mut self) -> &mut ListState {
        match self.mode {
            AppMode::CompletedView => &mut self.completed_list_state,
            _ if self.use_tree_view => &mut self.tree_list_state,
            _ => &mut self.list_state,
        }
    }

    fn get_selected_todo(&self) -> Option<&Todo> {
        match self.mode {
            AppMode::ListFind => self.get_selected_search_todo(),
            AppMode::CompletedView => {
                // Completed view should always use completed_todos list, not tree
                let selected = self.completed_list_state.selected()?;
                self.completed_todos.get(selected)
            }
            AppMode::TreeSearch => {
                // In tree search mode, still use tree selection
                if self.use_tree_view {
                    let selected = self.tree_list_state.selected()?;
                    let rendered_lines = self.tree_manager.get_rendered_lines();
                    let line = rendered_lines.get(selected)?;
                    self.tree_manager.get_todo_by_id(line.todo_id)
                } else {
                    let todos = self.get_current_todos();
                    let selected = self.get_current_list_state().selected()?;
                    todos.get(selected)
                }
            }
            _ => {
                if self.use_tree_view {
                    let selected = self.tree_list_state.selected()?;
                    let rendered_lines = self.tree_manager.get_rendered_lines();
                    let line = rendered_lines.get(selected)?;
                    self.tree_manager.get_todo_by_id(line.todo_id)
                } else {
                    let todos = self.get_current_todos();
                    let selected = self.get_current_list_state().selected()?;
                    todos.get(selected)
                }
            }
        }
    }

    pub fn handle_key_event(&mut self, key: KeyCode) -> anyhow::Result<()> {
        self.error_message = None;

        match self.mode {
            AppMode::List => self.handle_list_key(key)?,
            AppMode::CompletedView => self.handle_completed_view_key(key)?,
            AppMode::View => self.handle_view_key(key)?,
            AppMode::Edit => self.handle_edit_key(key)?,
            AppMode::Create => self.handle_create_key(key)?,
            AppMode::ConfirmDelete => self.handle_delete_key(key)?,
            AppMode::ListFind => self.handle_list_find_key(key)?,
            AppMode::TreeSearch => self.handle_tree_search_key(key)?,
            AppMode::ParentSearch => self.handle_parent_search_key(key)?,
        }
        Ok(())
    }

    fn handle_list_key(&mut self, key: KeyCode) -> anyhow::Result<()> {
        match key {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('t') => {
                if self.use_tree_view {
                    // Branch-level toggle: expand/collapse the selected item
                    if let Some(selected) = self.tree_list_state.selected() {
                        if let Some(line) = self.tree_manager.get_rendered_lines().get(selected) {
                            if line.has_children {
                                self.tree_manager.toggle_expansion(line.todo_id);
                                // Maintain selection after toggle
                                self.update_tree_selection_after_toggle(selected);
                            }
                        }
                    }
                } else {
                    // Switch to tree view
                    self.use_tree_view = true;
                    if !self.tree_manager.get_rendered_lines().is_empty() {
                        self.tree_list_state.select(Some(0));
                    }
                }
            }
            KeyCode::Char('f') => {
                // List Find: flat search results view
                self.mode = AppMode::ListFind;
                self.search_query.clear();
                self.search_results.clear();
                self.search_list_state.select(None);
            }
            KeyCode::Char('c') => {
                if self.mode == AppMode::CompletedView {
                    self.mode = AppMode::List;
                } else {
                    self.mode = AppMode::CompletedView;
                    if !self.completed_todos.is_empty() && self.completed_list_state.selected().is_none() {
                        self.completed_list_state.select(Some(0));
                    }
                }
            }
            KeyCode::Char('/') => {
                // Tree Search: live highlighting in tree view
                self.mode = AppMode::TreeSearch;
                self.search_query.clear();
            }
            KeyCode::Char('n') => {
                self.mode = AppMode::Create;
                self.input_title.clear();
                self.input_parent.clear();
                self.input_description.clear();
                self.selected_parent_id = None;
                self.create_field_focus = CreateFieldFocus::Title;
            }
            KeyCode::Char('d') => {
                if self.get_current_list_state().selected().is_some() {
                    self.mode = AppMode::ConfirmDelete;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.use_tree_view {
                    self.next_tree_item();
                } else {
                    self.next_todo();
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.use_tree_view {
                    self.previous_tree_item();
                } else {
                    self.previous_todo();
                }
            }
            KeyCode::Char(' ') => {
                if let Some(todo) = self.get_selected_todo() {
                    let todo_id = todo.id;
                    let is_currently_completed = todo.is_completed();
                    
                    if is_currently_completed {
                        self.database.uncomplete_todo(todo_id)?;
                    } else {
                        self.database.complete_todo(todo_id)?;
                    }
                    
                    if self.use_tree_view {
                        // Update tree manager directly for visual feedback
                        self.tree_manager.update_todo_completion(todo_id, !is_currently_completed);
                    }
                    
                    self.refresh_todos()?;
                    self.update_selection_after_refresh();
                }
            }
            KeyCode::Enter => {
                if let Some(todo) = self.get_selected_todo() {
                    self.glow_pending = Some(todo.clone());
                }
            }
            KeyCode::Right | KeyCode::Char('l') => {
                if let Some(todo) = self.get_selected_todo() {
                    self.current_parent = Some(todo.id);
                    self.refresh_todos()?;
                    if !self.incomplete_todos.is_empty() {
                        self.list_state.select(Some(0));
                        if self.use_tree_view {
                            self.tree_list_state.select(Some(0));
                        }
                    }
                }
            }
            KeyCode::Left | KeyCode::Char('h') => {
                if self.current_parent.is_some() {
                    self.current_parent = None;
                    self.refresh_todos()?;
                    if !self.incomplete_todos.is_empty() {
                        self.list_state.select(Some(0));
                        if self.use_tree_view {
                            self.tree_list_state.select(Some(0));
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_completed_view_key(&mut self, key: KeyCode) -> anyhow::Result<()> {
        match key {
            KeyCode::Esc | KeyCode::Char('q') => self.mode = AppMode::List,
            KeyCode::Char('c') => self.mode = AppMode::List,
            KeyCode::Down | KeyCode::Char('j') => self.next_todo(),
            KeyCode::Up | KeyCode::Char('k') => self.previous_todo(),
            KeyCode::Enter => {
                if let Some(todo) = self.get_selected_todo() {
                    self.glow_pending = Some(todo.clone());
                }
            }
            KeyCode::Char(' ') => {
                // Allow uncompleting todos from completed view
                if let Some(todo) = self.get_selected_todo() {
                    let todo_id = todo.id;
                    self.database.uncomplete_todo(todo_id)?;
                    self.refresh_todos()?;
                    self.update_selection_after_refresh();
                }
            }
            _ => {}
        }
        Ok(())
    }


    fn update_selection_after_refresh(&mut self) {
        match self.mode {
            AppMode::CompletedView => {
                if self.completed_todos.is_empty() {
                    self.completed_list_state.select(None);
                } else {
                    let selected = self.completed_list_state.selected().unwrap_or(0);
                    if selected >= self.completed_todos.len() {
                        self.completed_list_state.select(Some(self.completed_todos.len() - 1));
                    }
                }
            }
            _ => {
                if self.use_tree_view {
                    let lines_len = self.tree_manager.get_rendered_lines().len();
                    if lines_len == 0 {
                        self.tree_list_state.select(None);
                    } else {
                        let selected = self.tree_list_state.selected().unwrap_or(0);
                        if selected >= lines_len {
                            self.tree_list_state.select(Some(lines_len - 1));
                        }
                    }
                } else {
                    if self.incomplete_todos.is_empty() {
                        self.list_state.select(None);
                    } else {
                        let selected = self.list_state.selected().unwrap_or(0);
                        if selected >= self.incomplete_todos.len() {
                            self.list_state.select(Some(self.incomplete_todos.len() - 1));
                        }
                    }
                }
            }
        }
    }

    fn handle_view_key(&mut self, key: KeyCode) -> anyhow::Result<()> {
        match key {
            KeyCode::Esc | KeyCode::Char('q') => {
                // Return to the previous mode (either List or CompletedView)
                if self.get_selected_todo().map(|t| t.is_completed()).unwrap_or(false) {
                    self.mode = AppMode::CompletedView;
                } else {
                    self.mode = AppMode::List;
                }
            }
            KeyCode::Char('e') => {
                if let Some(todo) = self.get_selected_todo() {
                    let todo_id = todo.id;
                    let todo_title = todo.title.clone();
                    let todo_description = todo.description.clone();
                    self.mode = AppMode::Edit;
                    self.editing_id = Some(todo_id);
                    self.input_title = todo_title;
                    self.input_description = todo_description;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_edit_key(&mut self, key: KeyCode) -> anyhow::Result<()> {
        match key {
            KeyCode::Esc => self.mode = AppMode::List,
            KeyCode::Enter => {
                if let Some(id) = self.editing_id {
                    if !self.input_title.trim().is_empty() {
                        self.database.update_todo(id, self.input_title.clone(), self.input_description.clone())?;
                        self.refresh_todos()?;
                        self.mode = AppMode::List;
                    } else {
                        self.error_message = Some("Title cannot be empty".to_string());
                    }
                }
            }
            KeyCode::Char(c) => {
                self.input_title.push(c);
            }
            KeyCode::Backspace => {
                self.input_title.pop();
            }
            KeyCode::Tab => {
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_create_key(&mut self, key: KeyCode) -> anyhow::Result<()> {
        match key {
            KeyCode::Esc => self.mode = AppMode::List,
            KeyCode::Enter => {
                if !self.input_title.trim().is_empty() {
                    let new_todo = NewTodo {
                        title: self.input_title.clone(),
                        description: self.input_description.clone(),
                        parent_id: self.selected_parent_id,
                    };
                    self.database.create_todo(new_todo)?;
                    self.refresh_todos()?;
                    self.mode = AppMode::List;
                    self.input_title.clear();
                    self.input_parent.clear();
                    self.input_description.clear();
                    self.selected_parent_id = None;
                    self.create_field_focus = CreateFieldFocus::Title;
                } else {
                    self.error_message = Some("Title cannot be empty".to_string());
                }
            }
            KeyCode::Tab => {
                match self.create_field_focus {
                    CreateFieldFocus::Title => {
                        self.create_field_focus = CreateFieldFocus::Parent;
                    }
                    CreateFieldFocus::Parent => {
                        self.create_field_focus = CreateFieldFocus::Description;
                    }
                    CreateFieldFocus::Description => {
                        self.create_field_focus = CreateFieldFocus::Title;
                    }
                }
            }
            KeyCode::Char(c) => {
                match self.create_field_focus {
                    CreateFieldFocus::Title => {
                        self.input_title.push(c);
                    }
                    CreateFieldFocus::Description => {
                        self.input_description.push(c);
                    }
                    CreateFieldFocus::Parent => {
                        // Enter parent search mode when typing in parent field
                        self.mode = AppMode::ParentSearch;
                        self.search_query.clear();
                        self.search_query.push(c);
                        self.update_search_results()?;
                    }
                }
            }
            KeyCode::Backspace => {
                match self.create_field_focus {
                    CreateFieldFocus::Title => {
                        self.input_title.pop();
                    }
                    CreateFieldFocus::Description => {
                        self.input_description.pop();
                    }
                    CreateFieldFocus::Parent => {
                        // Clear parent selection
                        self.input_parent.clear();
                        self.selected_parent_id = None;
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_delete_key(&mut self, key: KeyCode) -> anyhow::Result<()> {
        match key {
            KeyCode::Char('y') => {
                if let Some(todo) = self.get_selected_todo() {
                    self.database.delete_todo(todo.id)?;
                    self.refresh_todos()?;
                    self.update_selection_after_refresh();
                }
                self.mode = AppMode::List;
            }
            KeyCode::Char('n') | KeyCode::Esc => {
                self.mode = AppMode::List;
            }
            _ => {}
        }
        Ok(())
    }

    fn next_todo(&mut self) {
        let todos_len = self.get_current_todos().len();
        if todos_len == 0 {
            return;
        }

        let list_state = self.get_current_list_state_mut();
        let i = match list_state.selected() {
            Some(i) => {
                if i >= todos_len - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        list_state.select(Some(i));
    }

    fn previous_todo(&mut self) {
        let todos_len = self.get_current_todos().len();
        if todos_len == 0 {
            return;
        }

        let list_state = self.get_current_list_state_mut();
        let i = match list_state.selected() {
            Some(i) => {
                if i == 0 {
                    todos_len - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        list_state.select(Some(i));
    }

    fn next_tree_item(&mut self) {
        let lines_len = self.tree_manager.get_rendered_lines().len();
        if lines_len == 0 {
            return;
        }

        let i = match self.tree_list_state.selected() {
            Some(i) => {
                if i >= lines_len - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.tree_list_state.select(Some(i));
    }

    fn previous_tree_item(&mut self) {
        let lines_len = self.tree_manager.get_rendered_lines().len();
        if lines_len == 0 {
            return;
        }

        let i = match self.tree_list_state.selected() {
            Some(i) => {
                if i == 0 {
                    lines_len - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.tree_list_state.select(Some(i));
    }

    fn update_tree_selection_after_toggle(&mut self, previous_selected: usize) {
        let lines_len = self.tree_manager.get_rendered_lines().len();
        if lines_len == 0 {
            self.tree_list_state.select(None);
        } else {
            // Keep selection on same item if possible, otherwise adjust to valid range
            let new_selected = if previous_selected >= lines_len {
                lines_len - 1
            } else {
                previous_selected
            };
            self.tree_list_state.select(Some(new_selected));
        }
    }

    fn handle_list_find_key(&mut self, key: KeyCode) -> anyhow::Result<()> {
        match key {
            KeyCode::Esc => {
                self.mode = AppMode::List;
                self.search_query.clear();
                self.search_results.clear();
            }
            KeyCode::Enter => {
                // If there's a selected result, view/edit it with glow
                if let Some(selected) = self.search_list_state.selected() {
                    if let Some(todo) = self.search_results.get(selected) {
                        self.glow_pending = Some(todo.clone());
                    }
                }
            }
            KeyCode::Down | KeyCode::Char('j') => self.next_search_result(),
            KeyCode::Up | KeyCode::Char('k') => self.previous_search_result(),
            KeyCode::Char(c) => {
                self.search_query.push(c);
                self.update_search_results()?;
            }
            KeyCode::Backspace => {
                self.search_query.pop();
                self.update_search_results()?;
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_tree_search_key(&mut self, key: KeyCode) -> anyhow::Result<()> {
        match key {
            KeyCode::Esc => {
                self.mode = AppMode::List;
                self.search_query.clear();
                self.search_matches.clear();
            }
            KeyCode::Enter => {
                // If there's a selected todo in tree, view/edit it with glow
                if let Some(todo) = self.get_selected_todo() {
                    self.glow_pending = Some(todo.clone());
                }
            }
            // Only arrow keys for navigation during search
            KeyCode::Down => {
                if self.use_tree_view {
                    self.next_tree_item();
                } else {
                    self.next_todo();
                }
            }
            KeyCode::Up => {
                if self.use_tree_view {
                    self.previous_tree_item();
                } else {
                    self.previous_todo();
                }
            }
            // Special keys that should NOT be added to search
            KeyCode::Tab => {
                // Allow tree expansion/collapse during search with Tab key
                if self.use_tree_view {
                    if let Some(selected) = self.tree_list_state.selected() {
                        if let Some(line) = self.tree_manager.get_rendered_lines().get(selected) {
                            if line.has_children {
                                self.tree_manager.toggle_expansion(line.todo_id);
                                self.update_tree_search_matches()?;
                                self.update_tree_selection_after_toggle(selected);
                            }
                        }
                    }
                }
            }
            KeyCode::Backspace => {
                self.search_query.pop();
                self.update_tree_search_matches()?;
            }
            // ALL printable characters (including j, k, h, l, t, etc.) go to search
            KeyCode::Char(c) => {
                self.search_query.push(c);
                self.update_tree_search_matches()?;
            }
            _ => {}
        }
        Ok(())
    }

    fn next_search_result(&mut self) {
        if self.search_results.is_empty() {
            return;
        }

        let i = match self.search_list_state.selected() {
            Some(i) => {
                if i >= self.search_results.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.search_list_state.select(Some(i));
    }

    fn previous_search_result(&mut self) {
        if self.search_results.is_empty() {
            return;
        }

        let i = match self.search_list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.search_results.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.search_list_state.select(Some(i));
    }

    fn get_selected_search_todo(&self) -> Option<&Todo> {
        let selected = self.search_list_state.selected()?;
        self.search_results.get(selected)
    }

    fn handle_parent_search_key(&mut self, key: KeyCode) -> anyhow::Result<()> {
        match key {
            KeyCode::Esc => {
                // Return to create mode
                self.mode = AppMode::Create;
                self.create_field_focus = CreateFieldFocus::Parent;
                self.search_query.clear();
                self.search_results.clear();
            }
            KeyCode::Enter => {
                // Select the highlighted parent
                if let Some(selected) = self.search_list_state.selected() {
                    if let Some(todo) = self.search_results.get(selected) {
                        self.selected_parent_id = Some(todo.id);
                        // Truncate to 40 characters
                        let parent_display = if todo.title.len() > 40 {
                            format!("{}...", &todo.title[..37])
                        } else {
                            todo.title.clone()
                        };
                        self.input_parent = format!("ID:{} {}", todo.id, parent_display);
                        self.mode = AppMode::Create;
                        self.create_field_focus = CreateFieldFocus::Parent;
                    }
                }
            }
            KeyCode::Down | KeyCode::Char('j') => self.next_search_result(),
            KeyCode::Up | KeyCode::Char('k') => self.previous_search_result(),
            KeyCode::Char(c) => {
                self.search_query.push(c);
                self.update_search_results()?;
            }
            KeyCode::Backspace => {
                self.search_query.pop();
                self.update_search_results()?;
            }
            _ => {}
        }
        Ok(())
    }

    pub fn draw(&mut self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(3)])
            .split(f.area());

        match self.mode {
            AppMode::List => {
                if self.use_tree_view {
                    self.draw_tree_view(f, chunks[0]);
                } else {
                    self.draw_split_todo_lists(f, chunks[0]);
                }
            }
            AppMode::TreeSearch => {
                if self.use_tree_view {
                    self.draw_tree_search_view(f, chunks[0]);
                } else {
                    self.draw_split_todo_lists(f, chunks[0]);
                }
            }
            AppMode::CompletedView => self.draw_completed_view(f, chunks[0]),
            AppMode::View => self.draw_todo_view(f, chunks[0]),
            AppMode::Edit => self.draw_edit_mode(f, chunks[0]),
            AppMode::Create => self.draw_create_mode(f, chunks[0]),
            AppMode::ConfirmDelete => self.draw_confirm_delete(f, chunks[0]),
            AppMode::ListFind => self.draw_list_find_mode(f, chunks[0]),
            AppMode::ParentSearch => self.draw_parent_search_mode(f, chunks[0]),
        }

        self.draw_help(f, chunks[1]);
    }

    fn draw_split_todo_lists(&mut self, f: &mut Frame, area: Rect) {
        // Use the full area for incomplete todos (or tree view)
        if self.use_tree_view {
            self.draw_tree_view(f, area);
        } else {
            self.draw_incomplete_todos(f, area);
        }
    }

    fn draw_incomplete_todos(&mut self, f: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .incomplete_todos
            .iter()
            .map(|todo| {
                let created_time = todo.created_at.with_timezone(&Local).format("%m/%d %H:%M").to_string();
                let parent_title = self.database.get_parent_title(todo.parent_id)
                    .unwrap_or(None)
                    .unwrap_or_else(|| "null".to_string());
                
                ListItem::new(Line::from(vec![
                    Span::styled(format!("{} [ ] ", todo.id), Style::default().fg(CatppuccinFrappe::SUBTEXT1)),
                    Span::styled(todo.title.clone(), Style::default().fg(CatppuccinFrappe::INCOMPLETE)),
                    Span::styled(format!(" | Created: {} | Parent: {}", created_time, parent_title), 
                               Style::default().fg(CatppuccinFrappe::CREATION_TIME)),
                ]))
            })
            .collect();

        let title = if let Some(parent_id) = self.current_parent {
            format!("Incomplete Todos (Parent: {})", parent_id)
        } else {
            "Incomplete Todos".to_string()
        };

        let highlight_style = Style::default()
            .bg(CatppuccinFrappe::SELECTED_BG)
            .fg(CatppuccinFrappe::SELECTED);

        let list = List::new(items)
            .block(Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(CatppuccinFrappe::BORDER)))
            .highlight_style(highlight_style)
            .highlight_symbol("▶ ");

        f.render_stateful_widget(list, area, &mut self.list_state);
    }


    fn draw_tree_view(&mut self, f: &mut Frame, area: Rect) {
        let rendered_lines = self.tree_manager.get_rendered_lines();
        
        let items: Vec<ListItem> = rendered_lines
            .iter()
            .map(|line| {
                if let Some(todo) = self.tree_manager.get_todo_by_id(line.todo_id) {
                    let created_time = todo.created_at.with_timezone(&Local).format("%m/%d %H:%M").to_string();
                    
                    let (display_style, prefix_style) = if todo.is_completed() {
                        (
                            Style::default().fg(CatppuccinFrappe::COMPLETED).add_modifier(Modifier::CROSSED_OUT),
                            Style::default().fg(CatppuccinFrappe::SURFACE2)
                        )
                    } else {
                        (
                            Style::default().fg(CatppuccinFrappe::INCOMPLETE),
                            Style::default().fg(CatppuccinFrappe::PARENT_INDICATOR)
                        )
                    };

                    ListItem::new(Line::from(vec![
                        Span::styled(&line.prefix, prefix_style),
                        Span::styled(&line.display_text, display_style),
                        Span::styled(format!(" | Created: {}", created_time), 
                                   Style::default().fg(CatppuccinFrappe::CREATION_TIME)),
                    ]))
                } else {
                    ListItem::new(Line::from(Span::styled(
                        format!("{}ERROR: Todo not found", line.prefix),
                        Style::default().fg(CatppuccinFrappe::ERROR)
                    )))
                }
            })
            .collect();

        let title = "Todo Tree View (All Items)";
        let list = List::new(items)
            .block(Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(CatppuccinFrappe::BORDER)))
            .highlight_style(Style::default()
                .bg(CatppuccinFrappe::SELECTED_BG)
                .fg(CatppuccinFrappe::SELECTED))
            .highlight_symbol("▶ ");

        f.render_stateful_widget(list, area, &mut self.tree_list_state);
    }

    fn draw_tree_search_view(&mut self, f: &mut Frame, area: Rect) {
        // Split area to make room for search input at bottom
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(3)])
            .split(area);

        // Draw tree view with highlighting in the main area
        self.draw_tree_view_with_highlights(f, chunks[0]);

        // Draw search input at bottom
        let search_input = Paragraph::new(self.search_query.as_str())
            .block(Block::default()
                .borders(Borders::ALL)
                .title("Tree Search")
                .border_style(Style::default().fg(CatppuccinFrappe::YELLOW)))
            .style(Style::default().fg(CatppuccinFrappe::TEXT));
        f.render_widget(search_input, chunks[1]);
    }

    fn draw_tree_view_with_highlights(&mut self, f: &mut Frame, area: Rect) {
        let rendered_lines = self.tree_manager.get_rendered_lines();
        
        let items: Vec<ListItem> = rendered_lines
            .iter()
            .map(|line| {
                if let Some(todo) = self.tree_manager.get_todo_by_id(line.todo_id) {
                    let created_time = todo.created_at.with_timezone(&Local).format("%m/%d %H:%M").to_string();
                    
                    // Check if this todo matches the search
                    let is_match = self.search_matches.contains(&line.todo_id);
                    
                    let (display_style, prefix_style) = if todo.is_completed() {
                        (
                            if is_match {
                                Style::default().fg(CatppuccinFrappe::PEACH).add_modifier(Modifier::CROSSED_OUT).add_modifier(Modifier::BOLD)
                            } else {
                                Style::default().fg(CatppuccinFrappe::COMPLETED).add_modifier(Modifier::CROSSED_OUT)
                            },
                            Style::default().fg(CatppuccinFrappe::SURFACE2)
                        )
                    } else {
                        (
                            if is_match {
                                Style::default().fg(CatppuccinFrappe::YELLOW).add_modifier(Modifier::BOLD)
                            } else {
                                Style::default().fg(CatppuccinFrappe::INCOMPLETE)
                            },
                            Style::default().fg(CatppuccinFrappe::PARENT_INDICATOR)
                        )
                    };

                    ListItem::new(Line::from(vec![
                        Span::styled(&line.prefix, prefix_style),
                        Span::styled(&line.display_text, display_style),
                        Span::styled(format!(" | Created: {}", created_time), 
                                   Style::default().fg(CatppuccinFrappe::CREATION_TIME)),
                    ]))
                } else {
                    ListItem::new(Line::from(Span::styled(
                        format!("{}ERROR: Todo not found", line.prefix),
                        Style::default().fg(CatppuccinFrappe::ERROR)
                    )))
                }
            })
            .collect();

        let title = if self.search_query.is_empty() {
            "Todo Tree View".to_string()
        } else {
            format!("Tree Search - {} matches", self.search_matches.len())
        };
        
        let list = List::new(items)
            .block(Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(CatppuccinFrappe::BORDER)))
            .highlight_style(Style::default()
                .bg(CatppuccinFrappe::SELECTED_BG)
                .fg(CatppuccinFrappe::SELECTED))
            .highlight_symbol("▶ ");

        f.render_stateful_widget(list, area, &mut self.tree_list_state);
    }

    fn draw_completed_view(&mut self, f: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .completed_todos
            .iter()
            .map(|todo| {
                let completed_time = if let Some(completed_at) = todo.completed_at {
                    completed_at.with_timezone(&Local).format("%m/%d %H:%M").to_string()
                } else {
                    "Unknown".to_string()
                };
                let created_time = todo.created_at.with_timezone(&Local).format("%m/%d %H:%M").to_string();
                let parent_title = self.database.get_parent_title(todo.parent_id)
                    .unwrap_or(None)
                    .unwrap_or_else(|| "null".to_string());
                
                ListItem::new(Line::from(vec![
                    Span::styled(format!("{} [✓] ", todo.id), 
                               Style::default().fg(CatppuccinFrappe::COMPLETED)),
                    Span::styled(
                        todo.title.clone(), 
                        Style::default().fg(CatppuccinFrappe::COMPLETED).add_modifier(Modifier::CROSSED_OUT)
                    ),
                    Span::styled(
                        format!(" | Created: {} | Completed: {} | Parent: {}", 
                               created_time, completed_time, parent_title),
                        Style::default().fg(CatppuccinFrappe::SUBTEXT0)
                    ),
                ]))
            })
            .collect();

        let title = format!("All Completed Todos ({} total)", self.completed_todos.len());
        let highlight_style = Style::default()
            .bg(CatppuccinFrappe::SELECTED_BG)
            .fg(CatppuccinFrappe::SELECTED);

        let list = List::new(items)
            .block(Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(CatppuccinFrappe::BORDER)))
            .highlight_style(highlight_style)
            .highlight_symbol("▶ ");

        f.render_stateful_widget(list, area, &mut self.completed_list_state);
    }

    fn draw_todo_view(&self, f: &mut Frame, area: Rect) {
        if let Some(todo) = self.get_selected_todo() {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Header info
                    Constraint::Length(3), // Title
                    Constraint::Length(5), // Metadata 
                    Constraint::Min(0),    // Description
                ])
                .split(area);

            // Header: ID, Status, and basic info
            let status_text = if todo.is_completed() {
                "✓ COMPLETED"
            } else {
                "○ INCOMPLETE"
            };
            
            let header_text = format!("Todo #{} | Status: {}", todo.id, status_text);
            let header_style = if todo.is_completed() {
                Style::default().fg(CatppuccinFrappe::COMPLETED)
            } else {
                Style::default().fg(CatppuccinFrappe::INCOMPLETE)
            };

            let header_block = Paragraph::new(header_text)
                .block(Block::default()
                    .borders(Borders::ALL)
                    .title("Todo Information")
                    .border_style(Style::default().fg(CatppuccinFrappe::BORDER)))
                .style(header_style)
                .wrap(Wrap { trim: true });
            f.render_widget(header_block, chunks[0]);

            // Title section
            let title_block = Paragraph::new(todo.title.clone())
                .block(Block::default()
                    .borders(Borders::ALL)
                    .title("Title")
                    .border_style(Style::default().fg(CatppuccinFrappe::BORDER)))
                .style(Style::default().fg(CatppuccinFrappe::TEXT))
                .wrap(Wrap { trim: true });
            f.render_widget(title_block, chunks[1]);

            // Metadata section: Dates and relationships
            let created_time = todo.created_at.with_timezone(&Local).format("%A, %B %d, %Y at %I:%M %p").to_string();
            
            let completion_text = if let Some(completed_at) = todo.completed_at {
                let completed_time = completed_at.with_timezone(&Local).format("%A, %B %d, %Y at %I:%M %p").to_string();
                let duration = completed_at.signed_duration_since(todo.created_at);
                let duration_text = if duration.num_days() > 0 {
                    format!("{} days", duration.num_days())
                } else if duration.num_hours() > 0 {
                    format!("{} hours", duration.num_hours())
                } else {
                    format!("{} minutes", duration.num_minutes())
                };
                format!("Completed: {}\nDuration: {}", completed_time, duration_text)
            } else {
                let duration = chrono::Utc::now().signed_duration_since(todo.created_at);
                let age_text = if duration.num_days() > 0 {
                    format!("{} days ago", duration.num_days())
                } else if duration.num_hours() > 0 {
                    format!("{} hours ago", duration.num_hours())
                } else {
                    format!("{} minutes ago", duration.num_minutes())
                };
                format!("Still pending (created {})", age_text)
            };

            let parent_text = if let Some(parent_id) = todo.parent_id {
                match self.database.get_parent_title(Some(parent_id)) {
                    Ok(Some(parent_title)) => format!("Parent: #{} - {}", parent_id, parent_title),
                    _ => format!("Parent: #{} (title not found)", parent_id),
                }
            } else {
                "Parent: None (root todo)".to_string()
            };

            let metadata_text = format!(
                "Created: {}\n{}\n{}",
                created_time, completion_text, parent_text
            );

            let metadata_block = Paragraph::new(metadata_text)
                .block(Block::default()
                    .borders(Borders::ALL)
                    .title("Metadata")
                    .border_style(Style::default().fg(CatppuccinFrappe::BORDER)))
                .style(Style::default().fg(CatppuccinFrappe::SUBTEXT1))
                .wrap(Wrap { trim: true });
            f.render_widget(metadata_block, chunks[2]);

            // Description section
            let description_text = if todo.description.trim().is_empty() {
                "(No description provided)".to_string()
            } else {
                todo.description.clone()
            };

            let description_block = Paragraph::new(description_text)
                .block(Block::default()
                    .borders(Borders::ALL)
                    .title("Description")
                    .border_style(Style::default().fg(CatppuccinFrappe::BORDER)))
                .style(Style::default().fg(CatppuccinFrappe::TEXT))
                .wrap(Wrap { trim: true });
            f.render_widget(description_block, chunks[3]);
        }
    }

    fn draw_edit_mode(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(area);

        let title_input = Paragraph::new(self.input_title.as_str())
            .block(Block::default()
                .borders(Borders::ALL)
                .title("Edit Title")
                .border_style(Style::default().fg(CatppuccinFrappe::YELLOW)))
            .style(Style::default().fg(CatppuccinFrappe::TEXT));
        f.render_widget(title_input, chunks[0]);

        let description_input = Paragraph::new(self.input_description.as_str())
            .block(Block::default()
                .borders(Borders::ALL)
                .title("Description")
                .border_style(Style::default().fg(CatppuccinFrappe::YELLOW)))
            .style(Style::default().fg(CatppuccinFrappe::TEXT));
        f.render_widget(description_input, chunks[1]);
    }

    fn draw_create_mode(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Length(3), Constraint::Min(0)])
            .split(area);

        // Title field
        let title_style = if self.create_field_focus == CreateFieldFocus::Title {
            Style::default().fg(CatppuccinFrappe::YELLOW)
        } else {
            Style::default().fg(CatppuccinFrappe::BORDER)
        };
        let title_input = Paragraph::new(self.input_title.as_str())
            .block(Block::default().borders(Borders::ALL).title("Title").border_style(title_style))
            .style(Style::default().fg(CatppuccinFrappe::TEXT));
        f.render_widget(title_input, chunks[0]);

        // Parent field  
        let parent_style = if self.create_field_focus == CreateFieldFocus::Parent {
            Style::default().fg(CatppuccinFrappe::YELLOW)
        } else {
            Style::default().fg(CatppuccinFrappe::BORDER)
        };
        let parent_display = if self.input_parent.is_empty() {
            "Press Tab to focus, type to search for parent...".to_string()
        } else {
            self.input_parent.clone()
        };
        let parent_input = Paragraph::new(parent_display.as_str())
            .block(Block::default().borders(Borders::ALL).title("Parent (optional)").border_style(parent_style))
            .style(Style::default().fg(CatppuccinFrappe::TEXT));
        f.render_widget(parent_input, chunks[1]);

        // Description field
        let desc_style = if self.create_field_focus == CreateFieldFocus::Description {
            Style::default().fg(CatppuccinFrappe::YELLOW)
        } else {
            Style::default().fg(CatppuccinFrappe::BORDER)
        };
        let description_input = Paragraph::new(self.input_description.as_str())
            .block(Block::default().borders(Borders::ALL).title("Description (optional)").border_style(desc_style))
            .style(Style::default().fg(CatppuccinFrappe::TEXT));
        f.render_widget(description_input, chunks[2]);
    }

    fn draw_confirm_delete(&self, f: &mut Frame, area: Rect) {
        let popup_area = centered_rect(50, 20, area);
        f.render_widget(Clear, popup_area);
        
        let block = Block::default()
            .title("Confirm Delete")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(CatppuccinFrappe::RED))
            .style(Style::default().bg(CatppuccinFrappe::BASE));
        
        let paragraph = Paragraph::new("Are you sure you want to delete this todo?\n\nPress 'y' to confirm, 'n' to cancel")
            .block(block)
            .style(Style::default().fg(CatppuccinFrappe::TEXT))
            .wrap(Wrap { trim: true });
        
        f.render_widget(paragraph, popup_area);
    }

    fn draw_list_find_mode(&mut self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(area);

        // Search input box
        let search_input = Paragraph::new(self.search_query.as_str())
            .block(Block::default()
                .borders(Borders::ALL)
                .title("Search (regex supported)")
                .border_style(Style::default().fg(CatppuccinFrappe::SAPPHIRE)))
            .style(Style::default().fg(CatppuccinFrappe::TEXT));
        f.render_widget(search_input, chunks[0]);

        // Search results
        let items: Vec<ListItem> = self
            .search_results
            .iter()
            .map(|todo| {
                let created_time = todo.created_at.with_timezone(&Local).format("%m/%d %H:%M").to_string();
                let completed_time = if let Some(completed_at) = todo.completed_at {
                    format!(" | Completed: {}", completed_at.with_timezone(&Local).format("%m/%d %H:%M"))
                } else {
                    String::new()
                };
                let parent_title = self.database.get_parent_title(todo.parent_id)
                    .unwrap_or(None)
                    .unwrap_or_else(|| "null".to_string());
                
                let status_icon = if todo.is_completed() { "[✓]" } else { "[ ]" };
                let title_style = if todo.is_completed() {
                    Style::default().fg(Color::Gray).add_modifier(Modifier::CROSSED_OUT)
                } else {
                    Style::default()
                };
                
                ListItem::new(Line::from(vec![
                    Span::raw(format!("{} {} ", todo.id, status_icon)),
                    Span::styled(todo.title.clone(), title_style),
                    Span::raw(format!(" | Created: {}{} | Parent: {}", created_time, completed_time, parent_title)),
                ]))
            })
            .collect();

        let results_title = format!("Search Results ({} found)", self.search_results.len());
        let list = List::new(items)
            .block(Block::default()
                .borders(Borders::ALL)
                .title(results_title)
                .border_style(Style::default().fg(CatppuccinFrappe::BORDER)))
            .highlight_style(Style::default()
                .bg(CatppuccinFrappe::SELECTED_BG)
                .fg(CatppuccinFrappe::SELECTED))
            .highlight_symbol("▶ ");

        f.render_stateful_widget(list, chunks[1], &mut self.search_list_state);
    }

    fn draw_parent_search_mode(&mut self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(area);

        // Search input box
        let search_input = Paragraph::new(self.search_query.as_str())
            .block(Block::default().borders(Borders::ALL).title("Search for Parent Todo (regex supported)"));
        f.render_widget(search_input, chunks[0]);

        // Search results - same as regular search but with different title
        let items: Vec<ListItem> = self
            .search_results
            .iter()
            .map(|todo| {
                let created_time = todo.created_at.with_timezone(&Local).format("%m/%d %H:%M").to_string();
                let completed_time = if let Some(completed_at) = todo.completed_at {
                    format!(" | Completed: {}", completed_at.with_timezone(&Local).format("%m/%d %H:%M"))
                } else {
                    String::new()
                };
                let parent_title = self.database.get_parent_title(todo.parent_id)
                    .unwrap_or(None)
                    .unwrap_or_else(|| "null".to_string());
                
                let status_icon = if todo.is_completed() { "[✓]" } else { "[ ]" };
                let title_style = if todo.is_completed() {
                    Style::default().fg(Color::Gray).add_modifier(Modifier::CROSSED_OUT)
                } else {
                    Style::default()
                };
                
                ListItem::new(Line::from(vec![
                    Span::raw(format!("{} {} ", todo.id, status_icon)),
                    Span::styled(todo.title.clone(), title_style),
                    Span::raw(format!(" | Created: {}{} | Parent: {}", created_time, completed_time, parent_title)),
                ]))
            })
            .collect();

        let results_title = format!("Select Parent ({} found)", self.search_results.len());
        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title(results_title))
            .highlight_style(Style::default().bg(Color::Green).fg(Color::White))
            .highlight_symbol(">> ");

        f.render_stateful_widget(list, chunks[1], &mut self.search_list_state);
    }

    fn draw_help(&self, f: &mut Frame, area: Rect) {
        let help_text = match self.mode {
            AppMode::List => {
                if self.use_tree_view {
                    "/: Tree Search | f: List Find | n: New | t: Expand/Collapse | c: Show Completed | Enter: View/Edit | d: Delete | Space: Toggle Completion | h/l: Navigate | q: Quit"
                } else {
                    "/: Tree Search | f: List Find | n: New | c: Show Completed | Enter: View/Edit | d: Delete | Space: Toggle Completion | h/l: Navigate | q: Quit"
                }
            }
            AppMode::TreeSearch => "Type to search tree | Tab: Expand/Collapse | Enter: View/Edit | ↑/↓: Navigate | Esc: Back",
            AppMode::CompletedView => "Enter: View/Edit | Space: Uncomplete | j/k: Navigate | c: Back to List | q/Esc: Back",
            AppMode::View => "e: Edit | q/Esc: Back",
            AppMode::Edit => "Enter: Save | Esc: Cancel",
            AppMode::Create => "Tab: Next Field | Enter: Save | Esc: Cancel",
            AppMode::ConfirmDelete => "y: Yes | n/Esc: No",
            AppMode::ListFind => "Type to search all | Enter: View/Edit | j/k: Navigate | Esc: Back",
            AppMode::ParentSearch => "Type to search | Enter: Select Parent | j/k: Navigate | Esc: Back",
        };

        let help = Paragraph::new(help_text)
            .block(Block::default()
                .borders(Borders::ALL)
                .title("Help")
                .border_style(Style::default().fg(CatppuccinFrappe::BORDER)))
            .style(Style::default().fg(CatppuccinFrappe::SUBTEXT1));
        
        let mut help_area = area;
        if let Some(error) = &self.error_message {
            let error_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(1), Constraint::Length(2)])
                .split(area);
            
            let error_paragraph = Paragraph::new(error.as_str())
                .style(Style::default().fg(CatppuccinFrappe::ERROR));
            f.render_widget(error_paragraph, error_chunks[0]);
            
            help_area = error_chunks[1];
        }
        
        f.render_widget(help, help_area);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}