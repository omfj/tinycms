use serde::{Deserialize, Serialize};

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
}
