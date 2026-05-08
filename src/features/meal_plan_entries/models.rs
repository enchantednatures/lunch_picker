use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, PartialEq, Serialize)]
pub struct MealPlanEntry {
    pub id: i32,
    pub user_id: i32,
    pub recipe_id: i32,
    pub recipe_name: String,
    pub date: chrono::NaiveDate,
    pub meal_slot: MealSlot,
}

#[derive(Debug, PartialEq, FromRow)]
pub struct MealPlanEntryRow {
    pub id: i32,
    pub user_id: i32,
    pub recipe_id: i32,
    pub recipe_name: String,
    pub date: chrono::NaiveDate,
    pub meal_slot: String,
}

impl From<MealPlanEntryRow> for MealPlanEntry {
    fn from(row: MealPlanEntryRow) -> Self {
        Self {
            id: row.id,
            user_id: row.user_id,
            recipe_id: row.recipe_id,
            recipe_name: row.recipe_name,
            date: row.date,
            meal_slot: row.meal_slot.as_str().into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum MealSlot {
    Breakfast,
    Lunch,
    Dinner,
    Snack,
}

impl MealSlot {
    pub fn as_str(&self) -> &'static str {
        match self {
            MealSlot::Breakfast => "breakfast",
            MealSlot::Lunch => "lunch",
            MealSlot::Dinner => "dinner",
            MealSlot::Snack => "snack",
        }
    }
}

impl From<&str> for MealSlot {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "breakfast" => MealSlot::Breakfast,
            "lunch" => MealSlot::Lunch,
            "dinner" => MealSlot::Dinner,
            "snack" => MealSlot::Snack,
            _ => MealSlot::Dinner,
        }
    }
}
