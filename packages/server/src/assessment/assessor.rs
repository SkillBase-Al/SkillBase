use serde::{Deserialize, Serialize};

use crate::llm::provider::{LlmClient, LlmMessage};

/// Results from an LLM skill assessment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssessResult {
    pub description_score: u32,
    pub instructions_score: u32,
    pub scripts_score: u32,
    pub total_score: u32,
}

/// Assess a skill's content using the configured LLM provider.
///
/// Sends the skill content to the LLM and parses the response for
/// three quality scores (description, instructions, scripts) and a total.
pub async fn assess(
    llm: &LlmClient,
    skill_content: &str,
) -> Result<AssessResult, Box<dyn std::error::Error + Send + Sync>> {
    let system_prompt = "\
You are a skill assessment AI. Evaluate the following skill/automation content and rate \
it on three dimensions:

1. **description_score** (0-100): How well is the skill described? Does it have a clear purpose, usage instructions, and examples?
2. **instructions_score** (0-100): How clear and complete are the installation and usage instructions? Are prerequisites listed?
3. **scripts_score** (0-100): How well-written and robust are any scripts or automation code? Are there error checks, edge cases handled?

Return ONLY valid JSON in this exact format, with no additional text or markdown formatting:
{\"description_score\": 85, \"instructions_score\": 70, \"scripts_score\": 90, \"total_score\": 245}

Where total_score is the sum of the three individual scores.";

    let user_message = format!(
        "Please evaluate this skill content:\n\n{}",
        skill_content
    );

    let response = llm
        .complete(
            system_prompt,
            vec![LlmMessage {
                role: "user".to_string(),
                content: user_message,
            }],
            500,
        )
        .await?;

    // Extract JSON from the response (handle potential markdown code fences)
    let json_str = extract_json(&response.content)?;

    let result: AssessResult = serde_json::from_str(&json_str).map_err(|e| {
        format!(
            "Failed to parse LLM response as AssessResult: {}. Raw: {}",
            e,
            response.content.chars().take(200).collect::<String>()
        )
    })?;

    // Validate score ranges
    let description_score = result.description_score.min(100);
    let instructions_score = result.instructions_score.min(100);
    let scripts_score = result.scripts_score.min(100);
    let total_score = description_score + instructions_score + scripts_score;

    tracing::info!(
        "Assessment complete: desc={}, instr={}, scripts={}, total={} (tokens: {} prompt / {} completion)",
        description_score,
        instructions_score,
        scripts_score,
        total_score,
        response.prompt_tokens,
        response.completion_tokens,
    );

    Ok(AssessResult {
        description_score,
        instructions_score,
        scripts_score,
        total_score,
    })
}

/// Extract a JSON object from the LLM response text.
///
/// Handles responses that may be wrapped in markdown code fences.
fn extract_json(text: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let text = text.trim();

    // Try to extract from ```json ... ``` fences
    if let Some(start) = text.find("```json") {
        let after_fence = &text[start + 7..];
        if let Some(end) = after_fence.find("```") {
            return Ok(after_fence[..end].trim().to_string());
        }
    }

    // Try to extract from ``` ... ``` fences (generic)
    if let Some(start) = text.find("```") {
        let after_fence = &text[start + 3..];
        if let Some(end) = after_fence.find("```") {
            return Ok(after_fence[..end].trim().to_string());
        }
    }

    // Assume the text itself is JSON
    // Find the first '{' and last '}'
    if let Some(start) = text.find('{') {
        if let Some(end) = text.rfind('}') {
            return Ok(text[start..=end].to_string());
        }
    }

    Err(format!("No JSON object found in response: {}", text.chars().take(100).collect::<String>()).into())
}
