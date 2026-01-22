use sea_orm_migration_spanner::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

// Users table
#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
    Email,
    Nickname,
    ProfileImageUrl,
    BirthDate,
    Gender,
    Country,
    CreatedAt,
    UpdatedAt,
}

// User settings table
#[derive(DeriveIden)]
enum UserSettings {
    Table,
    UserId,
    DefaultExport,
    Theme,
    FontSize,
    Language,
    AgentTone,
    PlanningStyle,
}

// Chat sessions table
#[derive(DeriveIden)]
enum ChatSessions {
    Table,
    Id,
    UserId,
    CalendarId,
    Title,
    CreatedAt,
    LastMessageAt,
}

// Chat messages table
#[derive(DeriveIden)]
enum ChatMessages {
    Table,
    Id,
    SessionId,
    Role,
    Content,
    InputType,
    TokenUsage,
    LatencyMs,
    CreatedAt,
}

// Generated schedules table
#[derive(DeriveIden)]
enum GeneratedSchedules {
    Table,
    Id,
    UserId,
    SessionId,
    MessageId,
    Title,
    DateStart,
    DateEnd,
    Version,
    Exported,
    ExportTarget,
    ExportedAt,
    CreatedAt,
    UpdatedAt,
}

// Schedule events table
#[derive(DeriveIden)]
enum ScheduleEvents {
    Table,
    Id,
    ScheduleId,
    Title,
    StartTime,
    EndTime,
    LocationName,
    LocationAddress,
    LocationLat,
    LocationLng,
    Category,
    Icon,
    Notes,
    ConfidenceScore,
    SortOrder,
}

// User preferences table
#[derive(DeriveIden)]
enum UserPreferences {
    Table,
    UserId,
    PersonalityTags,
    Likes,
    Dislikes,
    CustomInstructions,
    UpdatedAt,
}

// Agent memories table
#[derive(DeriveIden)]
enum AgentMemories {
    Table,
    Id,
    UserId,
    Fact,
    Source,
    SourceId,
    LearnedAt,
}

// Feedbacks table
#[derive(DeriveIden)]
enum Feedbacks {
    Table,
    Id,
    UserId,
    TargetType,
    TargetId,
    Rating,
    Action,
    Details,
    CreatedAt,
}

// Daily quizzes table
#[derive(DeriveIden)]
enum DailyQuizzes {
    Table,
    Id,
    Question,
    Options,
    ContextField,
    PriorityTier,
    Active,
}

// Quiz responses table
#[derive(DeriveIden)]
enum QuizResponses {
    Table,
    Id,
    UserId,
    QuizId,
    Answer,
    PointsEarned,
    AnsweredAt,
}

// User points table
#[derive(DeriveIden)]
enum UserPoints {
    Table,
    UserId,
    TotalPoints,
    UsedPoints,
}

// LLM requests table
#[derive(DeriveIden)]
enum LlmRequests {
    Table,
    Id,
    UserId,
    SessionId,
    Intent,
    InputTokens,
    OutputTokens,
    LatencyMs,
    Status,
    ErrorMessage,
    CreatedAt,
    CompletedAt,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Users table
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Users::Id).uuid().not_null().primary_key())
                    .col(
                        ColumnDef::new(Users::Email)
                            .string_len(255)
                            .unique_key()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Users::Nickname).string_len(100))
                    .col(ColumnDef::new(Users::ProfileImageUrl).text())
                    .col(ColumnDef::new(Users::BirthDate).date())
                    .col(ColumnDef::new(Users::Gender).string_len(20))
                    .col(ColumnDef::new(Users::Country).string_len(10))
                    .col(ColumnDef::new(Users::CreatedAt).timestamp())
                    .col(ColumnDef::new(Users::UpdatedAt).timestamp())
                    .to_owned(),
            )
            .await?;

        // User settings table
        manager
            .create_table(
                Table::create()
                    .table(UserSettings::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(UserSettings::UserId)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(UserSettings::DefaultExport)
                            .string_len(20)
                            .default("timetree"),
                    )
                    .col(
                        ColumnDef::new(UserSettings::Theme)
                            .string_len(10)
                            .default("system"),
                    )
                    .col(
                        ColumnDef::new(UserSettings::FontSize)
                            .string_len(10)
                            .default("medium"),
                    )
                    .col(
                        ColumnDef::new(UserSettings::Language)
                            .string_len(5)
                            .default("ko"),
                    )
                    .col(
                        ColumnDef::new(UserSettings::AgentTone)
                            .string_len(20)
                            .default("friendly"),
                    )
                    .col(
                        ColumnDef::new(UserSettings::PlanningStyle)
                            .string_len(20)
                            .default("relaxed"),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-user_settings-user_id")
                            .from(UserSettings::Table, UserSettings::UserId)
                            .to(Users::Table, Users::Id),
                    )
                    .to_owned(),
            )
            .await?;

        // Chat sessions table
        manager
            .create_table(
                Table::create()
                    .table(ChatSessions::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ChatSessions::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ChatSessions::UserId).uuid().not_null())
                    .col(
                        ColumnDef::new(ChatSessions::CalendarId)
                            .string_len(100)
                            .not_null(),
                    )
                    .col(ColumnDef::new(ChatSessions::Title).string_len(500))
                    .col(ColumnDef::new(ChatSessions::CreatedAt).timestamp())
                    .col(ColumnDef::new(ChatSessions::LastMessageAt).timestamp())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-chat_sessions-user_id")
                            .from(ChatSessions::Table, ChatSessions::UserId)
                            .to(Users::Table, Users::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_chat_sessions_user")
                    .table(ChatSessions::Table)
                    .col(ChatSessions::UserId)
                    .col(ChatSessions::LastMessageAt)
                    .to_owned(),
            )
            .await?;

        // Chat messages table
        manager
            .create_table(
                Table::create()
                    .table(ChatMessages::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ChatMessages::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ChatMessages::SessionId).uuid().not_null())
                    .col(ColumnDef::new(ChatMessages::Role).string_len(20).not_null())
                    .col(ColumnDef::new(ChatMessages::Content).text().not_null())
                    .col(
                        ColumnDef::new(ChatMessages::InputType)
                            .string_len(20)
                            .default("text"),
                    )
                    .col(ColumnDef::new(ChatMessages::TokenUsage).integer())
                    .col(ColumnDef::new(ChatMessages::LatencyMs).integer())
                    .col(ColumnDef::new(ChatMessages::CreatedAt).timestamp())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-chat_messages-session_id")
                            .from(ChatMessages::Table, ChatMessages::SessionId)
                            .to(ChatSessions::Table, ChatSessions::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_chat_messages_session")
                    .table(ChatMessages::Table)
                    .col(ChatMessages::SessionId)
                    .col(ChatMessages::CreatedAt)
                    .to_owned(),
            )
            .await?;

        // Generated schedules table
        manager
            .create_table(
                Table::create()
                    .table(GeneratedSchedules::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(GeneratedSchedules::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(GeneratedSchedules::UserId).uuid().not_null())
                    .col(ColumnDef::new(GeneratedSchedules::SessionId).uuid())
                    .col(ColumnDef::new(GeneratedSchedules::MessageId).uuid())
                    .col(ColumnDef::new(GeneratedSchedules::Title).string_len(500))
                    .col(
                        ColumnDef::new(GeneratedSchedules::DateStart)
                            .date()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(GeneratedSchedules::DateEnd)
                            .date()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(GeneratedSchedules::Version)
                            .integer()
                            .default(1),
                    )
                    .col(
                        ColumnDef::new(GeneratedSchedules::Exported)
                            .boolean()
                            .default(false),
                    )
                    .col(ColumnDef::new(GeneratedSchedules::ExportTarget).string_len(20))
                    .col(ColumnDef::new(GeneratedSchedules::ExportedAt).timestamp())
                    .col(ColumnDef::new(GeneratedSchedules::CreatedAt).timestamp())
                    .col(ColumnDef::new(GeneratedSchedules::UpdatedAt).timestamp())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-generated_schedules-user_id")
                            .from(GeneratedSchedules::Table, GeneratedSchedules::UserId)
                            .to(Users::Table, Users::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-generated_schedules-session_id")
                            .from(GeneratedSchedules::Table, GeneratedSchedules::SessionId)
                            .to(ChatSessions::Table, ChatSessions::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-generated_schedules-message_id")
                            .from(GeneratedSchedules::Table, GeneratedSchedules::MessageId)
                            .to(ChatMessages::Table, ChatMessages::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_schedules_user")
                    .table(GeneratedSchedules::Table)
                    .col(GeneratedSchedules::UserId)
                    .col(GeneratedSchedules::CreatedAt)
                    .to_owned(),
            )
            .await?;

        // Schedule events table
        manager
            .create_table(
                Table::create()
                    .table(ScheduleEvents::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ScheduleEvents::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ScheduleEvents::ScheduleId).uuid().not_null())
                    .col(
                        ColumnDef::new(ScheduleEvents::Title)
                            .string_len(500)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ScheduleEvents::StartTime)
                            .timestamp()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ScheduleEvents::EndTime)
                            .timestamp()
                            .not_null(),
                    )
                    .col(ColumnDef::new(ScheduleEvents::LocationName).string_len(500))
                    .col(ColumnDef::new(ScheduleEvents::LocationAddress).text())
                    .col(ColumnDef::new(ScheduleEvents::LocationLat).decimal_len(10, 8))
                    .col(ColumnDef::new(ScheduleEvents::LocationLng).decimal_len(11, 8))
                    .col(ColumnDef::new(ScheduleEvents::Category).string_len(50))
                    .col(ColumnDef::new(ScheduleEvents::Icon).string_len(50))
                    .col(ColumnDef::new(ScheduleEvents::Notes).text())
                    .col(ColumnDef::new(ScheduleEvents::ConfidenceScore).decimal_len(3, 2))
                    .col(ColumnDef::new(ScheduleEvents::SortOrder).integer())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-schedule_events-schedule_id")
                            .from(ScheduleEvents::Table, ScheduleEvents::ScheduleId)
                            .to(GeneratedSchedules::Table, GeneratedSchedules::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_events_schedule")
                    .table(ScheduleEvents::Table)
                    .col(ScheduleEvents::ScheduleId)
                    .col(ScheduleEvents::StartTime)
                    .to_owned(),
            )
            .await?;

        // User preferences table
        manager
            .create_table(
                Table::create()
                    .table(UserPreferences::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(UserPreferences::UserId)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(UserPreferences::PersonalityTags).json())
                    .col(ColumnDef::new(UserPreferences::Likes).json())
                    .col(ColumnDef::new(UserPreferences::Dislikes).json())
                    .col(ColumnDef::new(UserPreferences::CustomInstructions).text())
                    .col(ColumnDef::new(UserPreferences::UpdatedAt).timestamp())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-user_preferences-user_id")
                            .from(UserPreferences::Table, UserPreferences::UserId)
                            .to(Users::Table, Users::Id),
                    )
                    .to_owned(),
            )
            .await?;

        // Agent memories table
        manager
            .create_table(
                Table::create()
                    .table(AgentMemories::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AgentMemories::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(AgentMemories::UserId).uuid().not_null())
                    .col(ColumnDef::new(AgentMemories::Fact).text().not_null())
                    .col(ColumnDef::new(AgentMemories::Source).string_len(50))
                    .col(ColumnDef::new(AgentMemories::SourceId).uuid())
                    .col(ColumnDef::new(AgentMemories::LearnedAt).timestamp())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-agent_memories-user_id")
                            .from(AgentMemories::Table, AgentMemories::UserId)
                            .to(Users::Table, Users::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_memories_user")
                    .table(AgentMemories::Table)
                    .col(AgentMemories::UserId)
                    .col(AgentMemories::LearnedAt)
                    .to_owned(),
            )
            .await?;

        // Feedbacks table
        manager
            .create_table(
                Table::create()
                    .table(Feedbacks::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Feedbacks::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Feedbacks::UserId).uuid().not_null())
                    .col(
                        ColumnDef::new(Feedbacks::TargetType)
                            .string_len(50)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Feedbacks::TargetId).uuid().not_null())
                    .col(ColumnDef::new(Feedbacks::Rating).string_len(20))
                    .col(ColumnDef::new(Feedbacks::Action).string_len(20))
                    .col(ColumnDef::new(Feedbacks::Details).json())
                    .col(ColumnDef::new(Feedbacks::CreatedAt).timestamp())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-feedbacks-user_id")
                            .from(Feedbacks::Table, Feedbacks::UserId)
                            .to(Users::Table, Users::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_feedbacks_target")
                    .table(Feedbacks::Table)
                    .col(Feedbacks::TargetType)
                    .col(Feedbacks::TargetId)
                    .to_owned(),
            )
            .await?;

        // Daily quizzes table
        manager
            .create_table(
                Table::create()
                    .table(DailyQuizzes::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(DailyQuizzes::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(DailyQuizzes::Question).text().not_null())
                    .col(ColumnDef::new(DailyQuizzes::Options).json().not_null())
                    .col(ColumnDef::new(DailyQuizzes::ContextField).string_len(100))
                    .col(
                        ColumnDef::new(DailyQuizzes::PriorityTier)
                            .integer()
                            .default(3),
                    )
                    .col(ColumnDef::new(DailyQuizzes::Active).boolean().default(true))
                    .to_owned(),
            )
            .await?;

        // Quiz responses table
        manager
            .create_table(
                Table::create()
                    .table(QuizResponses::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(QuizResponses::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(QuizResponses::UserId).uuid().not_null())
                    .col(ColumnDef::new(QuizResponses::QuizId).uuid().not_null())
                    .col(ColumnDef::new(QuizResponses::Answer).json().not_null())
                    .col(ColumnDef::new(QuizResponses::PointsEarned).integer())
                    .col(ColumnDef::new(QuizResponses::AnsweredAt).timestamp())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-quiz_responses-user_id")
                            .from(QuizResponses::Table, QuizResponses::UserId)
                            .to(Users::Table, Users::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-quiz_responses-quiz_id")
                            .from(QuizResponses::Table, QuizResponses::QuizId)
                            .to(DailyQuizzes::Table, DailyQuizzes::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_quiz_responses_user")
                    .table(QuizResponses::Table)
                    .col(QuizResponses::UserId)
                    .col(QuizResponses::AnsweredAt)
                    .to_owned(),
            )
            .await?;

        // User points table
        manager
            .create_table(
                Table::create()
                    .table(UserPoints::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(UserPoints::UserId)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(UserPoints::TotalPoints).integer().default(0))
                    .col(ColumnDef::new(UserPoints::UsedPoints).integer().default(0))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-user_points-user_id")
                            .from(UserPoints::Table, UserPoints::UserId)
                            .to(Users::Table, Users::Id),
                    )
                    .to_owned(),
            )
            .await?;

        // LLM requests table
        manager
            .create_table(
                Table::create()
                    .table(LlmRequests::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(LlmRequests::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(LlmRequests::UserId).uuid().not_null())
                    .col(ColumnDef::new(LlmRequests::SessionId).uuid())
                    .col(ColumnDef::new(LlmRequests::Intent).string_len(50))
                    .col(ColumnDef::new(LlmRequests::InputTokens).integer())
                    .col(ColumnDef::new(LlmRequests::OutputTokens).integer())
                    .col(ColumnDef::new(LlmRequests::LatencyMs).integer())
                    .col(
                        ColumnDef::new(LlmRequests::Status)
                            .string_len(20)
                            .default("pending"),
                    )
                    .col(ColumnDef::new(LlmRequests::ErrorMessage).text())
                    .col(ColumnDef::new(LlmRequests::CreatedAt).timestamp())
                    .col(ColumnDef::new(LlmRequests::CompletedAt).timestamp())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-llm_requests-user_id")
                            .from(LlmRequests::Table, LlmRequests::UserId)
                            .to(Users::Table, Users::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-llm_requests-session_id")
                            .from(LlmRequests::Table, LlmRequests::SessionId)
                            .to(ChatSessions::Table, ChatSessions::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_llm_requests_user_status")
                    .table(LlmRequests::Table)
                    .col(LlmRequests::UserId)
                    .col(LlmRequests::Status)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop tables in reverse order due to foreign key constraints
        manager
            .drop_table(
                Table::drop()
                    .table(LlmRequests::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(
                Table::drop()
                    .table(UserPoints::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(
                Table::drop()
                    .table(QuizResponses::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(
                Table::drop()
                    .table(DailyQuizzes::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(Feedbacks::Table).if_exists().to_owned())
            .await?;

        manager
            .drop_table(
                Table::drop()
                    .table(AgentMemories::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(
                Table::drop()
                    .table(UserPreferences::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(
                Table::drop()
                    .table(ScheduleEvents::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(
                Table::drop()
                    .table(GeneratedSchedules::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(
                Table::drop()
                    .table(ChatMessages::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(
                Table::drop()
                    .table(ChatSessions::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(
                Table::drop()
                    .table(UserSettings::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(Users::Table).if_exists().to_owned())
            .await?;

        Ok(())
    }
}
