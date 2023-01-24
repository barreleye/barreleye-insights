use eyre::Result;
use sea_orm::{
	entity::{prelude::*, *},
	ConnectionTrait, FromQueryResult, QuerySelect,
};
use sea_orm_migration::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
	models::{entity_tag, BasicModel, PrimaryId, PrimaryIds},
	utils, IdPrefix,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, DeriveEntityModel)]
#[sea_orm(table_name = "tags")]
#[serde(rename_all = "camelCase")]
pub struct Model {
	#[sea_orm(primary_key)]
	#[serde(skip_serializing, skip_deserializing)]
	pub tag_id: PrimaryId,
	pub id: String,
	pub name: String,
	#[sea_orm(nullable)]
	#[serde(skip_serializing)]
	pub updated_at: Option<DateTime>,
	pub created_at: DateTime,

	#[sea_orm(ignore)]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub entities: Option<Vec<String>>,
}

impl From<Vec<Model>> for PrimaryIds {
	fn from(m: Vec<Model>) -> PrimaryIds {
		let mut ids: Vec<PrimaryId> = m.iter().map(|m| m.tag_id).collect();

		ids.sort_unstable();
		ids.dedup();

		PrimaryIds(ids)
	}
}

#[derive(Clone, FromQueryResult)]
pub struct JoinedModel {
	pub tag_id: PrimaryId,
	pub id: String,
	pub name: String,
	pub updated_at: Option<DateTime>,
	pub created_at: DateTime,
	pub entity_id: PrimaryId,
}

impl From<Vec<JoinedModel>> for PrimaryIds {
	fn from(m: Vec<JoinedModel>) -> PrimaryIds {
		let mut ids: Vec<PrimaryId> = m.iter().map(|m| m.tag_id).collect();

		ids.sort_unstable();
		ids.dedup();

		PrimaryIds(ids)
	}
}

impl From<JoinedModel> for Model {
	fn from(m: JoinedModel) -> Model {
		Model {
			tag_id: m.tag_id,
			id: m.id,
			name: m.name,
			updated_at: m.updated_at,
			created_at: m.created_at,
			entities: None,
		}
	}
}

pub use ActiveModel as TagActiveModel;
pub use JoinedModel as JoinedTag;
pub use Model as Tag;

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
	EntityTag,
}

impl RelationTrait for Relation {
	fn def(&self) -> RelationDef {
		match self {
			Self::EntityTag => Entity::belongs_to(entity_tag::Entity)
				.from(Column::TagId)
				.to(entity_tag::Column::TagId)
				.into(),
		}
	}
}

impl ActiveModelBehavior for ActiveModel {}

impl BasicModel for Model {
	type ActiveModel = ActiveModel;
}

impl Model {
	pub fn new_model(name: &str) -> ActiveModel {
		ActiveModel {
			id: Set(utils::new_unique_id(IdPrefix::Tag)),
			name: Set(name.to_string()),
			..Default::default()
		}
	}

	pub async fn get_by_name<C>(c: &C, name: &str) -> Result<Option<Self>>
	where
		C: ConnectionTrait,
	{
		Ok(Entity::find()
			.filter(
				Condition::all()
					.add(Func::lower(Expr::col(Column::Name)).equals(name.trim().to_lowercase())),
			)
			.one(c)
			.await?)
	}

	pub async fn get_all_by_tag_ids<C>(c: &C, tag_ids: PrimaryIds) -> Result<Vec<Self>>
	where
		C: ConnectionTrait,
	{
		Ok(Entity::find().filter(Column::TagId.is_in(tag_ids)).all(c).await?)
	}

	pub async fn get_all_by_entity_ids<C>(c: &C, entity_ids: PrimaryIds) -> Result<Vec<JoinedModel>>
	where
		C: ConnectionTrait,
	{
		Ok(Entity::find()
			.column_as(entity_tag::Column::EntityId, "entity_id")
			.join(JoinType::LeftJoin, Relation::EntityTag.def())
			.filter(entity_tag::Column::EntityId.is_in(entity_ids))
			.into_model::<JoinedModel>()
			.all(c)
			.await?)
	}
}
