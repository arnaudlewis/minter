use crate::core::content;
use crate::model::{GuideTopic, VALID_GUIDE_TOPICS};

/// Shared core: returns guide content for a topic string.
/// Used by both CLI and MCP.
pub fn run_guide_topic(topic: &str) -> Result<String, String> {
    match topic {
        "workflow" => Ok(content::guide_workflow().to_string()),
        "authoring" => Ok(content::guide_authoring().to_string()),
        "smells" => Ok(content::guide_smells().to_string()),
        "nfr" => Ok(content::guide_nfr().to_string()),
        "context" => Ok(content::guide_context().to_string()),
        "methodology" => Ok(content::methodology().to_string()),
        "coverage" => Ok(content::guide_coverage().to_string()),
        other => Err(format!(
            "Unknown topic '{}'. Valid topics: {}",
            other,
            VALID_GUIDE_TOPICS.join(", ")
        )),
    }
}

/// CLI entry point: print guide content for the given topic.
pub fn run_guide(topic: &GuideTopic) -> i32 {
    match run_guide_topic(topic.as_str()) {
        Ok(text) => {
            println!("{}", text);
            0
        }
        Err(msg) => {
            eprintln!("{}", msg);
            1
        }
    }
}
