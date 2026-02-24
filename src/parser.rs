use crate::model::*;
use std::fmt;

#[derive(Debug, Clone)]
pub struct ParseError {
    pub line: usize,
    pub message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}: {}", self.line, self.message)
    }
}

/// Parse a .spec DSL string into a Spec model.
pub fn parse(source: &str) -> Result<Spec, Vec<ParseError>> {
    let mut parser = Parser::new(source);
    parser.parse_spec()
}

struct Parser<'a> {
    lines: Vec<&'a str>,
    pos: usize,
}

// ── Core navigation ─────────────────────────────────────

impl<'a> Parser<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            lines: source.lines().collect(),
            pos: 0,
        }
    }

    fn line_num(&self) -> usize {
        self.pos + 1
    }

    fn at_end(&self) -> bool {
        self.pos >= self.lines.len()
    }

    fn current_line(&self) -> &'a str {
        self.lines[self.pos]
    }

    fn advance(&mut self) {
        self.pos += 1;
    }

    fn skip_blank_lines(&mut self) {
        while !self.at_end() {
            let trimmed = self.current_line().trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn err(&self, msg: impl Into<String>) -> Vec<ParseError> {
        vec![ParseError {
            line: self.line_num(),
            message: msg.into(),
        }]
    }
}

// ── Top-level parse ─────────────────────────────────────

impl<'a> Parser<'a> {
    fn parse_spec(&mut self) -> Result<Spec, Vec<ParseError>> {
        // Reject tab indentation anywhere in the file
        let tab_errors: Vec<ParseError> = self
            .lines
            .iter()
            .enumerate()
            .filter(|(_, line)| line.starts_with('\t'))
            .map(|(i, _)| ParseError {
                line: i + 1,
                message: "tab indentation is not allowed, use two spaces".to_string(),
            })
            .collect();
        if !tab_errors.is_empty() {
            return Err(tab_errors);
        }

        self.skip_blank_lines();
        if self.at_end() {
            return Err(self.err("Empty input — expected 'spec <name> v<version>'"));
        }

        let (name, version) = self.parse_spec_line()?;
        self.skip_blank_lines();
        let title = self.parse_title_line()?;
        self.skip_blank_lines();
        let description = self.parse_text_block("description")?;
        self.skip_blank_lines();
        let motivation = self.parse_text_block("motivation")?;
        self.skip_blank_lines();

        let behaviors = self.parse_behaviors()?;
        self.skip_blank_lines();

        let dependencies = if !self.at_end() {
            self.parse_dependencies()?
        } else {
            vec![]
        };

        Ok(Spec {
            name,
            version,
            title,
            description,
            motivation,
            behaviors,
            dependencies,
        })
    }
}

// ── Header parsing ──────────────────────────────────────

impl<'a> Parser<'a> {
    fn parse_spec_line(&mut self) -> Result<(String, String), Vec<ParseError>> {
        let line = self.current_line().trim();
        if !line.starts_with("spec ") {
            return Err(self.err(format!(
                "Expected 'spec <name> v<version>', got '{}'",
                first_word(line)
            )));
        }
        let rest = line.strip_prefix("spec ").unwrap().trim();
        let parts: Vec<&str> = rest.splitn(2, ' ').collect();
        if parts.len() < 2 {
            return Err(self.err("Expected 'spec <name> v<version>'"));
        }
        let name = parts[0].to_string();
        let version = parts[1]
            .strip_prefix('v')
            .unwrap_or(parts[1])
            .to_string();
        self.advance();
        Ok((name, version))
    }

    fn parse_title_line(&mut self) -> Result<String, Vec<ParseError>> {
        if self.at_end() {
            return Err(self.err("Expected 'title \"...\"'"));
        }
        let line = self.current_line().trim();
        if !line.starts_with("title ") {
            return Err(self.err(format!(
                "Expected 'title \"...\"', got '{}'",
                first_word(line)
            )));
        }
        let rest = line.strip_prefix("title ").unwrap().trim();
        let title = parse_quoted_string(rest)
            .ok_or_else(|| self.err("Unclosed quote in title"))?;
        self.advance();
        Ok(title)
    }

    fn parse_text_block(&mut self, keyword: &str) -> Result<String, Vec<ParseError>> {
        if self.at_end() {
            return Err(self.err(format!("Expected '{keyword}'")));
        }
        let line = self.current_line().trim();
        if line != keyword {
            return Err(self.err(format!(
                "Expected '{keyword}', got '{}'",
                first_word(line)
            )));
        }
        self.advance();
        let mut lines = Vec::new();
        while !self.at_end() {
            let line = self.current_line();
            if line.starts_with("  ") {
                lines.push(line.trim());
                self.advance();
            } else {
                break;
            }
        }
        Ok(lines.join("\n"))
    }
}

// ── Behavior parsing ────────────────────────────────────

impl<'a> Parser<'a> {
    fn parse_behaviors(&mut self) -> Result<Vec<Behavior>, Vec<ParseError>> {
        let mut behaviors = Vec::new();
        while !self.at_end() {
            let line = self.current_line().trim();
            if line.starts_with("behavior ") {
                behaviors.push(self.parse_one_behavior()?);
                self.skip_blank_lines();
            } else if line.starts_with("depends on ") {
                break;
            } else if line.is_empty() || line.starts_with('#') {
                self.advance();
            } else {
                return Err(self.err(format!(
                    "Expected 'behavior' declaration, got '{}'",
                    first_word(line)
                )));
            }
        }
        if behaviors.is_empty() {
            return Err(self.err("No behaviors defined — at least one is required"));
        }
        Ok(behaviors)
    }

    fn parse_one_behavior(&mut self) -> Result<Behavior, Vec<ParseError>> {
        let (name, category) = self.parse_behavior_header()?;
        self.skip_blank_lines();
        let description = self.parse_behavior_description()?;
        self.skip_blank_lines();
        let preconditions = self.parse_given_section()?;
        self.skip_blank_lines();
        let action = self.parse_when_section()?;
        self.skip_blank_lines();
        let postconditions = self.parse_then_sections()?;

        Ok(Behavior {
            name,
            category,
            description,
            preconditions,
            action,
            postconditions,
        })
    }

    fn parse_behavior_header(&mut self) -> Result<(String, BehaviorCategory), Vec<ParseError>> {
        let line = self.current_line().trim();
        let rest = line.strip_prefix("behavior ").unwrap();
        let (name_part, bracket) = rest
            .rsplit_once('[')
            .ok_or_else(|| self.err("Expected 'behavior <name> [<category>]'"))?;
        let name = name_part.trim().to_string();
        let cat_str = bracket
            .strip_suffix(']')
            .ok_or_else(|| self.err("Expected closing ']' for category"))?
            .trim();
        let category = match cat_str {
            "happy_path" => BehaviorCategory::HappyPath,
            "error_case" => BehaviorCategory::ErrorCase,
            "edge_case" => BehaviorCategory::EdgeCase,
            other => {
                return Err(self.err(format!(
                    "Unknown category '{}' — expected happy_path, error_case, or edge_case",
                    other
                )))
            }
        };
        self.advance();
        Ok((name, category))
    }

    fn parse_behavior_description(&mut self) -> Result<String, Vec<ParseError>> {
        if self.at_end() {
            return Err(self.err("Expected quoted behavior description"));
        }
        let line = self.current_line().trim();
        let desc = parse_quoted_string(line)
            .ok_or_else(|| self.err("Expected quoted behavior description"))?;
        self.advance();
        Ok(desc)
    }
}

// ── Given section ───────────────────────────────────────

impl<'a> Parser<'a> {
    fn parse_given_section(&mut self) -> Result<Vec<Precondition>, Vec<ParseError>> {
        if self.at_end() {
            return Err(self.err("Expected 'given' section"));
        }
        let line = self.current_line().trim();
        if line == "when" || line.starts_with("when ") {
            return Err(self.err("Expected 'given' section before 'when'"));
        }
        if line == "then" || line.starts_with("then ") {
            return Err(self.err("Expected 'given' section before 'then'"));
        }
        if line != "given" {
            return Err(self.err(format!("Expected 'given', got '{}'", first_word(line))));
        }
        self.advance();
        let mut preconditions = Vec::new();
        while !self.at_end() {
            let line = self.current_line();
            let trimmed = line.trim();
            if !is_indented(line) || trimmed.is_empty() {
                break;
            }
            if trimmed.starts_with('@') {
                preconditions.push(self.parse_alias_declaration()?);
            } else {
                preconditions.push(Precondition::Prose(trimmed.to_string()));
                self.advance();
            }
        }
        Ok(preconditions)
    }

    fn parse_alias_declaration(&mut self) -> Result<Precondition, Vec<ParseError>> {
        let line = self.current_line().trim();
        let rest = line.strip_prefix('@').unwrap();
        let (alias_name, after_eq) = rest
            .split_once('=')
            .ok_or_else(|| self.err("Expected '@alias = Entity { ... }'"))?;
        let alias_name = alias_name.trim().to_string();
        let after_eq = after_eq.trim();

        // Entity name must come before '{'
        let (entity, props_str) = after_eq
            .split_once('{')
            .ok_or_else(|| self.err("Expected 'Entity { ... }' after '='"))?;
        let entity = entity.trim().to_string();
        if entity.is_empty() || !entity.chars().next().unwrap().is_uppercase() {
            return Err(self.err(format!(
                "Expected entity type name (e.g. User), got '{}'",
                entity
            )));
        }
        let props_str = props_str
            .strip_suffix('}')
            .ok_or_else(|| self.err("Expected closing '}' in alias declaration"))?
            .trim();
        let properties = parse_properties(props_str);
        self.advance();
        Ok(Precondition::Alias {
            name: alias_name,
            entity,
            properties,
        })
    }
}

// ── When section ────────────────────────────────────────

impl<'a> Parser<'a> {
    fn parse_when_section(&mut self) -> Result<Action, Vec<ParseError>> {
        if self.at_end() {
            return Err(self.err("Expected 'when' section"));
        }
        let line = self.current_line().trim();
        if line == "then" || line.starts_with("then ") {
            return Err(self.err("Expected 'when' section before 'then'"));
        }
        if !line.starts_with("when") {
            return Err(self.err(format!("Expected 'when', got '{}'", first_word(line))));
        }
        let action_name = line
            .strip_prefix("when")
            .unwrap()
            .trim()
            .to_string();
        self.advance();

        let mut inputs = Vec::new();
        while !self.at_end() {
            let line = self.current_line();
            let trimmed = line.trim();
            if !is_indented(line) || trimmed.is_empty() {
                break;
            }
            inputs.push(self.parse_action_input()?);
        }
        Ok(Action {
            name: action_name,
            inputs,
        })
    }

    fn parse_action_input(&mut self) -> Result<ActionInput, Vec<ParseError>> {
        let line = self.current_line().trim();
        let (name, value_str) = line
            .split_once('=')
            .ok_or_else(|| self.err("Expected 'name = value' in when section"))?;
        let name = name.trim().to_string();
        let value_str = value_str.trim();

        self.advance();

        if value_str.starts_with('@') {
            let ref_str = value_str.strip_prefix('@').unwrap();
            let (alias, field) = ref_str.split_once('.').ok_or_else(|| {
                vec![ParseError {
                    line: self.line_num() - 1,
                    message: format!(
                        "Malformed alias reference '@{}' — expected '@alias.field'",
                        ref_str
                    ),
                }]
            })?;
            if alias.is_empty() || field.is_empty() {
                return Err(vec![ParseError {
                    line: self.line_num() - 1,
                    message: format!("Malformed alias reference '@{ref_str}'"),
                }]);
            }
            Ok(ActionInput::AliasRef {
                name,
                alias: alias.to_string(),
                field: field.to_string(),
            })
        } else {
            let value = unquote(value_str);
            Ok(ActionInput::Value { name, value })
        }
    }
}

// ── Then section ────────────────────────────────────────

impl<'a> Parser<'a> {
    fn parse_then_sections(&mut self) -> Result<Vec<Postcondition>, Vec<ParseError>> {
        let mut postconditions = Vec::new();
        while !self.at_end() {
            let trimmed = self.current_line().trim();
            if trimmed.starts_with("then ") || trimmed == "then" {
                postconditions.push(self.parse_one_then()?);
                self.skip_blank_lines();
            } else if trimmed.is_empty() {
                self.advance();
            } else {
                break;
            }
        }
        if postconditions.is_empty() {
            return Err(self.err("Expected at least one 'then' section"));
        }
        Ok(postconditions)
    }

    fn parse_one_then(&mut self) -> Result<Postcondition, Vec<ParseError>> {
        let line = self.current_line().trim();
        let rest = line.strip_prefix("then").unwrap().trim();
        let kind = parse_postcondition_kind(rest);
        self.advance();

        let mut assertions = Vec::new();
        while !self.at_end() {
            let line = self.current_line();
            let trimmed = line.trim();
            if !is_indented(line) || trimmed.is_empty() {
                break;
            }
            if trimmed.starts_with("assert ") || trimmed == "assert" {
                assertions.push(self.parse_assertion()?);
            } else {
                self.advance();
            }
        }
        Ok(Postcondition { kind, assertions })
    }

    fn parse_assertion(&mut self) -> Result<Assertion, Vec<ParseError>> {
        let line = self.current_line().trim();
        let rest = line.strip_prefix("assert").unwrap().trim();
        self.advance();

        if rest.is_empty() || rest.starts_with("==") || rest.starts_with(">=") {
            return Err(malformed_assertion_error(self.line_num() - 1));
        }

        let tokens: Vec<&str> = rest.split_whitespace().collect();
        let op_idx = tokens.iter().position(|t| {
            matches!(*t, "==" | "is_present" | "contains" | "in_range" | "matches_pattern" | ">=")
        });

        match op_idx {
            Some(0) => Err(malformed_assertion_error(self.line_num() - 1)),
            Some(idx) => {
                let field = tokens[..idx].join(" ");
                let remainder = if idx + 1 < tokens.len() {
                    tokens[idx + 1..].join(" ")
                } else {
                    String::new()
                };
                parse_typed_assertion(field, tokens[idx], &remainder)
            }
            None => parse_prose_or_unknown(rest, &tokens, self.line_num() - 1),
        }
    }
}

// ── Assertion helpers ───────────────────────────────────

fn malformed_assertion_error(line: usize) -> Vec<ParseError> {
    vec![ParseError {
        line,
        message: "Malformed assertion — expected 'assert <field> <operator> [<value>]'".to_string(),
    }]
}

fn parse_typed_assertion(field: String, operator: &str, remainder: &str) -> Result<Assertion, Vec<ParseError>> {
    match operator {
        "==" => parse_equals_assertion(field, remainder),
        "is_present" => Ok(Assertion::IsPresent { field }),
        "contains" => Ok(Assertion::Contains { field, value: unquote(remainder) }),
        "in_range" => parse_range_assertion(field, remainder),
        "matches_pattern" => Ok(Assertion::MatchesPattern { field, pattern: unquote(remainder) }),
        ">=" => Ok(Assertion::GreaterOrEqual { field, value: unquote(remainder) }),
        _ => unreachable!(),
    }
}

fn parse_equals_assertion(field: String, remainder: &str) -> Result<Assertion, Vec<ParseError>> {
    if remainder.starts_with('@') {
        let ref_str = remainder.strip_prefix('@').unwrap();
        let (alias, alias_field) = ref_str.split_once('.').unwrap_or((ref_str, ""));
        Ok(Assertion::EqualsRef {
            field,
            alias: alias.to_string(),
            alias_field: alias_field.to_string(),
        })
    } else {
        Ok(Assertion::Equals { field, value: unquote(remainder) })
    }
}

fn parse_range_assertion(field: String, remainder: &str) -> Result<Assertion, Vec<ParseError>> {
    let (min, max) = remainder.split_once("..").unwrap_or((remainder, ""));
    Ok(Assertion::InRange {
        field,
        min: min.to_string(),
        max: max.to_string(),
    })
}

fn parse_prose_or_unknown(rest: &str, tokens: &[&str], line: usize) -> Result<Assertion, Vec<ParseError>> {
    if tokens.len() == 3 && tokens[2].starts_with('"') {
        return Err(vec![ParseError {
            line,
            message: format!(
                "Unknown assertion operator '{}' — expected ==, is_present, contains, in_range, matches_pattern, or >=",
                tokens[1]
            ),
        }]);
    }
    Ok(Assertion::Prose(rest.to_string()))
}

// ── Dependency parsing ──────────────────────────────────

impl<'a> Parser<'a> {
    fn parse_dependencies(&mut self) -> Result<Vec<Dependency>, Vec<ParseError>> {
        let mut deps = Vec::new();
        while !self.at_end() {
            let line = self.current_line().trim();
            if line.is_empty() || line.starts_with('#') {
                self.advance();
                continue;
            }
            if line.starts_with("depends on ") {
                deps.push(self.parse_one_dependency()?);
            } else {
                break;
            }
        }
        Ok(deps)
    }

    fn parse_one_dependency(&mut self) -> Result<Dependency, Vec<ParseError>> {
        let line = self.current_line().trim();
        let rest = line.strip_prefix("depends on ").unwrap().trim();
        let parts: Vec<&str> = rest.splitn(3, ' ').collect();
        let spec_name = parts[0].to_string();
        if parts.len() < 3 || parts[1] != ">=" {
            return Err(self.err(format!(
                "Expected 'depends on {} >= <version>'",
                spec_name
            )));
        }
        let version_constraint = parts[2].to_string();
        self.advance();
        Ok(Dependency {
            spec_name,
            version_constraint,
        })
    }
}

// ── Utility functions ───────────────────────────────────

fn first_word(s: &str) -> &str {
    s.split_whitespace().next().unwrap_or(s)
}

fn is_indented(line: &str) -> bool {
    line.starts_with("  ")
}

fn parse_quoted_string(s: &str) -> Option<String> {
    let s = s.trim();
    if !s.starts_with('"') {
        return None;
    }
    let rest = &s[1..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn unquote(s: &str) -> String {
    let s = s.trim();
    if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

fn parse_properties(s: &str) -> Vec<(String, String)> {
    if s.is_empty() {
        return vec![];
    }
    s.split(',')
        .filter_map(|part| {
            let part = part.trim();
            let (key, value) = part.split_once(':')?;
            Some((key.trim().to_string(), unquote(value.trim())))
        })
        .collect()
}

fn parse_postcondition_kind(rest: &str) -> PostconditionKind {
    if rest.starts_with("returns ") {
        PostconditionKind::Returns(rest.strip_prefix("returns ").unwrap().to_string())
    } else if rest.starts_with("emits ") {
        PostconditionKind::Emits(rest.strip_prefix("emits ").unwrap().to_string())
    } else if rest == "side_effect" || rest.starts_with("side_effect") {
        PostconditionKind::SideEffect
    } else if rest == "returns" {
        PostconditionKind::Returns(String::new())
    } else {
        // Default: treat as returns with description
        PostconditionKind::Returns(rest.to_string())
    }
}

#[cfg(test)]
#[path = "parser.test.rs"]
mod tests;
