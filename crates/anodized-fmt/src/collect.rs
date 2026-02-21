use crop::Rope;
use syn::{
    Attribute, File,
    spanned::Spanned,
    visit::{self, Visit},
};

/// Information about a #[spec] attribute found in the source code.
#[derive(Debug)]
pub struct SpecAttr<'a> {
    /// The attribute itself
    pub attr: &'a Attribute,
    /// The indentation before the attribute
    pub base_indent: ParentIndent,
}

/// Tracks the indentation (tabs and spaces) before an attribute.
#[derive(Debug, Default, Clone)]
pub struct ParentIndent {
    pub tabs: usize,
    pub spaces: usize,
}

impl ParentIndent {
    /// Get the total indentation in spaces (assuming tab_width spaces per tab).
    pub fn total_spaces(&self, tab_width: usize) -> usize {
        self.tabs * tab_width + self.spaces
    }
}

/// Visitor that collects all #[spec(...)] attributes in a file.
struct SpecAttrVisitor<'a, 'ast> {
    attrs: Vec<SpecAttr<'ast>>,
    source: &'a Rope,
}

impl<'a, 'ast> SpecAttrVisitor<'a, 'ast> {
    fn new(source: &'a Rope) -> Self {
        Self {
            attrs: Vec::new(),
            source,
        }
    }

    /// Check if an attribute is a #[spec(...)] attribute and collect it.
    fn collect_if_spec(&mut self, attr: &'ast Attribute) {
        // Check if this is a #[spec(...)] attribute
        if !attr.path().is_ident("spec") {
            return;
        }

        let span_line = attr.span().start().line;
        if span_line == 0 {
            return; // Invalid span
        }

        let line = self.source.line(span_line - 1);

        // Calculate indentation by counting leading whitespace
        let indent_chars: Vec<_> = line
            .chars()
            .take_while(|&c| c == ' ' || c == '\t')
            .collect();

        let tabs = indent_chars.iter().filter(|&&c| c == '\t').count();
        let spaces = indent_chars.iter().filter(|&&c| c == ' ').count();

        self.attrs.push(SpecAttr {
            attr,
            base_indent: ParentIndent { tabs, spaces },
        });
    }
}

impl<'a, 'ast> Visit<'ast> for SpecAttrVisitor<'a, 'ast> {
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        // Check attributes on functions
        for attr in &node.attrs {
            self.collect_if_spec(attr);
        }
        visit::visit_item_fn(self, node);
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        // Check attributes on impl methods
        for attr in &node.attrs {
            self.collect_if_spec(attr);
        }
        visit::visit_impl_item_fn(self, node);
    }

    fn visit_trait_item_fn(&mut self, node: &'ast syn::TraitItemFn) {
        // Check attributes on trait methods
        for attr in &node.attrs {
            self.collect_if_spec(attr);
        }
        visit::visit_trait_item_fn(self, node);
    }

    fn visit_foreign_item_fn(&mut self, node: &'ast syn::ForeignItemFn) {
        // Check attributes on foreign functions
        for attr in &node.attrs {
            self.collect_if_spec(attr);
        }
        visit::visit_foreign_item_fn(self, node);
    }
}

/// Collect all #[spec(...)] attributes from a parsed Rust file.
///
/// This uses syn's visitor pattern to traverse the AST and find all
/// spec attributes, recording their indentation from the source.
pub fn collect_spec_attrs_in_file<'a, 'ast>(
    file: &'ast File,
    source: &'a Rope,
) -> Vec<SpecAttr<'ast>> {
    let mut visitor = SpecAttrVisitor::new(source);
    visitor.visit_file(file);
    visitor.attrs
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_file;

    #[test]
    fn test_collect_simple_spec() {
        let source = r#"
use anodized::spec;

#[spec(requires: x > 0)]
fn foo(x: i32) -> i32 {
    x + 1
}
"#;
        let rope = Rope::from(source);
        let ast = parse_file(source).unwrap();
        let specs = collect_spec_attrs_in_file(&ast, &rope);

        assert_eq!(specs.len(), 1);
        assert_eq!(specs[0].base_indent.spaces, 0);
        assert_eq!(specs[0].base_indent.tabs, 0);
    }

    #[test]
    fn test_collect_indented_spec() {
        let source = r#"
impl MyStruct {
    #[spec(requires: x > 0)]
    fn foo(x: i32) -> i32 {
        x + 1
    }
}
"#;
        let rope = Rope::from(source);
        let ast = parse_file(source).unwrap();
        let specs = collect_spec_attrs_in_file(&ast, &rope);

        assert_eq!(specs.len(), 1);
        // Should detect 4 spaces of indentation
        assert_eq!(specs[0].base_indent.spaces, 4);
    }

    #[test]
    fn test_collect_multiple_specs() {
        let source = r#"
#[spec(requires: x > 0)]
fn foo(x: i32) -> i32 {
    x + 1
}

#[spec(requires: y > 0)]
fn bar(y: i32) -> i32 {
    y + 2
}
"#;
        let rope = Rope::from(source);
        let ast = parse_file(source).unwrap();
        let specs = collect_spec_attrs_in_file(&ast, &rope);

        assert_eq!(specs.len(), 2);
    }

    #[test]
    fn test_ignore_non_spec_attributes() {
        let source = r#"
#[derive(Debug)]
#[spec(requires: x > 0)]
fn foo(x: i32) -> i32 {
    x + 1
}
"#;
        let rope = Rope::from(source);
        let ast = parse_file(source).unwrap();
        let specs = collect_spec_attrs_in_file(&ast, &rope);

        // Should only find the spec attribute, not derive
        assert_eq!(specs.len(), 1);
    }

    #[test]
    fn test_trait_methods() {
        let source = r#"
trait MyTrait {
    #[spec(requires: x > 0)]
    fn foo(&self, x: i32) -> i32;
}
"#;
        let rope = Rope::from(source);
        let ast = parse_file(source).unwrap();
        let specs = collect_spec_attrs_in_file(&ast, &rope);

        assert_eq!(specs.len(), 1);
    }
}
