use crate::task::Task;
use rusqlite::{params, Connection, OptionalExtension};

pub fn get_task_by_id(conn: &Connection, id: &i32) -> Option<Task> {
    let task: Option<Task> = conn
        .prepare("SELECT id, name, is_done FROM tasks WHERE id = ?1")
        .unwrap()
        .query_row(params![id], |row| {
            Ok(Task {
                id: row.get(0).unwrap(),
                name: row.get(1).unwrap(),
                is_done: row.get(2).unwrap(),
            })
        })
        .optional()
        .unwrap();
    task
}

pub fn get_task_by_name(conn: &Connection, name: &str) -> Option<Task> {
    let task: Option<Task> = conn
        .prepare("SELECT id, name, is_done FROM tasks WHERE LOWER(name) = LOWER(?1)")
        .unwrap()
        .query_row(params![name], |row| {
            Ok(Task {
                id: row.get(0).unwrap(),
                name: row.get(1).unwrap(),
                is_done: row.get(2).unwrap(),
            })
        })
        .optional()
        .unwrap();
    task
}

pub fn get_all_tasks(conn: &Connection) -> Vec<Task> {
    let tasks: Vec<Task> = conn
        .prepare("SELECT id, name, is_done FROM tasks")
        .unwrap()
        .query_map([], |row| {
            Ok(Task {
                id: row.get(0).unwrap(),
                name: row.get(1).unwrap(),
                is_done: row.get(2).unwrap(),
            })
        })
        .unwrap()
        .map(|task| task.unwrap())
        .collect();
    tasks
}
