//! board/mod.rs — Re-exports for the board model

pub mod board;
pub mod column;
pub mod priority;
pub mod task;

pub use board::Board;
#[allow(unused_imports)]
pub use column::Column;
pub use priority::Priority;
pub use task::Task;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_three_columns() {
        let b = Board::default();
        assert_eq!(b.column_count(), 3);
        assert_eq!(b.next_id, 5);
    }

    #[test]
    fn task_serde_roundtrip_with_new_fields() {
        let task = Task {
            id: 42,
            title: "Test".into(),
            description: "desc".into(),
            deadline: Some("2026-12-31".into()),
            priority: Priority::Urgent,
            tags: vec!["work".into(), "bug".into()],
            created_at: Some("2026-01-01".into()),
        };
        let json = serde_json::to_string(&task).unwrap();
        let back: Task = serde_json::from_str(&json).unwrap();
        assert_eq!(task, back);
    }

    #[test]
    fn priority_cycle() {
        assert_eq!(Priority::Low.next(), Priority::Medium);
        assert_eq!(Priority::Urgent.next(), Priority::Low);
    }

    #[test]
    fn board_column_ops() {
        let mut b = Board::default();
        let n = b.column_count();
        b.add_column("Review".into());
        assert_eq!(b.column_count(), n + 1);
        b.rename_column(n, "QA".into());
        assert_eq!(b.columns[n].name, "QA");
        let moved = b.move_column(n, -1).unwrap();
        assert_eq!(b.columns[moved].name, "QA");
        assert!(b.delete_column(moved).is_some());
        assert_eq!(b.column_count(), n);
    }

    #[test]
    fn board_can_be_empty_and_rebuilt() {
        let mut b = Board::default();
        while !b.is_empty() {
            assert!(b.delete_column(0).is_some());
        }
        assert_eq!(b.column_count(), 0);
        assert!(b.is_empty());
        b.add_column("Ideas".into());
        assert_eq!(b.column_count(), 1);
        assert_eq!(b.columns[0].name, "Ideas");
        b.reset_to_template();
        assert_eq!(b.column_count(), 3);
        b.clear();
        assert!(b.is_empty());
    }

    #[test]
    fn deadline_overdue_logic() {
        let t = Task {
            id: 1,
            title: "x".into(),
            description: "".into(),
            deadline: Some("2000-01-01".into()),
            priority: Priority::Medium,
            tags: vec![],
            created_at: None,
        };
        assert!(t.is_overdue());
        let t2 = Task {
            deadline: Some("2099-01-01".into()),
            ..t.clone()
        };
        assert!(!t2.is_overdue());
    }

    #[test]
    fn migration_old_json_still_loads() {
        let old = r#"{"columns":[{"name":"To Do","tasks":[{"id":1,"title":"Old","description":"hi"}]}],"next_id":2,"title":"Old Board"}"#;
        let b: Board = serde_json::from_str(old).unwrap();
        assert_eq!(b.columns[0].tasks[0].priority, Priority::Medium);
        assert!(b.columns[0].tasks[0].deadline.is_none());
        assert!(b.columns[0].tasks[0].tags.is_empty());
    }
}
