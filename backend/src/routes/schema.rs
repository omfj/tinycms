use axum::{Json, extract::State};
use serde::Serialize;

use crate::{error::Result, schema::TypeDef, state::SharedState};

#[derive(Serialize)]
pub struct AdminSchema {
    pub types: Vec<TypeDef>,
    pub storage_configured: bool,
}

pub async fn get(State(state): State<SharedState>) -> Result<Json<AdminSchema>> {
    Ok(Json(AdminSchema {
        types: state.schema.types.clone(),
        storage_configured: state.storage.is_some(),
    }))
}
