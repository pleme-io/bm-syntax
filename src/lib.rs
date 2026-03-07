//! bm-syntax: Tree-sitter powered Zsh syntax highlighting
//!
//! This library provides two integration modes:
//! 1. Native Zsh module (.dylib/.so) loaded via zmodload — highest performance
//! 2. External process called from ZLE widget — simpler, more portable
//!
//! Architecture:
//! - Parses the command line buffer using tree-sitter-bash (closest grammar to Zsh)
//! - Maps AST node types to colors from a theme configuration
//! - Returns region_highlight entries that Zsh applies to the command line
//!
//! The native module approach registers a zle-line-pre-redraw hook that:
//! 1. Reads BUFFER (the current command line text)
//! 2. Parses it with tree-sitter
//! 3. Walks the AST and maps node kinds to ANSI color codes
//! 4. Sets region_highlight with the colored regions

pub mod highlighter;
pub mod theme;

pub use highlighter::Highlighter;
pub use theme::Theme;
