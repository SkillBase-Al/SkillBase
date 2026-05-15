use crate::crawler::RawSkill;
use crate::db::models::*;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn list_skills(
    pool: &PgPool,
    page: u32,
    per_page: u32,
    category: Option<String>,
    sort: Option<String>,
    order: Option<String>,
) -> Result<PaginatedResponse<Skill>, sqlx::Error> {
    let offset = ((page.saturating_sub(1)) * per_page) as i64;
    let limit = per_page as i64;

    let sort_col = match sort.as_deref() {
        Some("name") => "name",
        Some("rating") => "rating",
        Some("install_count") => "install_count",
        Some("created_at") => "created_at",
        _ => "created_at",
    };

    let order_dir = match order.as_deref() {
        Some("asc") => "ASC",
        _ => "DESC",
    };

    let (total, skills) = if let Some(cat) = category {
        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM skills s
             JOIN skill_categories sc ON s.id = sc.skill_id
             JOIN categories c ON sc.category_id = c.id
             WHERE c.name = $1",
        )
        .bind(&cat)
        .fetch_one(pool)
        .await?;

        let query_str = format!(
            "SELECT s.id, s.name, s.description, s.source, s.source_url, s.license,
                    s.content_hash, s.skill_md_content, s.safety_level,
                    s.format_score, s.quality_score, s.rating, s.install_count,
                    s.created_at, s.updated_at
             FROM skills s
             JOIN skill_categories sc ON s.id = sc.skill_id
             JOIN categories c ON sc.category_id = c.id
             WHERE c.name = $1
             ORDER BY s.{} {}
             LIMIT $2 OFFSET $3",
            sort_col, order_dir
        );

        let skills = sqlx::query_as::<_, Skill>(&query_str)
            .bind(&cat)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?;

        (total, skills)
    } else {
        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM skills")
            .fetch_one(pool)
            .await?;

        let query_str = format!(
            "SELECT id, name, description, source, source_url, license, content_hash,
                    skill_md_content, safety_level, format_score, quality_score, rating,
                    install_count, created_at, updated_at
             FROM skills
             ORDER BY {} {}
             LIMIT $1 OFFSET $2",
            sort_col, order_dir
        );

        let skills = sqlx::query_as::<_, Skill>(&query_str)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?;

        (total, skills)
    };

    Ok(PaginatedResponse {
        data: skills,
        total,
        page,
        per_page,
    })
}

pub async fn search_skills(
    pool: &PgPool,
    query: &str,
    category: Option<String>,
    license_filter: Option<String>,
    page: u32,
    per_page: u32,
) -> Result<PaginatedResponse<Skill>, sqlx::Error> {
    let offset = ((page.saturating_sub(1)) * per_page) as i64;
    let limit = per_page as i64;
    let search_pattern = format!("%{}%", query);

    let (total, skills) = match (category, license_filter) {
        (Some(cat), Some(lic)) => {
            let total: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM skills s
                 JOIN skill_categories sc ON s.id = sc.skill_id
                 JOIN categories c ON sc.category_id = c.id
                 WHERE c.name = $1 AND s.license = $2
                 AND (s.name ILIKE $3 OR s.description ILIKE $3 OR s.skill_md_content ILIKE $3)",
            )
            .bind(&cat)
            .bind(&lic)
            .bind(&search_pattern)
            .fetch_one(pool)
            .await?;

            let skills = sqlx::query_as::<_, Skill>(
                "SELECT s.id, s.name, s.description, s.source, s.source_url, s.license,
                        s.content_hash, s.skill_md_content, s.safety_level,
                        s.format_score, s.quality_score, s.rating, s.install_count,
                        s.created_at, s.updated_at
                 FROM skills s
                 JOIN skill_categories sc ON s.id = sc.skill_id
                 JOIN categories c ON sc.category_id = c.id
                 WHERE c.name = $1 AND s.license = $2
                 AND (s.name ILIKE $3 OR s.description ILIKE $3 OR s.skill_md_content ILIKE $3)
                 ORDER BY s.created_at DESC
                 LIMIT $4 OFFSET $5",
            )
            .bind(&cat)
            .bind(&lic)
            .bind(&search_pattern)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?;

            (total, skills)
        }
        (Some(cat), None) => {
            let total: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM skills s
                 JOIN skill_categories sc ON s.id = sc.skill_id
                 JOIN categories c ON sc.category_id = c.id
                 WHERE c.name = $1
                 AND (s.name ILIKE $2 OR s.description ILIKE $2 OR s.skill_md_content ILIKE $2)",
            )
            .bind(&cat)
            .bind(&search_pattern)
            .fetch_one(pool)
            .await?;

            let skills = sqlx::query_as::<_, Skill>(
                "SELECT s.id, s.name, s.description, s.source, s.source_url, s.license,
                        s.content_hash, s.skill_md_content, s.safety_level,
                        s.format_score, s.quality_score, s.rating, s.install_count,
                        s.created_at, s.updated_at
                 FROM skills s
                 JOIN skill_categories sc ON s.id = sc.skill_id
                 JOIN categories c ON sc.category_id = c.id
                 WHERE c.name = $1
                 AND (s.name ILIKE $2 OR s.description ILIKE $2 OR s.skill_md_content ILIKE $2)
                 ORDER BY s.created_at DESC
                 LIMIT $3 OFFSET $4",
            )
            .bind(&cat)
            .bind(&search_pattern)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?;

            (total, skills)
        }
        (None, Some(lic)) => {
            let total: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM skills
                 WHERE license = $1
                 AND (name ILIKE $2 OR description ILIKE $2 OR skill_md_content ILIKE $2)",
            )
            .bind(&lic)
            .bind(&search_pattern)
            .fetch_one(pool)
            .await?;

            let skills = sqlx::query_as::<_, Skill>(
                "SELECT id, name, description, source, source_url, license, content_hash,
                        skill_md_content, safety_level, format_score, quality_score, rating,
                        install_count, created_at, updated_at
                 FROM skills
                 WHERE license = $1
                 AND (name ILIKE $2 OR description ILIKE $2 OR skill_md_content ILIKE $2)
                 ORDER BY created_at DESC
                 LIMIT $3 OFFSET $4",
            )
            .bind(&lic)
            .bind(&search_pattern)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?;

            (total, skills)
        }
        (None, None) => {
            let total: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM skills
                 WHERE name ILIKE $1 OR description ILIKE $1 OR skill_md_content ILIKE $1",
            )
            .bind(&search_pattern)
            .fetch_one(pool)
            .await?;

            let skills = sqlx::query_as::<_, Skill>(
                "SELECT id, name, description, source, source_url, license, content_hash,
                        skill_md_content, safety_level, format_score, quality_score, rating,
                        install_count, created_at, updated_at
                 FROM skills
                 WHERE name ILIKE $1 OR description ILIKE $1 OR skill_md_content ILIKE $1
                 ORDER BY created_at DESC
                 LIMIT $2 OFFSET $3",
            )
            .bind(&search_pattern)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?;

            (total, skills)
        }
    };

    Ok(PaginatedResponse {
        data: skills,
        total,
        page,
        per_page,
    })
}

pub async fn get_skill_by_id(pool: &PgPool, skill_id: Uuid) -> Result<Skill, sqlx::Error> {
    let skill = sqlx::query_as::<_, Skill>(
        "SELECT id, name, description, source, source_url, license, content_hash,
                skill_md_content, safety_level, format_score, quality_score, rating,
                install_count, created_at, updated_at
         FROM skills
         WHERE id = $1",
    )
    .bind(skill_id)
    .fetch_one(pool)
    .await?;

    Ok(skill)
}

pub async fn get_categories_for_skill(
    pool: &PgPool,
    skill_id: Uuid,
) -> Result<Vec<Category>, sqlx::Error> {
    let categories = sqlx::query_as::<_, Category>(
        "SELECT c.id, c.name, c.display_name
         FROM categories c
         JOIN skill_categories sc ON c.id = sc.category_id
         WHERE sc.skill_id = $1",
    )
    .bind(skill_id)
    .fetch_all(pool)
    .await?;

    Ok(categories)
}

pub async fn get_similar_skills(
    pool: &PgPool,
    skill_id: Uuid,
    limit: u32,
) -> Result<Vec<Skill>, sqlx::Error> {
    let skills = sqlx::query_as::<_, Skill>(
        "SELECT DISTINCT s.id, s.name, s.description, s.source, s.source_url, s.license,
                s.content_hash, s.skill_md_content, s.safety_level,
                s.format_score, s.quality_score, s.rating, s.install_count,
                s.created_at, s.updated_at
         FROM skills s
         JOIN skill_categories sc ON s.id = sc.skill_id
         WHERE sc.category_id IN (
             SELECT sc2.category_id
             FROM skill_categories sc2
             WHERE sc2.skill_id = $1
         )
         AND s.id != $1
         ORDER BY s.rating DESC NULLS LAST
         LIMIT $2",
    )
    .bind(skill_id)
    .bind(limit as i64)
    .fetch_all(pool)
    .await?;

    Ok(skills)
}

pub async fn get_categories(pool: &PgPool) -> Result<Vec<Category>, sqlx::Error> {
    let categories = sqlx::query_as::<_, Category>(
        "SELECT id, name, display_name FROM categories ORDER BY display_name",
    )
    .fetch_all(pool)
    .await?;

    Ok(categories)
}

pub async fn upsert_skill(pool: &PgPool, raw: &RawSkill) -> Result<Skill, sqlx::Error> {
    let skill = sqlx::query_as::<_, Skill>(
        "INSERT INTO skills (name, description, source, source_url, license, content_hash, skill_md_content)
         VALUES ($1, $2, $3, $4, $5, $6, $7)
         ON CONFLICT (content_hash)
         DO UPDATE SET
             name = EXCLUDED.name,
             description = EXCLUDED.description,
             source_url = EXCLUDED.source_url,
             skill_md_content = EXCLUDED.skill_md_content,
             license = EXCLUDED.license,
             updated_at = NOW()
         RETURNING id, name, description, source, source_url, license, content_hash,
                   skill_md_content, safety_level, format_score, quality_score, rating,
                   install_count, created_at, updated_at",
    )
    .bind(&raw.name)
    .bind(&raw.description)
    .bind(&raw.source)
    .bind(&raw.source_url)
    .bind(&raw.license)
    .bind(&raw.content_hash)
    .bind(&raw.content)
    .fetch_one(pool)
    .await?;

    Ok(skill)
}

pub async fn insert_category(pool: &PgPool, name: &str, display_name: &str) -> Result<Category, sqlx::Error> {
    let category = sqlx::query_as::<_, Category>(
        "INSERT INTO categories (name, display_name)
         VALUES ($1, $2)
         ON CONFLICT (name)
         DO UPDATE SET display_name = EXCLUDED.display_name
         RETURNING id, name, display_name",
    )
    .bind(name)
    .bind(display_name)
    .fetch_one(pool)
    .await?;

    Ok(category)
}

pub async fn get_stats(pool: &PgPool) -> Result<Stats, sqlx::Error> {
    let stats = sqlx::query_as::<_, Stats>(
        "SELECT
             COUNT(*)::bigint as total_skills,
             COUNT(DISTINCT source)::bigint as total_sources,
             (SELECT COUNT(*)::bigint FROM categories) as total_categories,
             AVG(rating) as avg_rating,
             AVG(quality_score) as avg_quality_score,
             AVG(format_score) as avg_format_score
         FROM skills",
    )
    .fetch_one(pool)
    .await?;

    Ok(stats)
}
