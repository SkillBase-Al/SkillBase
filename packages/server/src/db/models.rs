use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Skill {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub source: String,
    pub source_url: Option<String>,
    pub license: Option<String>,
    pub content_hash: String,
    pub skill_md_content: Option<String>,
    pub safety_level: Option<String>,
    pub format_score: Option<i32>,
    pub quality_score: Option<i32>,
    pub rating: Option<f64>,
    pub install_count: i32,
    #[sqlx(skip)]
    pub embedding: Option<Vec<f64>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Category {
    pub id: Uuid,
    pub name: String,
    pub display_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SkillCategory {
    pub skill_id: Uuid,
    pub category_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Stats {
    pub total_skills: i64,
    pub total_sources: i64,
    pub total_categories: i64,
    pub avg_rating: Option<f64>,
    pub avg_quality_score: Option<f64>,
    pub avg_format_score: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillWithCategories {
    pub skill: Skill,
    pub categories: Vec<Category>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total: i64,
    pub page: u32,
    pub per_page: u32,
}
