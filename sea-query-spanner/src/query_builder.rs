use sea_query::{
    backend::{EscapeBuilder, OperLeftAssocDecider, PrecedenceDecider, QueryBuilder, QuotedBuilder, TableRefBuilder},
    BinOper, Oper, Quote, SimpleExpr, SqlWriter, SubQueryStatement, Value,
};

pub struct SpannerQueryBuilder;

impl SpannerQueryBuilder {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SpannerQueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl QuotedBuilder for SpannerQueryBuilder {
    fn quote(&self) -> Quote {
        Quote::new(b'`')
    }
}

impl EscapeBuilder for SpannerQueryBuilder {
    fn escape_string(&self, string: &str) -> String {
        string.replace('\'', "''")
    }

    fn unescape_string(&self, string: &str) -> String {
        string.replace("''", "'")
    }
}

impl TableRefBuilder for SpannerQueryBuilder {}

impl PrecedenceDecider for SpannerQueryBuilder {
    fn inner_expr_well_known_greater_precedence(
        &self,
        inner: &SimpleExpr,
        _outer_oper: &Oper,
    ) -> bool {
        matches!(
            inner,
            SimpleExpr::Column(_)
                | SimpleExpr::Tuple(_)
                | SimpleExpr::Constant(_)
                | SimpleExpr::FunctionCall(_)
                | SimpleExpr::Value(_)
                | SimpleExpr::Keyword(_)
                | SimpleExpr::Case(_)
                | SimpleExpr::SubQuery(_, _)
        )
    }
}

impl OperLeftAssocDecider for SpannerQueryBuilder {
    fn well_known_left_associative(&self, op: &BinOper) -> bool {
        matches!(
            op,
            BinOper::And
                | BinOper::Or
                | BinOper::Add
                | BinOper::Sub
                | BinOper::Mul
                | BinOper::Div
                | BinOper::Mod
        )
    }
}

impl QueryBuilder for SpannerQueryBuilder {
    fn placeholder(&self) -> (&str, bool) {
        ("@p", true)
    }

    fn prepare_query_statement(&self, query: &SubQueryStatement, sql: &mut dyn SqlWriter) {
        match query {
            SubQueryStatement::SelectStatement(stmt) => self.prepare_select_statement(stmt, sql),
            SubQueryStatement::InsertStatement(stmt) => self.prepare_insert_statement(stmt, sql),
            SubQueryStatement::UpdateStatement(stmt) => self.prepare_update_statement(stmt, sql),
            SubQueryStatement::DeleteStatement(stmt) => self.prepare_delete_statement(stmt, sql),
            SubQueryStatement::WithStatement(stmt) => self.prepare_with_query(stmt, sql),
        }
    }

    fn prepare_value(&self, value: &Value, sql: &mut dyn SqlWriter) {
        sql.push_param(value.clone(), self as _);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_query::{Alias, Expr, Query};

    #[test]
    fn test_select_basic() {
        let query = Query::select()
            .column(Alias::new("name"))
            .from(Alias::new("users"))
            .to_string(SpannerQueryBuilder);

        assert_eq!(query, r#"SELECT `name` FROM `users`"#);
    }

    #[test]
    fn test_select_with_where() {
        let query = Query::select()
            .column(Alias::new("name"))
            .from(Alias::new("users"))
            .and_where(Expr::col(Alias::new("id")).eq(1))
            .to_string(SpannerQueryBuilder);

        assert_eq!(query, r#"SELECT `name` FROM `users` WHERE `id` = 1"#);
    }

    #[test]
    fn test_insert() {
        let query = Query::insert()
            .into_table(Alias::new("users"))
            .columns([Alias::new("name"), Alias::new("email")])
            .values_panic(["Alice".into(), "alice@example.com".into()])
            .to_string(SpannerQueryBuilder);

        assert_eq!(
            query,
            r#"INSERT INTO `users` (`name`, `email`) VALUES ('Alice', 'alice@example.com')"#
        );
    }

    #[test]
    fn test_update() {
        let query = Query::update()
            .table(Alias::new("users"))
            .value(Alias::new("name"), "Bob")
            .and_where(Expr::col(Alias::new("id")).eq(1))
            .to_string(SpannerQueryBuilder);

        assert_eq!(
            query,
            r#"UPDATE `users` SET `name` = 'Bob' WHERE `id` = 1"#
        );
    }

    #[test]
    fn test_delete() {
        let query = Query::delete()
            .from_table(Alias::new("users"))
            .and_where(Expr::col(Alias::new("id")).eq(1))
            .to_string(SpannerQueryBuilder);

        assert_eq!(query, r#"DELETE FROM `users` WHERE `id` = 1"#);
    }

    #[test]
    fn test_placeholder() {
        let builder = SpannerQueryBuilder::new();
        assert_eq!(builder.placeholder(), ("@p", true));
    }

    #[test]
    fn test_select_with_params() {
        let (sql, values) = Query::select()
            .column(Alias::new("name"))
            .from(Alias::new("users"))
            .and_where(Expr::col(Alias::new("id")).eq(1))
            .and_where(Expr::col(Alias::new("active")).eq(true))
            .build(SpannerQueryBuilder);

        assert_eq!(sql, r#"SELECT `name` FROM `users` WHERE (`id` = @p1) AND (`active` = @p2)"#);
        assert_eq!(values.0.len(), 2);
    }
}
