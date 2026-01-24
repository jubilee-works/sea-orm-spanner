mod common;
mod entity;

use common::setup_test_database;
use entity::array_types;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use serial_test::serial;

mod int64_array_tests {
    use super::*;

    #[tokio::test]
    #[serial]
    async fn test_int64_array_insert_and_select() {
        let db = setup_test_database().await;
        let id = uuid::Uuid::new_v4().to_string();

        let model = array_types::ActiveModel {
            id: Set(id.clone()),
            int64_array: Set(vec![1, 2, 3, i64::MAX, i64::MIN]),
            int64_array_nullable: Set(Some(vec![-100, 0, 100])),
            float64_array: Set(vec![1.0]),
            float64_array_nullable: Set(None),
            string_array: Set(vec!["a".to_string()]),
            string_array_nullable: Set(None),
            bool_array: Set(vec![true]),
            bool_array_nullable: Set(None),
        };

        let inserted = model.insert(&db).await.expect("Insert failed");
        assert_eq!(inserted.int64_array, vec![1, 2, 3, i64::MAX, i64::MIN]);
        assert_eq!(inserted.int64_array_nullable, Some(vec![-100, 0, 100]));

        let selected = array_types::Entity::find_by_id(&id)
            .one(&db)
            .await
            .expect("Select failed")
            .expect("Entity not found");
        assert_eq!(selected.int64_array, vec![1, 2, 3, i64::MAX, i64::MIN]);
        assert_eq!(selected.int64_array_nullable, Some(vec![-100, 0, 100]));
    }

    #[tokio::test]
    #[serial]
    async fn test_int64_array_nullable_null() {
        let db = setup_test_database().await;
        let id = uuid::Uuid::new_v4().to_string();

        let model = array_types::ActiveModel {
            id: Set(id.clone()),
            int64_array: Set(vec![42]),
            int64_array_nullable: Set(None),
            float64_array: Set(vec![1.0]),
            float64_array_nullable: Set(None),
            string_array: Set(vec!["test".to_string()]),
            string_array_nullable: Set(None),
            bool_array: Set(vec![true]),
            bool_array_nullable: Set(None),
        };

        let inserted = model.insert(&db).await.expect("Insert failed");
        assert!(inserted.int64_array_nullable.is_none());

        let selected = array_types::Entity::find_by_id(&id)
            .one(&db)
            .await
            .expect("Select failed")
            .expect("Entity not found");
        assert!(selected.int64_array_nullable.is_none());
    }
}

mod float64_array_tests {
    use super::*;

    #[tokio::test]
    #[serial]
    async fn test_float64_array_insert_and_select() {
        let db = setup_test_database().await;
        let id = uuid::Uuid::new_v4().to_string();

        let model = array_types::ActiveModel {
            id: Set(id.clone()),
            int64_array: Set(vec![1]),
            int64_array_nullable: Set(None),
            float64_array: Set(vec![1.1, 2.2, 3.3, std::f64::consts::PI]),
            float64_array_nullable: Set(Some(vec![0.0, -1.5, 999.999])),
            string_array: Set(vec!["a".to_string()]),
            string_array_nullable: Set(None),
            bool_array: Set(vec![true]),
            bool_array_nullable: Set(None),
        };

        let inserted = model.insert(&db).await.expect("Insert failed");
        assert_eq!(inserted.float64_array.len(), 4);
        assert!((inserted.float64_array[3] - std::f64::consts::PI).abs() < f64::EPSILON);

        let selected = array_types::Entity::find_by_id(&id)
            .one(&db)
            .await
            .expect("Select failed")
            .expect("Entity not found");
        assert_eq!(selected.float64_array.len(), 4);
    }

    #[tokio::test]
    #[serial]
    async fn test_float64_array_nullable_null() {
        let db = setup_test_database().await;
        let id = uuid::Uuid::new_v4().to_string();

        let model = array_types::ActiveModel {
            id: Set(id.clone()),
            int64_array: Set(vec![1]),
            int64_array_nullable: Set(None),
            float64_array: Set(vec![1.0, 2.0]),
            float64_array_nullable: Set(None),
            string_array: Set(vec!["test".to_string()]),
            string_array_nullable: Set(None),
            bool_array: Set(vec![true]),
            bool_array_nullable: Set(None),
        };

        let inserted = model.insert(&db).await.expect("Insert failed");
        assert!(inserted.float64_array_nullable.is_none());

        let selected = array_types::Entity::find_by_id(&id)
            .one(&db)
            .await
            .expect("Select failed")
            .expect("Entity not found");
        assert!(selected.float64_array_nullable.is_none());
    }
}

mod string_array_tests {
    use super::*;

    #[tokio::test]
    #[serial]
    async fn test_string_array_insert_and_select() {
        let db = setup_test_database().await;
        let id = uuid::Uuid::new_v4().to_string();

        let model = array_types::ActiveModel {
            id: Set(id.clone()),
            int64_array: Set(vec![1]),
            int64_array_nullable: Set(None),
            float64_array: Set(vec![1.0]),
            float64_array_nullable: Set(None),
            string_array: Set(vec![
                "hello".to_string(),
                "world".to_string(),
                "日本語".to_string(),
            ]),
            string_array_nullable: Set(Some(vec!["optional".to_string()])),
            bool_array: Set(vec![true]),
            bool_array_nullable: Set(None),
        };

        let inserted = model.insert(&db).await.expect("Insert failed");
        assert_eq!(inserted.string_array, vec!["hello", "world", "日本語"]);
        assert_eq!(
            inserted.string_array_nullable,
            Some(vec!["optional".to_string()])
        );

        let selected = array_types::Entity::find_by_id(&id)
            .one(&db)
            .await
            .expect("Select failed")
            .expect("Entity not found");
        assert_eq!(selected.string_array, vec!["hello", "world", "日本語"]);
    }

    #[tokio::test]
    #[serial]
    async fn test_string_array_with_special_chars() {
        let db = setup_test_database().await;
        let id = uuid::Uuid::new_v4().to_string();

        let model = array_types::ActiveModel {
            id: Set(id.clone()),
            int64_array: Set(vec![1]),
            int64_array_nullable: Set(None),
            float64_array: Set(vec![1.0]),
            float64_array_nullable: Set(None),
            string_array: Set(vec![
                "quotes: 'single' \"double\"".to_string(),
                "newline\nand\ttab".to_string(),
                "emoji: 🎉🚀".to_string(),
            ]),
            string_array_nullable: Set(None),
            bool_array: Set(vec![true]),
            bool_array_nullable: Set(None),
        };

        let inserted = model.insert(&db).await.expect("Insert failed");
        assert_eq!(inserted.string_array.len(), 3);
        assert!(inserted.string_array[2].contains("🎉"));

        let selected = array_types::Entity::find_by_id(&id)
            .one(&db)
            .await
            .expect("Select failed")
            .expect("Entity not found");
        assert_eq!(selected.string_array.len(), 3);
    }

    #[tokio::test]
    #[serial]
    async fn test_string_array_nullable_null() {
        let db = setup_test_database().await;
        let id = uuid::Uuid::new_v4().to_string();

        let model = array_types::ActiveModel {
            id: Set(id.clone()),
            int64_array: Set(vec![1]),
            int64_array_nullable: Set(None),
            float64_array: Set(vec![1.0]),
            float64_array_nullable: Set(None),
            string_array: Set(vec!["test".to_string()]),
            string_array_nullable: Set(None),
            bool_array: Set(vec![true]),
            bool_array_nullable: Set(None),
        };

        let inserted = model.insert(&db).await.expect("Insert failed");
        assert!(inserted.string_array_nullable.is_none());

        let selected = array_types::Entity::find_by_id(&id)
            .one(&db)
            .await
            .expect("Select failed")
            .expect("Entity not found");
        assert!(selected.string_array_nullable.is_none());
    }
}

mod bool_array_tests {
    use super::*;

    #[tokio::test]
    #[serial]
    async fn test_bool_array_insert_and_select() {
        let db = setup_test_database().await;
        let id = uuid::Uuid::new_v4().to_string();

        let model = array_types::ActiveModel {
            id: Set(id.clone()),
            int64_array: Set(vec![1]),
            int64_array_nullable: Set(None),
            float64_array: Set(vec![1.0]),
            float64_array_nullable: Set(None),
            string_array: Set(vec!["a".to_string()]),
            string_array_nullable: Set(None),
            bool_array: Set(vec![true, false, true, true, false]),
            bool_array_nullable: Set(Some(vec![false, false])),
        };

        let inserted = model.insert(&db).await.expect("Insert failed");
        assert_eq!(inserted.bool_array, vec![true, false, true, true, false]);
        assert_eq!(inserted.bool_array_nullable, Some(vec![false, false]));

        let selected = array_types::Entity::find_by_id(&id)
            .one(&db)
            .await
            .expect("Select failed")
            .expect("Entity not found");
        assert_eq!(selected.bool_array, vec![true, false, true, true, false]);
    }

    #[tokio::test]
    #[serial]
    async fn test_bool_array_nullable_null() {
        let db = setup_test_database().await;
        let id = uuid::Uuid::new_v4().to_string();

        let model = array_types::ActiveModel {
            id: Set(id.clone()),
            int64_array: Set(vec![1]),
            int64_array_nullable: Set(None),
            float64_array: Set(vec![1.0]),
            float64_array_nullable: Set(None),
            string_array: Set(vec!["test".to_string()]),
            string_array_nullable: Set(None),
            bool_array: Set(vec![true]),
            bool_array_nullable: Set(None),
        };

        let inserted = model.insert(&db).await.expect("Insert failed");
        assert!(inserted.bool_array_nullable.is_none());

        let selected = array_types::Entity::find_by_id(&id)
            .one(&db)
            .await
            .expect("Select failed")
            .expect("Entity not found");
        assert!(selected.bool_array_nullable.is_none());
    }
}

mod all_arrays_test {
    use super::*;

    #[tokio::test]
    #[serial]
    async fn test_all_array_types_together() {
        let db = setup_test_database().await;
        let id = uuid::Uuid::new_v4().to_string();

        let model = array_types::ActiveModel {
            id: Set(id.clone()),
            int64_array: Set(vec![1, 2, 3]),
            int64_array_nullable: Set(Some(vec![10, 20])),
            float64_array: Set(vec![1.1, 2.2]),
            float64_array_nullable: Set(Some(vec![3.3])),
            string_array: Set(vec!["a".to_string(), "b".to_string()]),
            string_array_nullable: Set(Some(vec!["c".to_string()])),
            bool_array: Set(vec![true, false]),
            bool_array_nullable: Set(Some(vec![true])),
        };

        let inserted = model.insert(&db).await.expect("Insert failed");
        assert_eq!(inserted.int64_array, vec![1, 2, 3]);
        assert_eq!(inserted.float64_array, vec![1.1, 2.2]);
        assert_eq!(inserted.string_array, vec!["a", "b"]);
        assert_eq!(inserted.bool_array, vec![true, false]);

        let selected = array_types::Entity::find_by_id(&id)
            .one(&db)
            .await
            .expect("Select failed")
            .expect("Entity not found");
        assert_eq!(selected.int64_array, vec![1, 2, 3]);
        assert_eq!(selected.float64_array, vec![1.1, 2.2]);
        assert_eq!(selected.string_array, vec!["a", "b"]);
        assert_eq!(selected.bool_array, vec![true, false]);
    }

    #[tokio::test]
    #[serial]
    async fn test_all_nullable_arrays_as_null() {
        let db = setup_test_database().await;
        let id = uuid::Uuid::new_v4().to_string();

        let model = array_types::ActiveModel {
            id: Set(id.clone()),
            int64_array: Set(vec![1]),
            int64_array_nullable: Set(None),
            float64_array: Set(vec![1.0]),
            float64_array_nullable: Set(None),
            string_array: Set(vec!["test".to_string()]),
            string_array_nullable: Set(None),
            bool_array: Set(vec![true]),
            bool_array_nullable: Set(None),
        };

        let inserted = model.insert(&db).await.expect("Insert failed");
        assert!(inserted.int64_array_nullable.is_none());
        assert!(inserted.float64_array_nullable.is_none());
        assert!(inserted.string_array_nullable.is_none());
        assert!(inserted.bool_array_nullable.is_none());

        let selected = array_types::Entity::find_by_id(&id)
            .one(&db)
            .await
            .expect("Select failed")
            .expect("Entity not found");
        assert!(selected.int64_array_nullable.is_none());
        assert!(selected.float64_array_nullable.is_none());
        assert!(selected.string_array_nullable.is_none());
        assert!(selected.bool_array_nullable.is_none());
    }
}
