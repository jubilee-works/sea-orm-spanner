mod common;
mod entity;

use chrono::Utc;
use common::setup_test_database;
use entity::{category, post, product, user};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, ModelTrait, PaginatorTrait,
    QueryFilter, QueryOrder, QuerySelect, Set,
};
use serial_test::serial;

mod insert_tests {
    use super::*;

    #[tokio::test]
    #[serial]
    async fn test_insert_single_entity() {
        let db = setup_test_database().await;

        let new_user = user::ActiveModel {
            id: Set(uuid::Uuid::new_v4().to_string()),
            name: Set("Alice".to_string()),
            email: Set("alice@example.com".to_string()),
            age: Set(Some(25)),
            active: Set(true),
            created_at: Set(Utc::now()),
        };

        let result = new_user.insert(&db).await;
        assert!(result.is_ok());

        let inserted = result.unwrap();
        assert_eq!(inserted.name, "Alice");
        assert_eq!(inserted.email, "alice@example.com");
    }

    #[tokio::test]
    #[serial]
    async fn test_insert_with_null_fields() {
        
        let db = setup_test_database().await;

        let new_user = user::ActiveModel {
            id: Set(uuid::Uuid::new_v4().to_string()),
            name: Set("Bob".to_string()),
            email: Set("bob@example.com".to_string()),
            age: Set(None),
            active: Set(false),
            created_at: Set(Utc::now()),
        };

        let result = new_user.insert(&db).await;
        assert!(result.is_ok());

        let inserted = result.unwrap();
        assert!(inserted.age.is_none());
    }

    #[tokio::test]
    #[serial]
    async fn test_insert_multiple_entities() {
        
        let db = setup_test_database().await;

        for i in 0..5 {
            let cat = category::ActiveModel {
                id: Set(uuid::Uuid::new_v4().to_string()),
                name: Set(format!("Category {}", i)),
                description: Set(Some(format!("Description for category {}", i))),
            };
            cat.insert(&db).await.expect("Insert failed");
        }

        let count = category::Entity::find().count(&db).await.unwrap();
        assert_eq!(count, 5);
    }
}

mod select_tests {
    use super::*;

    async fn setup_users(db: &sea_orm::DatabaseConnection) -> Vec<String> {
        let mut ids = Vec::new();
        let users = vec![
            ("Alice", "alice@example.com", Some(25), true),
            ("Bob", "bob@example.com", Some(30), true),
            ("Charlie", "charlie@example.com", None, false),
            ("Diana", "diana@example.com", Some(28), true),
            ("Eve", "eve@example.com", Some(35), false),
        ];

        for (name, email, age, active) in users {
            let id = uuid::Uuid::new_v4().to_string();
            ids.push(id.clone());
            let u = user::ActiveModel {
                id: Set(id),
                name: Set(name.to_string()),
                email: Set(email.to_string()),
                age: Set(age),
                active: Set(active),
                created_at: Set(Utc::now()),
            };
            u.insert(db).await.unwrap();
        }
        ids
    }

    #[tokio::test]
    #[serial]
    async fn test_find_by_id() {
        
        let db = setup_test_database().await;
        let ids = setup_users(&db).await;

        let user = user::Entity::find_by_id(&ids[0]).one(&db).await.unwrap();
        assert!(user.is_some());
        assert_eq!(user.unwrap().name, "Alice");
    }

    #[tokio::test]
    #[serial]
    async fn test_find_all() {
        
        let db = setup_test_database().await;
        setup_users(&db).await;

        let users = user::Entity::find().all(&db).await.unwrap();
        assert_eq!(users.len(), 5);
    }

    #[tokio::test]
    #[serial]
    async fn test_find_with_filter() {
        
        let db = setup_test_database().await;
        setup_users(&db).await;

        let active_users = user::Entity::find()
            .filter(user::Column::Active.eq(true))
            .all(&db)
            .await
            .unwrap();
        assert_eq!(active_users.len(), 3);
    }

    #[tokio::test]
    #[serial]
    async fn test_find_with_order_by() {
        
        let db = setup_test_database().await;
        setup_users(&db).await;

        let users = user::Entity::find()
            .order_by_asc(user::Column::Name)
            .all(&db)
            .await
            .unwrap();

        assert_eq!(users[0].name, "Alice");
        assert_eq!(users[4].name, "Eve");
    }

    #[tokio::test]
    #[serial]
    async fn test_find_with_limit() {
        
        let db = setup_test_database().await;
        setup_users(&db).await;

        let users = user::Entity::find()
            .order_by_asc(user::Column::Name)
            .limit(2)
            .all(&db)
            .await
            .unwrap();

        assert_eq!(users.len(), 2);
    }

    #[tokio::test]
    #[serial]
    async fn test_find_with_offset() {
        
        let db = setup_test_database().await;
        setup_users(&db).await;

        let users = user::Entity::find()
            .order_by_asc(user::Column::Name)
            .offset(2)
            .limit(2)
            .all(&db)
            .await
            .unwrap();

        assert_eq!(users.len(), 2);
        assert_eq!(users[0].name, "Charlie");
    }

    #[tokio::test]
    #[serial]
    async fn test_find_one() {
        
        let db = setup_test_database().await;
        setup_users(&db).await;

        let user = user::Entity::find()
            .filter(user::Column::Email.eq("bob@example.com"))
            .one(&db)
            .await
            .unwrap();

        assert!(user.is_some());
        assert_eq!(user.unwrap().name, "Bob");
    }

    #[tokio::test]
    #[serial]
    async fn test_count() {
        
        let db = setup_test_database().await;
        setup_users(&db).await;

        let total = user::Entity::find().count(&db).await.unwrap();
        assert_eq!(total, 5);

        let active_count = user::Entity::find()
            .filter(user::Column::Active.eq(true))
            .count(&db)
            .await
            .unwrap();
        assert_eq!(active_count, 3);
    }

    #[tokio::test]
    #[serial]
    async fn test_find_with_contains() {
        
        let db = setup_test_database().await;
        setup_users(&db).await;

        let users = user::Entity::find()
            .filter(user::Column::Name.contains("a"))
            .all(&db)
            .await
            .unwrap();

        assert!(users.len() >= 2);
    }

    #[tokio::test]
    #[serial]
    async fn test_find_with_in_list() {
        
        let db = setup_test_database().await;
        setup_users(&db).await;

        let users = user::Entity::find()
            .filter(user::Column::Name.is_in(["Alice", "Bob"]))
            .all(&db)
            .await
            .unwrap();

        assert_eq!(users.len(), 2);
    }

    #[tokio::test]
    #[serial]
    async fn test_find_with_is_null() {
        
        let db = setup_test_database().await;
        setup_users(&db).await;

        let users = user::Entity::find()
            .filter(user::Column::Age.is_null())
            .all(&db)
            .await
            .unwrap();

        assert_eq!(users.len(), 1);
        assert_eq!(users[0].name, "Charlie");
    }
}

mod update_tests {
    use super::*;

    #[tokio::test]
    #[serial]
    async fn test_update_single_field() {
        
        let db = setup_test_database().await;

        let id = uuid::Uuid::new_v4().to_string();
        let new_user = user::ActiveModel {
            id: Set(id.clone()),
            name: Set("Original".to_string()),
            email: Set("original@example.com".to_string()),
            age: Set(Some(20)),
            active: Set(true),
            created_at: Set(Utc::now()),
        };
        new_user.insert(&db).await.unwrap();

        let user = user::Entity::find_by_id(&id).one(&db).await.unwrap().unwrap();
        let mut active: user::ActiveModel = user.into_active_model();
        active.name = Set("Updated".to_string());
        active.update(&db).await.unwrap();

        let updated = user::Entity::find_by_id(&id).one(&db).await.unwrap().unwrap();
        assert_eq!(updated.name, "Updated");
        assert_eq!(updated.email, "original@example.com");
    }

    #[tokio::test]
    #[serial]
    async fn test_update_multiple_fields() {
        
        let db = setup_test_database().await;

        let id = uuid::Uuid::new_v4().to_string();
        let new_user = user::ActiveModel {
            id: Set(id.clone()),
            name: Set("Original".to_string()),
            email: Set("original@example.com".to_string()),
            age: Set(Some(20)),
            active: Set(true),
            created_at: Set(Utc::now()),
        };
        new_user.insert(&db).await.unwrap();

        let user = user::Entity::find_by_id(&id).one(&db).await.unwrap().unwrap();
        let mut active: user::ActiveModel = user.into_active_model();
        active.name = Set("New Name".to_string());
        active.email = Set("new@example.com".to_string());
        active.age = Set(Some(30));
        active.update(&db).await.unwrap();

        let updated = user::Entity::find_by_id(&id).one(&db).await.unwrap().unwrap();
        assert_eq!(updated.name, "New Name");
        assert_eq!(updated.email, "new@example.com");
        assert_eq!(updated.age, Some(30));
    }

    #[tokio::test]
    #[serial]
    async fn test_update_to_null() {
        
        let db = setup_test_database().await;

        let id = uuid::Uuid::new_v4().to_string();
        let new_user = user::ActiveModel {
            id: Set(id.clone()),
            name: Set("Test".to_string()),
            email: Set("test@example.com".to_string()),
            age: Set(Some(25)),
            active: Set(true),
            created_at: Set(Utc::now()),
        };
        new_user.insert(&db).await.unwrap();

        let user = user::Entity::find_by_id(&id).one(&db).await.unwrap().unwrap();
        let mut active: user::ActiveModel = user.into_active_model();
        active.age = Set(None);
        active.update(&db).await.unwrap();

        let updated = user::Entity::find_by_id(&id).one(&db).await.unwrap().unwrap();
        assert!(updated.age.is_none());
    }
}

mod delete_tests {
    use super::*;

    #[tokio::test]
    #[serial]
    async fn test_delete_single_entity() {
        
        let db = setup_test_database().await;

        let id = uuid::Uuid::new_v4().to_string();
        let new_user = user::ActiveModel {
            id: Set(id.clone()),
            name: Set("ToDelete".to_string()),
            email: Set("delete@example.com".to_string()),
            age: Set(None),
            active: Set(true),
            created_at: Set(Utc::now()),
        };
        new_user.insert(&db).await.unwrap();

        let user = user::Entity::find_by_id(&id).one(&db).await.unwrap().unwrap();
        user.delete(&db).await.unwrap();

        let deleted = user::Entity::find_by_id(&id).one(&db).await.unwrap();
        assert!(deleted.is_none());
    }

    #[tokio::test]
    #[serial]
    async fn test_delete_by_filter() {
        
        let db = setup_test_database().await;

        for i in 0..5 {
            let cat = category::ActiveModel {
                id: Set(uuid::Uuid::new_v4().to_string()),
                name: Set(format!("Category {}", i)),
                description: Set(if i % 2 == 0 {
                    Some("Even".to_string())
                } else {
                    Some("Odd".to_string())
                }),
            };
            cat.insert(&db).await.unwrap();
        }

        let result = category::Entity::delete_many()
            .filter(category::Column::Description.eq("Even"))
            .exec(&db)
            .await
            .unwrap();

        assert_eq!(result.rows_affected, 3);

        let remaining = category::Entity::find().count(&db).await.unwrap();
        assert_eq!(remaining, 2);
    }
}

mod relation_tests {
    use super::*;

    async fn setup_users_and_posts(db: &sea_orm::DatabaseConnection) -> (String, Vec<String>) {
        let user_id = uuid::Uuid::new_v4().to_string();
        let u = user::ActiveModel {
            id: Set(user_id.clone()),
            name: Set("Author".to_string()),
            email: Set("author@example.com".to_string()),
            age: Set(Some(30)),
            active: Set(true),
            created_at: Set(Utc::now()),
        };
        u.insert(db).await.unwrap();

        let mut post_ids = Vec::new();
        for i in 0..3 {
            let post_id = uuid::Uuid::new_v4().to_string();
            post_ids.push(post_id.clone());
            let p = post::ActiveModel {
                id: Set(post_id),
                user_id: Set(user_id.clone()),
                title: Set(format!("Post {}", i)),
                content: Set(format!("Content of post {}", i)),
                published: Set(i % 2 == 0),
                created_at: Set(Utc::now()),
            };
            p.insert(db).await.unwrap();
        }

        (user_id, post_ids)
    }

    #[tokio::test]
    #[serial]
    async fn test_find_related() {
        
        let db = setup_test_database().await;
        let (user_id, _) = setup_users_and_posts(&db).await;

        let user = user::Entity::find_by_id(&user_id)
            .one(&db)
            .await
            .unwrap()
            .unwrap();

        let posts = user.find_related(post::Entity).all(&db).await.unwrap();
        assert_eq!(posts.len(), 3);
    }

    #[tokio::test]
    #[serial]
    async fn test_find_with_related() {
        
        let db = setup_test_database().await;
        setup_users_and_posts(&db).await;

        let users_with_posts = user::Entity::find()
            .find_with_related(post::Entity)
            .all(&db)
            .await
            .unwrap();

        assert_eq!(users_with_posts.len(), 1);
        assert_eq!(users_with_posts[0].1.len(), 3);
    }
}

mod pagination_tests {
    use super::*;

    #[tokio::test]
    #[serial]
    async fn test_paginator() {
        
        let db = setup_test_database().await;

        for i in 0..25 {
            let cat = category::ActiveModel {
                id: Set(uuid::Uuid::new_v4().to_string()),
                name: Set(format!("Category {}", i)),
                description: Set(None),
            };
            cat.insert(&db).await.unwrap();
        }

        let paginator = category::Entity::find()
            .order_by_asc(category::Column::Name)
            .paginate(&db, 10);

        let page1 = paginator.fetch_page(0).await.unwrap();
        assert_eq!(page1.len(), 10);

        let page2 = paginator.fetch_page(1).await.unwrap();
        assert_eq!(page2.len(), 10);

        let page3 = paginator.fetch_page(2).await.unwrap();
        assert_eq!(page3.len(), 5);

        let total_pages = paginator.num_pages().await.unwrap();
        assert_eq!(total_pages, 3);
    }
}

mod complex_query_tests {
    use super::*;

    async fn setup_products(db: &sea_orm::DatabaseConnection) {
        let cat1_id = uuid::Uuid::new_v4().to_string();
        let cat2_id = uuid::Uuid::new_v4().to_string();

        category::ActiveModel {
            id: Set(cat1_id.clone()),
            name: Set("Electronics".to_string()),
            description: Set(Some("Electronic devices".to_string())),
        }
        .insert(db)
        .await
        .unwrap();

        category::ActiveModel {
            id: Set(cat2_id.clone()),
            name: Set("Books".to_string()),
            description: Set(None),
        }
        .insert(db)
        .await
        .unwrap();

        let products = vec![
            (&cat1_id, "Phone", 999.99, 50, true),
            (&cat1_id, "Laptop", 1499.99, 30, true),
            (&cat1_id, "Tablet", 599.99, 100, false),
            (&cat2_id, "Novel", 19.99, 200, true),
            (&cat2_id, "Textbook", 79.99, 50, true),
        ];

        for (cat_id, name, price, qty, active) in products {
            product::ActiveModel {
                id: Set(uuid::Uuid::new_v4().to_string()),
                category_id: Set(cat_id.clone()),
                name: Set(name.to_string()),
                price: Set(price),
                quantity: Set(qty),
                active: Set(active),
            }
            .insert(db)
            .await
            .unwrap();
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_complex_filter() {
        
        let db = setup_test_database().await;
        setup_products(&db).await;

        let expensive_active = product::Entity::find()
            .filter(product::Column::Price.gt(100.0))
            .filter(product::Column::Active.eq(true))
            .all(&db)
            .await
            .unwrap();

        assert_eq!(expensive_active.len(), 2);
    }

    #[tokio::test]
    #[serial]
    async fn test_or_filter() {
        
        let db = setup_test_database().await;
        setup_products(&db).await;

        use sea_orm::Condition;

        let products = product::Entity::find()
            .filter(
                Condition::any()
                    .add(product::Column::Price.lt(50.0))
                    .add(product::Column::Quantity.gt(100)),
            )
            .all(&db)
            .await
            .unwrap();

        assert!(products.len() >= 1);
    }

    #[tokio::test]
    #[serial]
    async fn test_select_only() {
        
        let db = setup_test_database().await;
        setup_products(&db).await;

        let names: Vec<String> = product::Entity::find()
            .select_only()
            .column(product::Column::Name)
            .into_tuple()
            .all(&db)
            .await
            .unwrap();

        assert_eq!(names.len(), 5);
    }
}
