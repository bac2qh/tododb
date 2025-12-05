use crate::database::{Database, NewTodo, Todo};
use crate::tree::TodoTreeManager;
use crate::colors::CatppuccinFrappe;
use chrono::{Local, Utc, DateTime, Duration};
use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Clear, List, ListItem, ListState, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap,
    },
    Frame,
};

#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    List,
    CompletedView,
    Create,
    ConfirmDelete,
    ListFind,
    TreeSearch,
    ParentSearch,
    Move,
    Help,
    IdModGoto,
}


#[derive(Debug, Clone, PartialEq)]
pub enum CreateFieldFocus {
    Title,
    DueDateRelative,
    DueDateAbsolute,
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
    pub previous_mode: AppMode,
    pub input_title: String,
    pub input_description: String,
    pub input_due_date_relative: String,
    pub input_due_date_absolute: String,
    pub current_parent: Option<i64>,
    pub should_quit: bool,
    pub error_message: Option<String>,
    pub search_query: String,
    pub search_results: Vec<Todo>,
    pub search_list_state: ListState,
    pub search_matches: Vec<i64>,
    pub current_match_index: Option<usize>,
    pub search_opened_nodes: std::collections::HashSet<i64>,
    pub pre_search_expansion_state: std::collections::HashMap<i64, bool>,
    pub input_parent: String,
    pub selected_parent_id: Option<i64>,
    pub create_field_focus: CreateFieldFocus,
    pub use_tree_view: bool,
    pub search_input_mode: bool,
    pub move_todo_id: Option<i64>,
    pub editor_pending: Option<Todo>,
    pub show_hidden_items: bool,
    pub goto_query: String,
    pub goto_matches: Vec<i64>,
    pub goto_current_match_index: Option<usize>,
    pub list_scrollbar_state: ScrollbarState,
    pub tree_scrollbar_state: ScrollbarState,
    pub completed_scrollbar_state: ScrollbarState,
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
        let due_date_text = if let Some(due_by) = todo.due_by {
            due_by.with_timezone(&Local).format("%Y-%m-%d %H:%M").to_string()
        } else {
            "Not set".to_string()
        };

        let markdown_content = format!(
            "# {}\n\n## Due Date\n{}\n\n## Description\n{}\n\n## Metadata\n- **ID:** {}\n- **Status:** {}\n- **Created:** {} UTC\n",
            todo.title,
            due_date_text,
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
    
    fn get_editor_command(&self) -> String {
        std::env::var("EDITOR")
            .or_else(|_| std::env::var("VISUAL"))
            .unwrap_or_else(|_| {
                // Fallback priority: vim -> nano -> vi
                if std::process::Command::new("vim").arg("--version").output().is_ok() {
                    "vim".to_string()
                } else if std::process::Command::new("nano").arg("--version").output().is_ok() {
                    "nano".to_string()
                } else {
                    "vi".to_string()
                }
            })
    }

    pub fn launch_editor<B: ratatui::backend::Backend + std::io::Write>(&mut self, todo: &Todo, terminal: &mut ratatui::Terminal<B>) -> Result<(), String> {
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
        
        // Get editor command
        let editor_cmd = self.get_editor_command();
        
        // Launch editor and WAIT for it to complete (foreground process)
        let status = Command::new(&editor_cmd)
            .arg(&file_path)
            .stdin(std::process::Stdio::inherit())
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .status()
            .map_err(|e| format!("Failed to launch editor '{}': {}", editor_cmd, e))?;
        
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
            return Err(format!("Editor '{}' exited with error", editor_cmd));
        }
        
        // Read back the edited content and update database
        if let Ok(edited_content) = std::fs::read_to_string(&file_path) {
            match self.parse_markdown(&edited_content) {
                Ok((new_title, new_description, new_due_date)) => {
                    if new_title != todo.title || new_description != todo.description || new_due_date != todo.due_by {
                        if let Err(e) = self.database.update_todo(todo.id, new_title, new_description, new_due_date) {
                            return Err(format!("Failed to update todo: {}", e));
                        } else {
                            // Force a checkpoint to ensure changes are written to disk immediately
                            let _ = self.database.checkpoint();
                            let _ = self.refresh_todos();
                        }
                    }
                }
                Err(e) => {
                    // Show error message to user about parsing failure
                    return Err(e);
                }
            }
        }
        
        Ok(())
    }
    
    fn parse_markdown(&self, content: &str) -> Result<(String, String, Option<DateTime<Utc>>), String> {
        let lines: Vec<&str> = content.lines().collect();
        let mut title = String::new();
        let mut description = String::new();
        let mut due_date = None;

        let mut in_description = false;
        let mut in_due_date = false;

        for line in lines {
            if line.starts_with("# ") && title.is_empty() {
                title = line[2..].trim().to_string();
            } else if line.starts_with("## Due Date") {
                in_due_date = true;
                in_description = false;
            } else if line.starts_with("## Description") {
                in_description = true;
                in_due_date = false;
            } else if line.starts_with("## Metadata") {
                // Stop collecting description when we hit the metadata section
                in_description = false;
                in_due_date = false;
            } else if in_due_date && !line.trim().is_empty() {
                // Parse due date from the line
                let date_str = line.trim();
                if date_str != "Not set" {
                    due_date = Self::parse_due_date(date_str);
                    // If parsing failed and it wasn't "Not set", return error
                    if due_date.is_none() {
                        return Err(format!("Invalid due date format: '{}'. Expected format: 'YYYY-MM-DD HH:MM', '2d', '1w', etc., or 'Not set'", date_str));
                    }
                }
                in_due_date = false; // Only parse first non-empty line
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

        Ok((title, description.trim().to_string(), due_date))
    }

    fn parse_due_date(input: &str) -> Option<DateTime<Utc>> {
        let input = input.trim();
        if input.is_empty() {
            return None;
        }

        // Try relative date parsing first (e.g., "2d", "1w", "3h", "30m")
        if let Some(duration) = Self::parse_relative_duration(input) {
            return Some(Utc::now() + duration);
        }

        // Try absolute date parsing
        // Format: "YYYY-MM-DD" or "YYYY-MM-DD HH:MM"
        if let Ok(dt) = chrono::NaiveDate::parse_from_str(input, "%Y-%m-%d") {
            // Parse date only, set time to end of day (23:59:59)
            let naive_datetime = dt.and_hms_opt(23, 59, 59)?;
            return Some(DateTime::<Utc>::from_naive_utc_and_offset(naive_datetime, Utc));
        }

        if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(input, "%Y-%m-%d %H:%M") {
            return Some(DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc));
        }

        None
    }

    fn parse_relative_duration(input: &str) -> Option<Duration> {
        let input = input.trim().to_lowercase();

        if input.is_empty() {
            return None;
        }

        // First, try parsing as a bare number (default to days)
        if let Ok(number) = input.parse::<i64>() {
            return Some(Duration::days(number));
        }

        // Extract number and unit
        let len = input.len();
        if len < 2 {
            return None;
        }

        let unit = &input[len - 1..];
        let number_str = &input[..len - 1];

        let number: i64 = number_str.parse().ok()?;

        match unit {
            "m" => Some(Duration::minutes(number)),
            "h" => Some(Duration::hours(number)),
            "d" => Some(Duration::days(number)),
            "w" => Some(Duration::weeks(number)),
            _ => None,
        }
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
            previous_mode: AppMode::List,
            input_title: String::new(),
            input_description: String::new(),
            input_due_date_relative: String::new(),
            input_due_date_absolute: String::new(),
            current_parent: None,
            should_quit: false,
            error_message: None,
            search_query: String::new(),
            search_results: Vec::new(),
            search_list_state: ListState::default(),
            search_matches: Vec::new(),
            current_match_index: None,
            search_opened_nodes: std::collections::HashSet::new(),
            pre_search_expansion_state: std::collections::HashMap::new(),
            input_parent: String::new(),
            selected_parent_id: None,
            create_field_focus: CreateFieldFocus::Title,
            use_tree_view: true,
            search_input_mode: false,
            move_todo_id: None,
            editor_pending: None,
            show_hidden_items: false,
            goto_query: String::new(),
            goto_matches: Vec::new(),
            goto_current_match_index: None,
            list_scrollbar_state: ScrollbarState::default(),
            tree_scrollbar_state: ScrollbarState::default(),
            completed_scrollbar_state: ScrollbarState::default(),
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
        self.tree_manager.rebuild_from_todos_with_hidden_filter(all_todos, self.show_hidden_items);
        
        // Initialize tree selection if we have items
        if !self.tree_manager.get_rendered_lines().is_empty() && self.tree_list_state.selected().is_none() {
            self.tree_list_state.select(Some(0));
        }
        
        Ok(())
    }

    pub fn update_scrollbar_states(&mut self) {
        // Update list scrollbar
        let list_len = self.incomplete_todos.len();
        self.list_scrollbar_state = self.list_scrollbar_state
            .content_length(list_len)
            .position(self.list_state.selected().unwrap_or(0));

        // Update tree scrollbar
        let tree_len = self.tree_manager.get_rendered_lines().len();
        self.tree_scrollbar_state = self.tree_scrollbar_state
            .content_length(tree_len)
            .position(self.tree_list_state.selected().unwrap_or(0));

        // Update completed scrollbar
        let completed_len = self.completed_todos.len();
        self.completed_scrollbar_state = self.completed_scrollbar_state
            .content_length(completed_len)
            .position(self.completed_list_state.selected().unwrap_or(0));
    }

    fn get_due_date_style(&self, todo: &Todo) -> Color {
        // Only color incomplete todos based on due date
        if todo.is_completed() {
            return CatppuccinFrappe::COMPLETED;
        }

        if let Some(due_by) = todo.due_by {
            let now = Utc::now();
            let diff = due_by.signed_duration_since(now);

            if diff.num_seconds() < 0 {
                // Past due - RED
                CatppuccinFrappe::RED
            } else if diff.num_days() < 7 {
                // Due within 1 week (less than 7 days) - TEAL
                CatppuccinFrappe::TEAL
            } else {
                // More than 1 week away (>= 7 days) - default color
                CatppuccinFrappe::INCOMPLETE
            }
        } else {
            // No due date - default color
            CatppuccinFrappe::INCOMPLETE
        }
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
            self.current_match_index = None;
        } else {
            let new_matches: Vec<i64> = self.database.search_todos(&self.search_query)?
                .into_iter()
                .map(|todo| todo.id)
                .collect();
            
            // Only re-sort if matches have actually changed
            if new_matches != self.search_matches {
                self.search_matches = new_matches;
                // Sort matches by their appearance order in the tree
                self.sort_matches_by_tree_order();
                
                // Find closest match to current selection or start from first match
                self.current_match_index = if self.search_matches.is_empty() {
                    None
                } else {
                    Some(self.find_closest_match_index())
                };

                // Automatically move cursor to the current match
                if let Some(current_match_index) = self.current_match_index {
                    if let Some(&match_todo_id) = self.search_matches.get(current_match_index) {
                        if let Some(line_index) = self.tree_manager.get_line_index_for_todo(match_todo_id) {
                            self.tree_list_state.select(Some(line_index));
                        }
                    }
                }
            }
        }
        Ok(())
    }
    
    fn sort_matches_by_tree_order(&mut self) {
        let rendered_lines = self.tree_manager.get_rendered_lines();
        let line_order: std::collections::HashMap<i64, usize> = rendered_lines
            .iter()
            .enumerate()
            .map(|(idx, line)| (line.todo_id, idx))
            .collect();
            
        self.search_matches.sort_by_key(|&todo_id| {
            line_order.get(&todo_id).copied().unwrap_or(usize::MAX)
        });
    }
    
    fn find_closest_match_index(&self) -> usize {
        if self.search_matches.is_empty() {
            return 0;
        }
        
        let current_selection = self.tree_list_state.selected().unwrap_or(0);
        let rendered_lines = self.tree_manager.get_rendered_lines();
        
        if let Some(current_line) = rendered_lines.get(current_selection) {
            let current_todo_id = current_line.todo_id;
            
            // If current selection is a match, use it
            if let Some(pos) = self.search_matches.iter().position(|&id| id == current_todo_id) {
                return pos;
            }
        }
        
        // Otherwise, find the closest match by tree position
        let line_positions: std::collections::HashMap<i64, usize> = rendered_lines
            .iter()
            .enumerate()
            .map(|(idx, line)| (line.todo_id, idx))
            .collect();
            
        let mut best_match = 0;
        let mut best_distance = usize::MAX;
        
        for (idx, &match_id) in self.search_matches.iter().enumerate() {
            if let Some(&match_pos) = line_positions.get(&match_id) {
                let distance = if match_pos >= current_selection {
                    match_pos - current_selection
                } else {
                    current_selection - match_pos
                };
                
                if distance < best_distance {
                    best_distance = distance;
                    best_match = idx;
                }
            }
        }
        
        best_match
    }
    
    fn navigate_to_next_match(&mut self) {
        if self.search_matches.is_empty() {
            return;
        }
        
        let start_index = self.current_match_index.unwrap_or(0);
        let matches_len = self.search_matches.len();
        
        // Try to find the next visible match, starting from the next position
        for i in 1..=matches_len {
            let next_index = (start_index + i) % matches_len;
            
            if let Some(&match_todo_id) = self.search_matches.get(next_index) {
                // First check if it's already visible
                if let Some(line_index) = self.tree_manager.get_line_index_for_todo(match_todo_id) {
                    self.current_match_index = Some(next_index);
                    self.tree_list_state.select(Some(line_index));
                    return;
                }
                
                // If not visible, expand parent nodes to make it visible
                let opened_nodes = self.expand_path_to_todo(match_todo_id);
                if !opened_nodes.is_empty() {
                    // Track which nodes we opened for search
                    for node_id in opened_nodes {
                        self.search_opened_nodes.insert(node_id);
                    }
                    
                    // After expanding, it should be visible now
                    if let Some(line_index) = self.tree_manager.get_line_index_for_todo(match_todo_id) {
                        self.current_match_index = Some(next_index);
                        self.tree_list_state.select(Some(line_index));
                        return;
                    }
                }
            }
        }
    }
    
    fn navigate_to_previous_match(&mut self) {
        if self.search_matches.is_empty() {
            return;
        }
        
        let start_index = self.current_match_index.unwrap_or(0);
        let matches_len = self.search_matches.len();
        
        // Try to find the previous visible match, starting from the previous position
        for i in 1..=matches_len {
            let prev_index = if start_index >= i {
                start_index - i
            } else {
                matches_len - (i - start_index)
            };
            
            if let Some(&match_todo_id) = self.search_matches.get(prev_index) {
                // First check if it's already visible
                if let Some(line_index) = self.tree_manager.get_line_index_for_todo(match_todo_id) {
                    self.current_match_index = Some(prev_index);
                    self.tree_list_state.select(Some(line_index));
                    return;
                }
                
                // If not visible, expand parent nodes to make it visible
                let opened_nodes = self.expand_path_to_todo(match_todo_id);
                if !opened_nodes.is_empty() {
                    // Track which nodes we opened for search
                    for node_id in opened_nodes {
                        self.search_opened_nodes.insert(node_id);
                    }
                    
                    // After expanding, it should be visible now
                    if let Some(line_index) = self.tree_manager.get_line_index_for_todo(match_todo_id) {
                        self.current_match_index = Some(prev_index);
                        self.tree_list_state.select(Some(line_index));
                        return;
                    }
                }
            }
        }
    }
    
    
    fn expand_path_to_todo(&mut self, todo_id: i64) -> Vec<i64> {
        self.tree_manager.expand_path_to_todo(todo_id)
    }
    
    fn restore_pre_search_expansion_state(&mut self) {
        let mut needs_rebuild = false;
        
        // For each node we opened during search, restore its original state
        for &node_id in &self.search_opened_nodes {
            // Get the original state (default to false if it wasn't tracked)
            let original_state = self.pre_search_expansion_state.get(&node_id).copied().unwrap_or(false);
            let current_state = self.tree_manager.expansion_states.get(&node_id).copied().unwrap_or(false);
            
            // Only change if current state differs from original
            if current_state != original_state {
                if original_state {
                    self.tree_manager.expansion_states.insert(node_id, true);
                } else {
                    self.tree_manager.expansion_states.remove(&node_id);
                }
                needs_rebuild = true;
            }
        }
        
        // Clear tracking data
        self.search_opened_nodes.clear();
        self.pre_search_expansion_state.clear();
        
        // Rebuild tree if we made changes
        if needs_rebuild {
            let all_todos = self.tree_manager.todos.values().cloned().collect();
            self.tree_manager.rebuild_from_todos_with_hidden_filter(all_todos, self.show_hidden_items);
        }
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

    fn update_goto_matches(&mut self) -> anyhow::Result<()> {
        if self.goto_query.is_empty() {
            self.goto_matches.clear();
            self.goto_current_match_index = None;
        } else {
            // Parse the goto query as a number
            if let Ok(target_id_mod) = self.goto_query.parse::<i64>() {
                // Only search within currently visible todos in the tree
                let rendered_lines = self.tree_manager.get_rendered_lines();
                let new_matches: Vec<i64> = rendered_lines
                    .iter()
                    .filter_map(|line| {
                        self.tree_manager.get_todo_by_id(line.todo_id)
                            .filter(|todo| todo.id_mod() == target_id_mod)
                            .map(|_| line.todo_id)
                    })
                    .collect();

                // Only re-sort if matches have actually changed
                if new_matches != self.goto_matches {
                    self.goto_matches = new_matches;
                    // Sort matches by their appearance order in the tree
                    self.sort_goto_matches_by_tree_order();

                    // Find closest match to current selection or start from first match
                    self.goto_current_match_index = if self.goto_matches.is_empty() {
                        None
                    } else {
                        Some(self.find_closest_goto_match_index())
                    };

                    // Automatically move cursor to the current match
                    if let Some(current_match_index) = self.goto_current_match_index {
                        if let Some(&match_todo_id) = self.goto_matches.get(current_match_index) {
                            if let Some(line_index) = self.tree_manager.get_line_index_for_todo(match_todo_id) {
                                self.tree_list_state.select(Some(line_index));
                            }
                        }
                    }
                }
            } else {
                self.goto_matches.clear();
                self.goto_current_match_index = None;
            }
        }
        Ok(())
    }

    fn sort_goto_matches_by_tree_order(&mut self) {
        let rendered_lines = self.tree_manager.get_rendered_lines();
        let line_order: std::collections::HashMap<i64, usize> = rendered_lines
            .iter()
            .enumerate()
            .map(|(idx, line)| (line.todo_id, idx))
            .collect();

        self.goto_matches.sort_by_key(|&todo_id| {
            line_order.get(&todo_id).copied().unwrap_or(usize::MAX)
        });
    }

    fn find_closest_goto_match_index(&self) -> usize {
        if self.goto_matches.is_empty() {
            return 0;
        }

        let current_selection = self.tree_list_state.selected().unwrap_or(0);
        let rendered_lines = self.tree_manager.get_rendered_lines();

        if let Some(current_line) = rendered_lines.get(current_selection) {
            let current_todo_id = current_line.todo_id;

            // If current selection is a match, use it
            if let Some(pos) = self.goto_matches.iter().position(|&id| id == current_todo_id) {
                return pos;
            }
        }

        // Otherwise, find the closest match by tree position
        let line_positions: std::collections::HashMap<i64, usize> = rendered_lines
            .iter()
            .enumerate()
            .map(|(idx, line)| (line.todo_id, idx))
            .collect();

        let mut best_match = 0;
        let mut best_distance = usize::MAX;

        for (idx, &match_id) in self.goto_matches.iter().enumerate() {
            if let Some(&match_pos) = line_positions.get(&match_id) {
                let distance = if match_pos >= current_selection {
                    match_pos - current_selection
                } else {
                    current_selection - match_pos
                };

                if distance < best_distance {
                    best_distance = distance;
                    best_match = idx;
                }
            }
        }

        best_match
    }

    fn navigate_to_next_goto_match(&mut self) {
        if self.goto_matches.is_empty() {
            return;
        }

        let start_index = self.goto_current_match_index.unwrap_or(0);
        let matches_len = self.goto_matches.len();
        let next_index = (start_index + 1) % matches_len;

        if let Some(&match_todo_id) = self.goto_matches.get(next_index) {
            if let Some(line_index) = self.tree_manager.get_line_index_for_todo(match_todo_id) {
                self.goto_current_match_index = Some(next_index);
                self.tree_list_state.select(Some(line_index));
            }
        }
    }

    fn navigate_to_previous_goto_match(&mut self) {
        if self.goto_matches.is_empty() {
            return;
        }

        let start_index = self.goto_current_match_index.unwrap_or(0);
        let matches_len = self.goto_matches.len();
        let prev_index = if start_index == 0 {
            matches_len - 1
        } else {
            start_index - 1
        };

        if let Some(&match_todo_id) = self.goto_matches.get(prev_index) {
            if let Some(line_index) = self.tree_manager.get_line_index_for_todo(match_todo_id) {
                self.goto_current_match_index = Some(prev_index);
                self.tree_list_state.select(Some(line_index));
            }
        }
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

                    // In move mode, index 0 is the virtual ROOT entry
                    if self.mode == AppMode::Move && selected == 0 {
                        return None; // ROOT is not a real todo
                    }

                    let rendered_lines = self.tree_manager.get_rendered_lines();
                    let tree_index = if self.mode == AppMode::Move { selected - 1 } else { selected };
                    let line = rendered_lines.get(tree_index)?;
                    self.tree_manager.get_todo_by_id(line.todo_id)
                } else {
                    let todos = self.get_current_todos();
                    let selected = self.get_current_list_state().selected()?;
                    todos.get(selected)
                }
            }
        }
    }

    pub fn handle_key_event(&mut self, key: KeyCode, modifiers: KeyModifiers) -> anyhow::Result<()> {
        self.error_message = None;

        // Global help key - available from any mode except Help itself and text input modes
        let is_in_text_input_mode = match self.mode {
            AppMode::Create => true,
            AppMode::ListFind if self.search_input_mode => true,
            AppMode::TreeSearch if self.search_input_mode => true,
            AppMode::IdModGoto if self.search_input_mode => true,
            AppMode::ParentSearch => true,
            _ => false,
        };

        if key == KeyCode::Char('a') && self.mode != AppMode::Help && !is_in_text_input_mode {
            self.previous_mode = self.mode.clone();
            self.mode = AppMode::Help;
            return Ok(());
        }

        // Handle Ctrl+d: half-page scroll down
        if key == KeyCode::Char('d') && modifiers.contains(KeyModifiers::CONTROL) && !is_in_text_input_mode {
            self.half_page_down();
            return Ok(());
        }

        // Handle Ctrl+u: half-page scroll up
        if key == KeyCode::Char('u') && modifiers.contains(KeyModifiers::CONTROL) && !is_in_text_input_mode {
            self.half_page_up();
            return Ok(());
        }

        // Handle 'h' key: toggle hidden status of selected todo in tree view
        if key == KeyCode::Char('h') && self.mode != AppMode::Help && !is_in_text_input_mode && self.use_tree_view {
            if let Some(todo) = self.get_selected_todo() {
                let todo_id = todo.id;
                if let Err(e) = self.database.toggle_todo_hidden(todo_id) {
                    self.error_message = Some(format!("Failed to toggle hidden status: {}", e));
                } else {
                    self.refresh_todos()?;
                    self.update_selection_after_refresh();
                }
            }
            return Ok(());
        }

        // Handle 'H' key: toggle showing/hiding hidden items in tree view
        if key == KeyCode::Char('H') && self.mode != AppMode::Help && !is_in_text_input_mode && self.use_tree_view {
            self.show_hidden_items = !self.show_hidden_items;
            self.refresh_todos()?;
            self.update_selection_after_refresh();
            return Ok(());
        }

        // Handle 'g' key: goto mode for id_mod navigation in tree view
        if key == KeyCode::Char('g') && self.mode != AppMode::Help && !is_in_text_input_mode && self.use_tree_view {
            self.mode = AppMode::IdModGoto;
            self.goto_query.clear();
            self.goto_matches.clear();
            self.goto_current_match_index = None;
            self.search_input_mode = true;
            return Ok(());
        }

        match self.mode {
            AppMode::List => self.handle_list_key(key)?,
            AppMode::CompletedView => self.handle_completed_view_key(key)?,
            AppMode::Create => self.handle_create_key(key)?,
            AppMode::ConfirmDelete => self.handle_delete_key(key)?,
            AppMode::ListFind => self.handle_list_find_key(key)?,
            AppMode::TreeSearch => self.handle_tree_search_key(key)?,
            AppMode::ParentSearch => self.handle_parent_search_key(key)?,
            AppMode::Move => self.handle_move_key(key)?,
            AppMode::Help => self.handle_help_key(key)?,
            AppMode::IdModGoto => self.handle_idmod_goto_key(key)?,
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
                self.search_input_mode = true;
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
                self.search_matches.clear();
                self.current_match_index = None;
                
                // Capture current expansion state before starting search
                self.pre_search_expansion_state = self.tree_manager.expansion_states.clone();
                self.search_opened_nodes.clear();
                
                self.search_input_mode = true;
            }
            KeyCode::Char('n') => {
                self.mode = AppMode::Create;
                self.input_title.clear();
                self.input_description.clear();
                self.input_due_date_relative.clear();
                self.input_due_date_absolute.clear();
                self.create_field_focus = CreateFieldFocus::Title;
                
                // Auto-fill parent field with currently highlighted task
                if let Some(selected_todo) = self.get_selected_todo() {
                    let todo_id = selected_todo.id;
                    let todo_title = selected_todo.title.clone();
                    
                    self.selected_parent_id = Some(todo_id);
                    let parent_display = if todo_title.len() > 40 {
                        format!("{}...", &todo_title[..37])
                    } else {
                        todo_title
                    };
                    self.input_parent = format!("ID:{} {}", todo_id, parent_display);
                } else {
                    // No selection, clear parent fields
                    self.input_parent.clear();
                    self.selected_parent_id = None;
                }
            }
            KeyCode::Char('d') => {
                if self.get_current_list_state().selected().is_some() {
                    self.mode = AppMode::ConfirmDelete;
                }
            }
            KeyCode::Char('m') => {
                if self.use_tree_view {
                    if let Some(todo) = self.get_selected_todo() {
                        self.move_todo_id = Some(todo.id);
                        self.mode = AppMode::Move;
                        // Find and highlight the current parent (or first valid parent if root)
                        self.highlight_current_parent_for_move();
                    }
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
                    self.editor_pending = Some(todo.clone());
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
                    self.editor_pending = Some(todo.clone());
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



    fn handle_create_key(&mut self, key: KeyCode) -> anyhow::Result<()> {
        match key {
            KeyCode::Esc => self.mode = AppMode::List,
            KeyCode::Enter => {
                if !self.input_title.trim().is_empty() {
                    // Try parsing from relative field first, then absolute field
                    let due_by = if !self.input_due_date_relative.trim().is_empty() {
                        Self::parse_due_date(&self.input_due_date_relative)
                    } else if !self.input_due_date_absolute.trim().is_empty() {
                        Self::parse_due_date(&self.input_due_date_absolute)
                    } else {
                        None
                    };
                    let new_todo = NewTodo {
                        title: self.input_title.clone(),
                        description: self.input_description.clone(),
                        parent_id: self.selected_parent_id,
                        due_by,
                    };
                    self.database.create_todo(new_todo)?;
                    self.refresh_todos()?;
                    self.mode = AppMode::List;
                    self.input_title.clear();
                    self.input_parent.clear();
                    self.input_description.clear();
                    self.input_due_date_relative.clear();
                    self.input_due_date_absolute.clear();
                    self.selected_parent_id = None;
                    self.create_field_focus = CreateFieldFocus::Title;
                } else {
                    self.error_message = Some("Title cannot be empty".to_string());
                }
            }
            KeyCode::Tab => {
                match self.create_field_focus {
                    CreateFieldFocus::Title => {
                        self.create_field_focus = CreateFieldFocus::DueDateRelative;
                    }
                    CreateFieldFocus::DueDateRelative => {
                        self.create_field_focus = CreateFieldFocus::DueDateAbsolute;
                    }
                    CreateFieldFocus::DueDateAbsolute => {
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
                    CreateFieldFocus::DueDateRelative => {
                        self.input_due_date_relative.push(c);
                        // Sync to absolute field
                        if let Some(due_date) = Self::parse_due_date(&self.input_due_date_relative) {
                            self.input_due_date_absolute = due_date.with_timezone(&Local).format("%Y-%m-%d %H:%M").to_string();
                        }
                    }
                    CreateFieldFocus::DueDateAbsolute => {
                        self.input_due_date_absolute.push(c);
                        // Sync to relative field - calculate time difference in days (default unit)
                        if let Some(due_date) = Self::parse_due_date(&self.input_due_date_absolute) {
                            let now = Utc::now();
                            let diff = due_date.signed_duration_since(now);
                            let days = diff.num_days();

                            // Default to days, show 0 if less than a day
                            self.input_due_date_relative = format!("{}", days.max(0));
                        }
                    }
                    CreateFieldFocus::Description => {
                        self.input_description.push(c);
                    }
                    CreateFieldFocus::Parent => {
                        if c == 'r' {
                            // Clear parent field on 'r' key
                            self.input_parent.clear();
                            self.selected_parent_id = None;
                        } else {
                            // Enter parent search mode when typing in parent field
                            self.mode = AppMode::ParentSearch;
                            self.search_query.clear();
                            self.search_query.push(c);
                            self.update_search_results()?;
                        }
                    }
                }
            }
            KeyCode::Backspace => {
                match self.create_field_focus {
                    CreateFieldFocus::Title => {
                        self.input_title.pop();
                    }
                    CreateFieldFocus::DueDateRelative => {
                        self.input_due_date_relative.pop();
                        // Sync to absolute field
                        if let Some(due_date) = Self::parse_due_date(&self.input_due_date_relative) {
                            self.input_due_date_absolute = due_date.with_timezone(&Local).format("%Y-%m-%d %H:%M").to_string();
                        } else {
                            self.input_due_date_absolute.clear();
                        }
                    }
                    CreateFieldFocus::DueDateAbsolute => {
                        self.input_due_date_absolute.pop();
                        // Sync to relative field - calculate time difference in days (default unit)
                        if let Some(due_date) = Self::parse_due_date(&self.input_due_date_absolute) {
                            let now = Utc::now();
                            let diff = due_date.signed_duration_since(now);
                            let days = diff.num_days();

                            // Default to days, show 0 if less than a day
                            self.input_due_date_relative = format!("{}", days.max(0));
                        } else {
                            self.input_due_date_relative.clear();
                        }
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
                    // Check if the task has children before deleting
                    if self.database.has_children(todo.id)? {
                        self.error_message = Some("Cannot delete: task has children. Delete children first.".to_string());
                    } else {
                        self.database.delete_todo(todo.id)?;
                        self.refresh_todos()?;
                        self.update_selection_after_refresh();
                    }
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

    fn half_page_down(&mut self) {
        if self.use_tree_view {
            self.half_page_down_tree();
        } else {
            self.half_page_down_list();
        }
    }

    fn half_page_up(&mut self) {
        if self.use_tree_view {
            self.half_page_up_tree();
        } else {
            self.half_page_up_list();
        }
    }

    fn half_page_down_tree(&mut self) {
        let lines_len = self.tree_manager.get_rendered_lines().len();
        if lines_len == 0 {
            return;
        }

        let current = self.tree_list_state.selected().unwrap_or(0);
        let jump_size = 10; // Half page size - could be made configurable
        let new_pos = std::cmp::min(current + jump_size, lines_len.saturating_sub(1));
        self.tree_list_state.select(Some(new_pos));
    }

    fn half_page_up_tree(&mut self) {
        let lines_len = self.tree_manager.get_rendered_lines().len();
        if lines_len == 0 {
            return;
        }

        let current = self.tree_list_state.selected().unwrap_or(0);
        let jump_size = 10; // Half page size - could be made configurable
        let new_pos = if current >= jump_size {
            current - jump_size
        } else {
            0
        };
        self.tree_list_state.select(Some(new_pos));
    }

    fn half_page_down_list(&mut self) {
        let list_len = self.incomplete_todos.len();
        if list_len == 0 {
            return;
        }

        let current = self.list_state.selected().unwrap_or(0);
        let jump_size = 10; // Half page size - could be made configurable
        let new_pos = std::cmp::min(current + jump_size, list_len.saturating_sub(1));
        self.list_state.select(Some(new_pos));
    }

    fn half_page_up_list(&mut self) {
        let list_len = self.incomplete_todos.len();
        if list_len == 0 {
            return;
        }

        let current = self.list_state.selected().unwrap_or(0);
        let jump_size = 10; // Half page size - could be made configurable
        let new_pos = if current >= jump_size {
            current - jump_size
        } else {
            0
        };
        self.list_state.select(Some(new_pos));
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
                self.search_input_mode = false;
            }
            KeyCode::Enter => {
                if self.search_input_mode {
                    // Finish input mode, enable navigation
                    self.search_input_mode = false;
                    self.update_search_results()?;
                } else {
                    // If there's a selected result, view/edit it with editor
                    if let Some(selected) = self.search_list_state.selected() {
                        if let Some(todo) = self.search_results.get(selected) {
                            self.editor_pending = Some(todo.clone());
                        }
                    }
                }
            }
            KeyCode::Backspace => {
                if self.search_input_mode {
                    self.search_query.pop();
                    self.update_search_results()?;
                }
            }
            KeyCode::Char(c) => {
                if self.search_input_mode {
                    // In input mode, all characters go to search
                    self.search_query.push(c);
                    self.update_search_results()?;
                } else {
                    // In navigation mode, handle navigation keys
                    match c {
                        'j' => self.next_search_result(),
                        'k' => self.previous_search_result(),
                        _ => {
                            // Any other character goes to search input when not in input mode
                            // Re-enter input mode
                            self.search_input_mode = true;
                            self.search_query.push(c);
                            self.update_search_results()?;
                        }
                    }
                }
            }
            // Arrow keys always work for navigation regardless of mode
            KeyCode::Down => {
                if !self.search_input_mode {
                    self.next_search_result();
                }
            }
            KeyCode::Up => {
                if !self.search_input_mode {
                    self.previous_search_result();
                }
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
                self.current_match_index = None;
                self.search_input_mode = false;
                
                // Restore original expansion state for nodes we opened during search
                self.restore_pre_search_expansion_state();
            }
            KeyCode::Enter => {
                if self.search_input_mode {
                    // Finish input mode, enable navigation
                    self.search_input_mode = false;
                    self.update_tree_search_matches()?;
                } else {
                    // If there's a selected todo in tree, view/edit it with editor
                    if let Some(todo) = self.get_selected_todo() {
                        self.editor_pending = Some(todo.clone());
                    }
                }
            }
            KeyCode::Backspace => {
                if self.search_input_mode {
                    self.search_query.pop();
                    self.update_tree_search_matches()?;
                }
            }
            KeyCode::Char(c) => {
                if self.search_input_mode {
                    // In input mode, all characters go to search
                    self.search_query.push(c);
                    self.update_tree_search_matches()?;
                } else {
                    // In navigation mode, handle navigation keys
                    match c {
                        'j' => {
                            if self.use_tree_view {
                                self.next_tree_item();
                            } else {
                                self.next_todo();
                            }
                        }
                        'k' => {
                            if self.use_tree_view {
                                self.previous_tree_item();
                            } else {
                                self.previous_todo();
                            }
                        }
                        'h' => {
                            if self.current_parent.is_some() {
                                self.current_parent = None;
                                self.refresh_todos()?;
                                self.update_tree_search_matches()?;
                                if !self.incomplete_todos.is_empty() {
                                    self.list_state.select(Some(0));
                                    if self.use_tree_view {
                                        self.tree_list_state.select(Some(0));
                                    }
                                }
                            }
                        }
                        'l' => {
                            if let Some(todo) = self.get_selected_todo() {
                                self.current_parent = Some(todo.id);
                                self.refresh_todos()?;
                                self.update_tree_search_matches()?;
                                if !self.incomplete_todos.is_empty() {
                                    self.list_state.select(Some(0));
                                    if self.use_tree_view {
                                        self.tree_list_state.select(Some(0));
                                    }
                                }
                            }
                        }
                        't' => {
                            // Allow tree expansion/collapse during search with 't' key
                            if self.use_tree_view {
                                if let Some(selected) = self.tree_list_state.selected() {
                                    if let Some(line) = self.tree_manager.get_rendered_lines().get(selected) {
                                        if line.has_children {
                                            self.tree_manager.toggle_expansion(line.todo_id);
                                            self.update_tree_selection_after_toggle(selected);
                                        }
                                    }
                                }
                            }
                        }
                        'n' => {
                            // Navigate to next search match (vim-like behavior)
                            self.navigate_to_next_match();
                        }
                        'N' => {
                            // Navigate to previous search match (vim-like behavior)
                            self.navigate_to_previous_match();
                        }
                        ' ' => {
                            // Allow toggling completion during search
                            if let Some(todo) = self.get_selected_todo() {
                                let todo_id = todo.id;
                                let is_currently_completed = todo.is_completed();
                                
                                if is_currently_completed {
                                    self.database.uncomplete_todo(todo_id)?;
                                } else {
                                    self.database.complete_todo(todo_id)?;
                                }
                                
                                if self.use_tree_view {
                                    self.tree_manager.update_todo_completion(todo_id, !is_currently_completed);
                                }
                                
                                self.refresh_todos()?;
                                self.update_selection_after_refresh();
                                self.update_tree_search_matches()?;
                            }
                        }
                        _ => {
                            // Any other character goes to search input when not in input mode
                            // Re-enter input mode
                            self.search_input_mode = true;
                            self.search_query.push(c);
                            self.update_tree_search_matches()?;
                        }
                    }
                }
            }
            // Arrow keys always work for navigation regardless of mode
            KeyCode::Down => {
                if !self.search_input_mode {
                    if self.use_tree_view {
                        self.next_tree_item();
                    } else {
                        self.next_todo();
                    }
                }
            }
            KeyCode::Up => {
                if !self.search_input_mode {
                    if self.use_tree_view {
                        self.previous_tree_item();
                    } else {
                        self.previous_todo();
                    }
                }
            }
            KeyCode::Left => {
                if !self.search_input_mode && self.current_parent.is_some() {
                    self.current_parent = None;
                    self.refresh_todos()?;
                    self.update_tree_search_matches()?;
                    if !self.incomplete_todos.is_empty() {
                        self.list_state.select(Some(0));
                        if self.use_tree_view {
                            self.tree_list_state.select(Some(0));
                        }
                    }
                }
            }
            KeyCode::Right => {
                if !self.search_input_mode {
                    if let Some(todo) = self.get_selected_todo() {
                        self.current_parent = Some(todo.id);
                        self.refresh_todos()?;
                        self.update_tree_search_matches()?;
                        if !self.incomplete_todos.is_empty() {
                            self.list_state.select(Some(0));
                            if self.use_tree_view {
                                self.tree_list_state.select(Some(0));
                            }
                        }
                    }
                }
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

    fn handle_idmod_goto_key(&mut self, key: KeyCode) -> anyhow::Result<()> {
        match key {
            KeyCode::Esc => {
                self.mode = AppMode::List;
                self.goto_query.clear();
                self.goto_matches.clear();
                self.goto_current_match_index = None;
                self.search_input_mode = false;
            }
            KeyCode::Enter => {
                if self.search_input_mode {
                    // Finish input mode, enable navigation
                    self.search_input_mode = false;
                    self.update_goto_matches()?;
                } else {
                    // If there's a selected todo, view/edit it with editor
                    if let Some(todo) = self.get_selected_todo() {
                        self.editor_pending = Some(todo.clone());
                    }
                }
            }
            KeyCode::Backspace => {
                if self.search_input_mode {
                    self.goto_query.pop();
                    self.update_goto_matches()?;
                }
            }
            KeyCode::Char(c) => {
                if self.search_input_mode {
                    // Only allow digits
                    if c.is_ascii_digit() {
                        self.goto_query.push(c);
                        self.update_goto_matches()?;
                    }
                } else {
                    // In navigation mode, handle navigation keys
                    match c {
                        'j' => {
                            if self.use_tree_view {
                                self.next_tree_item();
                            }
                        }
                        'k' => {
                            if self.use_tree_view {
                                self.previous_tree_item();
                            }
                        }
                        'n' => {
                            // Navigate to next goto match
                            self.navigate_to_next_goto_match();
                        }
                        'N' => {
                            // Navigate to previous goto match
                            self.navigate_to_previous_goto_match();
                        }
                        ' ' => {
                            // Allow toggling completion during goto
                            if let Some(todo) = self.get_selected_todo() {
                                let todo_id = todo.id;
                                let is_currently_completed = todo.is_completed();

                                if is_currently_completed {
                                    self.database.uncomplete_todo(todo_id)?;
                                } else {
                                    self.database.complete_todo(todo_id)?;
                                }

                                if self.use_tree_view {
                                    self.tree_manager.update_todo_completion(todo_id, !is_currently_completed);
                                }

                                self.refresh_todos()?;
                                self.update_selection_after_refresh();
                                self.update_goto_matches()?;
                            }
                        }
                        _ => {
                            // Any other character goes to goto input when not in input mode
                            // Re-enter input mode
                            if c.is_ascii_digit() {
                                self.search_input_mode = true;
                                self.goto_query.push(c);
                                self.update_goto_matches()?;
                            }
                        }
                    }
                }
            }
            // Arrow keys always work for navigation regardless of mode
            KeyCode::Down => {
                if !self.search_input_mode {
                    if self.use_tree_view {
                        self.next_tree_item();
                    }
                }
            }
            KeyCode::Up => {
                if !self.search_input_mode {
                    if self.use_tree_view {
                        self.previous_tree_item();
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_move_key(&mut self, key: KeyCode) -> anyhow::Result<()> {
        match key {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.mode = AppMode::List;
                self.move_todo_id = None;
            }
            KeyCode::Down | KeyCode::Char('j') => {
                // Move to next valid parent candidate in tree
                self.move_to_next_valid_parent();
            }
            KeyCode::Up | KeyCode::Char('k') => {
                // Move to previous valid parent candidate in tree
                self.move_to_previous_valid_parent();
            }
            KeyCode::Enter => {
                if let Some(move_todo_id) = self.move_todo_id {
                    let new_parent_id = if self.is_highlighting_root_position() {
                        None // Move to root level
                    } else if let Some(highlighted_todo) = self.get_selected_todo() {
                        Some(highlighted_todo.id)
                    } else {
                        return Ok(()); // No valid selection
                    };

                    match self.database.move_todo(move_todo_id, new_parent_id) {
                        Ok(()) => {
                            self.refresh_todos()?;
                            self.mode = AppMode::List;
                            self.move_todo_id = None;
                        }
                        Err(e) => {
                            self.error_message = Some(format!("Cannot move todo: {}", e));
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_help_key(&mut self, key: KeyCode) -> anyhow::Result<()> {
        match key {
            KeyCode::Esc | KeyCode::Char('a') | KeyCode::Char('q') => {
                self.mode = self.previous_mode.clone();
            }
            _ => {}
        }
        Ok(())
    }

    fn highlight_current_parent_for_move(&mut self) {
        if let Some(move_todo_id) = self.move_todo_id {
            // Find the todo being moved
            if let Some(todo) = self.incomplete_todos.iter().find(|t| t.id == move_todo_id) {
                if let Some(parent_id) = todo.parent_id {
                    // Find the parent in the tree and highlight it
                    if let Some(parent_index) = self.find_todo_index_in_tree(parent_id) {
                        self.tree_list_state.select(Some(parent_index));
                        return;
                    }
                } else {
                    // Todo has no parent, so it's at root level - highlight ROOT
                    self.tree_list_state.select(Some(0));
                    return;
                }
            }
            // If no parent or parent not found, highlight the first valid candidate
            self.move_to_first_valid_parent();
        }
    }

    fn move_to_next_valid_parent(&mut self) {
        let rendered_lines = self.tree_manager.get_rendered_lines();
        let total_items = if self.mode == AppMode::Move { rendered_lines.len() + 1 } else { rendered_lines.len() };

        if let Some(current_selection) = self.tree_list_state.selected() {
            let mut next_index = (current_selection + 1) % total_items;

            // Find next valid parent candidate
            while !self.is_valid_parent_candidate_at_index(next_index) {
                next_index = (next_index + 1) % total_items;
                if next_index == current_selection {
                    break; // Avoid infinite loop
                }
            }

            self.tree_list_state.select(Some(next_index));
        }
    }

    fn move_to_previous_valid_parent(&mut self) {
        let rendered_lines = self.tree_manager.get_rendered_lines();
        let total_items = if self.mode == AppMode::Move { rendered_lines.len() + 1 } else { rendered_lines.len() };

        if let Some(current_selection) = self.tree_list_state.selected() {
            let mut prev_index = if current_selection == 0 {
                total_items - 1
            } else {
                current_selection - 1
            };

            // Find previous valid parent candidate
            while !self.is_valid_parent_candidate_at_index(prev_index) {
                prev_index = if prev_index == 0 {
                    total_items - 1
                } else {
                    prev_index - 1
                };
                if prev_index == current_selection {
                    break; // Avoid infinite loop
                }
            }

            self.tree_list_state.select(Some(prev_index));
        }
    }

    fn move_to_first_valid_parent(&mut self) {
        let rendered_lines = self.tree_manager.get_rendered_lines();
        let total_items = if self.mode == AppMode::Move { rendered_lines.len() + 1 } else { rendered_lines.len() };

        for index in 0..total_items {
            if self.is_valid_parent_candidate_at_index(index) {
                self.tree_list_state.select(Some(index));
                return;
            }
        }
    }

    fn is_valid_parent_candidate_at_index(&self, index: usize) -> bool {
        if let Some(move_todo_id) = self.move_todo_id {
            // In move mode, index 0 is always the virtual ROOT entry
            if self.mode == AppMode::Move && index == 0 {
                return true; // ROOT is always a valid parent
            }

            let rendered_lines = self.tree_manager.get_rendered_lines();
            let tree_index = if self.mode == AppMode::Move { index - 1 } else { index };

            if tree_index < rendered_lines.len() {
                let line = &rendered_lines[tree_index];
                let todo_id = line.todo_id;

                // Cannot move to itself or its descendants
                return todo_id != move_todo_id && !self.is_descendant_of(todo_id, move_todo_id);
            }
        }
        false
    }

    fn is_highlighting_root_position(&self) -> bool {
        if self.mode == AppMode::Move {
            if let Some(selected) = self.tree_list_state.selected() {
                return selected == 0; // First item is the virtual ROOT
            }
        }
        false
    }

    fn find_todo_index_in_tree(&self, todo_id: i64) -> Option<usize> {
        let rendered_lines = self.tree_manager.get_rendered_lines();
        for (tree_index, line) in rendered_lines.iter().enumerate() {
            if line.todo_id == todo_id {
                // In move mode, add 1 to account for the virtual ROOT entry at index 0
                let index = if self.mode == AppMode::Move { tree_index + 1 } else { tree_index };
                return Some(index);
            }
        }
        None
    }

    fn is_descendant_of(&self, potential_descendant: i64, ancestor: i64) -> bool {
        // Check if potential_descendant is a descendant of ancestor
        for todo in &self.incomplete_todos {
            if todo.id == potential_descendant {
                let mut current_parent = todo.parent_id;
                while let Some(parent_id) = current_parent {
                    if parent_id == ancestor {
                        return true;
                    }
                    // Find the parent todo
                    if let Some(parent_todo) = self.incomplete_todos.iter().find(|t| t.id == parent_id) {
                        current_parent = parent_todo.parent_id;
                    } else {
                        break;
                    }
                }
                break;
            }
        }
        false
    }


    pub fn draw(&mut self, f: &mut Frame) {
        // Update scrollbar states before drawing
        self.update_scrollbar_states();

        if self.mode == AppMode::Help {
            // Help mode takes full screen
            self.draw_help_page(f, f.area());
            return;
        }

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
            AppMode::IdModGoto => {
                if self.use_tree_view {
                    self.draw_idmod_goto_view(f, chunks[0]);
                } else {
                    self.draw_split_todo_lists(f, chunks[0]);
                }
            }
            AppMode::CompletedView => self.draw_completed_view(f, chunks[0]),
            AppMode::Create => self.draw_create_mode(f, chunks[0]),
            AppMode::ConfirmDelete => self.draw_confirm_delete(f, chunks[0]),
            AppMode::ListFind => self.draw_list_find_mode(f, chunks[0]),
            AppMode::ParentSearch => self.draw_parent_search_mode(f, chunks[0]),
            AppMode::Move => {
                // In move mode, just draw the tree view with special highlighting
                if self.use_tree_view {
                    self.draw_tree_view(f, chunks[0]);
                } else {
                    self.draw_split_todo_lists(f, chunks[0]);
                }
            }
            AppMode::Help => {
                // This case is handled above, but needed for exhaustive matching
                unreachable!();
            }
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
                let due_by_text = if let Some(due_by) = todo.due_by {
                    format!(" | Due: {}", due_by.with_timezone(&Local).format("%m/%d %H:%M"))
                } else {
                    String::new()
                };
                let parent_title = self.database.get_parent_title(todo.parent_id)
                    .unwrap_or(None)
                    .unwrap_or_else(|| "null".to_string());

                ListItem::new(Line::from(vec![
                    Span::styled(format!("{} [ ] ", todo.id_mod()), Style::default().fg(CatppuccinFrappe::SUBTEXT1)),
                    Span::styled(todo.title.clone(), Style::default().fg(self.get_due_date_style(todo))),
                    Span::styled(format!(" | Created: {}{} | Parent: {}", created_time, due_by_text, parent_title),
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

        // Split area to make room for scrollbar
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(0), Constraint::Length(1)])
            .split(area);

        f.render_stateful_widget(list, chunks[0], &mut self.list_state);

        // Draw scrollbar
        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"))
            .style(Style::default().fg(CatppuccinFrappe::SURFACE2))
            .thumb_style(Style::default().fg(CatppuccinFrappe::SUBTEXT1));

        f.render_stateful_widget(scrollbar, chunks[1], &mut self.list_scrollbar_state);
    }


    fn draw_tree_view(&mut self, f: &mut Frame, area: Rect) {
        let rendered_lines = self.tree_manager.get_rendered_lines();

        let mut items: Vec<ListItem> = Vec::new();

        // Add virtual ROOT entry at the top in move mode
        if self.mode == AppMode::Move {
            let root_style = Style::default().fg(CatppuccinFrappe::GREEN).add_modifier(Modifier::BOLD);
            items.push(ListItem::new(Line::from(vec![
                Span::styled("ROOT", root_style),
                Span::styled(" (Move here to make top-level)", Style::default().fg(CatppuccinFrappe::SUBTEXT1)),
            ])));
        }

        // Add the regular tree items, adjusting index for move mode
        let tree_items: Vec<ListItem> = rendered_lines
            .iter()
            .enumerate()
            .map(|(tree_index, line)| {
                let index = if self.mode == AppMode::Move { tree_index + 1 } else { tree_index };
                if let Some(todo) = self.tree_manager.get_todo_by_id(line.todo_id) {
                    let created_time = todo.created_at.with_timezone(&Local).format("%m/%d %H:%M").to_string();
                    let due_by_text = if let Some(due_by) = todo.due_by {
                        format!(" | Due: {}", due_by.with_timezone(&Local).format("%m/%d %H:%M"))
                    } else {
                        String::new()
                    };

                    let (display_style, prefix_style) = if todo.hidden && self.show_hidden_items {
                        // Hidden items shown with italic styling
                        if todo.is_completed() {
                            (
                                Style::default().fg(CatppuccinFrappe::COMPLETED).add_modifier(Modifier::CROSSED_OUT).add_modifier(Modifier::ITALIC),
                                Style::default().fg(CatppuccinFrappe::SURFACE2).add_modifier(Modifier::ITALIC)
                            )
                        } else {
                            (
                                Style::default().fg(self.get_due_date_style(todo)).add_modifier(Modifier::ITALIC),
                                Style::default().fg(CatppuccinFrappe::PARENT_INDICATOR).add_modifier(Modifier::ITALIC)
                            )
                        }
                    } else if todo.is_completed() {
                        (
                            Style::default().fg(CatppuccinFrappe::COMPLETED).add_modifier(Modifier::CROSSED_OUT),
                            Style::default().fg(CatppuccinFrappe::SURFACE2)
                        )
                    } else {
                        // In move mode, highlight valid parent candidates differently
                        if self.mode == AppMode::Move && self.is_valid_parent_candidate_at_index(index) {
                            (
                                Style::default().fg(CatppuccinFrappe::GREEN), // Green for valid move targets
                                Style::default().fg(CatppuccinFrappe::GREEN)
                            )
                        } else if self.mode == AppMode::Move && Some(todo.id) == self.move_todo_id {
                            (
                                Style::default().fg(CatppuccinFrappe::YELLOW), // Yellow for item being moved
                                Style::default().fg(CatppuccinFrappe::YELLOW)
                            )
                        } else {
                            (
                                Style::default().fg(self.get_due_date_style(todo)),
                                Style::default().fg(CatppuccinFrappe::PARENT_INDICATOR)
                            )
                        }
                    };

                    ListItem::new(Line::from(vec![
                        Span::styled(&line.prefix, prefix_style),
                        Span::styled(&line.display_text, display_style),
                        Span::styled(format!(" | Created: {}{}", created_time, due_by_text),
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

        // Append tree items to the main items list
        items.extend(tree_items);

        let title = if self.mode == AppMode::Move {
            if let Some(move_todo_id) = self.move_todo_id {
                if let Some(todo) = self.incomplete_todos.iter().find(|t| t.id == move_todo_id) {
                    format!("Move '{}' - Green=Valid Parents, j/k=Navigate, Enter=Confirm", todo.title)
                } else {
                    "Move Mode - Green=Valid Parents, j/k=Navigate, Enter=Confirm".to_string()
                }
            } else {
                "Move Mode - Green=Valid Parents, j/k=Navigate, Enter=Confirm".to_string()
            }
        } else {
            if self.show_hidden_items {
                "Todo Tree View (All Items + Hidden)".to_string()
            } else {
                "Todo Tree View (All Items)".to_string()
            }
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

        // Split area to make room for scrollbar
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(0), Constraint::Length(1)])
            .split(area);

        f.render_stateful_widget(list, chunks[0], &mut self.tree_list_state);

        // Draw scrollbar
        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"))
            .style(Style::default().fg(CatppuccinFrappe::SURFACE2))
            .thumb_style(Style::default().fg(CatppuccinFrappe::SUBTEXT1));

        f.render_stateful_widget(scrollbar, chunks[1], &mut self.tree_scrollbar_state);
    }

    fn draw_idmod_goto_view(&mut self, f: &mut Frame, area: Rect) {
        // Split area to make room for goto input at bottom
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(3)])
            .split(area);

        // Draw tree view with goto highlighting in the main area
        self.draw_tree_view_with_goto_highlights(f, chunks[0]);

        // Draw goto input at bottom
        let goto_input = Paragraph::new(self.goto_query.as_str())
            .block(Block::default()
                .borders(Borders::ALL)
                .title("Goto ID (digits only)")
                .border_style(Style::default().fg(CatppuccinFrappe::SAPPHIRE)))
            .style(Style::default().fg(CatppuccinFrappe::TEXT));
        f.render_widget(goto_input, chunks[1]);
    }

    fn draw_tree_view_with_goto_highlights(&mut self, f: &mut Frame, area: Rect) {
        let rendered_lines = self.tree_manager.get_rendered_lines();

        let items: Vec<ListItem> = rendered_lines
            .iter()
            .map(|line| {
                if let Some(todo) = self.tree_manager.get_todo_by_id(line.todo_id) {
                    let created_time = todo.created_at.with_timezone(&Local).format("%m/%d %H:%M").to_string();
                    let due_by_text = if let Some(due_by) = todo.due_by {
                        format!(" | Due: {}", due_by.with_timezone(&Local).format("%m/%d %H:%M"))
                    } else {
                        String::new()
                    };

                    // Check if this todo matches the goto query
                    let is_match = self.goto_matches.contains(&line.todo_id);
                    let is_current_match = self.goto_current_match_index
                        .and_then(|idx| self.goto_matches.get(idx))
                        .map(|&match_id| match_id == line.todo_id)
                        .unwrap_or(false);

                    let (display_style, prefix_style) = if todo.hidden && self.show_hidden_items {
                        // Hidden items shown with italic styling
                        if todo.is_completed() {
                            (
                                Style::default().fg(CatppuccinFrappe::COMPLETED).add_modifier(Modifier::CROSSED_OUT).add_modifier(Modifier::ITALIC),
                                Style::default().fg(CatppuccinFrappe::SURFACE2).add_modifier(Modifier::ITALIC)
                            )
                        } else {
                            (
                                Style::default().fg(self.get_due_date_style(todo)).add_modifier(Modifier::ITALIC),
                                Style::default().fg(CatppuccinFrappe::PARENT_INDICATOR).add_modifier(Modifier::ITALIC)
                            )
                        }
                    } else if todo.is_completed() {
                        (
                            if is_current_match {
                                // Current match - bright yellow and underlined
                                Style::default().fg(CatppuccinFrappe::YELLOW).add_modifier(Modifier::CROSSED_OUT).add_modifier(Modifier::BOLD).add_modifier(Modifier::UNDERLINED)
                            } else if is_match {
                                // Other matches - highlighted but less prominent
                                Style::default().fg(CatppuccinFrappe::YELLOW).add_modifier(Modifier::CROSSED_OUT).add_modifier(Modifier::BOLD)
                            } else {
                                Style::default().fg(CatppuccinFrappe::COMPLETED).add_modifier(Modifier::CROSSED_OUT)
                            },
                            Style::default().fg(CatppuccinFrappe::SURFACE2)
                        )
                    } else {
                        (
                            if is_current_match {
                                // Current match - bright yellow and underlined
                                Style::default().fg(CatppuccinFrappe::YELLOW).add_modifier(Modifier::BOLD).add_modifier(Modifier::UNDERLINED)
                            } else if is_match {
                                // Other matches - yellow and bold
                                Style::default().fg(CatppuccinFrappe::YELLOW).add_modifier(Modifier::BOLD)
                            } else {
                                Style::default().fg(self.get_due_date_style(todo))
                            },
                            Style::default().fg(CatppuccinFrappe::PARENT_INDICATOR)
                        )
                    };

                    ListItem::new(Line::from(vec![
                        Span::styled(&line.prefix, prefix_style),
                        Span::styled(&line.display_text, display_style),
                        Span::styled(format!(" | Created: {}{}", created_time, due_by_text),
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

        let title = if self.goto_query.is_empty() {
            "Todo Tree View - Goto Mode".to_string()
        } else {
            match self.goto_current_match_index {
                Some(current_idx) if !self.goto_matches.is_empty() => {
                    format!("Goto {} - Match {}/{} (n: next, N: prev)",
                        self.goto_query, current_idx + 1, self.goto_matches.len())
                }
                _ => {
                    format!("Goto {} - {} matches (n: next, N: prev)",
                        self.goto_query, self.goto_matches.len())
                }
            }
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

        // Split area to make room for scrollbar
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(0), Constraint::Length(1)])
            .split(area);

        f.render_stateful_widget(list, chunks[0], &mut self.tree_list_state);

        // Draw scrollbar
        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"))
            .style(Style::default().fg(CatppuccinFrappe::SURFACE2))
            .thumb_style(Style::default().fg(CatppuccinFrappe::SUBTEXT1));

        f.render_stateful_widget(scrollbar, chunks[1], &mut self.tree_scrollbar_state);
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
                    let due_by_text = if let Some(due_by) = todo.due_by {
                        format!(" | Due: {}", due_by.with_timezone(&Local).format("%m/%d %H:%M"))
                    } else {
                        String::new()
                    };

                    // Check if this todo matches the search
                    let is_match = self.search_matches.contains(&line.todo_id);
                    let is_current_match = self.current_match_index
                        .and_then(|idx| self.search_matches.get(idx))
                        .map(|&match_id| match_id == line.todo_id)
                        .unwrap_or(false);
                    
                    let (display_style, prefix_style) = if todo.is_completed() {
                        (
                            if is_current_match {
                                // Current match - bright yellow and underlined
                                Style::default().fg(CatppuccinFrappe::RED).add_modifier(Modifier::CROSSED_OUT).add_modifier(Modifier::BOLD).add_modifier(Modifier::UNDERLINED)
                            } else if is_match {
                                // Other matches - highlighted but less prominent
                                Style::default().fg(CatppuccinFrappe::YELLOW).add_modifier(Modifier::CROSSED_OUT).add_modifier(Modifier::BOLD)
                            } else {
                                Style::default().fg(CatppuccinFrappe::COMPLETED).add_modifier(Modifier::CROSSED_OUT)
                            },
                            Style::default().fg(CatppuccinFrappe::SURFACE2)
                        )
                    } else {
                        (
                            if is_current_match {
                                // Current match - bright yellow and underlined
                                Style::default().fg(CatppuccinFrappe::YELLOW).add_modifier(Modifier::BOLD).add_modifier(Modifier::UNDERLINED)
                            } else if is_match {
                                // Other matches - yellow and bold
                                Style::default().fg(CatppuccinFrappe::YELLOW).add_modifier(Modifier::BOLD)
                            } else {
                                Style::default().fg(self.get_due_date_style(todo))
                            },
                            Style::default().fg(CatppuccinFrappe::PARENT_INDICATOR)
                        )
                    };

                    ListItem::new(Line::from(vec![
                        Span::styled(&line.prefix, prefix_style),
                        Span::styled(&line.display_text, display_style),
                        Span::styled(format!(" | Created: {}{}", created_time, due_by_text),
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
            match self.current_match_index {
                Some(current_idx) if !self.search_matches.is_empty() => {
                    format!("Tree Search - Match {}/{} (n: next, N: prev)", 
                        current_idx + 1, self.search_matches.len())
                }
                _ => {
                    format!("Tree Search - {} matches (n: next, N: prev)", self.search_matches.len())
                }
            }
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

        // Split area to make room for scrollbar
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(0), Constraint::Length(1)])
            .split(area);

        f.render_stateful_widget(list, chunks[0], &mut self.tree_list_state);

        // Draw scrollbar
        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"))
            .style(Style::default().fg(CatppuccinFrappe::SURFACE2))
            .thumb_style(Style::default().fg(CatppuccinFrappe::SUBTEXT1));

        f.render_stateful_widget(scrollbar, chunks[1], &mut self.tree_scrollbar_state);
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
                let due_by_text = if let Some(due_by) = todo.due_by {
                    format!(" | Due: {}", due_by.with_timezone(&Local).format("%m/%d %H:%M"))
                } else {
                    String::new()
                };
                let parent_title = self.database.get_parent_title(todo.parent_id)
                    .unwrap_or(None)
                    .unwrap_or_else(|| "null".to_string());

                ListItem::new(Line::from(vec![
                    Span::styled(format!("{} [✓] ", todo.id_mod()),
                               Style::default().fg(CatppuccinFrappe::COMPLETED)),
                    Span::styled(
                        todo.title.clone(),
                        Style::default().fg(CatppuccinFrappe::COMPLETED).add_modifier(Modifier::CROSSED_OUT)
                    ),
                    Span::styled(
                        format!(" | Created: {} | Completed: {}{} | Parent: {}",
                               created_time, completed_time, due_by_text, parent_title),
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

        // Split area to make room for scrollbar
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(0), Constraint::Length(1)])
            .split(area);

        f.render_stateful_widget(list, chunks[0], &mut self.completed_list_state);

        // Draw scrollbar
        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"))
            .style(Style::default().fg(CatppuccinFrappe::SURFACE2))
            .thumb_style(Style::default().fg(CatppuccinFrappe::SUBTEXT1));

        f.render_stateful_widget(scrollbar, chunks[1], &mut self.completed_scrollbar_state);
    }



    fn draw_create_mode(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Length(3), Constraint::Length(3), Constraint::Min(0)])
            .split(area);

        // Title field
        let title_style = if self.create_field_focus == CreateFieldFocus::Title {
            Style::default().fg(CatppuccinFrappe::YELLOW)
        } else {
            Style::default().fg(CatppuccinFrappe::BORDER)
        };
        let title_display = if self.input_title.is_empty() {
            "e.g., 'p0 Fix critical bug' (p0=highest priority)".to_string()
        } else {
            self.input_title.clone()
        };
        let title_input = Paragraph::new(title_display.as_str())
            .block(Block::default().borders(Borders::ALL).title("Title").border_style(title_style))
            .style(Style::default().fg(CatppuccinFrappe::TEXT));
        f.render_widget(title_input, chunks[0]);

        // Due Date fields - split into two side-by-side boxes
        let date_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[1]);

        // Relative date field (left)
        let relative_style = if self.create_field_focus == CreateFieldFocus::DueDateRelative {
            Style::default().fg(CatppuccinFrappe::YELLOW)
        } else {
            Style::default().fg(CatppuccinFrappe::BORDER)
        };
        let relative_display = if self.input_due_date_relative.is_empty() {
            "e.g., '2' (2 days), '1w', '3h'".to_string()
        } else {
            self.input_due_date_relative.clone()
        };
        let relative_input = Paragraph::new(relative_display.as_str())
            .block(Block::default().borders(Borders::ALL).title("Relative (optional)").border_style(relative_style))
            .style(Style::default().fg(CatppuccinFrappe::TEXT));
        f.render_widget(relative_input, date_chunks[0]);

        // Absolute date field (right)
        let absolute_style = if self.create_field_focus == CreateFieldFocus::DueDateAbsolute {
            Style::default().fg(CatppuccinFrappe::YELLOW)
        } else {
            Style::default().fg(CatppuccinFrappe::BORDER)
        };
        let absolute_display = if self.input_due_date_absolute.is_empty() {
            "e.g., '2025-10-20 14:30'".to_string()
        } else {
            self.input_due_date_absolute.clone()
        };
        let absolute_input = Paragraph::new(absolute_display.as_str())
            .block(Block::default().borders(Borders::ALL).title("Absolute (optional)").border_style(absolute_style))
            .style(Style::default().fg(CatppuccinFrappe::TEXT));
        f.render_widget(absolute_input, date_chunks[1]);

        // Parent field  
        let parent_style = if self.create_field_focus == CreateFieldFocus::Parent {
            Style::default().fg(CatppuccinFrappe::YELLOW)
        } else {
            Style::default().fg(CatppuccinFrappe::BORDER)
        };
        let parent_display = if self.input_parent.is_empty() {
            "Press Tab to focus, type to search for parent, 'r' to clear...".to_string()
        } else {
            self.input_parent.clone()
        };
        let parent_input = Paragraph::new(parent_display.as_str())
            .block(Block::default().borders(Borders::ALL).title("Parent (optional)").border_style(parent_style))
            .style(Style::default().fg(CatppuccinFrappe::TEXT));
        f.render_widget(parent_input, chunks[2]);

        // Description field
        let desc_style = if self.create_field_focus == CreateFieldFocus::Description {
            Style::default().fg(CatppuccinFrappe::YELLOW)
        } else {
            Style::default().fg(CatppuccinFrappe::BORDER)
        };
        let description_input = Paragraph::new(self.input_description.as_str())
            .block(Block::default().borders(Borders::ALL).title("Description (optional)").border_style(desc_style))
            .style(Style::default().fg(CatppuccinFrappe::TEXT));
        f.render_widget(description_input, chunks[3]);
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
                let due_by_text = if let Some(due_by) = todo.due_by {
                    format!(" | Due: {}", due_by.with_timezone(&Local).format("%m/%d %H:%M"))
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
                    Span::raw(format!("{} {} ", todo.id_mod(), status_icon)),
                    Span::styled(todo.title.clone(), title_style),
                    Span::raw(format!(" | Created: {}{}{} | Parent: {}", created_time, due_by_text, completed_time, parent_title)),
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
                let due_by_text = if let Some(due_by) = todo.due_by {
                    format!(" | Due: {}", due_by.with_timezone(&Local).format("%m/%d %H:%M"))
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
                    Span::raw(format!("{} {} ", todo.id_mod(), status_icon)),
                    Span::styled(todo.title.clone(), title_style),
                    Span::raw(format!(" | Created: {}{}{} | Parent: {}", created_time, due_by_text, completed_time, parent_title)),
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

    fn draw_help_page(&self, f: &mut Frame, area: Rect) {
        // Create a centered popup
        let popup_area = centered_rect(80, 70, area);
        
        // Clear the background
        f.render_widget(Clear, popup_area);
        
        let help_content = vec![
            "NAVIGATION".to_string(),
            "  j/k or ↑/↓      Navigate todos".to_string(),
            "  Ctrl+d/Ctrl+u   Half-page scroll down/up".to_string(),
            "  h/l or ←/→      Navigate hierarchy levels".to_string(),
            "  t               Expand/Collapse tree nodes".to_string(),
            "".to_string(),
            "ACTIONS".to_string(),
            "  Space           Toggle completion status".to_string(),
            "  Enter           View/Edit todo in $EDITOR".to_string(),
            "  n               Create new todo".to_string(),
            "  d               Delete selected todo".to_string(),
            "  m               Move todo (tree view only)".to_string(),
            "  c               Show/hide completed todos".to_string(),
            "  h               Toggle hidden status (tree view only)".to_string(),
            "  H               Toggle showing/hiding hidden todos (tree view only)".to_string(),
            "".to_string(),
            "SEARCH & MODES".to_string(),
            "  /               Tree search with live highlighting".to_string(),
            "  f               List search (flat view)".to_string(),
            "  g               Goto ID mode (tree view only)".to_string(),
            "  n/N             Navigate search matches (in search/goto mode)".to_string(),
            "".to_string(),
            "GENERAL".to_string(),
            "  a               Show/hide this help page".to_string(),
            "  q               Quit application".to_string(),
            "  Esc             Cancel current operation".to_string(),
            "".to_string(),
            "Press a, Esc, or q to close this help".to_string(),
        ];
        
        let help_text = help_content.join("\n");
        
        let help_block = Paragraph::new(help_text)
            .block(Block::default()
                .borders(Borders::ALL)
                .title("TodoDB Help")
                .border_style(Style::default().fg(CatppuccinFrappe::BLUE)))
            .style(Style::default().fg(CatppuccinFrappe::TEXT))
            .wrap(Wrap { trim: true });
        
        f.render_widget(help_block, popup_area);
    }

    fn draw_help(&self, f: &mut Frame, area: Rect) {
        let help_text = "Press a for help | q to quit";

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