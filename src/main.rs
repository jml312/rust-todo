mod db;
mod task;

use crate::db::{get_all_tasks, get_task_by_id, get_task_by_name};
use crate::task::Task;

use clap::{Arg, Command};
use rusqlite::{params, Connection, Result};

use std::env;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};

fn main() -> Result<()> {
    let matches: clap::ArgMatches = Command::new("ToDo App")
        .version("1.0")
        .author("Josh Levy")
        .about("A simple ToDo app in Rust")
        .subcommand(
            Command::new("add")
                .about("Adds a new task to the todo list")
                .arg(
                    Arg::new("name")
                        .help("The name of the task to add")
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(
            Command::new("load")
                .about("Loads a list of tasks from a file path")
                .arg(
                    Arg::new("file_path")
                        .help("The file path to load tasks from")
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(Command::new("list").about("Lists all tasks in the todo list"))
        .subcommand(
            Command::new("toggle")
                .about("Toggles a task as complete or incomplete")
                .arg(
                    Arg::new("id")
                        .help("The ID of the task to toggle")
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(Command::new("toggle_all").about("Toggles all tasks as complete or incomplete"))
        .subcommand(Command::new("clean").about("Removes all completed tasks from the todo list"))
        .subcommand(
            Command::new("reset").about("Resets the all tasks in the todo list to uncompleted"),
        )
        .subcommand(
            Command::new("delete")
                .about("Deletes a task from the todo list")
                .arg(
                    Arg::new("id")
                        .help("The ID of the task to delete")
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(Command::new("clear").about("Clears all tasks from the todo list"))
        .get_matches();

    let conn: Connection = Connection::open("local.db")?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS tasks (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                is_done BOOLEAN NOT NULL default 0
            )",
        [],
    )?;

    match matches.subcommand() {
        Some(("add", sub_matches)) => {
            let name: &str = sub_matches
                .get_one::<String>("name")
                .expect("Task name is required");
            add_task(&conn, name, true)?;
        }
        Some(("load", sub_matches)) => {
            let file_path: &str = sub_matches
                .get_one::<String>("file_path")
                .expect("File path is required");
            load_tasks(&conn, &file_path, true)?;
        }
        Some(("list", _)) => {
            list_tasks(&conn)?;
        }
        Some(("toggle", sub_matches)) => {
            let id: i32 = sub_matches
                .get_one::<String>("id")
                .expect("ID is required")
                .parse()
                .unwrap();
            toggle_task(&conn, &id)?;
        }
        Some(("toggle_all", _)) => {
            toggle_all_tasks(&conn)?;
        }
        Some(("clean", _)) => {
            clean_tasks(&conn)?;
        }
        Some(("delete", sub_matches)) => {
            let id: i32 = sub_matches
                .get_one::<String>("id")
                .expect("ID is required")
                .parse()
                .unwrap();
            delete_task(&conn, &id)?;
        }
        Some(("clear", _)) => {
            clear_tasks(&conn)?;
        }
        Some(("reset", _)) => {
            reset_tasks(&conn)?;
        }
        _ => {
            eprintln!("Unknown command. Use --help to see available commands.");
        }
    }

    Ok(())
}

fn add_task(conn: &Connection, name: &str, print: bool) -> Result<()> {
    let task: Option<Task> = get_task_by_name(conn, name);

    match task {
        Some(_) => {
            if print {
                println!("Task '{}' already exists", name);
            }
        }
        None => {
            conn.execute("INSERT INTO tasks (name) VALUES (?1)", params![name])?;
            if print {
                println!("Task added: '{}'", name);
            }
        }
    }
    if print {
        list_tasks(&conn)?;
    }
    Ok(())
}

fn load_tasks(conn: &Connection, file_path: &str, print: bool) -> Result<()> {
    let current_directory: PathBuf = env::current_dir().unwrap();

    let full_path: PathBuf = current_directory.join(file_path);

    if !full_path.exists() {
        eprintln!("The file '{}' does not exist.", file_path);
        return Ok(());
    }

    let path: &Path = &full_path;

    let file: Result<File, io::Error> = File::open(path);
    let reader: io::BufReader<File> = io::BufReader::new(file.unwrap());

    for line in reader.lines() {
        match line {
            Ok(line_content) => {
                if !line_content.is_empty() {
                    add_task(&conn, &line_content, false)?
                }
            }
            Err(e) => eprintln!("Error reading line: {}", e),
        }
    }

    if print {
        println!("Tasks loaded from '{}'", file_path);
        list_tasks(&conn)?;
    }

    Ok(())
}

fn list_tasks(conn: &Connection) -> Result<()> {
    let tasks: Vec<Task> = get_all_tasks(&conn);

    match tasks.len() {
        0 => {
            println!("No tasks found...");
        }
        _ => {
            println!("Your To-Do List:");
            for task in tasks {
                println!(
                    "[{}] {:<3} {}",
                    if task.is_done { "✔" } else { " " },
                    task.id,
                    task.name
                );
            }
        }
    }

    Ok(())
}

fn toggle_task(conn: &Connection, id: &i32) -> Result<()> {
    let task: Option<Task> = get_task_by_id(&conn, &id);

    match task {
        Some(task) => {
            let new_status: bool = !task.is_done;
            conn.execute(
                "UPDATE tasks SET is_done = ?1 WHERE id = ?2",
                params![new_status, id],
            )?;

            let status: &str = if new_status { "complete" } else { "incomplete" };

            println!("Task '{}' marked as {}", task.name, status);
        }
        None => {
            println!("Task with ID '{}' not found", id);
        }
    }

    list_tasks(&conn)?;
    Ok(())
}

fn toggle_all_tasks(conn: &Connection) -> Result<()> {
    let tasks: Vec<Task> = get_all_tasks(&conn);

    let all_completed: bool = tasks.iter().all(|task: &Task| task.is_done);

    if all_completed {
        conn.execute("UPDATE tasks SET is_done = 0", [])?;
        println!("All tasks marked as incomplete");
    } else {
        conn.execute("UPDATE tasks SET is_done = 1", [])?;
        println!("All tasks marked as complete");
    }

    list_tasks(&conn)?;
    Ok(())
}

fn clean_tasks(conn: &Connection) -> Result<()> {
    conn.execute("DELETE FROM tasks WHERE is_done = 1", [])?;
    println!("All completed tasks cleaned");
    list_tasks(conn)?;
    Ok(())
}

fn reset_tasks(conn: &Connection) -> Result<()> {
    conn.execute("UPDATE tasks SET is_done = 0", [])?;
    println!("All tasks reset");
    list_tasks(&conn)?;
    Ok(())
}

fn delete_task(conn: &Connection, id: &i32) -> Result<()> {
    let task: Option<Task> = get_task_by_id(&conn, &id);

    match task {
        Some(task) => {
            conn.execute("DELETE FROM tasks WHERE id = ?1", params![id])?;
            println!("Task deleted: '{}'", task.name);
        }
        None => {
            println!("Task with ID '{}' not found", id);
        }
    }

    list_tasks(&conn)?;
    Ok(())
}

fn clear_tasks(conn: &Connection) -> Result<()> {
    conn.execute("DELETE FROM tasks", [])?;
    println!("All tasks cleared");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE tasks (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                is_done BOOLEAN NOT NULL default 0
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
    fn test_add_task() {
        let conn = setup_db();
        add_task(&conn, "task3", false).unwrap();
        let task = get_task_by_name(&conn, "task3").unwrap();
        assert_eq!(task.name, "task3");
        assert_eq!(task.is_done, false);
    }

    #[test]
    fn test_load_tasks() {
        let file_path = "tasks.txt";
        let mut file: File = File::create(file_path).unwrap();
        file.write_all(b"task4\ntask5\ntask6").unwrap();

        let conn = setup_db();
        load_tasks(&conn, file_path, false).unwrap();
        let tasks = get_all_tasks(&conn);

        assert_eq!(tasks.len(), 5);

        std::fs::remove_file(file_path).unwrap();
    }

    #[test]
    fn test_list_tasks() {
        let conn = setup_db();
        list_tasks(&conn).unwrap();
    }

    #[test]
    fn test_toggle_task() {
        let conn = setup_db();
        toggle_task(&conn, &1).unwrap();
        let task = get_task_by_id(&conn, &1).unwrap();
        assert_eq!(task.is_done, true);
    }

    #[test]
    fn test_toggle_all_tasks() {
        let conn = setup_db();
        toggle_all_tasks(&conn).unwrap();
        let tasks = get_all_tasks(&conn);
        assert_eq!(tasks.iter().all(|task| task.is_done), true);
    }

    #[test]
    fn test_clean_tasks() {
        let conn = setup_db();
        clean_tasks(&conn).unwrap();
        let tasks = get_all_tasks(&conn);
        assert_eq!(tasks.len(), 1);
    }

    #[test]
    fn test_reset_tasks() {
        let conn = setup_db();
        toggle_task(&conn, &1).unwrap();
        reset_tasks(&conn).unwrap();
        let tasks = get_all_tasks(&conn);
        assert_eq!(tasks.iter().all(|task| task.is_done), false);
    }

    #[test]
    fn test_delete_task() {
        let conn = setup_db();
        delete_task(&conn, &1).unwrap();
        let tasks = get_all_tasks(&conn);
        assert_eq!(tasks.len(), 1);
    }

    #[test]
    fn test_clear_tasks() {
        let conn = setup_db();
        clear_tasks(&conn).unwrap();
        let tasks = get_all_tasks(&conn);
        assert_eq!(tasks.len(), 0);
    }

    #[test]
    fn test_main() {
        main().unwrap();
    }
}
