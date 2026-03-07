use crate::theme::Theme;
use tree_sitter::{Parser, Tree};

/// A region of the command line buffer to highlight
#[derive(Debug, Clone)]
pub struct HighlightRegion {
    /// Start byte offset
    pub start: usize,
    /// End byte offset
    pub end: usize,
    /// ANSI color code (e.g., "38;2;136;192;208" for 24-bit color)
    pub ansi_color: String,
}

impl HighlightRegion {
    /// Format as a Zsh region_highlight entry: "start end fg=color"
    pub fn to_zsh_region(&self) -> String {
        format!("{} {} fg=#{}", self.start, self.end, self.ansi_to_hex())
    }

    fn ansi_to_hex(&self) -> String {
        // Parse "38;2;R;G;B" back to hex for Zsh's region_highlight
        let parts: Vec<&str> = self.ansi_color.split(';').collect();
        if parts.len() == 5 {
            let r: u8 = parts[2].parse().unwrap_or(0);
            let g: u8 = parts[3].parse().unwrap_or(0);
            let b: u8 = parts[4].parse().unwrap_or(0);
            format!("{r:02x}{g:02x}{b:02x}")
        } else {
            "D8DEE9".into()
        }
    }
}

/// Main syntax highlighter
pub struct Highlighter {
    parser: Parser,
    theme: Theme,
}

impl Highlighter {
    pub fn new(theme: Theme) -> Self {
        let mut parser = Parser::new();
        let language = tree_sitter_bash::LANGUAGE;
        parser
            .set_language(&language.into())
            .expect("failed to set tree-sitter-bash language");
        Self { parser, theme }
    }

    /// Parse a command line and return highlight regions
    pub fn highlight(&mut self, input: &str) -> Vec<HighlightRegion> {
        let tree = match self.parser.parse(input, None) {
            Some(t) => t,
            None => return Vec::new(),
        };

        let mut regions = Vec::new();
        self.walk_tree(&tree, input, &mut regions);
        regions
    }

    fn walk_tree(&self, tree: &Tree, source: &str, regions: &mut Vec<HighlightRegion>) {
        let mut cursor = tree.walk();
        self.walk_node(&mut cursor, source, regions);
    }

    fn walk_node(
        &self,
        cursor: &mut tree_sitter::TreeCursor,
        source: &str,
        regions: &mut Vec<HighlightRegion>,
    ) {
        let node = cursor.node();
        let kind = node.kind();

        // Map tree-sitter-bash node kinds to our semantic roles
        let role = match kind {
            "command_name" => Some("command"),
            "word" if node.parent().map(|p| p.kind()) == Some("command_name") => Some("command"),
            "string" | "raw_string" | "ansi_c_string" => Some("string"),
            "string_expansion" | "command_substitution" | "process_substitution" => {
                Some("string_expansion")
            }
            "expansion" | "simple_expansion" => Some("variable"),
            "variable_name" => Some("variable_name"),
            "number" => Some("number"),
            "comment" => Some("comment"),
            "file_redirect" | "heredoc_redirect" | "herestring_redirect" => Some("redirection"),
            "|" | "||" | "&&" | "pipeline" => Some("pipe"),
            ";" | ";;" => Some("semicolon"),
            "(" | ")" | "{" | "}" | "[" | "]" | "[[" | "]]" => Some("bracket"),
            "if" | "then" | "else" | "elif" | "fi" | "for" | "while" | "do" | "done" | "case"
            | "esac" | "in" | "function" | "select" | "until" => Some("keyword"),
            ">" | "<" | ">>" | ">&" | "<&" | ">|" => Some("redirection"),
            "ERROR" => Some("error"),
            _ => None,
        };

        // Only add highlight if this is a leaf node or we have a specific role
        if let Some(role) = role {
            if node.child_count() == 0 || role == "error" || role == "comment" || role == "string" {
                let start = node.start_byte();
                let end = node.end_byte();
                if start < source.len() && end <= source.len() && start < end {
                    regions.push(HighlightRegion {
                        start,
                        end,
                        ansi_color: self.theme.color_for(role),
                    });
                }
            }
        }

        // Recurse into children
        if cursor.goto_first_child() {
            loop {
                self.walk_node(cursor, source, regions);
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
            cursor.goto_parent();
        }
    }

    /// Generate the full region_highlight array content for Zsh
    pub fn highlight_for_zsh(&mut self, input: &str) -> String {
        let regions = self.highlight(input);
        regions
            .iter()
            .map(|r| r.to_zsh_region())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_command() {
        let theme = Theme::default();
        let mut hl = Highlighter::new(theme);
        let regions = hl.highlight("echo hello");
        assert!(!regions.is_empty());
    }

    #[test]
    fn test_pipe() {
        let theme = Theme::default();
        let mut hl = Highlighter::new(theme);
        let regions = hl.highlight("cat foo | grep bar");
        assert!(!regions.is_empty());
    }

    #[test]
    fn test_string() {
        let theme = Theme::default();
        let mut hl = Highlighter::new(theme);
        let regions = hl.highlight("echo \"hello world\"");
        let has_string = regions.iter().any(|r| {
            r.ansi_color == Theme::hex_to_ansi_fg("#A3BE8C")
        });
        assert!(has_string);
    }

    #[test]
    fn test_comment() {
        let theme = Theme::default();
        let mut hl = Highlighter::new(theme);
        let regions = hl.highlight("# this is a comment");
        let has_comment = regions.iter().any(|r| {
            r.ansi_color == Theme::hex_to_ansi_fg("#4C566A")
        });
        assert!(has_comment);
    }

    #[test]
    fn test_empty_input() {
        let theme = Theme::default();
        let mut hl = Highlighter::new(theme);
        let regions = hl.highlight("");
        assert!(regions.is_empty());
    }
}
