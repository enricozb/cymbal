use anyhow::Context;
use tree_sitter::{Query as TreeSitterQuery, QueryMatch};

use crate::ext::ResultExt;

pub struct Template {
  items: Vec<Item>,
}

pub enum Item {
  Text(String),
  Index(u32),
}

impl Template {
  pub fn parse<S: AsRef<str>>(s: S, query: &TreeSitterQuery) -> Result<Self, anyhow::Error> {
    let s = s.as_ref();
    let mut items = Vec::new();
    let mut rest = s;

    while let Some(start) = rest.find('{') {
      if start > 0 {
        items.push(Item::Text(rest[..start].to_string()));
      }

      let Some(end) = rest[start..].find('}') else {
        anyhow::bail!("Unmatched '{{' in template")
      };

      let name = &rest[start + 1..start + end];
      let index = query
        .capture_index_for_name(name)
        .with_context(|| "non-captured name {name:?}")?;
      items.push(Item::Index(index));

      rest = &rest[start + end + 1..];
    }

    if !rest.is_empty() {
      items.push(Item::Text(rest.to_string()));
    }

    Ok(Template { items })
  }

  pub fn render(&self, m: &QueryMatch, content: &str) -> Result<String, anyhow::Error> {
    self
      .items
      .iter()
      .map(|item| match item {
        Item::Text(s) => s,
        Item::Index(idx) => m
          .captures
          .iter()
          .find(|c| *idx == c.index)
          .map(|c| &content[c.node.start_byte()..c.node.end_byte()])
          .unwrap_or(""),
      })
      .collect::<Vec<&str>>()
      .join("")
      .ok()
  }
}
