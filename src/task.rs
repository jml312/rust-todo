#[derive(Debug)]
pub struct Task {
    pub id: i32,
    pub name: String,
    pub is_done: bool,
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task() {
        let task = Task {
            id: 1,
            name: "task1".to_string(),
            is_done: false,
        };
        assert_eq!(task.id, 1);
        assert_eq!(task.name, "task1");
        assert_eq!(task.is_done, false);
    }
}
