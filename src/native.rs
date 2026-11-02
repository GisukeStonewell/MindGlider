//! Freeplane's standard style and condition XML. Unknown conditions stay in the source tree.
use crate::{
    conditions::{Condition, Field, Operator, Rule},
    mm::parse_node,
    model::{Document, Style},
};
use std::collections::BTreeMap;
use xmltree::{Element, XMLNode};
fn attr(e: &Element, k: &str) -> String {
    e.attributes.get(k).cloned().unwrap_or_default()
}
fn set(e: &mut Element, k: &str, v: impl ToString) {
    e.attributes.insert(k.into(), v.to_string());
}
fn first(e: &Element) -> Option<&Element> {
    e.children.iter().find_map(XMLNode::as_element)
}
fn field(s: &str) -> Option<Field> {
    Some(match s {
        "filter_note" => Field::Note,
        "filter_details" => Field::Details,
        "filter_node" | "" => Field::Text,
        _ => return None,
    })
}
pub fn condition(e: &Element) -> Option<Condition> {
    let case = attr(e, "MATCH_CASE") == "true";
    let value = attr(e, "VALUE");
    let f = field(&attr(e, "ITEM"));
    let (field, op, value) = match e.name.as_str() {
        "conjunct_condition" | "disjunct_condition" => {
            let items = e
                .children
                .iter()
                .filter_map(XMLNode::as_element)
                .map(condition)
                .collect::<Option<Vec<_>>>()?;
            return Some(if e.name == "conjunct_condition" {
                Condition::And { items }
            } else {
                Condition::Or { items }
            });
        }
        "negate_condition" => {
            return Some(Condition::Not {
                item: Box::new(condition(first(e)?)?),
            });
        }
        "node_contains_condition" | "match_case_node_contains_condition" => {
            if attr(e, "MATCH_APPROXIMATELY") == "true"
                || attr(e, "MATCH_WORDWISE") == "true"
                || attr(e, "IGNORE_DIACRITICS") == "true"
            {
                return None;
            }
            (f?, Operator::Contains, value)
        }
        "node_matches_regexp" => (f?, Operator::Regex, attr(e, "SEARCH_PATTERN")),
        "node_compare_condition" | "node_level_condition" => {
            if attr(e, "SUCCEED") != "true" {
                return None;
            }
            let op = match attr(e, "COMPARISON_RESULT").as_str() {
                "0" => Operator::Equals,
                "1" => Operator::Greater,
                "-1" => Operator::Less,
                _ => return None,
            };
            (
                if e.name == "node_level_condition" {
                    Field::Depth
                } else {
                    f?
                },
                op,
                value,
            )
        }
        "icon_contained_condition" => (Field::Icon, Operator::Contains, attr(e, "ICON")),
        "hyper_link_contains" => (Field::Link, Operator::Contains, attr(e, "TEXT")),
        _ => return None,
    };
    Some(Condition::Match {
        field,
        op,
        value,
        case_sensitive: case || e.name == "match_case_node_contains_condition",
    })
}
pub fn read(root: &Element) -> (BTreeMap<String, Style>, Vec<Rule>) {
    let mut styles = BTreeMap::new();
    let mut rules = vec![];
    fn styles_walk(e: &Element, styles: &mut BTreeMap<String, Style>) {
        if e.name == "stylenode" {
            let name = e
                .attributes
                .get("TEXT")
                .or_else(|| e.attributes.get("LOCALIZED_TEXT"));
            if let Some(name) = name {
                styles.insert(name.clone(), parse_node(e).style);
            }
        }
        for c in e.children.iter().filter_map(XMLNode::as_element) {
            styles_walk(c, styles);
        }
    }
    for hook in root
        .children
        .iter()
        .filter_map(XMLNode::as_element)
        .filter(|e| e.name == "hook" && attr(e, "NAME") == "MapStyle")
    {
        for c in hook.children.iter().filter_map(XMLNode::as_element) {
            if c.name == "map_styles" {
                styles_walk(c, &mut styles);
            }
            if c.name == "conditional_styles" {
                for r in c.children.iter().filter_map(XMLNode::as_element) {
                    if let Some(condition) = first(r).and_then(condition) {
                        rules.push(Rule {
                            enabled: attr(r, "ACTIVE") == "true",
                            stop: attr(r, "LAST") == "true",
                            style: r
                                .attributes
                                .get("STYLE_REF")
                                .or_else(|| r.attributes.get("LOCALIZED_STYLE_REF"))
                                .cloned()
                                .unwrap_or_default(),
                            condition,
                        });
                    }
                }
            }
        }
    }
    (styles, rules)
}
pub fn condition_xml(c: &Condition) -> Option<Element> {
    let mut e;
    match c {
        Condition::And { items } | Condition::Or { items } => {
            e = Element::new(if matches!(c, Condition::And { .. }) {
                "conjunct_condition"
            } else {
                "disjunct_condition"
            });
            for c in items {
                e.children.push(XMLNode::Element(condition_xml(c)?));
            }
        }
        Condition::Not { item } => {
            e = Element::new("negate_condition");
            e.children.push(XMLNode::Element(condition_xml(item)?));
        }
        Condition::Match {
            field,
            op,
            value,
            case_sensitive,
        } => {
            if *field == Field::Progress {
                let mut items = vec![];
                for n in [0, 25, 50, 75, 100] {
                    let s = n.to_string();
                    let matches = match op {
                        Operator::Equals => s == *value,
                        Operator::Contains => s.contains(value),
                        Operator::StartsWith => s.starts_with(value),
                        Operator::Regex => regex::Regex::new(value).is_ok_and(|r| r.is_match(&s)),
                        Operator::Greater => value.parse::<f32>().is_ok_and(|v| n as f32 > v),
                        Operator::Less => value.parse::<f32>().is_ok_and(|v| (n as f32) < v),
                    };
                    if matches {
                        let mut icon = Element::new("icon_contained_condition");
                        set(&mut icon, "ICON", format!("{n}%"));
                        items.push(XMLNode::Element(icon));
                    }
                }
                e = Element::new("disjunct_condition");
                e.children = items;
                return Some(e);
            }
            if *field == Field::Icon {
                if !matches!(op, Operator::Contains | Operator::Equals) {
                    return None;
                }
                e = Element::new("icon_contained_condition");
                set(&mut e, "ICON", value);
                return Some(e);
            }
            if *field == Field::Link {
                if *op != Operator::Contains {
                    return None;
                }
                e = Element::new("hyper_link_contains");
                set(&mut e, "TEXT", value);
                set(&mut e, "MATCH_CASE", case_sensitive);
                return Some(e);
            }
            e = Element::new(if *field == Field::Depth {
                "node_level_condition"
            } else {
                match op {
                    Operator::Contains => {
                        if *case_sensitive {
                            "match_case_node_contains_condition"
                        } else {
                            "node_contains_condition"
                        }
                    }
                    Operator::Regex | Operator::StartsWith => "node_matches_regexp",
                    _ => "node_compare_condition",
                }
            });
            set(
                &mut e,
                "ITEM",
                match field {
                    Field::Note => "filter_note",
                    Field::Details => "filter_details",
                    _ => "filter_node",
                },
            );
            set(&mut e, "MATCH_CASE", case_sensitive);
            if matches!(op, Operator::StartsWith | Operator::Regex) && *field != Field::Depth {
                set(
                    &mut e,
                    "SEARCH_PATTERN",
                    if *op == Operator::StartsWith {
                        format!("^{}", regex::escape(value))
                    } else {
                        value.clone()
                    },
                );
            } else {
                set(&mut e, "VALUE", value);
                if matches!(op, Operator::Equals | Operator::Greater | Operator::Less) {
                    set(
                        &mut e,
                        "COMPARISON_RESULT",
                        match op {
                            Operator::Greater => 1,
                            Operator::Less => -1,
                            _ => 0,
                        },
                    );
                    set(&mut e, "SUCCEED", "true");
                }
            }
        }
    }
    Some(e)
}
pub fn update(root: &mut Element, doc: &Document) {
    let previous = read(root);
    if previous.0 == doc.styles && previous.1 == doc.rules {
        return;
    }
    let pos = root.children.iter().position(|c| {
        c.as_element()
            .is_some_and(|e| e.name == "hook" && attr(e, "NAME") == "MapStyle")
    });
    let index = pos.unwrap_or_else(|| {
        let mut e = Element::new("hook");
        set(&mut e, "NAME", "MapStyle");
        root.children.push(XMLNode::Element(e));
        root.children.len() - 1
    });
    let hook = root.children[index].as_mut_element().unwrap();
    // Existing noneditable conditions remain verbatim, and are never interpreted as true.
    let mut conditional = hook
        .get_child("conditional_styles")
        .cloned()
        .unwrap_or_else(|| Element::new("conditional_styles"));
    conditional.children.retain(|c| {
        !c.as_element()
            .is_some_and(|r| first(r).and_then(condition).is_some())
    });
    for rule in &doc.rules {
        if let Some(c) = condition_xml(&rule.condition) {
            let mut e = Element::new("conditional_style");
            set(&mut e, "ACTIVE", rule.enabled);
            set(&mut e, "LAST", rule.stop);
            set(&mut e, "STYLE_REF", &rule.style);
            e.children.push(XMLNode::Element(c));
            conditional.children.push(XMLNode::Element(e));
        }
    }
    hook.children.retain(|c| {
        !c.as_element()
            .is_some_and(|e| e.name == "conditional_styles")
    });
    hook.children.push(XMLNode::Element(conditional));
    let mut map_styles = hook
        .get_child("map_styles")
        .cloned()
        .unwrap_or_else(|| Element::new("map_styles"));
    fn update_style(e: &mut Element, name: &str, style: &Style) -> bool {
        if e.name == "stylenode" && (attr(e, "TEXT") == name || attr(e, "LOCALIZED_TEXT") == name) {
            apply_style(e, style);
            return true;
        }
        for c in e.children.iter_mut().filter_map(XMLNode::as_mut_element) {
            if update_style(c, name, style) {
                return true;
            }
        }
        false
    }
    for (name, style) in &doc.styles {
        // Editing one rule must not materialize defaults into unrelated source styles.
        if previous.0.get(name) == Some(style) {
            continue;
        }
        if !update_style(&mut map_styles, name, style) {
            let mut node = Element::new("stylenode");
            set(&mut node, "TEXT", name);
            apply_style(&mut node, style);
            if map_styles.children.is_empty() {
                let mut root = Element::new("stylenode");
                set(&mut root, "LOCALIZED_TEXT", "styles.root_node");
                map_styles.children.push(XMLNode::Element(root));
            }
            let parent = map_styles
                .children
                .iter_mut()
                .find_map(XMLNode::as_mut_element)
                .unwrap();
            parent.children.push(XMLNode::Element(node));
        }
    }
    hook.children
        .retain(|c| !c.as_element().is_some_and(|e| e.name == "map_styles"));
    hook.children.push(XMLNode::Element(map_styles));
}
fn apply_style(e: &mut Element, s: &Style) {
    set(e, "COLOR", &s.color);
    set(e, "BACKGROUND_COLOR", &s.background);
    set(e, "STYLE", &s.shape);
    let mut font = e
        .get_child("font")
        .cloned()
        .unwrap_or_else(|| Element::new("font"));
    set(&mut font, "NAME", &s.font_family);
    set(&mut font, "SIZE", s.font_size);
    set(&mut font, "BOLD", s.bold);
    set(&mut font, "ITALIC", s.italic);
    e.children
        .retain(|c| !c.as_element().is_some_and(|e| e.name == "font"));
    e.children.push(XMLNode::Element(font));
    let mut edge = e
        .get_child("edge")
        .cloned()
        .unwrap_or_else(|| Element::new("edge"));
    set(&mut edge, "COLOR", &s.edge_color);
    e.children
        .retain(|c| !c.as_element().is_some_and(|e| e.name == "edge"));
    e.children.push(XMLNode::Element(edge));
}
