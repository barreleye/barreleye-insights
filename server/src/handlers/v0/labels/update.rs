use axum::{
	extract::{Path, State},
	http::StatusCode,
	Json,
};
use sea_orm::ActiveModelTrait;
use serde::Deserialize;
use std::sync::Arc;

use crate::{errors::ServerError, App, ServerResult};
use barreleye_common::models::{optional_set, BasicModel, Label, LabelActiveModel};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Payload {
	name: Option<String>,
}

pub async fn handler(
	State(app): State<Arc<App>>,
	Path(label_id): Path<String>,
	Json(payload): Json<Payload>,
) -> ServerResult<StatusCode> {
	match Label::get_by_id(&app.db, &label_id).await? {
		Some(label) if !label.is_deleted => {
			// check for duplicate name
			if let Some(name) = payload.name.clone() {
				if label_id != label.id &&
					label.name.trim().to_lowercase() == name.trim().to_lowercase()
				{
					return Err(ServerError::Duplicate { field: "name".to_string(), value: name });
				}
			}

			// update
			let update_data =
				LabelActiveModel { name: optional_set(payload.name), ..Default::default() };
			if update_data.is_changed() {
				Label::update_by_id(&app.db, &label_id, update_data).await?;
			}

			// ok
			Ok(StatusCode::NO_CONTENT)
		}
		_ => Err(ServerError::NotFound),
	}
}
