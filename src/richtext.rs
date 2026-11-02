use serde::{Deserialize, Serialize};
use xmltree::{Element, XMLNode};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct Marks {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strike: bool,
    pub color: Option<String>,
    pub size: Option<f32>,
    pub link: Option<String>,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Run {
    pub text: String,
    pub marks: Marks,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct RichText {
    pub runs: Vec<Run>,
    pub unsupported: bool,
}
impl RichText {
    pub fn plain(text: impl Into<String>) -> Self {
        Self {
            runs: vec![Run {
                text: text.into(),
                marks: Marks::default(),
            }],
            unsupported: false,
        }
    }
    pub fn text(&self) -> String {
        self.runs.iter().map(|r| r.text.as_str()).collect()
    }
    pub fn chars(&self) -> Vec<(char, Marks)> {
        self.runs
            .iter()
            .flat_map(|r| r.text.chars().map(|c| (c, r.marks.clone())))
            .collect()
    }
    fn from_chars(chars: Vec<(char, Marks)>) -> Self {
        let mut result = Self::default();
        for (c, m) in chars {
            if let Some(last) = result.runs.last_mut().filter(|r| r.marks == m) {
                last.text.push(c);
            } else {
                result.runs.push(Run {
                    text: c.to_string(),
                    marks: m,
                });
            }
        }
        result
    }
    pub fn edit(&mut self, text: &str) {
        let old = self.chars();
        let new: Vec<char> = text.chars().collect();
        let prefix = old
            .iter()
            .zip(&new)
            .take_while(|((a, _), b)| a == *b)
            .count();
        let suffix = old[prefix..]
            .iter()
            .rev()
            .zip(new[prefix..].iter().rev())
            .take_while(|((a, _), b)| a == *b)
            .count();
        let mark = old
            .get(prefix.saturating_sub(1))
            .map(|(_, m)| m.clone())
            .unwrap_or_default();
        let mut chars = old[..prefix].to_vec();
        chars.extend(
            new[prefix..new.len() - suffix]
                .iter()
                .map(|c| (*c, mark.clone())),
        );
        chars.extend_from_slice(&old[old.len() - suffix..]);
        *self = Self::from_chars(chars);
    }
    pub fn format(&mut self, start: usize, end: usize, mut apply: impl FnMut(&mut Marks)) {
        let mut chars = self.chars();
        for (_, m) in chars.iter_mut().take(end).skip(start) {
            apply(m);
        }
        *self = Self::from_chars(chars);
    }
    pub fn html(&self) -> String {
        let mut out = String::new();
        for r in &self.runs {
            let mut s = escape(&r.text).replace('\n', "<br />");
            if r.marks.bold {
                s = format!("<b>{s}</b>");
            }
            if r.marks.italic {
                s = format!("<i>{s}</i>");
            }
            if r.marks.underline {
                s = format!("<u>{s}</u>");
            }
            if r.marks.strike {
                s = format!("<s>{s}</s>");
            }
            let mut style = String::new();
            if let Some(c) = &r.marks.color {
                style.push_str(&format!("color:{};", escape(c)));
            }
            if let Some(size) = r.marks.size {
                style.push_str(&format!("font-size:{size}pt;"));
            }
            if !style.is_empty() {
                s = format!("<span style=\"{style}\">{s}</span>");
            }
            if let Some(link) = &r.marks.link {
                let lower = link.trim().to_lowercase();
                if !["javascript:", "vbscript:", "data:"]
                    .iter()
                    .any(|p| lower.starts_with(p))
                {
                    s = format!("<a href=\"{}\">{s}</a>", escape(link));
                }
            }
            out.push_str(&s);
        }
        out
    }
    pub fn from_html(element: &Element) -> Self {
        fn walk(e: &Element, mut m: Marks, out: &mut RichText) {
            match e.name.to_ascii_lowercase().as_str() {
                "b" | "strong" => m.bold = true,
                "i" | "em" => m.italic = true,
                "u" => m.underline = true,
                "s" | "strike" | "del" => m.strike = true,
                "a" => m.link = e.attributes.get("href").cloned(),
                "font" => {
                    m.color = e.attributes.get("color").cloned();
                    if let Some(s) = e.attributes.get("size") {
                        m.size = s.parse().ok();
                        out.unsupported = true;
                    }
                }
                "html" | "head" | "body" | "richcontent" | "p" | "div" | "span" | "br" => {}
                _ => out.unsupported = true,
            }
            if let Some(style) = e.attributes.get("style") {
                for part in style.split(';') {
                    if let Some((k, v)) = part.split_once(':') {
                        match k.trim() {
                            "color" => m.color = Some(v.trim().into()),
                            "font-size" => {
                                m.size = v
                                    .trim()
                                    .trim_end_matches("pt")
                                    .trim_end_matches("px")
                                    .parse()
                                    .ok()
                            }
                            "font-weight" => m.bold = v.trim() == "bold" || v.trim() == "700",
                            "font-style" => m.italic = v.trim() == "italic",
                            "text-decoration" => {
                                m.underline = v.contains("underline");
                                m.strike = v.contains("line-through");
                            }
                            _ => out.unsupported = true,
                        }
                    }
                }
            }
            if matches!(e.name.as_str(), "p" | "div" | "li" | "tr")
                && !out.runs.is_empty()
                && !out.text().ends_with('\n')
            {
                out.runs.push(Run {
                    text: "\n".into(),
                    marks: m.clone(),
                });
            }
            if e.name == "br" {
                out.runs.push(Run {
                    text: "\n".into(),
                    marks: m.clone(),
                });
            }
            if e.name == "head" {
                return;
            }
            for c in &e.children {
                match c {
                    XMLNode::Element(child) => walk(child, m.clone(), out),
                    XMLNode::Text(t) | XMLNode::CData(t) => out.runs.push(Run {
                        text: t.clone(),
                        marks: m.clone(),
                    }),
                    _ => {}
                }
            }
        }
        let mut out = Self::default();
        walk(element, Marks::default(), &mut out);
        let unsupported = out.unsupported;
        let mut canonical = Self::from_chars(out.chars());
        canonical.unsupported = unsupported;
        canonical
    }
}
pub fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
