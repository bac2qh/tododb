use crate::database::{Database, NewTodo};
use crate::tree::TodoTreeManager;

pub fn test_tree_functionality() -> anyhow::Result<()> {
    println!("Testing tree functionality...");
    
    let database = Database::new("todos.db")?;
    
    // Create some hierarchical test data
    println!("Creating test hierarchical todos...");
    
    let project_id = database.create_todo(NewTodo {
        title: "Build Web Application".to_string(),
        description: "Main project".to_string(),
        parent_id: None,
        due_by: None,
    })?;
    
    let frontend_id = database.create_todo(NewTodo {
        title: "Frontend Development".to_string(),
        description: "UI and client-side logic".to_string(),
        parent_id: Some(project_id),
        due_by: None,
    })?;
    
    let backend_id = database.create_todo(NewTodo {
        title: "Backend Development".to_string(),
        description: "Server-side logic".to_string(),
        parent_id: Some(project_id),
        due_by: None,
    })?;
    
    let _react_id = database.create_todo(NewTodo {
        title: "Setup React".to_string(),
        description: "Initialize React project".to_string(),
        parent_id: Some(frontend_id),
        due_by: None,
    })?;
    
    let _styling_id = database.create_todo(NewTodo {
        title: "Add Styling".to_string(),
        description: "CSS and design".to_string(),
        parent_id: Some(frontend_id),
        due_by: None,
    })?;
    
    let _api_id = database.create_todo(NewTodo {
        title: "Create REST API".to_string(),
        description: "Backend API endpoints".to_string(),
        parent_id: Some(backend_id),
        due_by: None,
    })?;
    
    let _db_id = database.create_todo(NewTodo {
        title: "Setup Database".to_string(),
        description: "Configure database schema".to_string(),
        parent_id: Some(backend_id),
        due_by: None,
    })?;
    
    // Test tree functionality
    let mut tree_manager = TodoTreeManager::new();
    let all_todos = database.get_all_todos()?;
    tree_manager.rebuild_from_todos(all_todos);
    
    println!("\n=== Tree Structure ===");
    for line in tree_manager.get_rendered_lines() {
        println!("{}{}", line.prefix, line.display_text);
    }
    
    // Test completion update
    println!("\n=== Testing Completion Toggle ===");
    database.complete_todo(frontend_id)?;
    tree_manager.update_todo_completion(frontend_id, true);
    
    println!("After completing 'Frontend Development':");
    for line in tree_manager.get_rendered_lines() {
        println!("{}{}", line.prefix, line.display_text);
    }
    
    println!("\nTree functionality test completed successfully!");
    Ok(())
}