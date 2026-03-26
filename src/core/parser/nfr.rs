use super::common::{first_word, parse_quoted_string, unquote};
use super::fr::ParseError;
use crate::model::*;

const VALID_CATEGORIES: &[&str] = VALID_NFR_CATEGORIES;

const VALID_SEVERITIES: &[&str] = &["critical", "high", "medium", "low"];

const VALID_THRESHOLD_OPS: &[&str] = &["<", ">", "<=", ">=", "=="];

/// Parse a .nfr DSL string into an NfrSpec model.
pub fn parse_nfr(source: &str) -> Result<NfrSpec, Vec<ParseError>> {
    let mut parser = NfrParser::new(source);
    parser.parse()
}

struct NfrParser<'a> {
    lines: Vec<&'a str>,
    pos: usize,
}

// ── Core navigation ─────────────────────────────────────

impl<'a> NfrParser<'a> {
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

impl<'a> NfrParser<'a> {
    fn parse(&mut self) -> Result<NfrSpec, Vec<ParseError>> {
        // Reject tab indentation anywhere
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
            return Err(self.err("Empty input — expected 'nfr <category> v<version>'"));
        }

        let (category, version) = self.parse_nfr_line()?;
        self.skip_blank_lines();
        let title = self.parse_title_line()?;
        self.skip_blank_lines();
        let description = self.parse_text_block("description")?;
        self.skip_blank_lines();
        let motivation = self.parse_text_block("motivation")?;
        self.skip_blank_lines();

        let constraints = self.parse_constraints()?;

        Ok(NfrSpec {
            category,
            version,
            title,
            description,
            motivation,
            constraints,
        })
    }
}

// ── Header parsing ──────────────────────────────────────

impl<'a> NfrParser<'a> {
    fn parse_nfr_line(&mut self) -> Result<(String, String), Vec<ParseError>> {
        let line = self.current_line().trim();
        if !line.starts_with("nfr ") {
            return Err(self.err(format!(
                "Expected 'nfr <category> v<version>', got '{}'",
                first_word(line)
            )));
        }
        let rest = line.strip_prefix("nfr ").unwrap().trim();
        let parts: Vec<&str> = rest.splitn(2, ' ').collect();
        let category = parts[0].to_string();

        if !VALID_CATEGORIES.contains(&category.as_str()) {
            return Err(self.err(format!(
                "Invalid category '{}' — valid categories: {}",
                category,
                VALID_CATEGORIES.join(", ")
            )));
        }

        if parts.len() < 2 {
            return Err(self.err(format!(
                "Expected 'nfr {} v<version>' — missing version",
                category
            )));
        }

        let version = parts[1].strip_prefix('v').unwrap_or(parts[1]).to_string();
        self.advance();
        Ok((category, version))
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
        let title = parse_quoted_string(rest).ok_or_else(|| self.err("Unclosed quote in title"))?;
        self.advance();
        Ok(title)
    }

    fn parse_text_block(&mut self, keyword: &str) -> Result<String, Vec<ParseError>> {
        if self.at_end() {
            return Err(self.err(format!("Expected '{keyword}'")));
        }
        let line = self.current_line().trim();
        if line != keyword {
            return Err(self.err(format!("Expected '{keyword}', got '{}'", first_word(line))));
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

// ── Constraint parsing ──────────────────────────────────

impl<'a> NfrParser<'a> {
    fn parse_constraints(&mut self) -> Result<Vec<NfrConstraint>, Vec<ParseError>> {
        let mut constraints = Vec::new();
        while !self.at_end() {
            let line = self.current_line().trim();
            if line.starts_with("constraint ") {
                constraints.push(self.parse_one_constraint()?);
                self.skip_blank_lines();
            } else if line.is_empty() || line.starts_with('#') {
                self.advance();
            } else if constraints.is_empty() {
                return Err(self.err(format!(
                    "Expected 'constraint' declaration, got '{}'",
                    first_word(line)
                )));
            } else {
                return Err(self.err(format!(
                    "Unexpected content after end of NFR spec (line {})",
                    self.line_num()
                )));
            }
        }
        if constraints.is_empty() {
            return Err(self.err("No constraints defined — at least one is required"));
        }
        Ok(constraints)
    }

    fn parse_one_constraint(&mut self) -> Result<NfrConstraint, Vec<ParseError>> {
        let (name, constraint_type) = self.parse_constraint_header()?;
        self.skip_blank_lines();
        let description = self.parse_constraint_description()?;
        self.skip_blank_lines();

        let body = match constraint_type {
            ConstraintType::Metric => self.parse_metric_body()?,
            ConstraintType::Rule => self.parse_rule_body()?,
        };

        self.skip_blank_lines();
        let violation = self.parse_violation()?;
        self.skip_blank_lines();
        let overridable = self.parse_overridable()?;

        Ok(NfrConstraint {
            name,
            constraint_type,
            description,
            body,
            violation,
            overridable,
        })
    }

    fn parse_constraint_header(&mut self) -> Result<(String, ConstraintType), Vec<ParseError>> {
        let line = self.current_line().trim();
        let rest = line.strip_prefix("constraint ").unwrap();
        let (name_part, bracket) = rest
            .rsplit_once('[')
            .ok_or_else(|| self.err("Expected 'constraint <name> [<type>]'"))?;
        let name = name_part.trim().to_string();
        let type_str = bracket
            .strip_suffix(']')
            .ok_or_else(|| self.err("Expected closing ']' for constraint type"))?
            .trim();
        let constraint_type = match type_str {
            "metric" => ConstraintType::Metric,
            "rule" => ConstraintType::Rule,
            other => {
                return Err(self.err(format!(
                    "Unknown constraint type '{}' — expected metric or rule",
                    other
                )));
            }
        };
        self.advance();
        Ok((name, constraint_type))
    }

    fn parse_constraint_description(&mut self) -> Result<String, Vec<ParseError>> {
        if self.at_end() {
            return Err(self.err("Expected quoted constraint description"));
        }
        let line = self.current_line().trim();
        let desc = parse_quoted_string(line)
            .ok_or_else(|| self.err("Expected quoted constraint description"))?;
        self.advance();
        Ok(desc)
    }
}

// ── Metric body ─────────────────────────────────────────

impl<'a> NfrParser<'a> {
    fn parse_metric_body(&mut self) -> Result<ConstraintBody, Vec<ParseError>> {
        let metric = self.parse_metric_field()?;
        self.skip_blank_lines();
        let (threshold_operator, threshold_value) = self.parse_threshold()?;
        self.skip_blank_lines();
        let verification = self.parse_metric_verification()?;

        Ok(ConstraintBody::Metric {
            metric,
            threshold_operator,
            threshold_value,
            verification,
        })
    }

    fn parse_metric_field(&mut self) -> Result<String, Vec<ParseError>> {
        if self.at_end() {
            return Err(self.err("Expected 'metric \"...\"'"));
        }
        let line = self.current_line().trim();
        if !line.starts_with("metric ") {
            return Err(self.err(format!(
                "Expected 'metric \"...\"', got '{}'",
                first_word(line)
            )));
        }
        let rest = line.strip_prefix("metric ").unwrap().trim();
        let value = parse_quoted_string(rest)
            .ok_or_else(|| self.err("Expected quoted string after 'metric'"))?;
        self.advance();
        Ok(value)
    }

    fn parse_threshold(&mut self) -> Result<(String, String), Vec<ParseError>> {
        if self.at_end() {
            return Err(self.err("Expected 'threshold <operator> <value>'"));
        }
        let line = self.current_line().trim();
        if !line.starts_with("threshold ") {
            return Err(self.err(format!(
                "Expected 'threshold <operator> <value>', got '{}'",
                first_word(line)
            )));
        }
        let rest = line.strip_prefix("threshold ").unwrap().trim();

        // Parse operator (may be 1 or 2 chars)
        let (op, value) =
            if rest.starts_with("<=") || rest.starts_with(">=") || rest.starts_with("==") {
                (&rest[..2], rest[2..].trim())
            } else if rest.starts_with('<') || rest.starts_with('>') {
                (&rest[..1], rest[1..].trim())
            } else if rest.starts_with("!=") {
                return Err(self.err(format!(
                    "Invalid threshold operator '!=' — valid operators: {}",
                    VALID_THRESHOLD_OPS.join(", ")
                )));
            } else {
                let op_end = rest.find(' ').unwrap_or(rest.len());
                let bad_op = &rest[..op_end];
                return Err(self.err(format!(
                    "Invalid threshold operator '{}' — valid operators: {}",
                    bad_op,
                    VALID_THRESHOLD_OPS.join(", ")
                )));
            };

        if value.is_empty() {
            return Err(self.err("Expected a value after threshold operator"));
        }

        self.advance();
        Ok((op.to_string(), value.to_string()))
    }

    fn parse_metric_verification(&mut self) -> Result<MetricVerification, Vec<ParseError>> {
        if self.at_end() {
            return Err(self.err("Expected 'verification' block"));
        }
        let line = self.current_line().trim();
        if line != "verification" {
            return Err(self.err(format!(
                "Expected 'verification', got '{}'",
                first_word(line)
            )));
        }
        self.advance();

        let mut environments = Vec::new();
        let mut benchmarks = Vec::new();
        let mut datasets = Vec::new();
        let mut passes = Vec::new();

        while !self.at_end() {
            let line = self.current_line();
            if !line.starts_with("    ") {
                break;
            }
            let trimmed = line.trim();
            if trimmed.is_empty() {
                self.advance();
                continue;
            }
            if trimmed.starts_with("environment ") {
                let rest = trimmed.strip_prefix("environment ").unwrap().trim();
                environments = rest.split(',').map(|s| s.trim().to_string()).collect();
                self.advance();
            } else if trimmed.starts_with("benchmark ") {
                let rest = trimmed.strip_prefix("benchmark ").unwrap().trim();
                benchmarks.push(unquote(rest));
                self.advance();
            } else if trimmed.starts_with("dataset ") {
                let rest = trimmed.strip_prefix("dataset ").unwrap().trim();
                datasets.push(unquote(rest));
                self.advance();
            } else if trimmed.starts_with("pass ") {
                let rest = trimmed.strip_prefix("pass ").unwrap().trim();
                passes.push(unquote(rest));
                self.advance();
            } else {
                break;
            }
        }

        if environments.is_empty() {
            return Err(self.err("Metric verification requires 'environment'"));
        }
        if benchmarks.is_empty() {
            return Err(self.err("Metric verification requires at least one 'benchmark'"));
        }
        if passes.is_empty() {
            return Err(self.err("Metric verification requires at least one 'pass'"));
        }

        Ok(MetricVerification {
            environments,
            benchmarks,
            datasets,
            passes,
        })
    }
}

// ── Rule body ───────────────────────────────────────────

impl<'a> NfrParser<'a> {
    fn parse_rule_body(&mut self) -> Result<ConstraintBody, Vec<ParseError>> {
        let rule_text = self.parse_rule_text()?;
        self.skip_blank_lines();
        let verification = self.parse_rule_verification()?;

        Ok(ConstraintBody::Rule {
            rule_text,
            verification,
        })
    }

    fn parse_rule_text(&mut self) -> Result<String, Vec<ParseError>> {
        if self.at_end() {
            return Err(self.err("Expected 'rule' block"));
        }
        let line = self.current_line().trim();
        if line != "rule" {
            return Err(self.err(format!("Expected 'rule', got '{}'", first_word(line))));
        }
        self.advance();

        let mut lines = Vec::new();
        while !self.at_end() {
            let line = self.current_line();
            if line.starts_with("    ") {
                lines.push(line.trim());
                self.advance();
            } else {
                break;
            }
        }
        Ok(lines.join("\n"))
    }

    fn parse_rule_verification(&mut self) -> Result<RuleVerification, Vec<ParseError>> {
        if self.at_end() {
            return Err(self.err("Expected 'verification' block"));
        }
        let line = self.current_line().trim();
        if line != "verification" {
            return Err(self.err(format!(
                "Expected 'verification', got '{}'",
                first_word(line)
            )));
        }
        self.advance();

        let mut statics = Vec::new();
        let mut runtimes = Vec::new();

        while !self.at_end() {
            let line = self.current_line();
            if !line.starts_with("    ") {
                break;
            }
            let trimmed = line.trim();
            if trimmed.is_empty() {
                self.advance();
                continue;
            }
            if trimmed.starts_with("static ") {
                let rest = trimmed.strip_prefix("static ").unwrap().trim();
                statics.push(unquote(rest));
                self.advance();
            } else if trimmed.starts_with("runtime ") {
                let rest = trimmed.strip_prefix("runtime ").unwrap().trim();
                runtimes.push(unquote(rest));
                self.advance();
            } else {
                break;
            }
        }

        if statics.is_empty() && runtimes.is_empty() {
            return Err(
                self.err("Rule verification requires at least one 'static' or 'runtime' check")
            );
        }

        Ok(RuleVerification { statics, runtimes })
    }
}

// ── Shared fields ───────────────────────────────────────

impl<'a> NfrParser<'a> {
    fn parse_violation(&mut self) -> Result<String, Vec<ParseError>> {
        if self.at_end() {
            return Err(self.err("Expected 'violation <severity>'"));
        }
        let line = self.current_line().trim();
        if !line.starts_with("violation ") {
            return Err(self.err(format!(
                "Expected 'violation <severity>', got '{}'",
                first_word(line)
            )));
        }
        let severity = line.strip_prefix("violation ").unwrap().trim().to_string();
        if !VALID_SEVERITIES.contains(&severity.as_str()) {
            return Err(self.err(format!(
                "Invalid violation severity '{}' — valid values: {}",
                severity,
                VALID_SEVERITIES.join(", ")
            )));
        }
        self.advance();
        Ok(severity)
    }

    fn parse_overridable(&mut self) -> Result<bool, Vec<ParseError>> {
        if self.at_end() {
            return Err(self.err("Expected 'overridable <yes|no>'"));
        }
        let line = self.current_line().trim();
        if !line.starts_with("overridable ") {
            return Err(self.err(format!(
                "Expected 'overridable <yes|no>', got '{}'",
                first_word(line)
            )));
        }
        let value = line.strip_prefix("overridable ").unwrap().trim();
        match value {
            "yes" => {
                self.advance();
                Ok(true)
            }
            "no" => {
                self.advance();
                Ok(false)
            }
            other => Err(self.err(format!(
                "Invalid overridable value '{}' — expected yes or no",
                other
            ))),
        }
    }
}

#[cfg(test)]
#[path = "nfr.test.rs"]
mod tests;
