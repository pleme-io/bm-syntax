use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Theme configuration mapping tree-sitter node kinds to colors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    /// Map of semantic role to hex color
    pub highlights: HashMap<String, String>,
}

impl Default for Theme {
    fn default() -> Self {
        let mut highlights = HashMap::new();
        // Nord-themed syntax highlighting
        highlights.insert("command".into(), "#88C0D0".into()); // frost blue
        highlights.insert("builtin".into(), "#81A1C1".into()); // frost medium
        highlights.insert("keyword".into(), "#81A1C1".into()); // frost medium
        highlights.insert("string".into(), "#A3BE8C".into()); // green
        highlights.insert("string_expansion".into(), "#EBCB8B".into()); // yellow
        highlights.insert("number".into(), "#B48EAD".into()); // purple
        highlights.insert("variable".into(), "#D8DEE9".into()); // snow
        highlights.insert("variable_name".into(), "#D8DEE9".into()); // snow
        highlights.insert("operator".into(), "#81A1C1".into()); // frost
        highlights.insert("redirection".into(), "#EBCB8B".into()); // yellow
        highlights.insert("pipe".into(), "#81A1C1".into()); // frost
        highlights.insert("argument".into(), "#D8DEE9".into()); // snow
        highlights.insert("flag".into(), "#8FBCBB".into()); // frost light
        highlights.insert("path".into(), "#88C0D0".into()); // frost blue
        highlights.insert("path_directory".into(), "#5E81AC".into()); // frost dark
        highlights.insert("glob".into(), "#EBCB8B".into()); // yellow
        highlights.insert("comment".into(), "#4C566A".into()); // gray
        highlights.insert("error".into(), "#BF616A".into()); // red
        highlights.insert("bracket".into(), "#88C0D0".into()); // frost blue
        highlights.insert("semicolon".into(), "#81A1C1".into()); // frost
        highlights.insert("default".into(), "#D8DEE9".into()); // snow

        Self {
            name: "nord".into(),
            highlights,
        }
    }
}

/// Error loading a theme file.
#[derive(Debug, thiserror::Error)]
pub enum ThemeError {
    #[error("read {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("parse {path}: {source}")]
    Parse {
        path: String,
        #[source]
        source: serde_yaml::Error,
    },
}

impl Theme {
    /// Load a theme YAML file.
    ///
    /// This is a simple single-file read — no env-var layering, no
    /// defaults merging. The previous figment wrapper was overkill for
    /// that; `serde_yaml::from_str` after a raw file read keeps the
    /// dep graph small.
    pub fn load(path: &Path) -> Result<Self, ThemeError> {
        let content = std::fs::read_to_string(path).map_err(|source| ThemeError::Io {
            path: path.display().to_string(),
            source,
        })?;
        serde_yaml::from_str(&content).map_err(|source| ThemeError::Parse {
            path: path.display().to_string(),
            source,
        })
    }

    /// Convert a hex color like "#88C0D0" to ANSI 24-bit escape sequence
    pub fn hex_to_ansi_fg(hex: &str) -> String {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return String::new();
        }
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
        format!("38;2;{r};{g};{b}")
    }

    /// Get the ANSI color code for a semantic role
    pub fn color_for(&self, role: &str) -> String {
        let hex = self
            .highlights
            .get(role)
            .or_else(|| self.highlights.get("default"))
            .map(|s| s.as_str())
            .unwrap_or("#D8DEE9");
        Self::hex_to_ansi_fg(hex)
    }
}
