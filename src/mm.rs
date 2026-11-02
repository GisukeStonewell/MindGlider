use crate::i18n::{msg, tr};
use crate::{conditions::Rule, model::*, richtext::RichText};
use anyhow::{Context, Result, ensure};
use std::{collections::BTreeMap, io::Cursor};
use xmltree::{Element, EmitterConfig, XMLNode};

fn child<'a>(e: &'a Element, name: &str) -> Option<&'a Element> {
    e.children
        .iter()
        .filter_map(XMLNode::as_element)
        .find(|x| x.name == name)
}
fn rich(e: &Element, kind: &str) -> Option<RichText> {
    e.children
        .iter()
        .filter_map(XMLNode::as_element)
        .find(|c| c.name == "richcontent" && c.attributes.get("TYPE").is_some_and(|t| t == kind))
        .map(RichText::from_html)
}
fn attr(e: &Element, k: &str, default: &str) -> String {
    e.attributes
        .get(k)
        .cloned()
        .unwrap_or_else(|| default.into())
}
fn number(e: &Element, k: &str, default: f32) -> f32 {
    e.attributes
        .get(k)
        .and_then(|s| s.parse::<f32>().ok())
        .filter(|n| n.is_finite())
        .unwrap_or(default)
}
fn bool_attr(e: &Element, k: &str) -> bool {
    e.attributes.get(k).is_some_and(|s| s == "true")
}
pub fn parse_node(e: &Element) -> Node {
    let mut n = Node::new("");
    n.id = attr(e, "ID", &new_id());
    n.raw = Some(e.clone());
    n.text = rich(e, "NODE").unwrap_or_else(|| {
        let text = attr(e, "TEXT", "");
        if text.trim_start().starts_with("<html") {
            if let Ok(html) = Element::parse(Cursor::new(&text)) {
                return RichText::from_html(&html);
            }
            let mut r = RichText::plain(text);
            r.unsupported = true;
            r
        } else {
            RichText::plain(text)
        }
    });
    n.note = rich(e, "NOTE").unwrap_or_default();
    n.details = rich(e, "DETAILS").unwrap_or_default();
    n.style.color = attr(e, "COLOR", &n.style.color);
    n.style.background = attr(e, "BACKGROUND_COLOR", &n.style.background);
    n.style.shape = attr(e, "STYLE", &n.style.shape);
    if let Some(f) = child(e, "font") {
        n.style.font_family = attr(f, "NAME", &n.style.font_family);
        n.style.font_size = number(f, "SIZE", 16.);
        n.style.bold = bool_attr(f, "BOLD");
        n.style.italic = bool_attr(f, "ITALIC");
    }
    if let Some(edge) = child(e, "edge") {
        n.style.edge_color = attr(edge, "COLOR", &n.style.edge_color);
    }
    n.left = attr(e, "POSITION", "") == "left";
    n.folded = bool_attr(e, "FOLDED");
    n.link = attr(e, "LINK", "");
    n.protected = ["ENCRYPTED_CONTENT", "CONTENT_ID", "TREE_ID"]
        .iter()
        .any(|a| e.attributes.contains_key(*a));
    if let Some(c) = child(e, "cloud") {
        n.cloud = Some(attr(c, "COLOR", "#e8eef8"));
    }
    for c in e.children.iter().filter_map(XMLNode::as_element) {
        if c.name == "icon" {
            let name = attr(c, "BUILTIN", "");
            if let Some(p) = name.strip_suffix('%').and_then(|p| p.parse::<u8>().ok()) {
                n.progress = Some(p.min(100));
            } else {
                n.icons.push(name);
            }
        }
        if c.name == "hook" && attr(c, "NAME", "") == "ExternalObject" {
            let uri = attr(c, "URI", "");
            let dims =
                image::image_dimensions(crate::storage::resolve(None, &uri)).unwrap_or((160, 100));
            let scale = number(c, "SIZE", 1.).max(0.01);
            n.image = Some(Picture {
                uri,
                width: number(
                    c,
                    "MINDGLIDER_WIDTH",
                    number(c, crate::compat::OLD_WIDTH, dims.0 as f32 * scale),
                ),
                height: number(
                    c,
                    "MINDGLIDER_HEIGHT",
                    number(c, crate::compat::OLD_HEIGHT, dims.1 as f32 * scale),
                ),
            });
        }
    }
    n
}
pub fn read(bytes: &[u8]) -> Result<Document> {
    ensure!(bytes.len() <= 100 * 1024 * 1024, "{}", msg("mm.226", &[]));
    let raw = Element::parse(Cursor::new(bytes)).context(tr("mm.229"))?;
    ensure!(raw.name == "map", "{}", msg("mm.225", &[]));
    let root = child(&raw, "node").context(tr("mm.228"))?;
    let mut doc = Document {
        root: String::new(),
        nodes: BTreeMap::new(),
        connectors: vec![],
        styles: BTreeMap::new(),
        rules: vec![],
        raw: raw.clone(),
    };
    fn load(
        e: &Element,
        parent: Option<String>,
        left: bool,
        depth: usize,
        doc: &mut Document,
    ) -> Result<String> {
        ensure!(depth < 512, "{}", msg("mm.224", &[]));
        let mut n = parse_node(e);
        ensure!(
            !doc.nodes.contains_key(&n.id),
            "{}",
            msg("mm.223", &[("0", n.id.to_string())])
        );
        n.parent = parent;
        n.left = if e.attributes.contains_key("POSITION") {
            n.left
        } else {
            left
        };
        let id = n.id.clone();
        // Keep unknown metadata, but children are reconstructed from the editable tree.
        n.raw
            .as_mut()
            .unwrap()
            .children
            .retain(|c| !c.as_element().is_some_and(|e| e.name == "node"));
        doc.nodes.insert(id.clone(), n);
        for c in e.children.iter().filter_map(XMLNode::as_element) {
            if c.name == "node" {
                let cid = load(c, Some(id.clone()), doc.nodes[&id].left, depth + 1, doc)?;
                doc.nodes.get_mut(&id).unwrap().children.push(cid);
            }
            if c.name == "arrowlink" {
                doc.connectors.push(Connector {
                    from: id.clone(),
                    to: attr(c, "DESTINATION", ""),
                    color: attr(c, "COLOR", "#667799"),
                    label: attr(c, "MIDDLE_LABEL", ""),
                });
            }
        }
        Ok(id)
    }
    doc.root = load(root, None, false, 0, &mut doc)?;
    (doc.styles, doc.rules) = crate::native::read(root);
    if let Some(data) = raw
        .children
        .iter()
        .filter_map(XMLNode::as_element)
        .find(|e| {
            e.name == "hook"
                && matches!(
                    attr(e, "NAME", "").as_str(),
                    "MindGlider" | crate::compat::OLD_APP
                )
        })
        && let Some(json) = data.get_text()
    {
        let meta: Meta = serde_json::from_str(&json).context(tr("mm.227"))?;
        if meta.native_signature.as_deref() == Some(&native_signature(root)) {
            doc.styles = meta.styles;
            doc.rules = meta.rules;
        }
    }
    // References into an unopened linked map are kept in raw XML, not silently deleted.
    doc.connectors.retain(|c| doc.nodes.contains_key(&c.to));
    doc.validate()?;
    Ok(doc)
}
#[derive(serde::Serialize, serde::Deserialize)]
struct Meta {
    styles: BTreeMap<String, Style>,
    rules: Vec<Rule>,
    #[serde(default)]
    native_signature: Option<String>,
}
fn native_signature(root: &Element) -> String {
    format!(
        "{:?}",
        root.children
            .iter()
            .filter_map(XMLNode::as_element)
            .find(|e| e.name == "hook" && attr(e, "NAME", "") == "MapStyle")
    )
}
pub fn read_at(bytes: &[u8], path: &std::path::Path) -> Result<Document> {
    let mut doc = read(bytes)?;
    for n in doc.nodes.values_mut() {
        if let Some(p) = &mut n.image
            && let Some(raw) = &n.raw
            && let Some(h) = raw
                .children
                .iter()
                .filter_map(XMLNode::as_element)
                .find(|c| c.name == "hook" && attr(c, "NAME", "") == "ExternalObject")
            && !h.attributes.contains_key("MINDGLIDER_WIDTH")
            && !h.attributes.contains_key(crate::compat::OLD_WIDTH)
            && let Ok((w, height)) =
                image::image_dimensions(crate::storage::resolve(Some(path), &p.uri))
        {
            let scale = number(h, "SIZE", 1.).max(0.01);
            p.width = w as f32 * scale;
            p.height = height as f32 * scale;
        }
    }
    Ok(doc)
}
fn set(e: &mut Element, k: &str, v: impl Into<String>) {
    e.attributes.insert(k.into(), v.into());
}
fn remove_children(e: &mut Element, mut predicate: impl FnMut(&Element) -> bool) {
    e.children
        .retain(|c| !c.as_element().is_some_and(&mut predicate));
}
fn replace_rich(e: &mut Element, kind: &str, r: &RichText) -> Result<()> {
    remove_children(e, |c| {
        c.name == "richcontent" && attr(c, "TYPE", "") == kind
    });
    if !r.text().is_empty() {
        let html = format!(
            "<richcontent TYPE=\"{kind}\"><html><head/><body><p>{}</p></body></html></richcontent>",
            r.html()
        );
        e.children
            .push(XMLNode::Element(Element::parse(Cursor::new(html))?));
    }
    Ok(())
}
pub fn node_element(doc: &Document, id: &str) -> Result<Element> {
    let n = &doc.nodes[id];
    let old = n.raw.as_ref().map(parse_node);
    let mut e = n.raw.clone().unwrap_or_else(|| Element::new("node"));
    set(&mut e, "ID", &n.id);
    let changed = |f: fn(&Node) -> String| old.as_ref().is_none_or(|o| f(o) != f(n));
    if old.as_ref().is_none_or(|o| o.text != n.text) {
        remove_children(&mut e, |c| {
            c.name == "richcontent" && attr(c, "TYPE", "") == "NODE"
        });
        if n.text.runs.iter().all(|r| r.marks == Default::default()) {
            set(&mut e, "TEXT", n.text.text());
        } else {
            e.attributes.shift_remove("TEXT");
            replace_rich(&mut e, "NODE", &n.text)?;
        }
    }
    if old.as_ref().is_none_or(|o| o.note != n.note) {
        replace_rich(&mut e, "NOTE", &n.note)?;
    }
    if old.as_ref().is_none_or(|o| o.details != n.details) {
        replace_rich(&mut e, "DETAILS", &n.details)?;
    }
    if changed(|n| n.link.clone()) {
        if n.link.is_empty() {
            e.attributes.shift_remove("LINK");
        } else {
            set(&mut e, "LINK", &n.link);
        }
    }
    if old.as_ref().is_none_or(|o| o.folded != n.folded) {
        set(&mut e, "FOLDED", n.folded.to_string());
    }
    if n.parent.as_deref() == Some(&doc.root) {
        set(&mut e, "POSITION", if n.left { "left" } else { "right" });
    }
    if old.as_ref().is_none_or(|o| o.style != n.style) {
        set(&mut e, "COLOR", &n.style.color);
        set(&mut e, "BACKGROUND_COLOR", &n.style.background);
        set(&mut e, "STYLE", &n.style.shape);
        let mut f = child(&e, "font")
            .cloned()
            .unwrap_or_else(|| Element::new("font"));
        set(&mut f, "NAME", &n.style.font_family);
        set(&mut f, "SIZE", n.style.font_size.to_string());
        set(&mut f, "BOLD", n.style.bold.to_string());
        set(&mut f, "ITALIC", n.style.italic.to_string());
        remove_children(&mut e, |c| c.name == "font");
        e.children.push(XMLNode::Element(f));
        let mut edge = child(&e, "edge")
            .cloned()
            .unwrap_or_else(|| Element::new("edge"));
        set(&mut edge, "COLOR", &n.style.edge_color);
        remove_children(&mut e, |c| c.name == "edge");
        e.children.push(XMLNode::Element(edge));
    }
    if old
        .as_ref()
        .is_none_or(|o| o.icons != n.icons || o.progress != n.progress)
    {
        remove_children(&mut e, |c| c.name == "icon");
        for icon in n
            .icons
            .iter()
            .cloned()
            .chain(n.progress.map(|p| format!("{p}%")))
        {
            let mut c = Element::new("icon");
            set(&mut c, "BUILTIN", icon);
            e.children.push(XMLNode::Element(c));
        }
    }
    if old.as_ref().is_none_or(|o| o.cloud != n.cloud) {
        remove_children(&mut e, |c| c.name == "cloud");
        if let Some(color) = &n.cloud {
            let mut c = Element::new("cloud");
            set(&mut c, "COLOR", color);
            set(&mut c, "SHAPE", "ARC");
            e.children.push(XMLNode::Element(c));
        }
    }
    if old.as_ref().is_none_or(|o| o.image != n.image) {
        let original = e
            .children
            .iter()
            .filter_map(XMLNode::as_element)
            .find(|c| c.name == "hook" && attr(c, "NAME", "") == "ExternalObject")
            .cloned();
        remove_children(&mut e, |c| {
            c.name == "hook" && attr(c, "NAME", "") == "ExternalObject"
        });
        if let Some(p) = &n.image {
            let mut c = original.unwrap_or_else(|| Element::new("hook"));
            set(&mut c, "NAME", "ExternalObject");
            let path = crate::storage::resolve(None, &p.uri);
            let uri = url::Url::from_file_path(&path)
                .map(|u| u.to_string())
                .unwrap_or_else(|_| p.uri.replace('\\', "/"));
            set(&mut c, "URI", uri);
            let width = image::image_dimensions(&path)
                .map(|d| d.0 as f32)
                .unwrap_or(p.width);
            set(&mut c, "SIZE", (p.width / width).to_string());
            set(&mut c, "MINDGLIDER_WIDTH", p.width.to_string());
            set(&mut c, "MINDGLIDER_HEIGHT", p.height.to_string());
            e.children.push(XMLNode::Element(c));
        }
    }
    for h in e.children.iter_mut().filter_map(XMLNode::as_mut_element) {
        if h.name == "hook" && attr(h, "NAME", "") == "ExternalObject" {
            for (old, new) in [
                (crate::compat::OLD_WIDTH, "MINDGLIDER_WIDTH"),
                (crate::compat::OLD_HEIGHT, "MINDGLIDER_HEIGHT"),
            ] {
                if let Some(value) = h.attributes.shift_remove(old) {
                    h.attributes.entry(new.into()).or_insert(value);
                }
            }
        }
    }
    // Preserve unknown arrowlink attributes on an existing connector.
    let arrows: Vec<_> = e
        .children
        .iter()
        .filter_map(XMLNode::as_element)
        .filter(|c| c.name == "arrowlink")
        .cloned()
        .collect();
    fn was_local(e: &Element, id: &str) -> bool {
        (e.name == "node" && e.attributes.get("ID").is_some_and(|i| i == id))
            || e.children
                .iter()
                .filter_map(XMLNode::as_element)
                .any(|e| was_local(e, id))
    }
    remove_children(&mut e, |c| {
        c.name == "arrowlink"
            && (doc.nodes.contains_key(&attr(c, "DESTINATION", ""))
                || was_local(&doc.raw, &attr(c, "DESTINATION", "")))
    });
    for c in doc.connectors.iter().filter(|c| c.from == id) {
        let mut a = arrows
            .iter()
            .find(|a| attr(a, "DESTINATION", "") == c.to)
            .cloned()
            .unwrap_or_else(|| Element::new("arrowlink"));
        set(&mut a, "DESTINATION", &c.to);
        set(&mut a, "COLOR", &c.color);
        set(&mut a, "MIDDLE_LABEL", &c.label);
        set(&mut a, "ENDARROW", "DEFAULT");
        e.children.push(XMLNode::Element(a));
    }
    remove_children(&mut e, |c| c.name == "node");
    for c in &n.children {
        e.children.push(XMLNode::Element(node_element(doc, c)?));
    }
    Ok(e)
}
pub fn write(doc: &Document) -> Result<Vec<u8>> {
    doc.validate()?;
    let mut e = doc.raw.clone();
    // Freeplane reads the first bytes using startsWith("<map version=\"...").
    // XML declarations, BOMs, leading whitespace and a different first attribute
    // all select its unknown-dialect path before its XML parser is even called.
    let original = std::mem::take(&mut e.attributes);
    set(
        &mut e,
        "version",
        original
            .get("version")
            .cloned()
            .unwrap_or_else(|| "freeplane 1.12.1".into()),
    );
    e.attributes
        .extend(original.into_iter().filter(|(key, _)| key != "version"));
    set(
        &mut e,
        "generator",
        format!("MindGlider {}", env!("CARGO_PKG_VERSION")),
    );
    remove_children(&mut e, |c| {
        c.name == "node"
            || (c.name == "hook"
                && matches!(
                    attr(c, "NAME", "").as_str(),
                    "MindGlider" | crate::compat::OLD_APP
                ))
    });
    let mut root = node_element(doc, &doc.root)?;
    crate::native::update(&mut root, doc);
    if !doc.rules.is_empty() || !doc.styles.is_empty() {
        let mut meta = Element::new("hook");
        set(&mut meta, "NAME", "MindGlider");
        meta.children
            .push(XMLNode::CData(serde_json::to_string(&Meta {
                styles: doc.styles.clone(),
                rules: doc.rules.clone(),
                native_signature: Some(native_signature(&root)),
            })?));
        e.children.push(XMLNode::Element(meta));
    }
    e.children.push(XMLNode::Element(root));
    let mut bytes = vec![];
    e.write_with_config(
        &mut bytes,
        EmitterConfig::new()
            .perform_indent(false)
            .write_document_declaration(false),
    )?;
    Ok(freeplane_characters(&String::from_utf8(bytes)?).into_bytes())
}

/// UTF-8 XML with numeric character references, like Freeplane's useAsciiCharset.
/// Keep comments, XML names and processing instructions intact. Non-ASCII CDATA
/// becomes equivalent escaped text: references inside CDATA would be literal.
fn freeplane_characters(xml: &str) -> String {
    use std::fmt::Write;
    fn character(out: &mut String, c: char) {
        if c.is_ascii() {
            out.push(c);
        } else {
            write!(out, "&#x{:x};", c as u32).unwrap();
        }
    }
    let mut out = String::with_capacity(xml.len());
    let mut rest = xml;
    let mut markup = false;
    let mut quote = None;
    while !rest.is_empty() {
        if quote.is_none() && (rest.starts_with("<!--") || rest.starts_with("<?")) {
            let end = if rest.starts_with("<!--") {
                "-->"
            } else {
                "?>"
            };
            let length = rest.find(end).map(|i| i + end.len()).unwrap_or(rest.len());
            out.push_str(&rest[..length]);
            rest = &rest[length..];
            continue;
        }
        if quote.is_none() && rest.starts_with("<![CDATA[") {
            let end = rest.find("]]>").unwrap();
            let text = &rest[9..end];
            if text.is_ascii() {
                out.push_str(&rest[..end + 3]);
            } else {
                for c in text.chars() {
                    match c {
                        '&' => out.push_str("&amp;"),
                        '<' => out.push_str("&lt;"),
                        '>' => out.push_str("&gt;"),
                        _ => character(&mut out, c),
                    }
                }
            }
            rest = &rest[end + 3..];
            continue;
        }
        let c = rest.chars().next().unwrap();
        rest = &rest[c.len_utf8()..];
        if let Some(delimiter) = quote {
            character(&mut out, c);
            if c == delimiter {
                quote = None;
            }
        } else if markup {
            out.push(c);
            if c == '"' || c == '\'' {
                quote = Some(c);
            } else if c == '>' {
                markup = false;
            }
        } else if c == '<' {
            out.push(c);
            markup = true;
        } else {
            character(&mut out, c);
        }
    }
    out
}
