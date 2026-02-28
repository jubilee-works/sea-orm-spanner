mod common;
mod entity;

use {
    common::setup_test_database,
    entity::numeric_types,
    rust_decimal::Decimal,
    rust_decimal_macros::dec,
    sea_orm::{ActiveModelTrait, EntityTrait, Set},
    serial_test::serial,
};

mod numeric_type_tests {
    use super::*;

    #[tokio::test]
    #[serial]
    async fn test_numeric_insert_and_select() {
        let db = setup_test_database().await;
        let id = uuid::Uuid::new_v4().to_string();

        let value = dec!(12345.6789);
        let model = numeric_types::ActiveModel {
            id: Set(id.clone()),
            numeric_val: Set(value),
            numeric_nullable: Set(Some(dec!(999.999))),
        };

        let inserted = model.insert(&db).await.expect("Insert failed");
        assert_eq!(inserted.numeric_val, value);
        assert_eq!(inserted.numeric_nullable, Some(dec!(999.999)));

        let selected = numeric_types::Entity::find_by_id(&id)
            .one(&db)
            .await
            .expect("Select failed")
            .expect("Entity not found");
        assert_eq!(selected.numeric_val, value);
        assert_eq!(selected.numeric_nullable, Some(dec!(999.999)));
    }

    #[tokio::test]
    #[serial]
    async fn test_numeric_negative() {
        let db = setup_test_database().await;
        let id = uuid::Uuid::new_v4().to_string();

        let negative = dec!(-12345.6789);
        let model = numeric_types::ActiveModel {
            id: Set(id.clone()),
            numeric_val: Set(negative),
            numeric_nullable: Set(Some(dec!(-0.001))),
        };

        let inserted = model.insert(&db).await.expect("Insert failed");
        assert_eq!(inserted.numeric_val, negative);
        assert_eq!(inserted.numeric_nullable, Some(dec!(-0.001)));

        let selected = numeric_types::Entity::find_by_id(&id)
            .one(&db)
            .await
            .expect("Select failed")
            .expect("Entity not found");
        assert_eq!(selected.numeric_val, negative);
    }

    #[tokio::test]
    #[serial]
    async fn test_numeric_high_precision() {
        let db = setup_test_database().await;
        let id = uuid::Uuid::new_v4().to_string();

        let large_val = dec!(9999999999999999999.999999999);
        let model = numeric_types::ActiveModel {
            id: Set(id.clone()),
            numeric_val: Set(large_val),
            numeric_nullable: Set(None),
        };

        let inserted = model.insert(&db).await.expect("Insert failed");
        assert_eq!(inserted.numeric_val, large_val);

        let selected = numeric_types::Entity::find_by_id(&id)
            .one(&db)
            .await
            .expect("Select failed")
            .expect("Entity not found");
        assert_eq!(selected.numeric_val, large_val);
    }

    #[tokio::test]
    #[serial]
    async fn test_numeric_from_string() {
        let db = setup_test_database().await;
        let id = uuid::Uuid::new_v4().to_string();

        let value: Decimal = "123456789.123456789".parse().unwrap();
        let model = numeric_types::ActiveModel {
            id: Set(id.clone()),
            numeric_val: Set(value),
            numeric_nullable: Set(Some(dec!(1.0))),
        };

        let inserted = model.insert(&db).await.expect("Insert failed");
        assert_eq!(inserted.numeric_val, value);

        let selected = numeric_types::Entity::find_by_id(&id)
            .one(&db)
            .await
            .expect("Select failed")
            .expect("Entity not found");
        assert_eq!(selected.numeric_val, value);
    }
}
