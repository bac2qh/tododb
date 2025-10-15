use crate::database::{Database, NewTodo};

pub fn test_functionality() -> anyhow::Result<()> {
    println!("Testing todo database functionality...");
    
    // Create test database
    let db = Database::new(":memory:")?;
    
    // Test creating todos
    let todo1_id = db.create_todo(NewTodo {
        title: "Learn Rust".to_string(),
        description: "Master the Rust programming language".to_string(),
        parent_id: None,
        due_by: None,
    })?;
    println!("Created todo 1 with ID: {}", todo1_id);
    
    let todo2_id = db.create_todo(NewTodo {
        title: "Build todo app".to_string(),
        description: "Create a terminal-based todo application".to_string(),
        parent_id: None,
        due_by: None,
    })?;
    println!("Created todo 2 with ID: {}", todo2_id);
    
    // Test creating subtodo
    let subtodo_id = db.create_todo(NewTodo {
        title: "Learn ownership".to_string(),
        description: "Understand Rust's ownership system".to_string(),
        parent_id: Some(todo1_id),
        due_by: None,
    })?;
    println!("Created subtodo with ID: {}", subtodo_id);
    
    // Test getting incomplete todos
    let incomplete = db.get_incomplete_todos(None)?;
    println!("Root incomplete todos: {}", incomplete.len());
    for todo in &incomplete {
        println!("  - {}: {}", todo.id, todo.title);
    }
    
    // Test completing a todo
    db.complete_todo(todo2_id)?;
    println!("Completed todo 2");
    
    // Test getting completed todos
    let completed = db.get_recent_completed_todos(None, 5)?;
    println!("Recent completed todos: {}", completed.len());
    for todo in &completed {
        println!("  - {}: {} (completed: {:?})", todo.id, todo.title, todo.completed_at);
    }
    
    // Test getting incomplete todos again
    let incomplete = db.get_incomplete_todos(None)?;
    println!("Root incomplete todos after completion: {}", incomplete.len());
    
    // Test subtodos
    let subtodos = db.get_incomplete_todos(Some(todo1_id))?;
    println!("Subtodos for todo 1: {}", subtodos.len());
    for todo in &subtodos {
        println!("  - {}: {}", todo.id, todo.title);
    }
    
    // Test WAL checkpoint functionality
    println!("Testing WAL checkpoint...");
    db.checkpoint()?;
    let (busy, log_size) = db.get_wal_info()?;
    println!("WAL info - busy: {}, log_size: {}", busy, log_size);
    
    // Final checkpoint before exit
    db.checkpoint_and_close()?;
    
    println!("All tests passed!");
    Ok(())
}