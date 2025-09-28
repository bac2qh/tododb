use crate::database::Todo;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct TreeNode {
    pub id: i64,
    pub children: Vec<TreeNode>,
    pub is_expanded: bool,
}

#[derive(Debug, Clone)]
pub struct RenderedLine {
    pub todo_id: i64,
    pub prefix: String,
    pub display_text: String,
    pub has_children: bool,
}

pub struct TodoTreeManager {
    pub tree: Vec<TreeNode>,
    pub todos: HashMap<i64, Todo>,
    pub rendered_lines: Vec<RenderedLine>,
    pub id_to_line: HashMap<i64, usize>,
    pub expansion_states: HashMap<i64, bool>,
}

impl TodoTreeManager {
    pub fn new() -> Self {
        Self {
            tree: Vec::new(),
            todos: HashMap::new(),
            rendered_lines: Vec::new(),
            id_to_line: HashMap::new(),
            expansion_states: HashMap::new(),
        }
    }

    pub fn rebuild_from_todos(&mut self, todos: Vec<Todo>) {
        self.rebuild_from_todos_with_hidden_filter(todos, false);
    }

    pub fn rebuild_from_todos_with_hidden_filter(&mut self, todos: Vec<Todo>, show_hidden: bool) {
        // Filter todos based on hidden status if show_hidden is false
        let filtered_todos: Vec<Todo> = if show_hidden {
            todos
        } else {
            todos.into_iter().filter(|todo| !todo.hidden).collect()
        };

        self.todos = filtered_todos.iter().map(|todo| (todo.id, todo.clone())).collect();
        self.tree = self.build_tree();
        self.rendered_lines = self.render_tree();
        self.id_to_line = self.rendered_lines
            .iter()
            .enumerate()
            .map(|(idx, line)| (line.todo_id, idx))
            .collect();
    }

    fn build_tree(&self) -> Vec<TreeNode> {
        let mut children_map: HashMap<Option<i64>, Vec<i64>> = HashMap::new();
        
        // Group todos by parent_id
        for todo in self.todos.values() {
            children_map.entry(todo.parent_id).or_insert_with(Vec::new).push(todo.id);
        }

        // Build tree starting from root nodes, but only include roots with incomplete work
        let root_nodes = self.build_subtree(&children_map, None);
        
        // Filter root nodes: only show those that are incomplete OR have incomplete descendants
        root_nodes.into_iter()
            .filter(|node| self.should_show_root_node(node))
            .collect()
    }

    fn build_subtree(&self, children_map: &HashMap<Option<i64>, Vec<i64>>, parent_id: Option<i64>) -> Vec<TreeNode> {
        let mut nodes = Vec::new();
        
        if let Some(child_ids) = children_map.get(&parent_id) {
            for &child_id in child_ids {
                if self.todos.contains_key(&child_id) {
                    let children = self.build_subtree(children_map, Some(child_id));
                    let has_incomplete_children = self.has_incomplete_descendants(&children);
                    
                    // Determine expansion state based on rules:
                    // - Expand if any child is incomplete
                    // - Collapse if all children are completed
                    // - Use saved state if exists, otherwise default based on children
                    let is_expanded = self.expansion_states.get(&child_id)
                        .copied()
                        .unwrap_or(has_incomplete_children);
                    
                    nodes.push(TreeNode {
                        id: child_id,
                        children,
                        is_expanded,
                    });
                }
            }
        }

        // Sort by creation time (most recent first)
        nodes.sort_by(|a, b| {
            let todo_a = &self.todos[&a.id];
            let todo_b = &self.todos[&b.id];
            todo_b.created_at.cmp(&todo_a.created_at)
        });

        nodes
    }

    fn should_show_root_node(&self, node: &TreeNode) -> bool {
        if let Some(todo) = self.todos.get(&node.id) {
            // Show root if it's incomplete OR has incomplete descendants
            !todo.is_completed() || self.has_incomplete_descendants(&node.children)
        } else {
            false
        }
    }

    fn has_incomplete_descendants(&self, children: &[TreeNode]) -> bool {
        for child in children {
            if let Some(todo) = self.todos.get(&child.id) {
                if !todo.is_completed() {
                    return true;
                }
            }
            
            // Recursively check descendants
            if self.has_incomplete_descendants(&child.children) {
                return true;
            }
        }
        false
    }

    fn render_tree(&self) -> Vec<RenderedLine> {
        let mut lines = Vec::new();
        
        for (i, root) in self.tree.iter().enumerate() {
            let is_last = i == self.tree.len() - 1;
            self.render_node(root, &mut lines, Vec::new(), is_last, 0);
        }
        
        lines
    }

    fn render_node(
        &self,
        node: &TreeNode,
        lines: &mut Vec<RenderedLine>,
        mut ancestor_continuations: Vec<bool>,
        is_last_sibling: bool,
        depth: usize,
    ) {
        if let Some(todo) = self.todos.get(&node.id) {
            // Generate prefix based on tree position
            let prefix = self.generate_prefix(&ancestor_continuations, is_last_sibling, depth);
            
            // Format todo display text with expansion indicator
            let status_icon = if todo.is_completed() { "[✓]" } else { "[ ]" };
            let expansion_indicator = if !node.children.is_empty() {
                if node.is_expanded { "▼ " } else { "▶ " }
            } else { "" };
            
            let display_text = format!("{} {} {}{}", todo.id_mod(), status_icon, expansion_indicator, todo.title);
            
            lines.push(RenderedLine {
                todo_id: node.id,
                prefix,
                display_text,
                has_children: !node.children.is_empty(),
            });

            // Render children only if expanded
            if !node.children.is_empty() && node.is_expanded {
                ancestor_continuations.push(!is_last_sibling);
                
                for (i, child) in node.children.iter().enumerate() {
                    let is_last_child = i == node.children.len() - 1;
                    self.render_node(child, lines, ancestor_continuations.clone(), is_last_child, depth + 1);
                }
                
                ancestor_continuations.pop();
            }
        }
    }

    fn generate_prefix(&self, ancestor_continuations: &[bool], is_last_sibling: bool, depth: usize) -> String {
        let mut prefix = String::new();
        
        // Add continuation lines for ancestor levels
        for &needs_continuation in ancestor_continuations {
            if needs_continuation {
                prefix.push_str("│   ");
            } else {
                prefix.push_str("    ");
            }
        }
        
        // Add the connector for this level
        if depth > 0 {
            if is_last_sibling {
                prefix.push_str("└── ");
            } else {
                prefix.push_str("├── ");
            }
        }
        
        prefix
    }

    pub fn get_rendered_lines(&self) -> &Vec<RenderedLine> {
        &self.rendered_lines
    }

    pub fn get_todo_by_id(&self, id: i64) -> Option<&Todo> {
        self.todos.get(&id)
    }

    pub fn get_line_index_for_todo(&self, todo_id: i64) -> Option<usize> {
        self.id_to_line.get(&todo_id).copied()
    }

    pub fn update_todo_completion(&mut self, todo_id: i64, is_completed: bool) {
        if let Some(todo) = self.todos.get_mut(&todo_id) {
            if is_completed {
                todo.completed_at = Some(chrono::Utc::now());
            } else {
                todo.completed_at = None;
            }
        }
        
        // Update only the affected line's display text (no tree rebuild needed)
        if let Some(&line_idx) = self.id_to_line.get(&todo_id) {
            if let Some(line) = self.rendered_lines.get_mut(line_idx) {
                if let Some(todo) = self.todos.get(&todo_id) {
                    let status_icon = if todo.is_completed() { "[✓]" } else { "[ ]" };
                    line.display_text = format!("{} {} {}", todo.id_mod(), status_icon, todo.title);
                }
            }
        }
    }

    pub fn toggle_expansion(&mut self, todo_id: i64) -> bool {
        // Find the node and toggle its expansion state
        if self.find_and_toggle_node(todo_id) {
            // Rebuild the rendered lines after toggling
            self.rendered_lines = self.render_tree();
            self.id_to_line = self.rendered_lines
                .iter()
                .enumerate()
                .map(|(idx, line)| (line.todo_id, idx))
                .collect();
            true
        } else {
            false
        }
    }

    fn find_and_toggle_node(&mut self, target_id: i64) -> bool {
        // Check if the node exists and has children
        if self.node_has_children(target_id) {
            // Toggle the expansion state in our tracking
            let current_state = self.expansion_states.get(&target_id).copied().unwrap_or(true);
            let new_state = !current_state;
            self.expansion_states.insert(target_id, new_state);
            
            // Rebuild the tree with new state
            self.tree = self.build_tree();
            true
        } else {
            false
        }
    }

    fn node_has_children(&self, target_id: i64) -> bool {
        // Check if this todo has any children in the database
        self.todos.values().any(|todo| todo.parent_id == Some(target_id))
    }
    
    pub fn expand_path_to_todo(&mut self, todo_id: i64) -> Vec<i64> {
        let mut opened_nodes = Vec::new();
        
        // Find the todo and expand all its ancestors
        if let Some(todo) = self.todos.get(&todo_id) {
            let mut current_parent_id = todo.parent_id;
            
            // Walk up the parent chain and expand each parent
            while let Some(parent_id) = current_parent_id {
                // Only expand if it wasn't already expanded
                let was_expanded = self.expansion_states.get(&parent_id).copied().unwrap_or(false);
                if !was_expanded {
                    self.expansion_states.insert(parent_id, true);
                    opened_nodes.push(parent_id);
                }
                
                // Find the next parent in the chain
                if let Some(parent_todo) = self.todos.get(&parent_id) {
                    current_parent_id = parent_todo.parent_id;
                } else {
                    break;
                }
            }
            
            if !opened_nodes.is_empty() {
                // Rebuild the tree with new expansion states
                self.tree = self.build_tree();
                self.rendered_lines = self.render_tree();
                self.id_to_line = self.rendered_lines
                    .iter()
                    .enumerate()
                    .map(|(idx, line)| (line.todo_id, idx))
                    .collect();
            }
        }
        
        opened_nodes
    }
}