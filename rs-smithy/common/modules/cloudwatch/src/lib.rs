use tree_sitter::Language;

pub mod model;
pub mod query;
pub mod viewer;

// I guess we're going to do a language server for CWL Insights?
// A proper implementation will need a good model of CWL in general
// Which could be re-used pretty easily for UI


extern "C" {
    fn tree_sitter_cloudwatch_insights() -> Language;
}

/// Get the tree-sitter [Language][] for this grammar.
///
/// [Language]: https://docs.rs/tree-sitter/*/tree_sitter/struct.Language.html
pub fn language() -> Language {
    unsafe { tree_sitter_cloudwatch_insights() }
}

pub fn parser() -> tree_sitter::Parser {
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(language()).unwrap();
    parser
}