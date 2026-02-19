use sea_orm_migration_spanner::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table_spanner(
                SpannerTableBuilder::new()
                    .table("users")
                    .uuid("id", true)
                    .string("email", Some(255), true)
                    .string("nickname", Some(100), false)
                    .string("profile_image_url", None, false)
                    .date("birth_date", false)
                    .string("gender", Some(20), false)
                    .string("country", Some(10), false)
                    .timestamp("created_at", false)
                    .timestamp("updated_at", false)
                    .primary_key(["id"]),
            )
            .await?;

        manager
            .create_table_spanner(
                SpannerTableBuilder::new()
                    .table("user_settings")
                    .uuid("user_id", true)
                    .string("default_export", Some(20), false)
                    .string("theme", Some(10), false)
                    .string("font_size", Some(10), false)
                    .string("language", Some(5), false)
                    .string("agent_tone", Some(20), false)
                    .string("planning_style", Some(20), false)
                    .primary_key(["user_id"]),
            )
            .await?;

        manager
            .execute(
                "ALTER TABLE user_settings ADD CONSTRAINT fk_user_settings_user_id \
                 FOREIGN KEY (user_id) REFERENCES users (id)",
            )
            .await?;

        manager
            .create_table_spanner(
                SpannerTableBuilder::new()
                    .table("chat_sessions")
                    .uuid("id", true)
                    .uuid("user_id", true)
                    .string("calendar_id", Some(100), true)
                    .string("title", Some(500), false)
                    .timestamp("created_at", false)
                    .timestamp("last_message_at", false)
                    .primary_key(["id"]),
            )
            .await?;

        manager
            .execute(
                "ALTER TABLE chat_sessions ADD CONSTRAINT fk_chat_sessions_user_id \
                 FOREIGN KEY (user_id) REFERENCES users (id)",
            )
            .await?;

        manager
            .create_index_spanner(
                SpannerIndexBuilder::new()
                    .name("idx_chat_sessions_user")
                    .table("chat_sessions")
                    .col("user_id")
                    .col("last_message_at"),
            )
            .await?;

        manager
            .create_table_spanner(
                SpannerTableBuilder::new()
                    .table("chat_messages")
                    .uuid("id", true)
                    .uuid("session_id", true)
                    .string("role", Some(20), true)
                    .string("content", None, true)
                    .string("input_type", Some(20), false)
                    .int64("token_usage", false)
                    .int64("latency_ms", false)
                    .timestamp("created_at", false)
                    .primary_key(["id"]),
            )
            .await?;

        manager
            .execute(
                "ALTER TABLE chat_messages ADD CONSTRAINT fk_chat_messages_session_id \
                 FOREIGN KEY (session_id) REFERENCES chat_sessions (id)",
            )
            .await?;

        manager
            .create_index_spanner(
                SpannerIndexBuilder::new()
                    .name("idx_chat_messages_session")
                    .table("chat_messages")
                    .col("session_id")
                    .col("created_at"),
            )
            .await?;

        manager
            .create_table_spanner(
                SpannerTableBuilder::new()
                    .table("generated_schedules")
                    .uuid("id", true)
                    .uuid("user_id", true)
                    .uuid("session_id", false)
                    .uuid("message_id", false)
                    .string("title", Some(500), false)
                    .date("date_start", true)
                    .date("date_end", true)
                    .int64("version", false)
                    .bool("exported", false)
                    .string("export_target", Some(20), false)
                    .timestamp("exported_at", false)
                    .timestamp("created_at", false)
                    .timestamp("updated_at", false)
                    .primary_key(["id"]),
            )
            .await?;

        manager
            .execute(
                "ALTER TABLE generated_schedules ADD CONSTRAINT fk_generated_schedules_user_id \
                 FOREIGN KEY (user_id) REFERENCES users (id)",
            )
            .await?;

        manager
            .execute(
                "ALTER TABLE generated_schedules ADD CONSTRAINT fk_generated_schedules_session_id \
                 FOREIGN KEY (session_id) REFERENCES chat_sessions (id)",
            )
            .await?;

        manager
            .execute(
                "ALTER TABLE generated_schedules ADD CONSTRAINT fk_generated_schedules_message_id \
                 FOREIGN KEY (message_id) REFERENCES chat_messages (id)",
            )
            .await?;

        manager
            .create_index_spanner(
                SpannerIndexBuilder::new()
                    .name("idx_schedules_user")
                    .table("generated_schedules")
                    .col("user_id")
                    .col("created_at"),
            )
            .await?;

        manager
            .create_table_spanner(
                SpannerTableBuilder::new()
                    .table("schedule_events")
                    .uuid("id", true)
                    .uuid("schedule_id", true)
                    .string("title", Some(500), true)
                    .timestamp("start_time", true)
                    .timestamp("end_time", true)
                    .string("location_name", Some(500), false)
                    .string("location_address", None, false)
                    .numeric("location_lat", false)
                    .numeric("location_lng", false)
                    .string("category", Some(50), false)
                    .string("icon", Some(50), false)
                    .string("notes", None, false)
                    .numeric("confidence_score", false)
                    .int64("sort_order", false)
                    .primary_key(["id"]),
            )
            .await?;

        manager
            .execute(
                "ALTER TABLE schedule_events ADD CONSTRAINT fk_schedule_events_schedule_id \
                 FOREIGN KEY (schedule_id) REFERENCES generated_schedules (id)",
            )
            .await?;

        manager
            .create_index_spanner(
                SpannerIndexBuilder::new()
                    .name("idx_events_schedule")
                    .table("schedule_events")
                    .col("schedule_id")
                    .col("start_time"),
            )
            .await?;

        manager
            .create_table_spanner(
                SpannerTableBuilder::new()
                    .table("user_preferences")
                    .uuid("user_id", true)
                    .json("personality_tags", false)
                    .json("likes", false)
                    .json("dislikes", false)
                    .string("custom_instructions", None, false)
                    .timestamp("updated_at", false)
                    .primary_key(["user_id"]),
            )
            .await?;

        manager
            .execute(
                "ALTER TABLE user_preferences ADD CONSTRAINT fk_user_preferences_user_id \
                 FOREIGN KEY (user_id) REFERENCES users (id)",
            )
            .await?;

        manager
            .create_table_spanner(
                SpannerTableBuilder::new()
                    .table("agent_memories")
                    .uuid("id", true)
                    .uuid("user_id", true)
                    .string("fact", None, true)
                    .string("source", Some(50), false)
                    .uuid("source_id", false)
                    .timestamp("learned_at", false)
                    .primary_key(["id"]),
            )
            .await?;

        manager
            .execute(
                "ALTER TABLE agent_memories ADD CONSTRAINT fk_agent_memories_user_id \
                 FOREIGN KEY (user_id) REFERENCES users (id)",
            )
            .await?;

        manager
            .create_index_spanner(
                SpannerIndexBuilder::new()
                    .name("idx_memories_user")
                    .table("agent_memories")
                    .col("user_id")
                    .col("learned_at"),
            )
            .await?;

        manager
            .create_table_spanner(
                SpannerTableBuilder::new()
                    .table("feedbacks")
                    .uuid("id", true)
                    .uuid("user_id", true)
                    .string("target_type", Some(50), true)
                    .uuid("target_id", true)
                    .string("rating", Some(20), false)
                    .string("action", Some(20), false)
                    .json("details", false)
                    .timestamp("created_at", false)
                    .primary_key(["id"]),
            )
            .await?;

        manager
            .execute(
                "ALTER TABLE feedbacks ADD CONSTRAINT fk_feedbacks_user_id \
                 FOREIGN KEY (user_id) REFERENCES users (id)",
            )
            .await?;

        manager
            .create_index_spanner(
                SpannerIndexBuilder::new()
                    .name("idx_feedbacks_target")
                    .table("feedbacks")
                    .col("target_type")
                    .col("target_id"),
            )
            .await?;

        manager
            .create_table_spanner(
                SpannerTableBuilder::new()
                    .table("daily_quizzes")
                    .uuid("id", true)
                    .string("question", None, true)
                    .json("options", true)
                    .string("context_field", Some(100), false)
                    .int64("priority_tier", false)
                    .bool("active", false)
                    .primary_key(["id"]),
            )
            .await?;

        manager
            .create_table_spanner(
                SpannerTableBuilder::new()
                    .table("quiz_responses")
                    .uuid("id", true)
                    .uuid("user_id", true)
                    .uuid("quiz_id", true)
                    .json("answer", true)
                    .int64("points_earned", false)
                    .timestamp("answered_at", false)
                    .primary_key(["id"]),
            )
            .await?;

        manager
            .execute(
                "ALTER TABLE quiz_responses ADD CONSTRAINT fk_quiz_responses_user_id \
                 FOREIGN KEY (user_id) REFERENCES users (id)",
            )
            .await?;

        manager
            .execute(
                "ALTER TABLE quiz_responses ADD CONSTRAINT fk_quiz_responses_quiz_id \
                 FOREIGN KEY (quiz_id) REFERENCES daily_quizzes (id)",
            )
            .await?;

        manager
            .create_index_spanner(
                SpannerIndexBuilder::new()
                    .name("idx_quiz_responses_user")
                    .table("quiz_responses")
                    .col("user_id")
                    .col("answered_at"),
            )
            .await?;

        manager
            .create_table_spanner(
                SpannerTableBuilder::new()
                    .table("user_points")
                    .uuid("user_id", true)
                    .int64("total_points", false)
                    .int64("used_points", false)
                    .primary_key(["user_id"]),
            )
            .await?;

        manager
            .execute(
                "ALTER TABLE user_points ADD CONSTRAINT fk_user_points_user_id \
                 FOREIGN KEY (user_id) REFERENCES users (id)",
            )
            .await?;

        manager
            .create_table_spanner(
                SpannerTableBuilder::new()
                    .table("llm_requests")
                    .uuid("id", true)
                    .uuid("user_id", true)
                    .uuid("session_id", false)
                    .string("intent", Some(50), false)
                    .int64("input_tokens", false)
                    .int64("output_tokens", false)
                    .int64("latency_ms", false)
                    .string("status", Some(20), false)
                    .string("error_message", None, false)
                    .timestamp("created_at", false)
                    .timestamp("completed_at", false)
                    .primary_key(["id"]),
            )
            .await?;

        manager
            .execute(
                "ALTER TABLE llm_requests ADD CONSTRAINT fk_llm_requests_user_id \
                 FOREIGN KEY (user_id) REFERENCES users (id)",
            )
            .await?;

        manager
            .execute(
                "ALTER TABLE llm_requests ADD CONSTRAINT fk_llm_requests_session_id \
                 FOREIGN KEY (session_id) REFERENCES chat_sessions (id)",
            )
            .await?;

        manager
            .create_index_spanner(
                SpannerIndexBuilder::new()
                    .name("idx_llm_requests_user_status")
                    .table("llm_requests")
                    .col("user_id")
                    .col("status"),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let indices = [
            "idx_llm_requests_user_status",
            "idx_quiz_responses_user",
            "idx_feedbacks_target",
            "idx_memories_user",
            "idx_events_schedule",
            "idx_schedules_user",
            "idx_chat_messages_session",
            "idx_chat_sessions_user",
        ];
        for idx in indices {
            manager.drop_index_if_exists(idx).await?;
        }

        let tables = [
            "llm_requests",
            "user_points",
            "quiz_responses",
            "daily_quizzes",
            "feedbacks",
            "agent_memories",
            "user_preferences",
            "schedule_events",
            "generated_schedules",
            "chat_messages",
            "chat_sessions",
            "user_settings",
            "users",
        ];
        for table in tables {
            manager.drop_table_if_exists(table).await?;
        }
        Ok(())
    }
}
