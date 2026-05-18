use tree_sitter::{Language, Parser, Node};
use tree_sitter_python::language as python_lang;
use tree_sitter_rust::language as rust_lang;
use tree_sitter_go::language as go_lang;
use tree_sitter_javascript::language as js_lang;
use tree_sitter_java::language as java_lang;
use tree_sitter_c::language as c_lang;
use tree_sitter_cpp::language as cpp_lang;
use std::path::Path;

pub fn get_language_for_file(path: &Path) -> Option<Language> {
    match path.extension()?.to_str()? {
        "py" => Some(python_lang()),
        "rs" => Some(rust_lang()),
        "go" => Some(go_lang()),
        "js" | "mjs" | "cjs" => Some(js_lang()),
        "java" => Some(java_lang()),
        "c" => Some(c_lang()),
        "cpp" | "cc" | "cxx" => Some(cpp_lang()),
        _ => None,
    }
}

pub fn extract_symbols(content: &str, lang: Language) -> Vec<(String, String, usize)> {
    let mut parser = Parser::new();
    parser.set_language(lang).unwrap();
    let tree = parser.parse(content, None).unwrap();
    let root = tree.root_node();

    let mut symbols = Vec::new();
    find_symbols(root, content, &mut symbols);
    symbols
}

fn find_symbols(node: Node, source: &str, symbols: &mut Vec<(String, String, usize)>) {
    match node.kind() {
        "function_definition" | "function_declaration"
        | "method_definition" | "method_declaration"
        | "class_definition" | "struct_definition"
        | "impl_item" => {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "identifier" || child.kind() == "type_identifier" {
                    if let Ok(name) = child.utf8_text(source.as_bytes()) {
                        let typ = match node.kind() {
                            "function_definition" | "function_declaration" => "function",
                            "method_definition" | "method_declaration" => "method",
                            "class_definition" => "class",
                            "struct_definition" => "struct",
                            "impl_item" => "impl",
                            _ => "symbol",
                        };
                        let line = child.start_position().row + 1;
                        symbols.push((name.to_string(), typ.to_string(), line));
                    }
                    break;
                }
            }
        }
        "function_item" => {
            for child in node.children(&mut node.walk()) {
                if child.kind() == "identifier" {
                    if let Ok(name) = child.utf8_text(source.as_bytes()) {
                        let line = child.start_position().row + 1;
                        symbols.push((name.to_string(), "function".to_string(), line));
                    }
                    break;
                }
            }
        }
        _ => {}
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        find_symbols(child, source, symbols);
    }
}