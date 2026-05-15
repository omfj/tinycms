use sqlparser::ast::Statement;
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::parser::Parser;

use super::error::QueryError;

pub fn parse(sql: &str) -> Result<Box<sqlparser::ast::Query>, QueryError> {
    let dialect = PostgreSqlDialect {};
    let mut stmts =
        Parser::parse_sql(&dialect, sql).map_err(|e| QueryError::Parse(e.to_string()))?;

    if stmts.len() != 1 {
        return Err(QueryError::Invalid(
            "exactly one statement is required".into(),
        ));
    }

    match stmts.remove(0) {
        Statement::Query(q) => Ok(q),
        _ => Err(QueryError::Forbidden(
            "only SELECT queries are allowed".into(),
        )),
    }
}
