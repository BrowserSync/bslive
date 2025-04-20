use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct ArchyNode {
    pub label: String,
    pub nodes: Vec<ArchyNode>,
}

impl ArchyNode {
    pub fn new(label: &str) -> Self {
        ArchyNode {
            label: label.to_string(),
            nodes: Vec::new(),
        }
    }

    pub fn from_string(s: &str) -> Self {
        ArchyNode {
            label: s.to_string(),
            nodes: Vec::new(),
        }
    }
}

#[derive(Debug)]
enum Token {
    Gap,
    ColumnStartNl,
    AfterLabelNl,
    Vert,
    Hori,
    RightTurn,
    RightTurnContinueDown,
    T,
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let t = match self {
            Token::Gap => ' ',
            Token::ColumnStartNl => '\n',
            Token::AfterLabelNl => '\n',
            Token::Vert => '│',
            Token::Hori => '─',
            Token::RightTurn => '└',
            Token::RightTurnContinueDown => '├',
            Token::T => '┬',
        };
        write!(f, "{}", t)
    }
}

pub fn archy(obj: &ArchyNode, line_prefix: Option<&str>) -> String {
    type T = Token;

    let line_prefix = line_prefix.unwrap_or("");

    let this_row_prefix = if obj.nodes.is_empty() {
        T::Gap
    } else {
        T::Vert
    };
    let label_lines_prefix = format!(
        "{}{}{}{}",
        T::ColumnStartNl,
        line_prefix,
        this_row_prefix,
        T::Gap
    );

    let label_lines = obj.label.split('\n').collect::<Vec<_>>();
    let label_lines_joined = label_lines.join(&label_lines_prefix);

    let last_index = obj.nodes.len().checked_sub(1).unwrap_or(0);
    let each_node = obj
        .nodes
        .iter()
        .enumerate()
        .map(|(ix, node)| {
            let last = ix == last_index;
            let more = !node.nodes.is_empty();
            let inner_line_prefix = if last { T::Gap } else { T::Vert };
            let inner_prefix = format!("{}{}{}", line_prefix, inner_line_prefix, T::Gap);
            let connector = format!(
                "{}{}",
                if last {
                    T::RightTurn
                } else {
                    T::RightTurnContinueDown
                },
                T::Hori
            );
            let branch = format!("{}{}", if more { T::T } else { T::Hori }, T::Gap);
            let subtree = archy(node, Some(&inner_prefix));
            let line_prefix_len = line_prefix.char_indices().count();
            let subtree_sliced = subtree
                .char_indices()
                .skip(line_prefix_len + 2)
                .map(|(_, c)| String::from(c))
                .collect::<Vec<_>>()
                .join("");

            format!("{}{}{}{}", line_prefix, connector, branch, subtree_sliced)
        })
        .collect::<Vec<_>>()
        .join("");

    format!(
        "{}{}{}{}",
        line_prefix,
        label_lines_joined,
        T::AfterLabelNl,
        each_node
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_archy_single_node() {
        let node = ArchyNode::new("Root");
        let result = archy(&node, None);
        let expected = String::from("Root\n");
        assert_eq!(result, expected);
    }

    #[test]
    fn test_archy_with_children() {
        let mut root = ArchyNode::new("Root");
        root.nodes.push(ArchyNode::new("Child 1"));
        root.nodes.push(ArchyNode::new("Child 2"));

        let result = archy(&root, None);
        let expected = "\
Root
├── Child 1
└── Child 2
";
        assert_eq!(result, expected);
    }

    #[test]
    fn test_archy_multi_level_structure() {
        let mut root = ArchyNode::new("Root");
        let mut child1 = ArchyNode::new("Child 1");
        child1.nodes.push(ArchyNode::new("Grandchild 1.1"));
        child1.nodes.push(ArchyNode::new("Grandchild 1.2"));
        root.nodes.push(child1);
        root.nodes.push(ArchyNode::new("Child 2"));

        let result = archy(&root, None);
        let expected = "\
Root
├─┬ Child 1
│ ├── Grandchild 1.1
│ └── Grandchild 1.2
└── Child 2
";
        assert_eq!(result, expected);
    }
}
