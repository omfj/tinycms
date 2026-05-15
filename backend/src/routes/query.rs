use axum::{Json, Router, extract::State, routing::post};
use serde::Deserialize;
use serde_json::Value;

use crate::{
    auth::AuthUser,
    error::{Error, Result},
    query::{executor, parser, preprocessor, translator, validator},
    state::SharedState,
};

pub fn router() -> Router<SharedState> {
    Router::new().route("/", post(handle_query))
}

#[derive(Deserialize)]
struct QueryRequest {
    q: String,
    params: Option<Value>,
}

async fn handle_query(
    user: AuthUser,
    State(state): State<SharedState>,
    Json(body): Json<QueryRequest>,
) -> Result<Json<Vec<Value>>> {
    let preprocessed = preprocessor::expand(&body.q, body.params.as_ref()).map_err(Error::from)?;

    let ast = parser::parse(&preprocessed.sql).map_err(Error::from)?;

    let validated = validator::validate(ast, &user.role).map_err(Error::from)?;

    let translated = translator::translate(validated, preprocessed.params).map_err(Error::from)?;

    let rows = executor::execute(&state.pool, translated)
        .await
        .map_err(Error::from)?;

    Ok(Json(rows))
}
