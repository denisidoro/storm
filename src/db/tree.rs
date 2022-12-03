use anyhow::{Context, Result};
use std::collections::HashSet;
use std::fmt::{self, Display};
use std::path::{Path, PathBuf};

type T = String;
pub type TreeIndex = usize;

#[derive(Debug, Clone)]
pub(super) struct TreeNode {
    value: String,
    children: Vec<TreeIndex>,
    parent: Option<TreeIndex>,
}

impl TreeNode {
    fn new(value: T) -> Self {
        Self {
            value,
            children: vec![],
            parent: None,
        }
    }
}

#[derive(Debug)]
pub struct Tree {
    pub(super) arena: Vec<TreeNode>,
}

impl Display for Tree {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        self.fmt_helper(fmt, 0, &mut HashSet::new(), 0)
    }
}

impl Tree {
    pub fn new() -> Self {
        let root_node = TreeNode::new("/".into());
        Self {
            arena: vec![root_node],
        }
    }

    pub fn from_lines(lines: &mut dyn Iterator<Item = String>) -> Result<Self> {
        let mut t = Self::new();

        let mut depth: u32 = 0;
        let mut parts: Vec<T> = vec![];

        for line in lines {
            if line.starts_with('=') {
                break;
            }

            if line.trim().is_empty() {
                continue;
            }

            let newdepth = number_tabs(&line) + 1;
            if depth >= newdepth {
                let n = depth - newdepth;
                for _ in 0..=n {
                    parts.pop();
                }
            }

            parts.push(line.trim().into());

            let path = PathBuf::from_iter(parts.clone());
            t.add_path(&path)?;

            depth = newdepth;
        }

        Ok(t)
    }

    pub(super) fn node(&self, i: usize) -> Result<&TreeNode> {
        self.arena.get(i).context("node not found")
    }

    fn node_opt(&self, i: usize) -> Option<&TreeNode> {
        self.arena.get(i)
    }

    pub(super) fn path(&self, i: usize) -> Result<PathBuf> {
        let mut parts = vec![];

        let mut j = i;

        loop {
            let node = self.node(j)?;
            parts.push(node.value.clone());
            if let Some(parent) = node.parent {
                j = parent;
            } else {
                break;
            }
        }

        parts.reverse();
        Ok(PathBuf::from_iter(parts.iter().skip(1).collect::<Vec<_>>()))
    }

    fn node_mut(&mut self, i: usize) -> Result<&mut TreeNode> {
        Ok(&mut self.arena[i])
    }

    pub(super) fn add_node(&mut self, child: TreeNode) -> TreeIndex {
        let index = self.arena.len();
        self.arena.push(child);
        index
    }

    fn fmt_helper(
        &self,
        fmt: &mut fmt::Formatter,
        i: TreeIndex,
        seen: &mut HashSet<TreeIndex>,
        depth: usize,
    ) -> fmt::Result {
        if seen.contains(&i) {
            return Ok(());
        }

        let node = self.node(i).map_err(|_| fmt::Error {})?;

        if i > 0 {
            let line = format!(
                "{}{}\n",
                String::from_utf8(vec![b' '; (depth - 1) * 2]).map_err(|_| fmt::Error {})?,
                &node.value
            );
            fmt.write_str(&line)?;
        }

        seen.insert(i);

        for child in &node.children {
            self.fmt_helper(fmt, *child, seen, depth + 1)?;
        }

        Ok(())
    }

    // pub(super) fn find_path(&self, path: &Path) -> Option<TreeIndex> {
    //     let mut node = 0;

    //     for osstr in path.iter() {
    //         let part = osstr.to_string_lossy().to_string();

    //         match self.find(node, part.clone()) {
    //             None => return None,
    //             Some(n) => {
    //                 node = n;
    //             }
    //         }
    //     }

    //     Some(node)
    // }

    pub(super) fn add_path(&mut self, path: &Path) -> Result<TreeIndex> {
        let mut node = 0;
        let mut newnode;

        for osstr in path.iter() {
            let part = osstr.to_string_lossy().to_string();

            newnode = match self.find(node, part.clone()) {
                Some(m) => m,
                None => self.add_node(TreeNode::new(part)),
            };

            let node_mut = self.node_mut(node)?;
            node_mut.children.push(newnode);
            node_mut.children.dedup();

            self.node_mut(newnode)?.parent = Some(node);

            node = newnode;
        }

        Ok(node)
    }

    fn find(&self, i: TreeIndex, value: T) -> Option<TreeIndex> {
        if let Some(node) = self.node_opt(i) {
            for j in &node.children {
                if let Some(child) = self.node_opt(*j) {
                    if child.value == value {
                        return Some(*j);
                    }
                }
            }
        }
        None
    }
}

fn number_tabs(line: &str) -> u32 {
    let mut chars = line.chars();
    let mut i = 0;
    while let Some(' ') = chars.next() {
        i += 1;
    }
    i / 2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_number_tabs() {
        let cases = [
            ("foo", 0),
            (" foo", 0),
            ("  foo", 1),
            ("   foo", 1),
            ("    foo", 2),
            ("foo bar", 0),
            ("foo bar  lorem", 0),
            ("  foo         ", 1),
        ];
        for (input, output) in cases {
            assert_eq!(number_tabs(input), output);
        }
    }
}
