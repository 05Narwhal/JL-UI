// * ---- JSONC Utils ----
use std::{error::Error, fs, io::{Error as IoError, ErrorKind}, path::PathBuf, sync::{OnceLock, RwLock}};

use serde::{Serialize, de::DeserializeOwned};

static GLOBAL_JSONC: OnceLock<RwLock<Jsonc>> = OnceLock::new();

fn get_global_jsonc() -> &'static RwLock<Jsonc> {
    GLOBAL_JSONC.get_or_init(|| RwLock::new(Jsonc::init()))
}

#[derive(Clone, Debug)]
pub enum CommentAnchor {
    Left(String),
    Below(String),
    Above(String),
}

#[derive(Clone, Debug)]
pub struct JsoncComment {
    pub line: usize,
    pub comment_content: String,
    pub anchor: CommentAnchor,
}

pub struct Jsonc {
    comments: std::collections::HashMap<String, Vec<JsoncComment>>,
}

impl Jsonc {
    pub fn init() -> Self {
        Self {
            comments: std::collections::HashMap::new(),
        }
    }

    /// Initializes the Jsonc system, preparing it to store comments for JSONC files.
    fn extract_comments(path: &PathBuf, text: &str) -> String {
        let mut out = String::with_capacity(text.len());
        let lines: Vec<&str> = text.lines().collect();

        let mut jsonc = get_global_jsonc().write().unwrap();
        let entry = jsonc.comments.entry(path.to_string_lossy().to_string()).or_default();

        for i in 0..lines.len() {
            let line = lines[i];

            if let Some(comment_start) = line.find("//") {
                let (left, comment) = line.split_at(comment_start);
                let comment = comment.trim_start_matches("//").trim().to_string();

                let left_trimmed = left.trim();
                let above = if i > 0 { Some(lines[i - 1].trim().to_string()) } else { None };
                let below = if i + 1 < lines.len() { Some(lines[i + 1].trim().to_string()) } else { None };

                // Determine anchor priority
                let anchor = if !left_trimmed.is_empty() {
                    CommentAnchor::Left(left_trimmed.to_string())
                } else if let Some(b) = &below {
                    if !b.is_empty() {
                        CommentAnchor::Below(b.clone())
                    } else if let Some(a) = &above {
                        CommentAnchor::Above(a.clone())
                    } else {
                        continue;
                    }
                } else if let Some(a) = &above {
                    CommentAnchor::Above(a.clone())
                } else {
                    continue;
                };

                entry.push(JsoncComment {
                    line: i,
                    comment_content: comment,
                    anchor,
                });

                // Remove comment from output
                out.push_str(left);
                out.push('\n');
            } else {
                out.push_str(line);
                out.push('\n');
            }
        }

        out
    }

    /// Reads JSONC from a file, returning the deserialized data or an error if parsing fails.
    pub fn read_jsonc<T: DeserializeOwned>(path: &PathBuf) -> Result<T, Box<dyn Error>> {
        if let Ok(json_str) = fs::read_to_string(path) {
            // let cleaned = Self::extract_comments(path, &json_str); TODO: Still in development
            let cleaned = Self::strip_jsonc_comments(&json_str);
            match serde_json::from_str::<T>(&cleaned) {
                Ok(data) => Ok(data),
                Err(e) => {
                    crate::ui_error!("Failed to parse JSONC from {}: {}", path.display(), e);
                    Err(Box::new(e))
                }
            }
        } else {
            crate::ui_error!("Failed to read file: {}", path.display());
            Err(Box::new(IoError::new(ErrorKind::NotFound, "File not found")))
        }
    }

    /// Writes JSON while attempting to preserve existing comments in the file. (still in developement) DO NOT USE YET.
    fn write_jsonc<T: Serialize>(path: &PathBuf, data: &T) -> bool {
        let json_str = match serde_json::to_string_pretty(data) {
            Ok(s) => s,
            Err(e) => {
                crate::ui_error!("Serialize error: {}", e);
                return false;
            }
        };

        let mut lines: Vec<String> = json_str.lines().map(|l| l.to_string()).collect();

        let jsonc = get_global_jsonc().read().unwrap();
        let comments = match jsonc.comments.get(&path.to_string_lossy().to_string()) {
            Some(c) => c,
            None => &Vec::new(),
        };

        for comment in comments {
            match &comment.anchor {
                CommentAnchor::Left(content) => {
                    for line in &mut lines {
                        if line.contains(content) {
                            *line = format!("{} // {}", line, comment.comment_content);
                            break;
                        }
                    }
                }
                CommentAnchor::Below(content) => {
                    for i in 0..lines.len() {
                        if lines[i].contains(content) {
                            lines.insert(i, format!("// {}", comment.comment_content));
                            break;
                        }
                    }
                }
                CommentAnchor::Above(content) => {
                    for i in 0..lines.len() {
                        if lines[i].contains(content) {
                            lines.insert(i, format!("// {}", comment.comment_content));
                            break;
                        }
                    }
                }
            }
        }

        let final_output = lines.join("\n");

        if let Err(e) = fs::write(path, final_output) {
            crate::ui_error!("Write error: {}", e);
            false
        } else {
            true
        }
    }

    /// Strips all comments from a JSONC string, returning clean JSON text.
    pub fn strip_jsonc_comments(text: &str) -> String {
        let mut out = String::with_capacity(text.len());
        let mut in_string = false;
        let mut escaped = false;
        let mut chars = text.chars().peekable();
        while let Some(c) = chars.next() {
            if escaped {
                escaped = false;
                out.push(c);
                continue;
            }
            if in_string {
                if c == '\\' {
                    escaped = true;
                    out.push(c);
                    continue;
                }
                if c == '"' {
                    in_string = false;
                }
                out.push(c);
                continue;
            }
            if c == '"' {
                in_string = true;
                out.push(c);
                continue;
            }
            if c == '/' && chars.peek() == Some(&'/') {
                for nc in chars.by_ref() {
                    if nc == '\n' {
                        out.push('\n');
                        break;
                    }
                }
                continue;
            }
            out.push(c);
        }
        out
    }
}