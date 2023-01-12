use eyre::Result;
use sea_orm::{
	entity::prelude::*,
	sea_query::{func::Func, Expr},
	Condition, Set,
};
use serde::{Deserialize, Serialize};

use crate::{
	models::{BasicModel, PrimaryId},
	utils, Db, IdPrefix,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, DeriveEntityModel)]
#[sea_orm(table_name = "labels")]
#[serde(rename_all = "camelCase")]
pub struct Model {
	#[sea_orm(primary_key)]
	#[serde(skip_serializing, skip_deserializing)]
	pub label_id: PrimaryId,
	pub id: String,
	pub name: String,
	pub description: String,
	#[serde(skip_serializing)]
	pub is_deleted: bool,
	#[sea_orm(nullable)]
	#[serde(skip_serializing)]
	pub updated_at: Option<DateTime>,
	pub created_at: DateTime,
}

pub use ActiveModel as LabelActiveModel;
pub use Model as Label;

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {}

impl RelationTrait for Relation {
	fn def(&self) -> RelationDef {
		panic!("No RelationDef")
	}
}

impl ActiveModelBehavior for ActiveModel {}

impl BasicModel for Model {
	type ActiveModel = ActiveModel;
}

impl Model {
	pub fn new_model(name: &str, description: &str) -> ActiveModel {
		ActiveModel {
			id: Set(utils::new_unique_id(IdPrefix::Label)),
			name: Set(name.to_string()),
			description: Set(description.to_string()),
			is_deleted: Set(false),
			..Default::default()
		}
	}

	pub async fn get_by_name(
		db: &Db,
		name: &str,
		is_deleted: Option<bool>,
	) -> Result<Option<Self>> {
		let mut q = Entity::find().filter(
			Condition::all()
				.add(Func::lower(Expr::col(Column::Name)).equals(name.trim().to_lowercase())),
		);

		if is_deleted.is_some() {
			q = q.filter(Column::IsDeleted.eq(is_deleted.unwrap()))
		}

		Ok(q.one(db.get()).await?)
	}
}
