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

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE tasks (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                is_done BOOLEAN NOT NULL
            )",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO tasks (name, is_done) VALUES (?1, ?2)",
            params!["task1", false],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO tasks (name, is_done) VALUES (?1, ?2)",
            params!["task2", true],
        )
        .unwrap();
        conn
    }

    #[test]
    fn test_get_task_by_id() {
        let conn = setup();
        let task = get_task_by_id(&conn, &1).unwrap();
        assert_eq!(task.id, 1);
        assert_eq!(task.name, "task1");
        assert_eq!(task.is_done, false);
    }

    #[test]
    fn test_get_task_by_name() {
        let conn = setup();
        let task = get_task_by_name(&conn, "task2").unwrap();
        assert_eq!(task.id, 2);
        assert_eq!(task.name, "task2");
        assert_eq!(task.is_done, true);
    }

    #[test]
    fn test_get_all_tasks() {
        let conn = setup();
        let tasks = get_all_tasks(&conn);
        assert_eq!(tasks.len(), 2);
        assert_eq!(tasks[0].id, 1);
        assert_eq!(tasks[0].name, "task1");
        assert_eq!(tasks[0].is_done, false);
        assert_eq!(tasks[1].id, 2);
        assert_eq!(tasks[1].name, "task2");
        assert_eq!(tasks[1].is_done, true);
    }
}
