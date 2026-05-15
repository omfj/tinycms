use axum::{
    Json, Router,
    extract::{Multipart, Path, State},
    routing::get,
};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    error::{Error, Result},
    models::media::{Media, UpdateMedia},
    state::SharedState,
};

pub fn router() -> Router<SharedState> {
    Router::new()
        .route("/", get(list).post(upload))
        .route("/{id}", axum::routing::patch(update).delete(remove))
}

async fn list(_user: AuthUser, State(state): State<SharedState>) -> Result<Json<Vec<Media>>> {
    let items = sqlx::query_as!(
        Media,
        "SELECT id, key, url, filename, content_type, size, label, uploaded_by, created_at
         FROM media
         ORDER BY created_at DESC",
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(items))
}

async fn upload(
    user: AuthUser,
    State(state): State<SharedState>,
    mut multipart: Multipart,
) -> Result<Json<Media>> {
    let storage = state
        .storage
        .as_ref()
        .ok_or_else(|| Error::BadRequest("storage is not configured".into()))?;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| Error::BadRequest(e.to_string()))?
    {
        if field.name() != Some("file") {
            continue;
        }

        let filename = field.file_name().unwrap_or("upload").to_string();
        let content_type = field
            .content_type()
            .map(str::to_owned)
            .unwrap_or_else(|| "application/octet-stream".into());

        let ext = std::path::Path::new(&filename)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("bin");

        let key = format!("uploads/{}.{}", Uuid::new_v4(), ext);
        let data = field
            .bytes()
            .await
            .map_err(|e| Error::BadRequest(e.to_string()))?
            .to_vec();

        let size = data.len() as i64;
        let url = storage.upload(&key, data, &content_type).await?;

        let media = sqlx::query_as!(
            Media,
            "INSERT INTO media (key, url, filename, content_type, size, uploaded_by)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id, key, url, filename, content_type, size, label, uploaded_by, created_at",
            key,
            url,
            filename,
            content_type,
            size,
            user.id,
        )
        .fetch_one(&state.pool)
        .await?;

        return Ok(Json(media));
    }

    Err(Error::BadRequest("no file field in multipart body".into()))
}

async fn update(
    _user: AuthUser,
    State(state): State<SharedState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateMedia>,
) -> Result<Json<Media>> {
    let media = sqlx::query_as!(
        Media,
        "UPDATE media SET label = $2
         WHERE id = $1
         RETURNING id, key, url, filename, content_type, size, label, uploaded_by, created_at",
        id,
        body.label,
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or(Error::NotFound)?;

    Ok(Json(media))
}

async fn remove(
    _user: AuthUser,
    State(state): State<SharedState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    let storage = state
        .storage
        .as_ref()
        .ok_or_else(|| Error::BadRequest("storage is not configured".into()))?;

    let media = sqlx::query_as!(
        Media,
        "DELETE FROM media WHERE id = $1
         RETURNING id, key, url, filename, content_type, size, label, uploaded_by, created_at",
        id,
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or(Error::NotFound)?;

    storage.delete(&media.key).await?;

    Ok(Json(serde_json::json!({ "deleted": true })))
}
