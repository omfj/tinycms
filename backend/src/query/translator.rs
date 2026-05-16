use sqlparser::ast::{
    Expr, FunctionArg, FunctionArgExpr, FunctionArguments, GroupByExpr, Ident, LimitClause, Query,
    SelectItem, SetExpr, Value as SqlValue,
};

use crate::schema::TypeDef;

use super::error::QueryError;
use super::preprocessor::Param;
use super::validator::ValidatedQuery;

const DOCUMENT_COLUMNS: &[&str] = &[
    "id",
    "type",
    "status",
    "data",
    "created_at",
    "updated_at",
    "published_at",
];

pub struct Translated {
    pub sql: String,
    pub params: Vec<Param>,
}

pub fn translate(
    validated: ValidatedQuery,
    mut params: Vec<Param>,
    types: &[TypeDef],
) -> Result<Translated, QueryError> {
    let query = validated.query;
    let next_param = &mut (params.len() + 1);
    let sql = translate_query(&query, &mut params, next_param, types)?;
    Ok(Translated { sql, params })
}

fn translate_query(
    query: &Query,
    params: &mut Vec<Param>,
    next: &mut usize,
    types: &[TypeDef],
) -> Result<String, QueryError> {
    let body = match &*query.body {
        SetExpr::Select(s) => translate_select(s, params, next, types)?,
        _ => unreachable!("validator already rejected non-SELECT"),
    };

    let mut sql = body;

    if let Some(order_by) = &query.order_by {
        use sqlparser::ast::OrderByKind;
        if let OrderByKind::Expressions(exprs) = &order_by.kind {
            let formatted: Result<Vec<_>, _> = exprs
                .iter()
                .map(|o| {
                    let e = translate_expr(&o.expr, params, next)?;
                    let dir = match o.options.asc {
                        Some(false) => " DESC",
                        _ => " ASC",
                    };
                    Ok(format!("{e}{dir}"))
                })
                .collect();
            sql.push_str(&format!(" ORDER BY {}", formatted?.join(", ")));
        }
    }

    if let Some(LimitClause::LimitOffset { limit, offset, .. }) = &query.limit_clause {
        if let Some(l) = limit {
            // LIMIT must be a non-negative integer — emit inline (safe, no injection risk).
            sql.push_str(&format!(" LIMIT {}", emit_numeric_expr(l)?));
        }
        if let Some(o) = offset {
            sql.push_str(&format!(" OFFSET {}", emit_numeric_expr(&o.value)?));
        }
    }

    Ok(sql)
}

fn translate_select(
    select: &sqlparser::ast::Select,
    params: &mut Vec<Param>,
    next: &mut usize,
    types: &[TypeDef],
) -> Result<String, QueryError> {
    let projection: Result<Vec<_>, _> = select
        .projection
        .iter()
        .map(|item| match item {
            SelectItem::Wildcard(_) => Ok("*".to_string()),
            SelectItem::QualifiedWildcard(name, _) => Ok(format!("{name}.*")),
            SelectItem::UnnamedExpr(e) => translate_expr(e, params, next),
            SelectItem::ExprWithAlias { expr, alias } => Ok(format!(
                "{} AS {alias}",
                translate_expr(expr, params, next)?
            )),
            SelectItem::ExprWithAliases { expr, aliases } => {
                let alias_list: Vec<_> = aliases.iter().map(|a| a.to_string()).collect();
                Ok(format!(
                    "{} AS {}",
                    translate_expr(expr, params, next)?,
                    alias_list.join(", ")
                ))
            }
        })
        .collect();
    let projection_sql = projection?.join(", ");

    let from: Result<Vec<_>, _> = select
        .from
        .iter()
        .map(|t| {
            let table = translate_table_factor(&t.relation, types)?;
            let joins: Result<Vec<_>, _> = t
                .joins
                .iter()
                .map(|j| {
                    let join_table = translate_table_factor(&j.relation, types)?;
                    match &j.join_operator {
                        sqlparser::ast::JoinOperator::Inner(c) => Ok(format!(
                            "JOIN {join_table}{}",
                            format_join_constraint(c, params, next)?
                        )),
                        sqlparser::ast::JoinOperator::Left(c)
                        | sqlparser::ast::JoinOperator::LeftOuter(c) => Ok(format!(
                            "LEFT JOIN {join_table}{}",
                            format_join_constraint(c, params, next)?
                        )),
                        _ => Err(QueryError::Forbidden(
                            "only INNER and LEFT joins are supported".into(),
                        )),
                    }
                })
                .collect();
            let joins = joins?;
            let mut out = table;
            if !joins.is_empty() {
                out.push(' ');
                out.push_str(&joins.join(" "));
            }
            Ok(out)
        })
        .collect();
    let from_sql = from?.join(", ");

    let mut sql = format!("SELECT {projection_sql} FROM {from_sql}");

    if let Some(ref expr) = select.selection {
        sql.push_str(&format!(" WHERE {}", translate_expr(expr, params, next)?));
    }

    match &select.group_by {
        GroupByExpr::Expressions(exprs, _) if !exprs.is_empty() => {
            let gb: Result<Vec<_>, _> = exprs
                .iter()
                .map(|e| translate_expr(e, params, next))
                .collect();
            sql.push_str(&format!(" GROUP BY {}", gb?.join(", ")));
        }
        _ => {}
    }

    if let Some(ref expr) = select.having {
        sql.push_str(&format!(" HAVING {}", translate_expr(expr, params, next)?));
    }

    Ok(sql)
}

/// Emit a LIMIT/OFFSET expression as an inline integer (safe — these are numeric only).
fn emit_numeric_expr(expr: &Expr) -> Result<String, QueryError> {
    match expr {
        Expr::Value(v) => match &v.value {
            SqlValue::Number(n, _) => Ok(n.clone()),
            other => Err(QueryError::Invalid(format!(
                "LIMIT/OFFSET must be a number, got: {other}"
            ))),
        },
        other => Err(QueryError::Invalid(format!(
            "LIMIT/OFFSET must be a literal number, got: {other}"
        ))),
    }
}

fn translate_table_factor(
    factor: &sqlparser::ast::TableFactor,
    types: &[TypeDef],
) -> Result<String, QueryError> {
    match factor {
        sqlparser::ast::TableFactor::Table { name, alias, .. } => {
            let table = name.to_string();
            let table_lower = table.to_lowercase();

            if let Some(type_def) = types.iter().find(|t| t.name.to_lowercase() == table_lower) {
                let alias_name = alias
                    .as_ref()
                    .map(|a| a.name.to_string())
                    .unwrap_or_else(|| table.clone());

                let mut cols: Vec<String> = DOCUMENT_COLUMNS
                    .iter()
                    .map(|c| {
                        if *c == "type" {
                            "\"type\"".to_string()
                        } else {
                            c.to_string()
                        }
                    })
                    .collect();

                for field in &type_def.fields {
                    let field_name = &field.base().name;
                    if !DOCUMENT_COLUMNS.contains(&field_name.to_lowercase().as_str()) {
                        cols.push(format!("data->>'{}' AS \"{}\"", field_name, field_name));
                    }
                }

                return Ok(format!(
                    "(SELECT {} FROM documents WHERE \"type\" = '{}') AS {}",
                    cols.join(", "),
                    table_lower,
                    alias_name
                ));
            }

            if let Some(alias) = alias {
                Ok(format!("{table} AS {}", alias.name))
            } else {
                Ok(table)
            }
        }
        _ => Err(QueryError::Invalid("unsupported table reference".into())),
    }
}

fn format_join_constraint(
    c: &sqlparser::ast::JoinConstraint,
    params: &mut Vec<Param>,
    next: &mut usize,
) -> Result<String, QueryError> {
    match c {
        sqlparser::ast::JoinConstraint::On(expr) => {
            Ok(format!(" ON {}", translate_expr(expr, params, next)?))
        }
        sqlparser::ast::JoinConstraint::Using(cols) => {
            let names: Vec<_> = cols
                .iter()
                .map(|c| {
                    c.0.last()
                        .and_then(|i| i.as_ident())
                        .map(|i| i.value.as_str())
                        .unwrap_or("")
                        .to_string()
                })
                .collect();
            Ok(format!(" USING ({})", names.join(", ")))
        }
        _ => Err(QueryError::Invalid("unsupported JOIN constraint".into())),
    }
}

fn translate_expr(
    expr: &Expr,
    params: &mut Vec<Param>,
    next: &mut usize,
) -> Result<String, QueryError> {
    match expr {
        Expr::CompoundIdentifier(parts)
            if parts.first().map(|i| i.value.as_str()) == Some("data") =>
        {
            Ok(rewrite_jsonb_path(parts))
        }

        Expr::Identifier(id) => Ok(quote_ident(&id.value)),

        Expr::CompoundIdentifier(parts) => Ok(parts
            .iter()
            .map(|i| quote_ident(&i.value))
            .collect::<Vec<_>>()
            .join(".")),

        Expr::Value(v) => parameterize_value(&v.value, params, next),

        Expr::BinaryOp { left, op, right } => {
            let l = translate_expr(left, params, next)?;
            let r = translate_expr(right, params, next)?;
            Ok(format!("{l} {op} {r}"))
        }

        Expr::UnaryOp { op, expr } => Ok(format!("{op} {}", translate_expr(expr, params, next)?)),

        Expr::IsNull(e) => Ok(format!("{} IS NULL", translate_expr(e, params, next)?)),
        Expr::IsNotNull(e) => Ok(format!("{} IS NOT NULL", translate_expr(e, params, next)?)),

        Expr::Like {
            negated,
            expr,
            pattern,
            ..
        } => {
            let neg = if *negated { "NOT " } else { "" };
            Ok(format!(
                "{} {}LIKE {}",
                translate_expr(expr, params, next)?,
                neg,
                translate_expr(pattern, params, next)?
            ))
        }

        Expr::ILike {
            negated,
            expr,
            pattern,
            ..
        } => {
            let neg = if *negated { "NOT " } else { "" };
            Ok(format!(
                "{} {}ILIKE {}",
                translate_expr(expr, params, next)?,
                neg,
                translate_expr(pattern, params, next)?
            ))
        }

        Expr::Between {
            expr,
            negated,
            low,
            high,
        } => {
            let neg = if *negated { "NOT " } else { "" };
            Ok(format!(
                "{} {}BETWEEN {} AND {}",
                translate_expr(expr, params, next)?,
                neg,
                translate_expr(low, params, next)?,
                translate_expr(high, params, next)?
            ))
        }

        Expr::InList {
            expr,
            list,
            negated,
        } => {
            let neg = if *negated { "NOT " } else { "" };
            let items: Result<Vec<_>, _> = list
                .iter()
                .map(|e| translate_expr(e, params, next))
                .collect();
            Ok(format!(
                "{} {}IN ({})",
                translate_expr(expr, params, next)?,
                neg,
                items?.join(", ")
            ))
        }

        Expr::Nested(e) => Ok(format!("({})", translate_expr(e, params, next)?)),

        Expr::Cast {
            expr, data_type, ..
        } => Ok(format!(
            "CAST({} AS {data_type})",
            translate_expr(expr, params, next)?
        )),

        Expr::Function(f) => {
            let name = f.name.to_string().to_lowercase();
            let args_sql = match &f.args {
                FunctionArguments::None => String::new(),
                FunctionArguments::List(list) => {
                    let args: Result<Vec<_>, _> = list
                        .args
                        .iter()
                        .map(|a| match a {
                            FunctionArg::Unnamed(FunctionArgExpr::Expr(e)) => {
                                translate_expr(e, params, next)
                            }
                            FunctionArg::Unnamed(FunctionArgExpr::Wildcard) => Ok("*".to_string()),
                            FunctionArg::Named {
                                name: n,
                                arg: FunctionArgExpr::Expr(e),
                                ..
                            } => Ok(format!("{n} => {}", translate_expr(e, params, next)?)),
                            _ => Ok("*".to_string()),
                        })
                        .collect();
                    args?.join(", ")
                }
                FunctionArguments::Subquery(_) => {
                    return Err(QueryError::Forbidden(
                        "subquery function arguments are not allowed".into(),
                    ));
                }
            };
            Ok(format!("{name}({args_sql})"))
        }

        Expr::Wildcard(_) => Ok("*".to_string()),

        other => Err(QueryError::Invalid(format!(
            "unsupported expression: {other}"
        ))),
    }
}

fn rewrite_jsonb_path(parts: &[Ident]) -> String {
    let path: Vec<&str> = parts[1..].iter().map(|i| i.value.as_str()).collect();
    if path.len() == 1 {
        format!("data->>'{}' ", path[0])
    } else {
        let json_path = path.join(",");
        format!("data#>>'{{{json_path}}}'")
    }
}

fn parameterize_value(
    v: &SqlValue,
    params: &mut Vec<Param>,
    next: &mut usize,
) -> Result<String, QueryError> {
    let param = match v {
        SqlValue::SingleQuotedString(s) | SqlValue::DoubleQuotedString(s) => Param::Text(s.clone()),
        SqlValue::Number(n, _) => {
            if let Ok(i) = n.parse::<i64>() {
                Param::Int(i)
            } else if let Ok(f) = n.parse::<f64>() {
                Param::Float(f)
            } else {
                return Err(QueryError::Invalid(format!("invalid number: {n}")));
            }
        }
        SqlValue::Boolean(b) => Param::Bool(*b),
        SqlValue::Null => Param::Null,
        SqlValue::Placeholder(p) => {
            // Already a $N placeholder from the preprocessor — emit as-is.
            return Ok(p.clone());
        }
        other => {
            return Err(QueryError::Invalid(format!(
                "unsupported literal value: {other}"
            )));
        }
    };

    let pos = *next;
    params.push(param);
    *next += 1;
    Ok(format!("${pos}"))
}

fn quote_ident(s: &str) -> String {
    let reserved = [
        "type", "select", "where", "from", "order", "group", "limit", "offset", "join",
    ];
    if reserved.contains(&s.to_lowercase().as_str()) {
        format!("\"{s}\"")
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::user::UserRole;
    use crate::query::{parser, validator};

    fn run(sql: &str) -> Result<Translated, QueryError> {
        let q = parser::parse(sql)?;
        let v = validator::validate(q, &UserRole::Admin, &[])?;
        translate(v, vec![], &[])
    }

    #[test]
    fn simple_select() {
        let t = run("SELECT id, status FROM documents WHERE status = 'published'").unwrap();
        assert!(t.sql.contains("SELECT"));
        assert_eq!(t.params.len(), 1);
    }

    #[test]
    fn jsonb_single_field() {
        let t = run("SELECT data.title FROM documents").unwrap();
        assert!(t.sql.contains("data->>'title'"), "got: {}", t.sql);
    }

    #[test]
    fn jsonb_nested_field() {
        let t = run("SELECT data.meta.seo FROM documents").unwrap();
        assert!(t.sql.contains("data#>>'{meta,seo}'"), "got: {}", t.sql);
    }

    #[test]
    fn default_limit_injected() {
        let t = run("SELECT * FROM documents").unwrap();
        assert!(t.sql.contains("LIMIT 50"), "got: {}", t.sql);
    }
}
