use crate::crawler::RawSkill;

/// Stub adapter for the SkillNet API.
///
/// This is a placeholder for future implementation. It currently returns
/// an empty vector, indicating no skills were fetched.
pub async fn fetch_skills() -> Result<Vec<RawSkill>, Box<dyn std::error::Error + Send + Sync>> {
    tracing::debug!("SkillNet adapter: stub implementation, returning empty results");
    Ok(Vec::new())
}
