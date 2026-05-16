use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};
use serde::Deserialize;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::{
    error::{Error, Result},
    models::document::{CreateDocument, Document, DocumentRevision, UpdateDocument},
    schema::TinyCmsConfig,
    state::SharedState,
};

fn validate_doc(schema: &TinyCmsConfig, doc_type: &str, data: &serde_json::Value) -> Result<()> {
    let Some(typedef) = schema.types.iter().find(|t| t.name == doc_type) else {
        return Ok(());
    };
    let errors = typedef.validate(data);
    if errors.is_empty() {
        Ok(())
    } else {
        Err(Error::Validation(errors))
    }
}

pub fn router() -> Router<SharedState> {
    Router::new()
        .route("/", get(list).post(create))
        .route("/{id}", get(get_one).put(update).delete(remove))
        .route("/{id}/revisions", get(list_revisions))
}

#[derive(Deserialize)]
pub struct ListQuery {
    #[serde(rename = "type")]
    pub doc_type: Option<String>,
    pub status: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

async fn list(
    State(state): State<SharedState>,
    Query(q): Query<ListQuery>,
) -> Result<Json<Vec<Document>>> {
    let limit = q.limit.unwrap_or(50).min(200);
    let offset = q.offset.unwrap_or(0);

    let docs = sqlx::query_as!(
        Document,
        r#"SELECT id, type AS doc_type, slug, status, data AS "data: Value",
                  created_at, updated_at, published_at
           FROM documents
           WHERE ($1::text IS NULL OR type = $1)
             AND ($2::text IS NULL OR status = $2)
           ORDER BY created_at DESC
           LIMIT $3 OFFSET $4"#,
        q.doc_type,
        q.status,
        limit,
        offset,
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(docs))
}

async fn get_one(State(state): State<SharedState>, Path(id): Path<Uuid>) -> Result<Json<Document>> {
    let doc = sqlx::query_as!(
        Document,
        r#"SELECT id, type AS doc_type, slug, status, data AS "data: Value",
                  created_at, updated_at, published_at
           FROM documents WHERE id = $1"#,
        id,
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or(Error::NotFound)?;

    Ok(Json(doc))
}

async fn create(
    State(state): State<SharedState>,
    Json(body): Json<CreateDocument>,
) -> Result<Json<Document>> {
    let data = body.data.unwrap_or_else(|| json!({}));
    validate_doc(&state.schema.borrow(), &body.doc_type, &data)?;

    let doc = sqlx::query_as!(
        Document,
        r#"INSERT INTO documents (type, slug, status, data)
           VALUES ($1, $2, $3, $4)
           RETURNING id, type AS doc_type, slug, status, data AS "data: Value",
                     created_at, updated_at, published_at"#,
        body.doc_type,
        body.slug,
        body.status.as_deref().unwrap_or("draft"),
        data,
    )
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(doc))
}

async fn update(
    State(state): State<SharedState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateDocument>,
) -> Result<Json<Document>> {
    if let Some(data) = &body.data {
        let row = sqlx::query!("SELECT type FROM documents WHERE id = $1", id)
            .fetch_optional(&state.pool)
            .await?
            .ok_or(Error::NotFound)?;
        validate_doc(&state.schema.borrow(), &row.r#type, data)?;
    }

    let doc = sqlx::query_as!(
        Document,
        r#"UPDATE documents SET
             slug         = COALESCE($2, slug),
             status       = COALESCE($3, status),
             data         = COALESCE($4, data),
             updated_at   = now(),
             published_at = CASE
               WHEN $3 = 'published' AND published_at IS NULL THEN now()
               ELSE published_at
             END
           WHERE id = $1
           RETURNING id, type AS doc_type, slug, status, data AS "data: Value",
                     created_at, updated_at, published_at"#,
        id,
        body.slug,
        body.status,
        body.data,
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or(Error::NotFound)?;

    Ok(Json(doc))
}

async fn remove(State(state): State<SharedState>, Path(id): Path<Uuid>) -> Result<Json<Value>> {
    let res = sqlx::query!("DELETE FROM documents WHERE id = $1", id)
        .execute(&state.pool)
        .await?;

    if res.rows_affected() == 0 {
        return Err(Error::NotFound);
    }

    Ok(Json(json!({ "deleted": true })))
}

async fn list_revisions(
    State(state): State<SharedState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<DocumentRevision>>> {
    let revisions = sqlx::query_as!(
        DocumentRevision,
        "SELECT id, document_id, data AS \"data: Value\", created_at, created_by
         FROM document_revisions
         WHERE document_id = $1
         ORDER BY created_at DESC",
        id,
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(revisions))
}
