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

/// CLI entry point: list available guide topics with descriptions.
pub fn list_topics() -> i32 {
    println!("Available topics:");
    println!();
    println!("  workflow      Quick workflow phase reference (spec → test → implement)");
    println!("  authoring     Spec authoring patterns and granularity rules");
    println!("  smells        Requirements smell detection (ambiguity, Observer Test, Swap Test)");
    println!("  nfr           NFR design: categories, constraints, FR/NFR decision tree");
    println!("  context       Context management protocol for lazy loading specs");
    println!("  methodology   Full spec-driven development reference");
    println!("  coverage      Coverage tagging guide for linking tests to spec behaviors");
    println!();
    println!("Usage: minter guide <topic>");
    0
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
