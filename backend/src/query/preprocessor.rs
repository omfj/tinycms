use serde_json::Value;

use super::error::QueryError;

/// A bound parameter value ready to pass to sqlx.
#[derive(Debug, Clone)]
pub enum Param {
    Text(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Null,
}

impl From<&Value> for Param {
    fn from(v: &Value) -> Self {
        match v {
            Value::String(s) => Param::Text(s.clone()),
            Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Param::Int(i)
                } else {
                    Param::Float(n.as_f64().unwrap_or(0.0))
                }
            }
            Value::Bool(b) => Param::Bool(*b),
            _ => Param::Null,
        }
    }
}

pub struct Preprocessed {
    pub sql: String,
    pub params: Vec<Param>,
}

/// Expand named (`:name`) or positional (`$N`) params supplied in the request
/// body into a normalized positional form, returning the rewritten SQL and the
/// ordered param list.
///
/// If `params` is `None` the SQL is returned as-is with an empty param list —
/// inline literal values in the SQL will be parameterized later by the translator.
pub fn expand(sql: &str, params: Option<&Value>) -> Result<Preprocessed, QueryError> {
    match params {
        None => Ok(Preprocessed {
            sql: sql.to_string(),
            params: vec![],
        }),
        Some(Value::Array(arr)) => {
            // Positional params: $1, $2, ... already in the SQL — validate count only.
            let expected = count_positional(sql);
            if arr.len() != expected {
                return Err(QueryError::Invalid(format!(
                    "query has {expected} positional parameter(s) but {} value(s) were supplied",
                    arr.len()
                )));
            }
            Ok(Preprocessed {
                sql: sql.to_string(),
                params: arr.iter().map(Param::from).collect(),
            })
        }
        Some(Value::Object(map)) => {
            // Named params: replace `:name` with `$N` in order of first appearance.
            let mut params_ordered: Vec<Param> = Vec::new();
            let mut name_to_pos: std::collections::HashMap<String, usize> =
                std::collections::HashMap::new();

            // Regex-free replacement: scan for `:ident` patterns.
            let mut out = String::with_capacity(sql.len());
            let chars: Vec<char> = sql.chars().collect();
            let mut i = 0;

            while i < chars.len() {
                // Skip string literals (don't replace inside quotes).
                if chars[i] == '\'' {
                    out.push(chars[i]);
                    i += 1;
                    while i < chars.len() {
                        out.push(chars[i]);
                        if chars[i] == '\'' {
                            i += 1;
                            break;
                        }
                        i += 1;
                    }
                    continue;
                }

                if chars[i] == ':' && i + 1 < chars.len() && chars[i + 1].is_alphabetic() {
                    // Collect the identifier.
                    let start = i + 1;
                    let mut end = start;
                    while end < chars.len() && (chars[end].is_alphanumeric() || chars[end] == '_') {
                        end += 1;
                    }
                    let name: String = chars[start..end].iter().collect();

                    let pos = if let Some(&p) = name_to_pos.get(&name) {
                        p
                    } else {
                        let val = map.get(&name).ok_or_else(|| {
                            QueryError::Invalid(format!("parameter :{name} not found in params"))
                        })?;
                        let pos = params_ordered.len() + 1;
                        name_to_pos.insert(name.clone(), pos);
                        params_ordered.push(Param::from(val));
                        pos
                    };

                    out.push('$');
                    out.push_str(&pos.to_string());
                    i = end;
                    continue;
                }

                out.push(chars[i]);
                i += 1;
            }

            Ok(Preprocessed {
                sql: out,
                params: params_ordered,
            })
        }
        Some(other) => Err(QueryError::Invalid(format!(
            "params must be an array or object, got {}",
            other
        ))),
    }
}

fn count_positional(sql: &str) -> usize {
    let mut max = 0usize;
    let bytes = sql.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'$' {
            let start = i + 1;
            let mut end = start;
            while end < bytes.len() && bytes[end].is_ascii_digit() {
                end += 1;
            }
            if end > start
                && let Ok(n) = sql[start..end].parse::<usize>()
            {
                max = max.max(n);
            }
            i = end;
        } else {
            i += 1;
        }
    }
    max
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn positional_correct_count() {
        let r = expand(
            "SELECT * FROM documents WHERE type = $1 AND status = $2",
            Some(&json!(["blog", "published"])),
        )
        .unwrap();
        assert_eq!(r.params.len(), 2);
    }

    #[test]
    fn positional_wrong_count_errors() {
        let r = expand(
            "SELECT * FROM documents WHERE type = $1",
            Some(&json!(["blog", "extra"])),
        );
        assert!(r.is_err());
    }

    #[test]
    fn named_replaces_correctly() {
        let r = expand(
            "SELECT * FROM documents WHERE type = :type AND status = :status",
            Some(&json!({ "type": "blog", "status": "published" })),
        )
        .unwrap();
        assert!(r.sql.contains("$1") || r.sql.contains("$2"));
        assert_eq!(r.params.len(), 2);
    }

    #[test]
    fn named_missing_param_errors() {
        let r = expand(
            "SELECT * FROM documents WHERE type = :missing",
            Some(&json!({ "type": "blog" })),
        );
        assert!(r.is_err());
    }

    #[test]
    fn no_params_passthrough() {
        let sql = "SELECT * FROM documents LIMIT 10";
        let r = expand(sql, None).unwrap();
        assert_eq!(r.sql, sql);
        assert!(r.params.is_empty());
    }
}
