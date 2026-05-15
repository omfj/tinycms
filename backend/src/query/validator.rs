use sqlparser::ast::{
    Expr, Function, FunctionArg, FunctionArgExpr, FunctionArguments, Query, Select, SelectItem,
    SetExpr, TableFactor, Value,
};

use crate::models::user::UserRole;

use super::error::QueryError;

const DENIED_TABLES: &[&str] = &[
    "sessions",
    "accounts",
    "users",
    "workspace_settings",
    "document_revisions",
    "api_tokens",
];

const ALLOWED_TABLES_ALL: &[&str] = &["documents"];
const ALLOWED_TABLES_EDITOR: &[&str] = &["documents", "media"];
const ALLOWED_TABLES_ADMIN: &[&str] = &["documents", "media"];

const ALLOWED_FUNCTIONS: &[&str] = &[
    "lower",
    "upper",
    "length",
    "coalesce",
    "count",
    "now",
    "json_extract_path_text",
    "min",
    "max",
    "sum",
    "avg",
];

const MAX_LIMIT: i64 = 200;
const DEFAULT_LIMIT: i64 = 50;

pub struct ValidatedQuery {
    pub query: Box<Query>,
}

pub fn validate(
    mut query: Box<Query>,
    role: &UserRole,
    type_names: &[&str],
) -> Result<ValidatedQuery, QueryError> {
    let allowed_tables = match role {
        UserRole::Admin => ALLOWED_TABLES_ADMIN,
        UserRole::Editor => ALLOWED_TABLES_EDITOR,
        UserRole::Viewer => ALLOWED_TABLES_ALL,
    };

    if query.with.is_some() {
        return Err(QueryError::Forbidden("CTEs are not allowed".into()));
    }

    if !matches!(*query.body, SetExpr::Select(_)) {
        return Err(QueryError::Forbidden(
            "UNION, INTERSECT, and EXCEPT are not allowed".into(),
        ));
    }

    let select = match *query.body {
        SetExpr::Select(ref s) => s,
        _ => unreachable!(),
    };

    validate_from(select, allowed_tables, type_names)?;
    validate_select_items(select)?;

    if let Some(ref expr) = select.selection {
        validate_expr(expr)?;
    }

    enforce_limit(&mut query)?;

    Ok(ValidatedQuery { query })
}

fn validate_from(
    select: &Select,
    allowed_tables: &[&str],
    type_names: &[&str],
) -> Result<(), QueryError> {
    for table_with_joins in &select.from {
        validate_table_factor(&table_with_joins.relation, allowed_tables, type_names)?;
        for join in &table_with_joins.joins {
            validate_table_factor(&join.relation, allowed_tables, type_names)?;
        }
    }
    Ok(())
}

fn validate_table_factor(
    factor: &TableFactor,
    allowed_tables: &[&str],
    type_names: &[&str],
) -> Result<(), QueryError> {
    match factor {
        TableFactor::Table { name, .. } => {
            let table = name
                .0
                .last()
                .and_then(|i| i.as_ident())
                .map(|i| i.value.to_lowercase())
                .unwrap_or_default();
            if DENIED_TABLES.contains(&table.as_str()) {
                return Err(QueryError::Forbidden(format!(
                    "table '{table}' is not accessible"
                )));
            }
            if allowed_tables.contains(&table.as_str()) || type_names.contains(&table.as_str()) {
                return Ok(());
            }
            Err(QueryError::Forbidden(format!(
                "table '{table}' is not accessible with your role"
            )))
        }
        TableFactor::Derived { .. } => Err(QueryError::Forbidden(
            "subqueries in FROM are not allowed".into(),
        )),
        _ => Err(QueryError::Invalid("unsupported FROM clause".into())),
    }
}

fn validate_select_items(select: &Select) -> Result<(), QueryError> {
    for item in &select.projection {
        match item {
            SelectItem::UnnamedExpr(e) | SelectItem::ExprWithAlias { expr: e, .. } => {
                validate_expr(e)?;
            }
            SelectItem::ExprWithAliases { expr: e, .. } => {
                validate_expr(e)?;
            }
            SelectItem::QualifiedWildcard(_, _) | SelectItem::Wildcard(_) => {}
        }
    }
    Ok(())
}

fn validate_expr(expr: &Expr) -> Result<(), QueryError> {
    match expr {
        Expr::Function(f) => validate_function(f),
        Expr::BinaryOp { left, right, .. } => {
            validate_expr(left)?;
            validate_expr(right)
        }
        Expr::UnaryOp { expr, .. } => validate_expr(expr),
        Expr::IsNull(e) | Expr::IsNotNull(e) => validate_expr(e),
        Expr::Like { expr, pattern, .. } | Expr::ILike { expr, pattern, .. } => {
            validate_expr(expr)?;
            validate_expr(pattern)
        }
        Expr::Between {
            expr, low, high, ..
        } => {
            validate_expr(expr)?;
            validate_expr(low)?;
            validate_expr(high)
        }
        Expr::InList { expr, list, .. } => {
            validate_expr(expr)?;
            for e in list {
                validate_expr(e)?;
            }
            Ok(())
        }
        Expr::Nested(e) => validate_expr(e),
        Expr::Cast { expr, .. } => validate_expr(expr),
        Expr::Subquery(_) => Err(QueryError::Forbidden("subqueries are not allowed".into())),
        _ => Ok(()),
    }
}

fn validate_function(f: &Function) -> Result<(), QueryError> {
    let name = f
        .name
        .0
        .last()
        .and_then(|i| i.as_ident())
        .map(|i| i.value.to_lowercase())
        .unwrap_or_default();

    if !ALLOWED_FUNCTIONS.contains(&name.as_str()) {
        return Err(QueryError::Forbidden(format!(
            "function '{name}' is not allowed"
        )));
    }

    if let FunctionArguments::List(list) = &f.args {
        for arg in &list.args {
            match arg {
                FunctionArg::Unnamed(FunctionArgExpr::Expr(e)) => validate_expr(e)?,
                FunctionArg::Named {
                    arg: FunctionArgExpr::Expr(e),
                    ..
                } => validate_expr(e)?,
                _ => {}
            }
        }
    }
    Ok(())
}

fn make_limit_expr(n: i64) -> Expr {
    Expr::Value(Value::Number(n.to_string(), false).with_empty_span())
}

fn enforce_limit(query: &mut Query) -> Result<(), QueryError> {
    use sqlparser::ast::LimitClause;
    match &query.limit_clause {
        Some(LimitClause::LimitOffset {
            limit: Some(expr), ..
        }) => {
            if let Expr::Value(v) = expr
                && let Value::Number(n, _) = &v.value
            {
                let parsed: i64 = n.parse().unwrap_or(MAX_LIMIT + 1);
                if parsed > MAX_LIMIT {
                    let offset = match &query.limit_clause {
                        Some(LimitClause::LimitOffset { offset, .. }) => offset.clone(),
                        _ => None,
                    };
                    query.limit_clause = Some(LimitClause::LimitOffset {
                        limit: Some(make_limit_expr(MAX_LIMIT)),
                        offset,
                        limit_by: vec![],
                    });
                }
            }
        }
        None => {
            query.limit_clause = Some(sqlparser::ast::LimitClause::LimitOffset {
                limit: Some(make_limit_expr(DEFAULT_LIMIT)),
                offset: None,
                limit_by: vec![],
            });
        }
        _ => {}
    }
    Ok(())
}
