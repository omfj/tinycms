use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TypeDef {
    pub name: String,
    #[serde(default)]
    pub fields: Vec<FieldDef>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FieldOption {
    pub label: String,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BaseField {
    pub name: String,
    #[serde(default)]
    pub required: bool,
    pub title: Option<String>,
    pub description: Option<String>,
    #[serde(default)]
    pub hidden: bool,
    #[serde(rename = "readOnly", default)]
    pub read_only: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StringFieldDef {
    #[serde(flatten)]
    pub base: BaseField,
    pub placeholder: Option<String>,
    pub options: Option<Vec<FieldOption>>,
    pub pattern: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TextFieldDef {
    #[serde(flatten)]
    pub base: BaseField,
    pub placeholder: Option<String>,
    pub rows: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RichtextFieldDef {
    #[serde(flatten)]
    pub base: BaseField,
    pub placeholder: Option<String>,
    pub rows: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NumberFieldDef {
    #[serde(flatten)]
    pub base: BaseField,
    pub placeholder: Option<String>,
    pub options: Option<Vec<FieldOption>>,
    pub min: Option<f64>,
    pub max: Option<f64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BooleanFieldDef {
    #[serde(flatten)]
    pub base: BaseField,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DateFieldDef {
    #[serde(flatten)]
    pub base: BaseField,
    pub min: Option<f64>,
    pub max: Option<f64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UrlFieldDef {
    #[serde(flatten)]
    pub base: BaseField,
    pub placeholder: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SlugFieldDef {
    #[serde(flatten)]
    pub base: BaseField,
    pub source: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ImageFieldDef {
    #[serde(flatten)]
    pub base: BaseField,
    pub placeholder: Option<String>,
    pub accept: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ReferenceFieldDef {
    #[serde(flatten)]
    pub base: BaseField,
    pub to: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ArrayFieldDef {
    #[serde(flatten)]
    pub base: BaseField,
    pub of: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum FieldDef {
    String(StringFieldDef),
    Text(TextFieldDef),
    Richtext(RichtextFieldDef),
    Number(NumberFieldDef),
    Boolean(BooleanFieldDef),
    Date(DateFieldDef),
    Url(UrlFieldDef),
    Slug(SlugFieldDef),
    Image(ImageFieldDef),
    Reference(ReferenceFieldDef),
    Array(ArrayFieldDef),
}

impl FieldDef {
    pub fn base(&self) -> &BaseField {
        match self {
            Self::String(f) => &f.base,
            Self::Text(f) => &f.base,
            Self::Richtext(f) => &f.base,
            Self::Number(f) => &f.base,
            Self::Boolean(f) => &f.base,
            Self::Date(f) => &f.base,
            Self::Url(f) => &f.base,
            Self::Slug(f) => &f.base,
            Self::Image(f) => &f.base,
            Self::Reference(f) => &f.base,
            Self::Array(f) => &f.base,
        }
    }

    pub fn field_type(&self) -> &'static str {
        match self {
            Self::String(_) => "string",
            Self::Text(_) => "text",
            Self::Richtext(_) => "richtext",
            Self::Number(_) => "number",
            Self::Boolean(_) => "boolean",
            Self::Date(_) => "date",
            Self::Url(_) => "url",
            Self::Slug(_) => "slug",
            Self::Image(_) => "image",
            Self::Reference(_) => "reference",
            Self::Array(_) => "array",
        }
    }

    fn validate_value(&self, value: Option<&Value>) -> Vec<FieldError> {
        let base = self.base();
        let name = &base.name;
        let label = base.title.as_deref().unwrap_or(name);
        let mut errors = Vec::new();

        let is_empty = matches!(value, None | Some(Value::Null))
            || matches!(value, Some(Value::String(s)) if s.is_empty())
            || matches!(value, Some(Value::Array(a)) if a.is_empty());

        if base.required && is_empty {
            errors.push(FieldError {
                field: name.clone(),
                message: format!("{label} is required"),
            });
            return errors;
        }

        if is_empty {
            return errors;
        }

        let val = value.unwrap();

        match self {
            FieldDef::String(f) => {
                let Some(s) = val.as_str() else {
                    errors.push(FieldError {
                        field: name.clone(),
                        message: format!("{label} must be a string"),
                    });
                    return errors;
                };
                if let Some(pattern) = &f.pattern {
                    match regex::Regex::new(pattern) {
                        Ok(re) if !re.is_match(s) => errors.push(FieldError {
                            field: name.clone(),
                            message: format!("{label} does not match the required pattern"),
                        }),
                        Err(_) => {
                            tracing::warn!("invalid regex pattern for field {name}: {pattern}")
                        }
                        _ => {}
                    }
                }
                if let Some(options) = &f.options
                    && !options.iter().any(|o| o.value == *val)
                {
                    errors.push(FieldError {
                        field: name.clone(),
                        message: format!("{label} must be one of the allowed values"),
                    });
                }
            }
            FieldDef::Number(f) => {
                let Some(n) = val.as_f64() else {
                    errors.push(FieldError {
                        field: name.clone(),
                        message: format!("{label} must be a number"),
                    });
                    return errors;
                };
                if let Some(min) = f.min
                    && n < min
                {
                    errors.push(FieldError {
                        field: name.clone(),
                        message: format!("{label} must be at least {min}"),
                    });
                }
                if let Some(max) = f.max
                    && n > max
                {
                    errors.push(FieldError {
                        field: name.clone(),
                        message: format!("{label} must be at most {max}"),
                    });
                }
                if let Some(options) = &f.options
                    && !options.iter().any(|o| o.value == *val)
                {
                    errors.push(FieldError {
                        field: name.clone(),
                        message: format!("{label} must be one of the allowed values"),
                    });
                }
            }
            FieldDef::Boolean(_) if !val.is_boolean() => {
                errors.push(FieldError {
                    field: name.clone(),
                    message: format!("{label} must be a boolean"),
                });
            }
            FieldDef::Date(f) => {
                let n = val
                    .as_f64()
                    .or_else(|| val.as_str().and_then(|s| s.parse().ok()));
                match n {
                    Some(n) => {
                        if let Some(min) = f.min
                            && n < min
                        {
                            errors.push(FieldError {
                                field: name.clone(),
                                message: format!("{label} is before the minimum allowed date"),
                            });
                        }
                        if let Some(max) = f.max
                            && n > max
                        {
                            errors.push(FieldError {
                                field: name.clone(),
                                message: format!("{label} is after the maximum allowed date"),
                            });
                        }
                    }
                    None if !val.is_string() => {
                        errors.push(FieldError {
                            field: name.clone(),
                            message: format!("{label} must be a date"),
                        });
                    }
                    _ => {}
                }
            }
            FieldDef::Url(_) => {
                let Some(s) = val.as_str() else {
                    errors.push(FieldError {
                        field: name.clone(),
                        message: format!("{label} must be a string"),
                    });
                    return errors;
                };
                if !s.starts_with("http://") && !s.starts_with("https://") {
                    errors.push(FieldError {
                        field: name.clone(),
                        message: format!("{label} must be a valid URL"),
                    });
                }
            }
            FieldDef::Slug(_) => {
                let Some(s) = val.as_str() else {
                    errors.push(FieldError {
                        field: name.clone(),
                        message: format!("{label} must be a string"),
                    });
                    return errors;
                };
                let re = regex::Regex::new(r"^[a-z0-9]+(?:-[a-z0-9]+)*$").unwrap();
                if !re.is_match(s) {
                    errors.push(FieldError {
                        field: name.clone(),
                        message: format!(
                            "{label} must contain only lowercase letters, numbers, and hyphens"
                        ),
                    });
                }
            }
            // Text, Richtext, Image, Reference, Array: required already handled above
            _ => {}
        }

        errors
    }
}

#[derive(Debug, Serialize)]
pub struct FieldError {
    pub field: String,
    pub message: String,
}

impl TypeDef {
    pub fn validate(&self, data: &Value) -> Vec<FieldError> {
        self.fields
            .iter()
            .filter(|f| !f.base().hidden && !f.base().read_only)
            .flat_map(|f| f.validate_value(data.get(&f.base().name)))
            .collect()
    }
}
