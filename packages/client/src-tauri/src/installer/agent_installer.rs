use std::fs;
use crate::db::models::InstalledSkill;
use crate::db::repository::{self, DbConn};
use crate::utils::paths;

/// Install a skill to specific agent directories (each sub-path in base_path)
pub fn install_to_agents(
    conn: &DbConn,
    skill: &InstalledSkill,
    skill_content: &str,
    agent_ids: &[String],
) -> Result<Vec<String>, String> {
    let agents = repository::get_all_agents(conn)?;
    let mut installed_to = Vec::new();

    for agent_id in agent_ids {
        let agent = agents.iter()
            .find(|a| a.id == *agent_id)
            .ok_or_else(|| format!("Agent not found: {}", agent_id))?;

        let sub_paths = paths::split_base_paths(&agent.base_path);
        for sub_path in &sub_paths {
            let agent_skill_dir = paths::get_agent_skill_dir(&agent.agent_type, sub_path);
            if !agent_skill_dir.exists() {
                fs::create_dir_all(&agent_skill_dir)
                    .map_err(|e| format!("Failed to create agent skill dir: {}", e))?;
            }

            // Create skill subdirectory
            let skill_dir = agent_skill_dir.join(&skill.name);
            fs::create_dir_all(&skill_dir)
                .map_err(|e| format!("Failed to create skill dir in agent: {}", e))?;

            // Write SKILL.md
            let skill_md_path = skill_dir.join("SKILL.md");
            fs::write(&skill_md_path, skill_content)
                .map_err(|e| format!("Failed to write SKILL.md to agent: {}", e))?;
        }

        // Record mapping once per agent
        repository::add_mapping(conn, &skill.id, agent_id)?;
        installed_to.push(agent.name.clone());
    }

    Ok(installed_to)
}

/// Uninstall a skill from all associated agents
pub fn uninstall_from_agents(
    conn: &DbConn,
    skill: &InstalledSkill,
) -> Result<Vec<String>, String> {
    let agent_ids = repository::get_mappings_for_skill(conn, &skill.id)?;
    let agents = repository::get_all_agents(conn)?;
    let mut removed_from = Vec::new();

    for agent_id in &agent_ids {
        if let Some(agent) = agents.iter().find(|a| a.id == *agent_id) {
            let sub_paths = paths::split_base_paths(&agent.base_path);
            for sub_path in &sub_paths {
                let agent_skill_dir = paths::get_agent_skill_dir(&agent.agent_type, sub_path);
                let skill_dir = agent_skill_dir.join(&skill.name);

                if skill_dir.exists() {
                    fs::remove_dir_all(&skill_dir)
                        .map_err(|e| format!("Failed to remove skill from agent: {}", e))?;
                }
            }

            removed_from.push(agent.name.clone());
        }
    }

    repository::remove_mappings_for_skill(conn, &skill.id)?;
    Ok(removed_from)
}

/// Uninstall a skill from specific agents only
pub fn uninstall_from_agents_by_ids(
    conn: &DbConn,
    skill: &InstalledSkill,
    agent_ids: &[String],
) -> Result<Vec<String>, String> {
    let agents = repository::get_all_agents(conn)?;
    let mut removed_from = Vec::new();

    for agent_id in agent_ids {
        if let Some(agent) = agents.iter().find(|a| a.id == *agent_id) {
            let sub_paths = paths::split_base_paths(&agent.base_path);
            for sub_path in &sub_paths {
                let agent_skill_dir = paths::get_agent_skill_dir(&agent.agent_type, sub_path);
                let skill_dir = agent_skill_dir.join(&skill.name);

                if skill_dir.exists() {
                    fs::remove_dir_all(&skill_dir)
                        .map_err(|e| format!("Failed to remove skill from agent: {}", e))?;
                }
            }

            removed_from.push(agent.name.clone());
        }
    }

    for agent_id in agent_ids {
        repository::remove_mapping(conn, &skill.id, agent_id)?;
    }

    Ok(removed_from)
}

/// Update a skill to all currently mapped agents
pub fn update_in_agents(
    conn: &DbConn,
    skill: &InstalledSkill,
    skill_content: &str,
) -> Result<Vec<String>, String> {
    let agent_ids = repository::get_mappings_for_skill(conn, &skill.id)?;
    install_to_agents(conn, skill, skill_content, &agent_ids)
}

/// Toggle skill enabled/disabled by renaming in agent dirs
pub fn toggle_in_agents(
    conn: &DbConn,
    skill: &InstalledSkill,
    enabled: bool,
) -> Result<(), String> {
    let agent_ids = repository::get_mappings_for_skill(conn, &skill.id)?;
    let agents = repository::get_all_agents(conn)?;

    for agent_id in &agent_ids {
        if let Some(agent) = agents.iter().find(|a| a.id == *agent_id) {
            let sub_paths = paths::split_base_paths(&agent.base_path);
            for sub_path in &sub_paths {
                let agent_skill_dir = paths::get_agent_skill_dir(&agent.agent_type, sub_path);
                let skill_dir = agent_skill_dir.join(&skill.name);
                let disabled_dir = agent_skill_dir.join(format!("{}.disabled", skill.name));

                if enabled {
                    if disabled_dir.exists() {
                        fs::rename(&disabled_dir, &skill_dir)
                            .map_err(|e| format!("Failed to enable skill: {}", e))?;
                    }
                } else {
                    if skill_dir.exists() {
                        fs::rename(&skill_dir, &disabled_dir)
                            .map_err(|e| format!("Failed to disable skill: {}", e))?;
                    }
                }
            }
        }
    }

    repository::toggle_skill(conn, &skill.id, enabled)?;
    Ok(())
}
