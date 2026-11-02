use crate::model::{Document, Style};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Field {
    Text,
    Note,
    Details,
    Depth,
    Icon,
    Progress,
    Link,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Operator {
    Equals,
    Contains,
    StartsWith,
    Regex,
    Greater,
    Less,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Condition {
    Match {
        field: Field,
        op: Operator,
        value: String,
        case_sensitive: bool,
    },
    And {
        items: Vec<Condition>,
    },
    Or {
        items: Vec<Condition>,
    },
    Not {
        item: Box<Condition>,
    },
}
impl Default for Condition {
    fn default() -> Self {
        Self::Match {
            field: Field::Text,
            op: Operator::Contains,
            value: String::new(),
            case_sensitive: false,
        }
    }
}
impl Condition {
    pub fn validate(&self) -> anyhow::Result<()> {
        match self {
            Self::Match {
                op: Operator::Regex,
                value,
                ..
            } => {
                regex::RegexBuilder::new(value)
                    .size_limit(1_000_000)
                    .build()?;
            }
            Self::And { items } | Self::Or { items } => {
                for i in items {
                    i.validate()?;
                }
            }
            Self::Not { item } => item.validate()?,
            _ => {}
        }
        Ok(())
    }
    pub fn matches(&self, doc: &Document, id: &str) -> bool {
        match self {
            Self::And { items } => items.iter().all(|i| i.matches(doc, id)),
            Self::Or { items } => items.iter().any(|i| i.matches(doc, id)),
            Self::Not { item } => !item.matches(doc, id),
            Self::Match {
                field,
                op,
                value,
                case_sensitive,
            } => {
                let n = &doc.nodes[id];
                let text = match field {
                    Field::Text => n.text.text(),
                    Field::Note => n.note.text(),
                    Field::Details => n.details.text(),
                    Field::Depth => doc.depth(id).to_string(),
                    Field::Icon => n.icons.join(" "),
                    Field::Progress => n.progress.map(|p| p.to_string()).unwrap_or_default(),
                    Field::Link => n.link.clone(),
                };
                if *op == Operator::Regex {
                    return regex::RegexBuilder::new(value)
                        .case_insensitive(!case_sensitive)
                        .size_limit(1_000_000)
                        .build()
                        .is_ok_and(|r| r.is_match(&text));
                }
                let (text, value) = if *case_sensitive {
                    (text, value.clone())
                } else {
                    (text.to_lowercase(), value.to_lowercase())
                };
                match op {
                    Operator::Equals => text == value,
                    Operator::Contains => text.contains(&value),
                    Operator::StartsWith => text.starts_with(&value),
                    Operator::Greater | Operator::Less => text
                        .parse::<f64>()
                        .ok()
                        .zip(value.parse::<f64>().ok())
                        .is_some_and(|(a, b)| {
                            if *op == Operator::Greater {
                                a > b
                            } else {
                                a < b
                            }
                        }),
                    _ => false,
                }
            }
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Rule {
    pub enabled: bool,
    pub stop: bool,
    pub style: String,
    pub condition: Condition,
}
pub fn resolved_style(doc: &Document, id: &str) -> Style {
    let mut style = doc.nodes[id].style.clone();
    if let Some(raw) = &doc.nodes[id].raw {
        let base = raw
            .attributes
            .get("STYLE_REF")
            .or_else(|| raw.attributes.get("LOCALIZED_STYLE_REF"))
            .and_then(|name| doc.styles.get(name))
            .or_else(|| doc.styles.get("default"));
        if let Some(base) = base {
            let own = style.clone();
            style = base.clone();
            if raw.attributes.contains_key("COLOR") {
                style.color = own.color;
            }
            if raw.attributes.contains_key("BACKGROUND_COLOR") {
                style.background = own.background;
            }
            if raw.attributes.contains_key("STYLE") {
                style.shape = own.shape;
            }
            if raw.get_child("font").is_some() {
                style.font_size = own.font_size;
                style.font_family = own.font_family;
                style.bold = own.bold;
                style.italic = own.italic;
            }
            if raw.get_child("edge").is_some() {
                style.edge_color = own.edge_color;
            }
            // A local edit takes precedence over inherited source properties.
            let original = crate::mm::parse_node(raw).style;
            if doc.nodes[id].style != original {
                style = doc.nodes[id].style.clone();
            }
        }
    }
    for r in &doc.rules {
        if r.enabled && r.condition.matches(doc, id) {
            if let Some(s) = doc.styles.get(&r.style) {
                style = s.clone();
            }
            if r.stop {
                break;
            }
        }
    }
    style
}
