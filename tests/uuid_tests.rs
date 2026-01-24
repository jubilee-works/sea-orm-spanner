mod common;
mod entity;

use common::setup_test_database;
use entity::uuid_types;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use serial_test::serial;
use uuid::Uuid;

#[tokio::test]
#[serial]
async fn test_uuid_insert_and_select() {
    let db = setup_test_database().await;

    let id = Uuid::new_v4().to_string();
    let uuid_val = Uuid::new_v4();
    let uuid_nullable = Some(Uuid::new_v4());

    let model = uuid_types::ActiveModel {
        id: Set(id.clone()),
        uuid_val: Set(uuid_val),
        uuid_nullable: Set(uuid_nullable),
    };

    model.insert(&db).await.expect("Failed to insert");

    let result = uuid_types::Entity::find_by_id(&id)
        .one(&db)
        .await
        .expect("Failed to query")
        .expect("Not found");

    assert_eq!(result.uuid_val, uuid_val);
    assert_eq!(result.uuid_nullable, uuid_nullable);
}

#[tokio::test]
#[serial]
async fn test_uuid_nullable_null() {
    let db = setup_test_database().await;

    let id = Uuid::new_v4().to_string();
    let uuid_val = Uuid::new_v4();

    let model = uuid_types::ActiveModel {
        id: Set(id.clone()),
        uuid_val: Set(uuid_val),
        uuid_nullable: Set(None),
    };

    model.insert(&db).await.expect("Failed to insert");

    let result = uuid_types::Entity::find_by_id(&id)
        .one(&db)
        .await
        .expect("Failed to query")
        .expect("Not found");

    assert_eq!(result.uuid_val, uuid_val);
    assert!(result.uuid_nullable.is_none());
}

#[tokio::test]
#[serial]
async fn test_uuid_update() {
    let db = setup_test_database().await;

    let id = Uuid::new_v4().to_string();
    let original_uuid = Uuid::new_v4();
    let updated_uuid = Uuid::new_v4();

    let model = uuid_types::ActiveModel {
        id: Set(id.clone()),
        uuid_val: Set(original_uuid),
        uuid_nullable: Set(None),
    };

    model.insert(&db).await.expect("Failed to insert");

    let mut active: uuid_types::ActiveModel = uuid_types::Entity::find_by_id(&id)
        .one(&db)
        .await
        .expect("Failed to query")
        .expect("Not found")
        .into();

    active.uuid_val = Set(updated_uuid);
    active.uuid_nullable = Set(Some(Uuid::new_v4()));
    active.update(&db).await.expect("Failed to update");

    let result = uuid_types::Entity::find_by_id(&id)
        .one(&db)
        .await
        .expect("Failed to query")
        .expect("Not found");

    assert_eq!(result.uuid_val, updated_uuid);
    assert!(result.uuid_nullable.is_some());
}

#[tokio::test]
#[serial]
async fn test_uuid_specific_value() {
    let db = setup_test_database().await;

    let id = Uuid::new_v4().to_string();
    let specific_uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();

    let model = uuid_types::ActiveModel {
        id: Set(id.clone()),
        uuid_val: Set(specific_uuid),
        uuid_nullable: Set(None),
    };

    model.insert(&db).await.expect("Failed to insert");

    let result = uuid_types::Entity::find_by_id(&id)
        .one(&db)
        .await
        .expect("Failed to query")
        .expect("Not found");

    assert_eq!(result.uuid_val, specific_uuid);
}

#[tokio::test]
#[serial]
async fn test_uuid_roundtrip() {
    let db = setup_test_database().await;

    let id = Uuid::new_v4().to_string();
    let uuid_val = Uuid::new_v4();

    let model = uuid_types::ActiveModel {
        id: Set(id.clone()),
        uuid_val: Set(uuid_val),
        uuid_nullable: Set(None),
    };

    model.insert(&db).await.expect("Failed to insert");

    let result = uuid_types::Entity::find_by_id(&id)
        .one(&db)
        .await
        .expect("Failed to query")
        .expect("Not found");

    assert_eq!(result.uuid_val, uuid_val);
    assert_eq!(result.uuid_val.to_string(), uuid_val.to_string());
}
