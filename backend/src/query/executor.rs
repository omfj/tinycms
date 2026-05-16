use serde_json::{Map, Value};
use sqlx::{Column, PgPool, Row, TypeInfo};

use super::error::QueryError;
use super::preprocessor::Param;
use super::translator::Translated;

pub async fn execute(pool: &PgPool, translated: Translated) -> Result<Vec<Value>, QueryError> {
    let mut q = sqlx::query(&translated.sql);
    for param in &translated.params {
        q = match param {
            Param::Text(s) => q.bind(s.as_str()),
            Param::Int(i) => q.bind(*i),
            Param::Float(f) => q.bind(*f),
            Param::Bool(b) => q.bind(*b),
            Param::Null => q.bind(Option::<String>::None),
        };
    }

    let rows = q
        .fetch_all(pool)
        .await
        .map_err(|e| QueryError::Invalid(e.to_string()))?;

    rows.iter().map(row_to_value).collect()
}

fn row_to_value(row: &sqlx::postgres::PgRow) -> Result<Value, QueryError> {
    let mut map = Map::new();
    for col in row.columns() {
        let name = col.name().to_string();
        let type_name = col.type_info().name();
        let val = pg_col_to_json(row, col.ordinal(), type_name)?;
        insert_nested(&mut map, &name, val);
    }
    Ok(Value::Object(map))
}

fn insert_nested(map: &mut Map<String, Value>, key: &str, val: Value) {
    let mut parts = key.splitn(2, "__");
    let head = parts.next().unwrap();
    match parts.next() {
        None => {
            map.insert(head.to_string(), val);
        }
        Some(rest) => {
            let child = map
                .entry(head.to_string())
                .or_insert_with(|| Value::Object(Map::new()));
            if let Value::Object(child_map) = child {
                insert_nested(child_map, rest, val);
            }
        }
    }
}

fn pg_col_to_json(
    row: &sqlx::postgres::PgRow,
    idx: usize,
    type_name: &str,
) -> Result<Value, QueryError> {
    // Try JSONB / JSON first.
    if type_name == "JSONB" || type_name == "JSON" {
        let v: Option<Value> = row
            .try_get(idx)
            .map_err(|e| QueryError::Invalid(e.to_string()))?;
        return Ok(v.unwrap_or(Value::Null));
    }

    // Boolean
    if type_name == "BOOL" {
        let v: Option<bool> = row
            .try_get(idx)
            .map_err(|e| QueryError::Invalid(e.to_string()))?;
        return Ok(v.map(Value::Bool).unwrap_or(Value::Null));
    }

    // Integer types
    if matches!(type_name, "INT2" | "INT4") {
        let v: Option<i32> = row
            .try_get(idx)
            .map_err(|e| QueryError::Invalid(e.to_string()))?;
        return Ok(v.map(|n| Value::Number(n.into())).unwrap_or(Value::Null));
    }
    if type_name == "INT8" {
        let v: Option<i64> = row
            .try_get(idx)
            .map_err(|e| QueryError::Invalid(e.to_string()))?;
        return Ok(v.map(|n| Value::Number(n.into())).unwrap_or(Value::Null));
    }

    // Float types
    if matches!(type_name, "FLOAT4" | "FLOAT8" | "NUMERIC") {
        let v: Option<f64> = row
            .try_get(idx)
            .map_err(|e| QueryError::Invalid(e.to_string()))?;
        return Ok(v
            .and_then(|f| serde_json::Number::from_f64(f).map(Value::Number))
            .unwrap_or(Value::Null));
    }

    // UUID — serialize as string
    if type_name == "UUID" {
        let v: Option<uuid::Uuid> = row
            .try_get(idx)
            .map_err(|e| QueryError::Invalid(e.to_string()))?;
        return Ok(v
            .map(|u| Value::String(u.to_string()))
            .unwrap_or(Value::Null));
    }

    // Timestamps — serialize as ISO 8601 string
    if matches!(type_name, "TIMESTAMPTZ" | "TIMESTAMP") {
        let v: Option<chrono::DateTime<chrono::Utc>> = row
            .try_get(idx)
            .map_err(|e| QueryError::Invalid(e.to_string()))?;
        return Ok(v
            .map(|dt| Value::String(dt.to_rfc3339()))
            .unwrap_or(Value::Null));
    }

    // Default: TEXT and everything else
    let v: Option<String> = row
        .try_get(idx)
        .map_err(|e| QueryError::Invalid(e.to_string()))?;
    Ok(v.map(Value::String).unwrap_or(Value::Null))
}
