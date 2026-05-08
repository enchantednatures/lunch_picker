use serde::Serialize;
use sqlx::FromRow;

use crate::features::meal_plan_entries::MealSlot;

#[derive(Debug, PartialEq, Serialize)]
pub struct MealPlanTemplate {
    pub id: i32,
    pub user_id: i32,
    pub name: String,
    pub day_of_week: i32,
    pub meal_slot: MealSlot,
    pub recipe_id: i32,
    pub recipe_name: String,
}

#[derive(Debug, PartialEq, FromRow)]
pub struct MealPlanTemplateRow {
    pub id: i32,
    pub user_id: i32,
    pub name: String,
    pub day_of_week: i32,
    pub meal_slot: String,
    pub recipe_id: i32,
    pub recipe_name: String,
}

impl From<MealPlanTemplateRow> for MealPlanTemplate {
    fn from(row: MealPlanTemplateRow) -> Self {
        Self {
            id: row.id,
            user_id: row.user_id,
            name: row.name,
            day_of_week: row.day_of_week,
            meal_slot: row.meal_slot.as_str().into(),
            recipe_id: row.recipe_id,
            recipe_name: row.recipe_name,
        }
    }
}
