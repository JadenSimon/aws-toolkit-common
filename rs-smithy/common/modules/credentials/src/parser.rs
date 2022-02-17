// Incremental parser for AWS 'ini' files

use std::borrow::BorrowMut;
use std::rc::Rc;
use std::cell::{RefCell, RefMut};
use regex::Regex;

#[derive(Debug)]
pub struct Position {
    line: usize,
    char: usize,
}

#[derive(Debug)]
pub struct Range {
    start: Position,
    end: Position,
}

#[derive(Debug)]
pub struct Section {
    pub name: String,
    pub position: Position,
    pub assignments: Vec<Assignment>,
}

#[derive(Debug)]
pub struct Assignment {
    key: String,
    value: String,
    position: Position,
}

// TODO: add literal
// TODO: move all into enum?

#[derive(Debug)]
pub enum ConfigNode {
    Section(Section),
    Assignment(Assignment),
    BadValue {
        message: String,
        range: Range,
    },
}

pub struct Parser {
    source: String,
}

impl Parser {
    pub fn new(source: String) -> Self {
        Parser { source }
    }

    fn parse_line(line: &str, index: usize) -> ConfigNode {
        let section = Regex::new(r"^\s*\[").unwrap();
        // wouldn't be able to invalidate bad keys like this
        let property = Regex::new(r"^\s*([A-Za-z0-9_-]+)=").unwrap();

        let is_section_start = section.is_match(line);
        let is_property_start = property.is_match(line);

        if is_section_start {
            // we will be pretty lenient with profile names
            let matcher = Regex::new(r"^\s*\[(profile\s*)?([^\[\]\s=]+)]\s*$").unwrap();
            let captures = matcher.captures(line).unwrap();

            return match captures.get(2) {
                Some(matched) => ConfigNode::Section(Section {
                    name: matched.as_str().to_string(),
                    position: Position { char: matched.start(), line: index },
                    assignments: Vec::new(),
                }),
                None => ConfigNode::BadValue {
                    message: String::from("Invalid profile name"),
                    range: Range { 
                        start: Position { char: 0, line: index },
                        end: Position { char: line.len(), line: index },
                    },
                },
            };
        }

        if is_property_start {
            let matcher = Regex::new(r"^\s*(.+?)\s*=\s*(.+?)\s*$").unwrap();
            let captures = matcher.captures(line).unwrap();

            return match (captures.get(1), captures.get(2)) {
                (Some(key), Some(value)) => ConfigNode::Assignment(Assignment {
                    key: key.as_str().to_string(),
                    value: value.as_str().to_string(),
                    position: Position { char: key.start(), line: index },
                }),
                (Some(key), None) => ConfigNode::BadValue {
                    message: String::from("Invalid value"),
                    range: Range {
                        start: Position { char: key.start(), line: index },
                        end: Position { char: key.end(), line: index },
                    },
                },
                _ => ConfigNode::BadValue {
                    message: String::from("Invalid assignment"),
                    range: Range {
                        start: Position { char: 0, line: index },
                        end: Position { char: line.len(), line: index },
                    },
                },
            };
        }

        ConfigNode::BadValue {
            message: String::from("Invalid assignment"),
            range: Range {
                start: Position { char: 0, line: index },
                end: Position { char: line.len(), line: index },
            },
        }
    }

    pub fn parse_file(&self) -> Vec<ConfigNode> {
        let mut nodes = Vec::<ConfigNode>::new();
        let mut scope: usize = 0;
        let comment_re = Regex::new(r"(^|\s)[;#]").unwrap();

        for (index, line) in self.source.lines().enumerate() {
            let filtered = comment_re.splitn(line, 1).next().unwrap_or("");
            if filtered.trim().len() == 0 {
                continue
            }

            let node = Self::parse_line(filtered, index);
            
            match node {
                ConfigNode::Assignment(assignment) => {
                    if let Some(ConfigNode::Section(section)) = nodes.get_mut(scope) {
                        section.assignments.push(assignment);
                    } else {
                        nodes.push(ConfigNode::BadValue {
                            message: String::from("Assignment without a section"),
                            range: Range {
                                start: Position { char: 0, line: index },
                                end: Position { char: line.len(), line: index },
                            },
                        });
                    }
                },
                ConfigNode::Section(_) => {
                    nodes.push(node);
                    scope = nodes.len() - 1;
                },
                _ => nodes.push(node),
            }
        }

        nodes
    }
}

// TODO:
// This is technically valid ???:
//
// [profile example]
// s3 =
//    max_concurrent_requests=10
//    max_queue_size=1000

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let example = "
[foo]
baz=qaz
        ".trim();

        let parser = Parser { source: String::from(example) };
        let result = parser.parse_file();

        assert_eq!(result.len(), 1);
        assert!(matches!(result[0], ConfigNode::Section {..}));
    }

    #[test]
    fn bad_assignment() {
        let example = "
[foo]
baz
        ".trim();

        let parser = Parser { source: String::from(example) };
        let result = parser.parse_file();

        assert_eq!(result.len(), 2);
        assert!(matches!(result[1], ConfigNode::BadValue {..}));
    }

    #[test]
    fn multiple_profiles() {
        let example = "
[foo]
baz=qaz

[oof]
qaz=baz
abc=efg
        ".trim();

        let parser = Parser { source: String::from(example) };
        let result = parser.parse_file();

        assert_eq!(result.len(), 2);
        assert!(matches!(result[0], ConfigNode::Section {..}));
        assert!(matches!(result[1], ConfigNode::Section {..}));
    }

    #[test]
    fn config_profile() {
        let example = "
[profile foo-bar]
baz=qaz
        ".trim();

        let parser = Parser { source: String::from(example) };
        let result = parser.parse_file();

        assert_eq!(result.len(), 1);
        if let ConfigNode::Section(section) = &result[0] {
            assert_eq!(section.name, "foo-bar");
        } else {
            panic!();
        }
    }
}