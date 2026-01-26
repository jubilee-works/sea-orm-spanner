mod common;
mod entity;

use chrono::{NaiveDate, Utc};
use common::setup_test_database;
use entity::all_types;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use serde_json::json;
use serial_test::serial;

fn create_test_model(id: &str) -> all_types::ActiveModel {
    all_types::ActiveModel {
        id: Set(id.to_string()),
        string_val: Set("test string".to_string()),
        string_nullable: Set(Some("nullable string".to_string())),
        int64_val: Set(9223372036854775807i64),
        int64_nullable: Set(Some(-9223372036854775808i64)),
        int32_val: Set(2147483647),
        int32_nullable: Set(Some(-2147483648)),
        float64_val: Set(3.141592653589793),
        float64_nullable: Set(Some(-1.7976931348623157e308)),
        float32_val: Set(3.14159f64),
        float32_nullable: Set(Some(-3.40282e38f64)),
        bool_val: Set(true),
        bool_nullable: Set(Some(false)),
        bytes_val: Set(vec![0x00, 0x01, 0xFF, 0xFE]),
        bytes_nullable: Set(Some(vec![0xDE, 0xAD, 0xBE, 0xEF])),
        timestamp_val: Set(Utc::now()),
        timestamp_nullable: Set(Some(Utc::now())),
        date_val: Set(NaiveDate::from_ymd_opt(2026, 1, 26).unwrap()),
        date_nullable: Set(Some(NaiveDate::from_ymd_opt(2025, 12, 25).unwrap())),
        json_val: Set(json!({"key": "value", "number": 42, "nested": {"a": 1}})),
        json_nullable: Set(Some(json!(["array", "of", "strings"]))),
    }
}

fn create_test_model_with_nulls(id: &str) -> all_types::ActiveModel {
    all_types::ActiveModel {
        id: Set(id.to_string()),
        string_val: Set("required string".to_string()),
        string_nullable: Set(None),
        int64_val: Set(i64::MIN),
        int64_nullable: Set(None),
        int32_val: Set(0),
        int32_nullable: Set(None),
        float64_val: Set(0.0),
        float64_nullable: Set(None),
        float32_val: Set(0.0f64),
        float32_nullable: Set(None),
        bool_val: Set(false),
        bool_nullable: Set(None),
        bytes_val: Set(vec![0]),
        bytes_nullable: Set(None),
        timestamp_val: Set(Utc::now()),
        timestamp_nullable: Set(None),
        date_val: Set(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()),
        date_nullable: Set(None),
        json_val: Set(json!({})),
        json_nullable: Set(None),
    }
}

mod string_type_tests {
    use super::*;

    #[tokio::test]
    #[serial]
    async fn test_string_insert_and_select() {
        let db = setup_test_database().await;
        let id = uuid::Uuid::new_v4().to_string();

        let model = all_types::ActiveModel {
            id: Set(id.clone()),
            string_val: Set("hello world".to_string()),
            string_nullable: Set(Some("optional".to_string())),
            ..create_test_model(&id)
        };

        let inserted = model.insert(&db).await.expect("Insert failed");
        assert_eq!(inserted.string_val, "hello world");
        assert_eq!(inserted.string_nullable, Some("optional".to_string()));

        let selected = all_types::Entity::find_by_id(&id)
            .one(&db)
            .await
            .expect("Select failed")
            .expect("Entity not found");
        assert_eq!(selected.string_val, "hello world");
        assert_eq!(selected.string_nullable, Some("optional".to_string()));
    }

    #[tokio::test]
    #[serial]
    async fn test_string_with_special_characters() {
        let db = setup_test_database().await;
        let id = uuid::Uuid::new_v4().to_string();

        let special_string = "Hello, 世界! 🌍 'quotes' \"double\" \\backslash\\ newline\nend";
        let model = all_types::ActiveModel {
            string_val: Set(special_string.to_string()),
            string_nullable: Set(None),
            ..create_test_model(&id)
        };

        let inserted = model.insert(&db).await.expect("Insert failed");
        assert_eq!(inserted.string_val, special_string);

        let selected = all_types::Entity::find_by_id(&id)
            .one(&db)
            .await
            .expect("Select failed")
            .expect("Entity not found");
        assert_eq!(selected.string_val, special_string);
    }

    #[tokio::test]
    #[serial]
    async fn test_string_nullable_null() {
        let db = setup_test_database().await;
        let id = uuid::Uuid::new_v4().to_string();

        let model = all_types::ActiveModel {
            string_nullable: Set(None),
            ..create_test_model(&id)
        };

        let inserted = model.insert(&db).await.expect("Insert failed");
        assert!(inserted.string_nullable.is_none());

        let selected = all_types::Entity::find_by_id(&id)
            .one(&db)
            .await
            .expect("Select failed")
            .expect("Entity not found");
        assert!(selected.string_nullable.is_none());
    }
}

mod integer_type_tests {
    use super::*;

    #[tokio::test]
    #[serial]
    async fn test_int64_insert_and_select() {
        let db = setup_test_database().await;
        let id = uuid::Uuid::new_v4().to_string();

        let model = all_types::ActiveModel {
            int64_val: Set(i64::MAX),
            int64_nullable: Set(Some(i64::MIN)),
            ..create_test_model(&id)
        };

        let inserted = model.insert(&db).await.expect("Insert failed");
        assert_eq!(inserted.int64_val, i64::MAX);
        assert_eq!(inserted.int64_nullable, Some(i64::MIN));

        let selected = all_types::Entity::find_by_id(&id)
            .one(&db)
            .await
            .expect("Select failed")
            .expect("Entity not found");
        assert_eq!(selected.int64_val, i64::MAX);
        assert_eq!(selected.int64_nullable, Some(i64::MIN));
    }

    #[tokio::test]
    #[serial]
    async fn test_int32_insert_and_select() {
        let db = setup_test_database().await;
        let id = uuid::Uuid::new_v4().to_string();

        let model = all_types::ActiveModel {
            int32_val: Set(i32::MAX),
            int32_nullable: Set(Some(i32::MIN)),
            ..create_test_model(&id)
        };

        let inserted = model.insert(&db).await.expect("Insert failed");
        assert_eq!(inserted.int32_val, i32::MAX);
        assert_eq!(inserted.int32_nullable, Some(i32::MIN));

        let selected = all_types::Entity::find_by_id(&id)
            .one(&db)
            .await
            .expect("Select failed")
            .expect("Entity not found");
        assert_eq!(selected.int32_val, i32::MAX);
        assert_eq!(selected.int32_nullable, Some(i32::MIN));
    }

    #[tokio::test]
    #[serial]
    async fn test_integer_zero_and_negative() {
        let db = setup_test_database().await;
        let id = uuid::Uuid::new_v4().to_string();

        let model = all_types::ActiveModel {
            int64_val: Set(i64::MIN),
            int64_nullable: Set(Some(i64::MAX)),
            int32_val: Set(-999),
            int32_nullable: Set(Some(0)),
            ..create_test_model(&id)
        };

        let inserted = model.insert(&db).await.expect("Insert failed");
        assert_eq!(inserted.int64_val, i64::MIN);
        assert_eq!(inserted.int64_nullable, Some(i64::MAX));
        assert_eq!(inserted.int32_val, -999);
        assert_eq!(inserted.int32_nullable, Some(0));

        let selected = all_types::Entity::find_by_id(&id)
            .one(&db)
            .await
            .expect("Select failed")
            .expect("Entity not found");
        assert_eq!(selected.int64_val, i64::MIN);
        assert_eq!(selected.int64_nullable, Some(i64::MAX));
        assert_eq!(selected.int32_val, -999);
        assert_eq!(selected.int32_nullable, Some(0));
    }

    #[tokio::test]
    #[serial]
    async fn test_integer_nullable_null() {
        let db = setup_test_database().await;
        let id = uuid::Uuid::new_v4().to_string();

        let model = all_types::ActiveModel {
            int64_nullable: Set(None),
            int32_nullable: Set(None),
            ..create_test_model(&id)
        };

        let inserted = model.insert(&db).await.expect("Insert failed");
        assert!(inserted.int64_nullable.is_none());
        assert!(inserted.int32_nullable.is_none());

        let selected = all_types::Entity::find_by_id(&id)
            .one(&db)
            .await
            .expect("Select failed")
            .expect("Entity not found");
        assert!(selected.int64_nullable.is_none());
        assert!(selected.int32_nullable.is_none());
    }
}

mod float_type_tests {
    use super::*;

    #[tokio::test]
    #[serial]
    async fn test_float64_insert_and_select() {
        let db = setup_test_database().await;
        let id = uuid::Uuid::new_v4().to_string();

        let model = all_types::ActiveModel {
            float64_val: Set(std::f64::consts::PI),
            float64_nullable: Set(Some(std::f64::consts::E)),
            ..create_test_model(&id)
        };

        let inserted = model.insert(&db).await.expect("Insert failed");
        assert!((inserted.float64_val - std::f64::consts::PI).abs() < f64::EPSILON);
        assert!((inserted.float64_nullable.unwrap() - std::f64::consts::E).abs() < f64::EPSILON);

        let selected = all_types::Entity::find_by_id(&id)
            .one(&db)
            .await
            .expect("Select failed")
            .expect("Entity not found");
        assert!((selected.float64_val - std::f64::consts::PI).abs() < f64::EPSILON);
    }

    #[tokio::test]
    #[serial]
    async fn test_float32_insert_and_select() {
        let db = setup_test_database().await;
        let id = uuid::Uuid::new_v4().to_string();

        let model = all_types::ActiveModel {
            float32_val: Set(std::f32::consts::PI as f64),
            float32_nullable: Set(Some(std::f32::consts::E as f64)),
            ..create_test_model(&id)
        };

        let inserted = model.insert(&db).await.expect("Insert failed");
        assert!((inserted.float32_val - std::f32::consts::PI as f64).abs() < f64::EPSILON);

        let selected = all_types::Entity::find_by_id(&id)
            .one(&db)
            .await
            .expect("Select failed")
            .expect("Entity not found");
        assert!((selected.float32_val - std::f32::consts::PI as f64).abs() < f64::EPSILON);
    }

    #[tokio::test]
    #[serial]
    async fn test_float_special_values() {
        let db = setup_test_database().await;
        let id = uuid::Uuid::new_v4().to_string();

        let model = all_types::ActiveModel {
            float64_val: Set(0.0),
            float64_nullable: Set(Some(-0.0)),
            float32_val: Set(0.0f64),
            float32_nullable: Set(None),
            ..create_test_model(&id)
        };

        let inserted = model.insert(&db).await.expect("Insert failed");
        assert_eq!(inserted.float64_val, 0.0);
        assert!(inserted.float32_nullable.is_none());

        let selected = all_types::Entity::find_by_id(&id)
            .one(&db)
            .await
            .expect("Select failed")
            .expect("Entity not found");
        assert_eq!(selected.float64_val, 0.0);
    }
}

mod bool_type_tests {
    use super::*;

    #[tokio::test]
    #[serial]
    async fn test_bool_true_insert_and_select() {
        let db = setup_test_database().await;
        let id = uuid::Uuid::new_v4().to_string();

        let model = all_types::ActiveModel {
            bool_val: Set(true),
            bool_nullable: Set(Some(true)),
            ..create_test_model(&id)
        };

        let inserted = model.insert(&db).await.expect("Insert failed");
        assert!(inserted.bool_val);
        assert_eq!(inserted.bool_nullable, Some(true));

        let selected = all_types::Entity::find_by_id(&id)
            .one(&db)
            .await
            .expect("Select failed")
            .expect("Entity not found");
        assert!(selected.bool_val);
        assert_eq!(selected.bool_nullable, Some(true));
    }

    #[tokio::test]
    #[serial]
    async fn test_bool_false_insert_and_select() {
        let db = setup_test_database().await;
        let id = uuid::Uuid::new_v4().to_string();

        let model = all_types::ActiveModel {
            bool_val: Set(false),
            bool_nullable: Set(Some(false)),
            ..create_test_model(&id)
        };

        let inserted = model.insert(&db).await.expect("Insert failed");
        assert!(!inserted.bool_val);
        assert_eq!(inserted.bool_nullable, Some(false));

        let selected = all_types::Entity::find_by_id(&id)
            .one(&db)
            .await
            .expect("Select failed")
            .expect("Entity not found");
        assert!(!selected.bool_val);
        assert_eq!(selected.bool_nullable, Some(false));
    }

    #[tokio::test]
    #[serial]
    async fn test_bool_nullable_null() {
        let db = setup_test_database().await;
        let id = uuid::Uuid::new_v4().to_string();

        let model = all_types::ActiveModel {
            bool_nullable: Set(None),
            ..create_test_model(&id)
        };

        let inserted = model.insert(&db).await.expect("Insert failed");
        assert!(inserted.bool_nullable.is_none());

        let selected = all_types::Entity::find_by_id(&id)
            .one(&db)
            .await
            .expect("Select failed")
            .expect("Entity not found");
        assert!(selected.bool_nullable.is_none());
    }
}

mod bytes_type_tests {
    use super::*;

    #[tokio::test]
    #[serial]
    async fn test_bytes_insert_and_select() {
        let db = setup_test_database().await;
        let id = uuid::Uuid::new_v4().to_string();

        let bytes_data = vec![0x00, 0x01, 0x02, 0xFF, 0xFE, 0xFD];
        let model = all_types::ActiveModel {
            bytes_val: Set(bytes_data.clone()),
            bytes_nullable: Set(Some(vec![0xDE, 0xAD, 0xBE, 0xEF])),
            ..create_test_model(&id)
        };

        let inserted = model.insert(&db).await.expect("Insert failed");
        assert_eq!(inserted.bytes_val, bytes_data);
        assert_eq!(inserted.bytes_nullable, Some(vec![0xDE, 0xAD, 0xBE, 0xEF]));

        let selected = all_types::Entity::find_by_id(&id)
            .one(&db)
            .await
            .expect("Select failed")
            .expect("Entity not found");
        assert_eq!(selected.bytes_val, bytes_data);
        assert_eq!(selected.bytes_nullable, Some(vec![0xDE, 0xAD, 0xBE, 0xEF]));
    }

    #[tokio::test]
    #[serial]
    async fn test_bytes_single_null() {
        let db = setup_test_database().await;
        let id = uuid::Uuid::new_v4().to_string();

        let model = all_types::ActiveModel {
            bytes_val: Set(vec![0]),
            bytes_nullable: Set(Some(vec![0, 0])),
            ..create_test_model(&id)
        };

        let inserted = model.insert(&db).await.expect("Insert failed");
        assert_eq!(inserted.bytes_val, vec![0]);
        assert_eq!(inserted.bytes_nullable, Some(vec![0, 0]));

        let selected = all_types::Entity::find_by_id(&id)
            .one(&db)
            .await
            .expect("Select failed")
            .expect("Entity not found");
        assert_eq!(selected.bytes_val, vec![0]);
        assert_eq!(selected.bytes_nullable, Some(vec![0, 0]));
    }

    #[tokio::test]
    #[serial]
    async fn test_bytes_nullable_null() {
        let db = setup_test_database().await;
        let id = uuid::Uuid::new_v4().to_string();

        let model = all_types::ActiveModel {
            bytes_nullable: Set(None),
            ..create_test_model(&id)
        };

        let inserted = model.insert(&db).await.expect("Insert failed");
        assert!(inserted.bytes_nullable.is_none());

        let selected = all_types::Entity::find_by_id(&id)
            .one(&db)
            .await
            .expect("Select failed")
            .expect("Entity not found");
        assert!(selected.bytes_nullable.is_none());
    }
}

mod timestamp_type_tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    #[tokio::test]
    #[serial]
    async fn test_timestamp_insert_and_select() {
        let db = setup_test_database().await;
        let id = uuid::Uuid::new_v4().to_string();

        let now = Utc::now();
        let model = all_types::ActiveModel {
            timestamp_val: Set(now),
            timestamp_nullable: Set(Some(now)),
            ..create_test_model(&id)
        };

        let inserted = model.insert(&db).await.expect("Insert failed");
        let time_diff = (inserted.timestamp_val - now).num_seconds().abs();
        assert!(
            time_diff < 2,
            "Timestamp difference too large: {} seconds",
            time_diff
        );

        let selected = all_types::Entity::find_by_id(&id)
            .one(&db)
            .await
            .expect("Select failed")
            .expect("Entity not found");
        let selected_diff = (selected.timestamp_val - now).num_seconds().abs();
        assert!(selected_diff < 2, "Selected timestamp difference too large");
    }

    #[tokio::test]
    #[serial]
    async fn test_timestamp_specific_date() {
        let db = setup_test_database().await;
        let id = uuid::Uuid::new_v4().to_string();

        let specific_time = Utc.with_ymd_and_hms(2024, 6, 15, 12, 30, 45).unwrap();
        let model = all_types::ActiveModel {
            timestamp_val: Set(specific_time),
            timestamp_nullable: Set(Some(specific_time)),
            ..create_test_model(&id)
        };

        let inserted = model.insert(&db).await.expect("Insert failed");
        assert_eq!(
            inserted.timestamp_val.date_naive(),
            specific_time.date_naive()
        );

        let selected = all_types::Entity::find_by_id(&id)
            .one(&db)
            .await
            .expect("Select failed")
            .expect("Entity not found");
        assert_eq!(
            selected.timestamp_val.date_naive(),
            specific_time.date_naive()
        );
    }

    #[tokio::test]
    #[serial]
    async fn test_timestamp_nullable_null() {
        let db = setup_test_database().await;
        let id = uuid::Uuid::new_v4().to_string();

        let model = all_types::ActiveModel {
            timestamp_nullable: Set(None),
            ..create_test_model(&id)
        };

        let inserted = model.insert(&db).await.expect("Insert failed");
        assert!(inserted.timestamp_nullable.is_none());

        let selected = all_types::Entity::find_by_id(&id)
            .one(&db)
            .await
            .expect("Select failed")
            .expect("Entity not found");
        assert!(selected.timestamp_nullable.is_none());
    }
}

mod date_type_tests {
    use super::*;

    #[tokio::test]
    #[serial]
    async fn test_date_insert_and_select() {
        let db = setup_test_database().await;
        let id = uuid::Uuid::new_v4().to_string();

        let date_val = NaiveDate::from_ymd_opt(2026, 1, 26).unwrap();
        let date_nullable = NaiveDate::from_ymd_opt(2025, 12, 25).unwrap();

        let model = all_types::ActiveModel {
            date_val: Set(date_val),
            date_nullable: Set(Some(date_nullable)),
            ..create_test_model(&id)
        };

        let inserted = model.insert(&db).await.expect("Insert failed");
        assert_eq!(inserted.date_val, date_val);
        assert_eq!(inserted.date_nullable, Some(date_nullable));

        let selected = all_types::Entity::find_by_id(&id)
            .one(&db)
            .await
            .expect("Select failed")
            .expect("Entity not found");
        assert_eq!(selected.date_val, date_val);
        assert_eq!(selected.date_nullable, Some(date_nullable));
    }

    #[tokio::test]
    #[serial]
    async fn test_date_nullable_null() {
        let db = setup_test_database().await;
        let id = uuid::Uuid::new_v4().to_string();

        let model = all_types::ActiveModel {
            date_nullable: Set(None),
            ..create_test_model(&id)
        };

        let inserted = model.insert(&db).await.expect("Insert failed");
        assert!(inserted.date_nullable.is_none());

        let selected = all_types::Entity::find_by_id(&id)
            .one(&db)
            .await
            .expect("Select failed")
            .expect("Entity not found");
        assert!(selected.date_nullable.is_none());
    }

    #[tokio::test]
    #[serial]
    async fn test_date_edge_cases() {
        let db = setup_test_database().await;
        let id = uuid::Uuid::new_v4().to_string();

        let min_date = NaiveDate::from_ymd_opt(1, 1, 1).unwrap();
        let model = all_types::ActiveModel {
            date_val: Set(min_date),
            date_nullable: Set(None),
            ..create_test_model(&id)
        };

        let inserted = model.insert(&db).await.expect("Insert failed");
        assert_eq!(inserted.date_val, min_date);

        let selected = all_types::Entity::find_by_id(&id)
            .one(&db)
            .await
            .expect("Select failed")
            .expect("Entity not found");
        assert_eq!(selected.date_val, min_date);
    }
}

mod json_type_tests {
    use super::*;

    #[tokio::test]
    #[serial]
    async fn test_json_object_insert_and_select() {
        let db = setup_test_database().await;
        let id = uuid::Uuid::new_v4().to_string();

        let json_obj = json!({
            "name": "test",
            "count": 42,
            "active": true,
            "tags": ["a", "b", "c"]
        });

        let model = all_types::ActiveModel {
            json_val: Set(json_obj.clone()),
            json_nullable: Set(Some(json_obj.clone())),
            ..create_test_model(&id)
        };

        let inserted = model.insert(&db).await.expect("Insert failed");
        assert_eq!(inserted.json_val, json_obj);
        assert_eq!(inserted.json_nullable, Some(json_obj.clone()));

        let selected = all_types::Entity::find_by_id(&id)
            .one(&db)
            .await
            .expect("Select failed")
            .expect("Entity not found");
        assert_eq!(selected.json_val, json_obj);
        assert_eq!(selected.json_nullable, Some(json_obj));
    }

    #[tokio::test]
    #[serial]
    async fn test_json_array_insert_and_select() {
        let db = setup_test_database().await;
        let id = uuid::Uuid::new_v4().to_string();

        let json_arr = json!([1, 2, 3, "four", true, null]);

        let model = all_types::ActiveModel {
            json_val: Set(json_arr.clone()),
            json_nullable: Set(Some(json_arr.clone())),
            ..create_test_model(&id)
        };

        let inserted = model.insert(&db).await.expect("Insert failed");
        assert_eq!(inserted.json_val, json_arr);

        let selected = all_types::Entity::find_by_id(&id)
            .one(&db)
            .await
            .expect("Select failed")
            .expect("Entity not found");
        assert_eq!(selected.json_val, json_arr);
    }

    #[tokio::test]
    #[serial]
    async fn test_json_nested_insert_and_select() {
        let db = setup_test_database().await;
        let id = uuid::Uuid::new_v4().to_string();

        let nested_json = json!({
            "level1": {
                "level2": {
                    "level3": {
                        "value": "deep"
                    }
                }
            },
            "array_of_objects": [
                {"id": 1, "name": "first"},
                {"id": 2, "name": "second"}
            ]
        });

        let model = all_types::ActiveModel {
            json_val: Set(nested_json.clone()),
            json_nullable: Set(None),
            ..create_test_model(&id)
        };

        let inserted = model.insert(&db).await.expect("Insert failed");
        assert_eq!(inserted.json_val, nested_json);

        let selected = all_types::Entity::find_by_id(&id)
            .one(&db)
            .await
            .expect("Select failed")
            .expect("Entity not found");
        assert_eq!(selected.json_val, nested_json);
    }

    #[tokio::test]
    #[serial]
    async fn test_json_primitive_values() {
        let db = setup_test_database().await;

        let test_cases = vec![json!(null), json!(true), json!(false), json!("a string")];

        for (i, json_val) in test_cases.into_iter().enumerate() {
            let id = uuid::Uuid::new_v4().to_string();

            let model = all_types::ActiveModel {
                json_val: Set(json_val.clone()),
                json_nullable: Set(Some(json_val.clone())),
                ..create_test_model(&id)
            };

            let inserted = model.insert(&db).await.expect("Insert failed");
            assert_eq!(inserted.json_val, json_val, "Failed for case {}", i);

            let selected = all_types::Entity::find_by_id(&id)
                .one(&db)
                .await
                .expect("Select failed")
                .expect("Entity not found");
            assert_eq!(selected.json_val, json_val, "Select failed for case {}", i);
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_json_empty_structures() {
        let db = setup_test_database().await;
        let id = uuid::Uuid::new_v4().to_string();

        let empty_obj = json!({});
        let model = all_types::ActiveModel {
            json_val: Set(empty_obj.clone()),
            json_nullable: Set(Some(json!([]))),
            ..create_test_model(&id)
        };

        let inserted = model.insert(&db).await.expect("Insert failed");
        assert_eq!(inserted.json_val, empty_obj);
        assert_eq!(inserted.json_nullable, Some(json!([])));

        let selected = all_types::Entity::find_by_id(&id)
            .one(&db)
            .await
            .expect("Select failed")
            .expect("Entity not found");
        assert_eq!(selected.json_val, empty_obj);
        assert_eq!(selected.json_nullable, Some(json!([])));
    }

    #[tokio::test]
    #[serial]
    async fn test_json_with_special_characters() {
        let db = setup_test_database().await;
        let id = uuid::Uuid::new_v4().to_string();

        let special_json = json!({
            "unicode": "日本語 🎉 émojis",
            "quotes": "He said \"hello\"",
            "backslash": "path\\to\\file",
            "newlines": "line1\nline2\ttab"
        });

        let model = all_types::ActiveModel {
            json_val: Set(special_json.clone()),
            json_nullable: Set(None),
            ..create_test_model(&id)
        };

        let inserted = model.insert(&db).await.expect("Insert failed");
        assert_eq!(inserted.json_val, special_json);

        let selected = all_types::Entity::find_by_id(&id)
            .one(&db)
            .await
            .expect("Select failed")
            .expect("Entity not found");
        assert_eq!(selected.json_val, special_json);
    }

    #[tokio::test]
    #[serial]
    async fn test_json_nullable_null() {
        let db = setup_test_database().await;
        let id = uuid::Uuid::new_v4().to_string();

        let model = all_types::ActiveModel {
            json_nullable: Set(None),
            ..create_test_model(&id)
        };

        let inserted = model.insert(&db).await.expect("Insert failed");
        assert!(inserted.json_nullable.is_none());

        let selected = all_types::Entity::find_by_id(&id)
            .one(&db)
            .await
            .expect("Select failed")
            .expect("Entity not found");
        assert!(selected.json_nullable.is_none());
    }
}

mod all_nulls_test {
    use super::*;

    #[tokio::test]
    #[serial]
    async fn test_all_nullable_fields_as_null() {
        let db = setup_test_database().await;
        let id = uuid::Uuid::new_v4().to_string();

        let model = create_test_model_with_nulls(&id);

        let inserted = model.insert(&db).await.expect("Insert failed");
        assert!(inserted.string_nullable.is_none());
        assert!(inserted.int64_nullable.is_none());
        assert!(inserted.int32_nullable.is_none());
        assert!(inserted.float64_nullable.is_none());
        assert!(inserted.float32_nullable.is_none());
        assert!(inserted.bool_nullable.is_none());
        assert!(inserted.bytes_nullable.is_none());
        assert!(inserted.timestamp_nullable.is_none());
        assert!(inserted.json_nullable.is_none());

        let selected = all_types::Entity::find_by_id(&id)
            .one(&db)
            .await
            .expect("Select failed")
            .expect("Entity not found");
        assert!(selected.string_nullable.is_none());
        assert!(selected.int64_nullable.is_none());
        assert!(selected.int32_nullable.is_none());
        assert!(selected.float64_nullable.is_none());
        assert!(selected.float32_nullable.is_none());
        assert!(selected.bool_nullable.is_none());
        assert!(selected.bytes_nullable.is_none());
        assert!(selected.timestamp_nullable.is_none());
        assert!(selected.json_nullable.is_none());
    }
}

mod all_values_test {
    use super::*;

    #[tokio::test]
    #[serial]
    async fn test_all_fields_with_values() {
        let db = setup_test_database().await;
        let id = uuid::Uuid::new_v4().to_string();

        let model = create_test_model(&id);

        let inserted = model.insert(&db).await.expect("Insert failed");
        assert_eq!(inserted.string_val, "test string");
        assert!(inserted.string_nullable.is_some());
        assert_eq!(inserted.int64_val, 9223372036854775807i64);
        assert!(inserted.int64_nullable.is_some());
        assert!(inserted.bool_val);
        assert!(inserted.bool_nullable.is_some());
        assert!(!inserted.bytes_val.is_empty());
        assert!(inserted.bytes_nullable.is_some());
        assert!(inserted.json_nullable.is_some());

        let selected = all_types::Entity::find_by_id(&id)
            .one(&db)
            .await
            .expect("Select failed")
            .expect("Entity not found");
        assert_eq!(selected.string_val, "test string");
        assert!(selected.string_nullable.is_some());
        assert_eq!(selected.int64_val, 9223372036854775807i64);
    }
}
