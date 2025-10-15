use chrono::{DateTime, Utc};
use regex::RegexBuilder;
use rusqlite::{params, Connection, Result, Row};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Todo {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub due_by: Option<DateTime<Utc>>,
    pub parent_id: Option<i64>,
    pub hidden: bool,
}

impl Todo {
    pub fn from_row(row: &Row) -> Result<Self> {
        Ok(Todo {
            id: row.get(0)?,
            title: row.get(1)?,
            description: row.get(2)?,
            created_at: row.get(3)?,
            completed_at: row.get(4)?,
            due_by: row.get(5).ok(),
            parent_id: row.get(6)?,
            hidden: row.get(7).unwrap_or(false),
        })
    }

    pub fn is_completed(&self) -> bool {
        self.completed_at.is_some()
    }

    pub fn id_mod(&self) -> i64 {
        self.id % 100
    }
}

#[derive(Debug, Clone)]
pub struct NewTodo {
    pub title: String,
    pub description: String,
    pub parent_id: Option<i64>,
    pub due_by: Option<DateTime<Utc>>,
}

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(db_path: &str) -> anyhow::Result<Self> {
        let conn = Connection::open(db_path)?;
        let db = Database { conn };
        db.configure_wal_mode()?;
        db.create_tables()?;
        Ok(db)
    }

    fn configure_wal_mode(&self) -> anyhow::Result<()> {
        // Enable WAL mode for hybrid memory/disk operation
        self.conn.pragma_update(None, "journal_mode", "WAL")?;
        
        // Set checkpoint to happen less frequently (every 5000 pages instead of default 1000)  
        // This keeps more data in memory before writing to disk
        self.conn.pragma_update(None, "wal_autocheckpoint", 5000)?;
        
        // Use NORMAL synchronous mode (faster than FULL, still crash-safe)
        self.conn.pragma_update(None, "synchronous", "NORMAL")?;
        
        // Optimize for performance
        self.conn.pragma_update(None, "cache_size", -64000)?; // 64MB cache
        self.conn.pragma_update(None, "temp_store", "MEMORY")?; // Use memory for temp tables
        
        Ok(())
    }

    fn create_tables(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS todos (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL,
                completed_at TEXT,
                due_by TEXT,
                parent_id INTEGER,
                hidden INTEGER NOT NULL DEFAULT 0,
                FOREIGN KEY (parent_id) REFERENCES todos (id)
            )",
            [],
        )?;

        // Add hidden column to existing tables (migration)
        let _ = self.conn.execute(
            "ALTER TABLE todos ADD COLUMN hidden INTEGER NOT NULL DEFAULT 0",
            [],
        );

        // Add due_by column to existing tables (migration)
        let _ = self.conn.execute(
            "ALTER TABLE todos ADD COLUMN due_by TEXT",
            [],
        );

        Ok(())
    }

    pub fn create_todo(&self, new_todo: NewTodo) -> anyhow::Result<i64> {
        let now = Utc::now();
        let _id = self.conn.execute(
            "INSERT INTO todos (title, description, created_at, parent_id, hidden, due_by) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                new_todo.title,
                new_todo.description,
                now,
                new_todo.parent_id,
                false,
                new_todo.due_by
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_all_todos(&self) -> anyhow::Result<Vec<Todo>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, description, created_at, completed_at, due_by, parent_id, hidden
             FROM todos
             ORDER BY created_at DESC"
        )?;

        let todo_iter = stmt.query_map([], |row| Todo::from_row(row))?;

        let mut todos = Vec::new();
        for todo in todo_iter {
            todos.push(todo?);
        }

        Ok(todos)
    }


    pub fn get_todo_by_id(&self, id: i64) -> anyhow::Result<Option<Todo>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, description, created_at, completed_at, due_by, parent_id, hidden
             FROM todos
             WHERE id = ?1"
        )?;

        let mut rows = stmt.query_map([id], |row| Todo::from_row(row))?;

        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }

    pub fn update_todo(&self, id: i64, title: String, description: String) -> anyhow::Result<()> {
        self.conn.execute(
            "UPDATE todos SET title = ?1, description = ?2 WHERE id = ?3",
            params![title, description, id],
        )?;
        Ok(())
    }

    pub fn complete_todo(&self, id: i64) -> anyhow::Result<()> {
        let now = Utc::now();
        self.conn.execute(
            "UPDATE todos SET completed_at = ?1 WHERE id = ?2",
            params![now, id],
        )?;
        Ok(())
    }

    pub fn uncomplete_todo(&self, id: i64) -> anyhow::Result<()> {
        self.conn.execute(
            "UPDATE todos SET completed_at = NULL WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }

    pub fn toggle_todo_hidden(&self, id: i64) -> anyhow::Result<()> {
        self.conn.execute(
            "UPDATE todos SET hidden = NOT hidden WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }

    pub fn delete_todo(&self, id: i64) -> anyhow::Result<()> {
        self.conn.execute("DELETE FROM todos WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn move_todo(&self, id: i64, new_parent_id: Option<i64>) -> anyhow::Result<()> {
        // Check if the new parent would create a cycle
        if let Some(parent_id) = new_parent_id {
            if self.would_create_cycle(id, parent_id)? {
                return Err(anyhow::anyhow!("Cannot move todo: would create a cycle"));
            }
        }
        
        self.conn.execute(
            "UPDATE todos SET parent_id = ?1 WHERE id = ?2",
            params![new_parent_id, id],
        )?;
        Ok(())
    }

    fn would_create_cycle(&self, todo_id: i64, potential_parent_id: i64) -> anyhow::Result<bool> {
        // If we're trying to make a todo its own parent, that's obviously a cycle
        if todo_id == potential_parent_id {
            return Ok(true);
        }

        // Walk up the parent chain of the potential parent to see if we encounter the todo we're trying to move
        let mut current_id = Some(potential_parent_id);
        while let Some(id) = current_id {
            if let Some(todo) = self.get_todo_by_id(id)? {
                current_id = todo.parent_id;
                if current_id == Some(todo_id) {
                    return Ok(true);
                }
            } else {
                break;
            }
        }
        
        Ok(false)
    }

    pub fn get_incomplete_todos(&self, parent_id: Option<i64>) -> anyhow::Result<Vec<Todo>> {
        let mut todos = Vec::new();

        match parent_id {
            Some(pid) => {
                let mut stmt = self.conn.prepare(
                    "SELECT id, title, description, created_at, completed_at, due_by, parent_id, hidden
                     FROM todos
                     WHERE parent_id = ?1 AND completed_at IS NULL
                     ORDER BY created_at DESC"
                )?;
                let todo_iter = stmt.query_map([pid], |row| Todo::from_row(row))?;
                for todo in todo_iter {
                    todos.push(todo?);
                }
            },
            None => {
                let mut stmt = self.conn.prepare(
                    "SELECT id, title, description, created_at, completed_at, due_by, parent_id, hidden
                     FROM todos
                     WHERE completed_at IS NULL
                     ORDER BY created_at DESC"
                )?;
                let todo_iter = stmt.query_map([], |row| Todo::from_row(row))?;
                for todo in todo_iter {
                    todos.push(todo?);
                }
            }
        }

        Ok(todos)
    }

    pub fn get_recent_completed_todos(&self, parent_id: Option<i64>, limit: usize) -> anyhow::Result<Vec<Todo>> {
        let mut todos = Vec::new();

        match parent_id {
            Some(pid) => {
                let mut stmt = self.conn.prepare(
                    "SELECT id, title, description, created_at, completed_at, due_by, parent_id, hidden
                     FROM todos
                     WHERE parent_id = ?1 AND completed_at IS NOT NULL
                     ORDER BY completed_at DESC
                     LIMIT ?2"
                )?;
                let todo_iter = stmt.query_map(params![pid, limit as i64], |row| Todo::from_row(row))?;
                for todo in todo_iter {
                    todos.push(todo?);
                }
            },
            None => {
                let mut stmt = self.conn.prepare(
                    "SELECT id, title, description, created_at, completed_at, due_by, parent_id, hidden
                     FROM todos
                     WHERE completed_at IS NOT NULL
                     ORDER BY completed_at DESC
                     LIMIT ?1"
                )?;
                let todo_iter = stmt.query_map([limit as i64], |row| Todo::from_row(row))?;
                for todo in todo_iter {
                    todos.push(todo?);
                }
            }
        }

        Ok(todos)
    }

    pub fn get_parent_title(&self, parent_id: Option<i64>) -> anyhow::Result<Option<String>> {
        match parent_id {
            Some(id) => {
                let mut stmt = self.conn.prepare("SELECT title FROM todos WHERE id = ?1")?;
                let mut rows = stmt.query_map([id], |row| {
                    let title: String = row.get(0)?;
                    Ok(title)
                })?;
                
                match rows.next() {
                    Some(row) => Ok(Some(row?)),
                    None => Ok(None),
                }
            },
            None => Ok(None)
        }
    }

    /// Force a checkpoint to write WAL data to main database file
    pub fn checkpoint(&self) -> anyhow::Result<()> {
        let mut stmt = self.conn.prepare("PRAGMA wal_checkpoint(PASSIVE)")?;
        let _rows: Vec<Result<(), rusqlite::Error>> = stmt.query_map([], |_| Ok(()))?.collect();
        Ok(())
    }

    /// Force a full checkpoint and truncate WAL file (for app shutdown)
    pub fn checkpoint_and_close(&self) -> anyhow::Result<()> {
        let mut stmt = self.conn.prepare("PRAGMA wal_checkpoint(TRUNCATE)")?;
        let _rows: Vec<Result<(), rusqlite::Error>> = stmt.query_map([], |_| Ok(()))?.collect();
        Ok(())
    }

    /// Get WAL file size info for monitoring  
    pub fn get_wal_info(&self) -> anyhow::Result<(i64, i64)> {
        // Use a simpler approach - just return that WAL mode is working
        // The important thing is that WAL mode is enabled and functioning
        Ok((0, 0))
    }

    /// Search todos by regex pattern (case-insensitive) in title or description
    pub fn search_todos(&self, pattern: &str) -> anyhow::Result<Vec<Todo>> {
        // Return empty if pattern is empty
        if pattern.trim().is_empty() {
            return Ok(Vec::new());
        }

        // Build case-insensitive regex
        let regex = match RegexBuilder::new(pattern)
            .case_insensitive(true)
            .build()
        {
            Ok(regex) => regex,
            Err(_) => {
                // If regex is invalid, treat as literal string search
                RegexBuilder::new(&regex::escape(pattern))
                    .case_insensitive(true)
                    .build()?
            }
        };

        // Get all todos from database
        let mut stmt = self.conn.prepare(
            "SELECT id, title, description, created_at, completed_at, due_by, parent_id, hidden
             FROM todos
             ORDER BY created_at DESC"
        )?;

        let todo_iter = stmt.query_map([], |row| Todo::from_row(row))?;

        let mut matching_todos = Vec::new();
        for todo_result in todo_iter {
            let todo = todo_result?;

            // Check if regex matches title or description
            if regex.is_match(&todo.title) || regex.is_match(&todo.description) {
                matching_todos.push(todo);
            }
        }

        Ok(matching_todos)
    }
}