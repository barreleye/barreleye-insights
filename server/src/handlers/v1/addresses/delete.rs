use axum::{extract::State, http::StatusCode, Json};
use sea_orm::ColumnTrait;
use serde::Deserialize;
use std::{collections::HashSet, sync::Arc};

use crate::{errors::ServerError, ServerResult};
use barreleye_common::{
	models::{set, Address, AddressActiveModel, AddressColumn, BasicModel, PrimaryId},
	App,
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Payload {
	addresses: HashSet<String>,
}

pub async fn handler(
	State(app): State<Arc<App>>,
	Json(payload): Json<Payload>,
) -> ServerResult<StatusCode> {
	// exit if no input
	if payload.addresses.is_empty() {
		return Ok(StatusCode::NO_CONTENT);
	}

	// get all addresses
	let all_addresses =
		Address::get_all_where(app.db(), AddressColumn::Id.is_in(payload.addresses)).await?;

	// proceed only when there's something to delete
	if all_addresses.is_empty() {
		return Ok(StatusCode::NO_CONTENT);
	}

	// make sure none of the addresses are locked (@TODO when "sanctions" mode is on)
	let invalid_ids = all_addresses
		.iter()
		.filter_map(|a| if a.is_locked { Some(a.id.clone()) } else { None })
		.collect::<Vec<String>>();
	if !invalid_ids.is_empty() {
		return Err(ServerError::InvalidValues {
			field: "addresses".to_string(),
			values: invalid_ids.join(", "),
		});
	}

	// soft-delete all associated addresses
	Address::update_all_where(
		app.db(),
		AddressColumn::AddressId
			.is_in(all_addresses.iter().map(|a| a.address_id).collect::<Vec<PrimaryId>>()),
		AddressActiveModel { is_deleted: set(true), ..Default::default() },
	)
	.await?;

	Ok(StatusCode::NO_CONTENT)
}
