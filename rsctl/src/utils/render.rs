use anyhow::{Result, anyhow};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Value {
    Str(String),
    Bool(bool),
}

impl Value {
    fn truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Str(s) => !s.is_empty(),
        }
    }

    fn as_str(&self) -> String {
        match self {
            Value::Bool(b) => {
                if *b {
                    "true".into()
                } else {
                    "false".into()
                }
            }
            Value::Str(s) => s.clone(),
        }
    }
}

/// Minimal Go-template-like context.
/// Keys are stored case-insensitively (lowercased).
#[derive(Debug, Clone, Default)]
pub struct Context {
    map: HashMap<String, Value>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn set_str(mut self, key: &str, val: impl Into<String>) -> Self {
        self.map
            .insert(key.to_ascii_lowercase(), Value::Str(val.into()));
        self
    }

    pub fn set_bool(mut self, key: &str, val: bool) -> Self {
        self.map.insert(key.to_ascii_lowercase(), Value::Bool(val));
        self
    }

    fn get(&self, key: &str) -> Option<&Value> {
        self.map.get(&key.to_ascii_lowercase())
    }
}

/// Render template string supporting:
/// - `{{.Var}}`
/// - `{{if .Cond}} ... {{else}} ... {{end}}`
///
/// Notes:
/// - Unknown vars render as empty.
/// - Conditions treat non-empty strings as true.
pub fn render(input: &str, ctx: &Context) -> Result<String> {
    let mut out = String::with_capacity(input.len());
    let mut i = 0usize;

    // stack of (is_current_branch_active, has_else_been_seen, parent_active)
    let mut stack: Vec<(bool, bool, bool)> = Vec::new();
    let mut active = true;

    while i < input.len() {
        // Find next action start.
        let Some(rel_start) = input[i..].find("{{") else {
            if active {
                out.push_str(&input[i..]);
            }
            break;
        };
        let start = i + rel_start;
        if active {
            out.push_str(&input[i..start]);
        }

        // Find action end.
        let after_open = start + 2;
        let Some(rel_end) = input[after_open..].find("}}") else {
            return Err(anyhow!("unterminated template action"));
        };
        let end = after_open + rel_end;
        let action = input[after_open..end].trim();

        // Advance .cursor past "}}"
        i = end + 2;

        // Control actions
        if let Some(rest) = action.strip_prefix("if ") {
            let cond = rest.trim();
            let key = cond.strip_prefix('.').unwrap_or(cond).trim();
            let cond_val = ctx.get(key).map(|v| v.truthy()).unwrap_or(false);
            let parent_active = active;
            let branch_active = parent_active && cond_val;
            stack.push((branch_active, false, parent_active));
            active = branch_active;
            continue;
        }
        if action == "else" {
            let Some((branch_active, has_else, parent_active)) = stack.pop() else {
                return Err(anyhow!("unexpected {{else}}"));
            };
            if has_else {
                return Err(anyhow!("duplicate {{else}}"));
            }
            let new_branch_active = parent_active && !branch_active;
            stack.push((new_branch_active, true, parent_active));
            active = new_branch_active;
            continue;
        }
        if action == "end" {
            let Some((_branch_active, _has_else, parent_active)) = stack.pop() else {
                return Err(anyhow!("unexpected {{end}}"));
            };
            active = parent_active;
            continue;
        }

        // Variable substitution
        if !active {
            continue;
        }
        let key = action.strip_prefix('.').unwrap_or(action).trim();
        if key.is_empty() {
            continue;
        }
        if let Some(v) = ctx.get(key) {
            out.push_str(&v.as_str());
        }
    }

    if !stack.is_empty() {
        return Err(anyhow!("unclosed {{if}} block"));
    }

    Ok(out)
}
