use eframe::egui::vec2;
use mindglider::{
    ai::{self, EditPlan, Operation, Provider, Snapshot},
    conditions::{Condition, Field, Operator, Rule},
    export,
    layout::LayoutCache,
    mm,
    model::*,
    richtext::RichText,
    storage,
};

fn sample() -> (Document, String, String) {
    let mut d = Document::default();
    let root = d.root.clone();
    let a = d.add(&root, "日本語ノード").unwrap();
    let b = d.add(&a, "子ノード").unwrap();
    (d, a, b)
}
#[test]
fn failed_move_is_atomic_and_undo_restores_ids() {
    let (d, a, b) = sample();
    let mut h = History::new(d.clone());
    assert!(h.edit("cycle", |d| d.move_node(&a, &b, 0)).is_err());
    assert_eq!(h.doc, d);
    h.edit("delete", |d| d.delete(&a)).unwrap();
    assert!(!h.doc.nodes.contains_key(&b));
    h.undo();
    assert_eq!(h.doc, d);
    assert!(!h.dirty);
    h.redo();
    assert!(h.dirty);
}
#[test]
fn richtext_edits_preserve_untouched_marks_and_unicode() {
    let mut r = RichText::plain("日本語abc");
    r.format(0, 3, |m| m.bold = true);
    r.edit("日本語xyabc");
    assert!(r.chars()[0].1.bold);
    assert_eq!(r.text(), "日本語xyabc");
    r.format(3, 5, |m| m.link = Some("https://example.org".into()));
    assert!(r.html().contains("href="));
}
#[test]
fn unknown_xml_and_unedited_html_survive() {
    let xml=br##"<map version="freeplane 1.12.1" CUSTOM="keep"><!--comment--><node ID="root" TEXT="root"><hook NAME="FuturePlugin"><custom x="1"><![CDATA[original]]></custom></hook><richcontent TYPE="NOTE"><html><body><table><tr><td>untouched</td></tr></table></body></html></richcontent><node ID="child" TEXT="before" EXTRA="value"><unknown foo="bar"/></node></node></map>"##;
    let mut d = mm::read(xml).unwrap();
    assert!(d.nodes["root"].note.unsupported);
    d.nodes.get_mut("child").unwrap().text.edit("after");
    let bytes = mm::write(&d).unwrap();
    let s = String::from_utf8(bytes.clone()).unwrap();
    for needle in [
        "FuturePlugin",
        "CUSTOM=\"keep\"",
        "EXTRA=\"value\"",
        "<table>",
        "original",
        "<!-- comment -->",
    ] {
        assert!(s.contains(needle), "missing {needle}: {s}");
    }
    let round = mm::read(&bytes).unwrap();
    assert_eq!(round.nodes["child"].text.text(), "after");
    assert_eq!(round.nodes["root"].note, d.nodes["root"].note);
}
#[test]
fn clone_and_encryption_protect_ancestors_from_destructive_edits() {
    for attr in [
        "CONTENT_ID=\"r\"",
        "TREE_ID=\"r\"",
        "ENCRYPTED_CONTENT=\"secret\"",
    ] {
        let xml = format!(
            "<map><node ID=\"r\" TEXT=\"Root\"><node ID=\"a\" TEXT=\"A\"><node ID=\"b\" {attr}/></node></node></map>"
        );
        let mut d = mm::read(xml.as_bytes()).unwrap();
        assert!(d.delete("a").is_err());
        assert!(d.move_node("a", "r", 0).is_err());
        assert!(d.add("b", "no").is_err());
        assert!(mm::write(&d).is_ok());
    }
}
#[test]
fn node_shapes_colors_notes_links_roundtrip() {
    let (mut d, a, _) = sample();
    let n = d.nodes.get_mut(&a).unwrap();
    n.text.format(0, 3, |m| {
        m.bold = true;
        m.italic = true;
        m.underline = true;
        m.color = Some("#cc2244".into());
        m.size = Some(24.);
    });
    n.note = RichText::plain("ノート\n説明");
    n.details = RichText::plain("詳細");
    n.link = "https://example.org/?x=1&y=2".into();
    n.cloud = Some("#eeeeff".into());
    n.style.shape = "oval".into();
    n.progress = Some(75);
    n.icons.push("idea".into());
    let r = mm::read(&mm::write(&d).unwrap()).unwrap();
    let n = &r.nodes[&a];
    assert_eq!(n.text, d.nodes[&a].text);
    assert_eq!(n.note, d.nodes[&a].note);
    assert_eq!(n.progress, Some(75));
    assert_eq!(n.style, d.nodes[&a].style);
    assert_eq!(n.link, d.nodes[&a].link);
}
#[test]
fn deleted_connector_does_not_resurrect_as_unknown_xml() {
    let xml=br##"<map><node ID="r" TEXT="R"><arrowlink DESTINATION="b" COLOR="#112233" CUSTOM="retained"/><node ID="b" TEXT="B"/></node></map>"##;
    let mut d = mm::read(xml).unwrap();
    let s = String::from_utf8(mm::write(&d).unwrap()).unwrap();
    assert!(s.contains("CUSTOM=\"retained\""));
    d.delete("b").unwrap();
    let s = String::from_utf8(mm::write(&d).unwrap()).unwrap();
    assert!(!s.contains("arrowlink"), "{s}");
}
fn cond(field: Field, op: Operator, value: &str) -> Condition {
    Condition::Match {
        field,
        op,
        value: value.into(),
        case_sensitive: false,
    }
}
#[test]
fn compound_conditions_and_ordered_styles() {
    let (mut d, a, b) = sample();
    let c = Condition::And {
        items: vec![
            cond(Field::Text, Operator::Contains, "日本"),
            Condition::Not {
                item: Box::new(cond(Field::Depth, Operator::Equals, "2")),
            },
        ],
    };
    assert!(c.matches(&d, &a));
    assert!(!c.matches(&d, &b));
    assert!(cond(Field::Text, Operator::Regex, "[").validate().is_err());
    let red = Style {
        color: "#ff0000".into(),
        ..Default::default()
    };
    d.styles.insert("red".into(), red.clone());
    d.rules.push(Rule {
        enabled: true,
        stop: true,
        style: "red".into(),
        condition: c,
    });
    assert_eq!(mindglider::conditions::resolved_style(&d, &a), red);
    let r = mm::read(&mm::write(&d).unwrap()).unwrap();
    assert_eq!(r.rules, d.rules);
    assert_eq!(r.styles["red"], red);
    let xml = String::from_utf8(mm::write(&d).unwrap()).unwrap();
    assert!(xml.contains("<conditional_styles>"));
    assert!(xml.contains("conjunct_condition"));
}
#[test]
fn reads_native_freeplane_conditions_without_metadata() {
    let xml=br##"<map><node ID="r" TEXT="Root"><hook NAME="MapStyle"><conditional_styles><conditional_style ACTIVE="true" STYLE_REF="hot" LAST="true"><node_contains_condition ITEM="filter_node" VALUE="match"/></conditional_style></conditional_styles><map_styles><stylenode TEXT="hot" COLOR="#ff0000"><font SIZE="24"/></stylenode></map_styles></hook><node ID="a" TEXT="match"/></node></map>"##;
    let d = mm::read(xml).unwrap();
    assert_eq!(d.rules.len(), 1);
    assert_eq!(
        mindglider::conditions::resolved_style(&d, "a").color,
        "#ff0000"
    );
}
#[test]
fn ai_applies_transaction_and_single_undo() {
    let (d, a, b) = sample();
    let snapshot = Snapshot::capture(&d, &a, "整理").unwrap();
    assert!(!snapshot.payload.contains(&d.root));
    let mut h = History::new(d.clone());
    let plan = EditPlan {
        operations: vec![
            Operation::Update {
                id: a.clone(),
                text: "整理済み".into(),
                note: None,
                details: None,
            },
            Operation::Add {
                id: "ID_new".into(),
                parent: a.clone(),
                text: "追加".into(),
            },
            Operation::Move {
                id: b,
                parent: "ID_new".into(),
                index: 0,
            },
        ],
    };
    assert_eq!(ai::apply(&mut h, &snapshot, &plan).unwrap(), 3);
    h.undo();
    assert_eq!(h.doc, d);
    h.redo();
    assert!(h.doc.nodes.contains_key("ID_new"));
}
#[test]
fn ai_rejects_scope_cycles_root_edits_duplicate_ids_and_rolls_back() {
    let (d, a, b) = sample();
    let snapshot = Snapshot::capture(&d, &a, "x").unwrap();
    for bad in [
        Operation::Delete { id: a.clone() },
        Operation::Update {
            id: d.root.clone(),
            text: "bad".into(),
            note: None,
            details: None,
        },
        Operation::Add {
            id: b.clone(),
            parent: a.clone(),
            text: "duplicate".into(),
        },
        Operation::Move {
            id: b.clone(),
            parent: b.clone(),
            index: 0,
        },
    ] {
        let mut h = History::new(d.clone());
        let plan = EditPlan {
            operations: vec![
                Operation::Update {
                    id: a.clone(),
                    text: "first valid edit".into(),
                    note: None,
                    details: None,
                },
                bad,
            ],
        };
        assert!(ai::apply(&mut h, &snapshot, &plan).is_err());
        assert_eq!(h.doc, d);
    }
}
#[test]
fn ai_conflict_checks_selected_branch_only() {
    let (mut d, a, _) = sample();
    let snapshot = Snapshot::capture(&d, &a, "x").unwrap();
    let root = d.root.clone();
    d.add(&root, "別の枝").unwrap();
    let mut h = History::new(d);
    let plan = EditPlan {
        operations: vec![Operation::Update {
            id: a.clone(),
            text: "ok".into(),
            note: None,
            details: None,
        }],
    };
    ai::apply(&mut h, &snapshot, &plan).unwrap();
    assert!(ai::apply(&mut h, &snapshot, &plan).is_err());
}
#[test]
fn ai_rejects_invalid_text_and_html_conversion() {
    let (mut d, a, _) = sample();
    let snapshot = Snapshot::capture(&d, &a, "x").unwrap();
    let mut h = History::new(d.clone());
    let plan = EditPlan {
        operations: vec![Operation::Update {
            id: a.clone(),
            text: "bad\0".into(),
            note: None,
            details: None,
        }],
    };
    assert!(ai::apply(&mut h, &snapshot, &plan).is_err());
    d.nodes.get_mut(&a).unwrap().text.unsupported = true;
    let snapshot = Snapshot::capture(&d, &a, "x").unwrap();
    let mut h = History::new(d);
    let plan = EditPlan {
        operations: vec![Operation::Update {
            id: a,
            text: "text".into(),
            note: None,
            details: None,
        }],
    };
    assert!(ai::apply(&mut h, &snapshot, &plan).is_err());
}
#[test]
fn provider_envelopes_and_invalid_json() {
    let plain = br##"{"operations":[]}"##;
    assert!(ai::parse_output(Provider::Codex, plain).is_ok());
    assert!(
        ai::parse_output(
            Provider::Claude,
            br##"{"is_error":false,"structured_output":{"operations":[]}}"##
        )
        .is_ok()
    );
    assert!(
        ai::parse_output(
            Provider::Claude,
            br##"{"is_error":true,"result":"login required"}"##
        )
        .is_err()
    );
    assert!(ai::parse_output(Provider::Codex, b"not json").is_err());
    assert!(ai::parse_output(Provider::Codex, br##"{"operations":[],"unexpected":1}"##).is_err());
}
#[test]
fn filtered_layout_retains_ancestors_and_avoids_overlap() {
    let (d, a, b) = sample();
    let mut cache = LayoutCache::default();
    let scene = cache.build(
        &d,
        Some(&cond(Field::Text, Operator::Contains, "子")),
        |s, size| vec2(s.chars().count() as f32 * size, size * 1.3),
    );
    assert_eq!(scene.nodes.len(), 3);
    assert!(scene.nodes.iter().any(|n| n.id == a));
    assert!(scene.nodes.iter().any(|n| n.id == b));
    for (i, n) in scene.nodes.iter().enumerate() {
        for other in &scene.nodes[i + 1..] {
            assert!(!n.rect.intersects(other.rect));
        }
    }
}
#[test]
fn exports_escape_content_and_embed_local_image() {
    let dir = test_dir();
    let image = dir.path().join("日本語 image.png");
    image::RgbaImage::from_pixel(10, 10, image::Rgba([255, 0, 0, 255]))
        .save(&image)
        .unwrap();
    let (mut d, a, _) = sample();
    let n = d.nodes.get_mut(&a).unwrap();
    n.text = RichText::plain("<script>&日本語");
    n.image = Some(Picture {
        uri: image.to_string_lossy().into(),
        width: 20.,
        height: 20.,
    });
    let html = export::outline(&d, &d.root, "html", None).unwrap();
    assert!(html.contains("&lt;script&gt;&amp;"));
    assert!(html.contains("data:image/png;base64,"));
    let opml = export::outline(&d, &d.root, "opml", None).unwrap();
    xmltree::Element::parse(opml.as_bytes()).unwrap();
    let mut cache = LayoutCache::default();
    let scene = cache.build(&d, None, |s, size| {
        vec2(s.chars().count() as f32 * size, size * 1.3)
    });
    let svg = export::svg(&d, &scene, None).unwrap();
    xmltree::Element::parse(svg.as_bytes()).unwrap();
    assert!(!export::raster(&svg, "png").unwrap().is_empty());
    assert!(!export::raster(&svg, "jpg").unwrap().is_empty());
    let pdf = export::pdf(&d, &scene, None, &Default::default()).unwrap();
    assert!(pdf.starts_with(b"%PDF"));
    assert!(String::from_utf8_lossy(&pdf).contains("/Type /Page"));
    std::fs::write(dir.path().join("test.pdf"), pdf).unwrap();
}
#[test]
fn image_dimensions_roundtrip_as_freeplane_zoom() {
    let dir = test_dir();
    let path = dir.path().join("image.png");
    image::RgbaImage::from_pixel(400, 200, image::Rgba([0, 0, 255, 255]))
        .save(&path)
        .unwrap();
    let (mut d, a, _) = sample();
    d.nodes.get_mut(&a).unwrap().image = Some(Picture {
        uri: path.to_string_lossy().into(),
        width: 200.,
        height: 100.,
    });
    let bytes = mm::write(&d).unwrap();
    assert!(String::from_utf8_lossy(&bytes).contains("SIZE=\"0.5\""));
    let r = mm::read(&bytes).unwrap();
    assert_eq!(r.nodes[&a].image.as_ref().unwrap().width, 200.);
}
#[test]
fn paginated_pdf_has_multiple_pages() {
    let (mut d, _, _) = sample();
    let root = d.root.clone();
    for i in 0..30 {
        d.add(&root, format!("ノード{i}")).unwrap();
    }
    let scene =
        LayoutCache::default().build(&d, None, |s, size| vec2(s.len() as f32 * size, size * 1.3));
    let setup = export::PageSetup {
        tile: true,
        ..Default::default()
    };
    let pages = export::page_views(&scene, &setup).unwrap();
    assert!(pages.len() > 1);
    let pdf = export::pdf(&d, &scene, None, &setup).unwrap();
    let text = String::from_utf8_lossy(&pdf);
    assert!(text.contains(&format!("/Count {}", pages.len())));
}
#[test]
fn save_rotates_ten_backups_and_preserves_original_on_error() {
    let dir = test_dir();
    let p = dir.path().join("map.mm");
    let mut d = Document::default();
    for i in 0..13 {
        let root = d.root.clone();
        d.nodes
            .get_mut(&root)
            .unwrap()
            .text
            .edit(&format!("version {i}"));
        storage::save(&p, &d).unwrap();
    }
    let backups = std::fs::read_dir(dir.path().join(".mindglider-backup"))
        .unwrap()
        .count();
    assert_eq!(backups, 10);
    let original = std::fs::read(&p).unwrap();
    d.nodes
        .get_mut(&d.root.clone())
        .unwrap()
        .text
        .edit("invalid\0");
    assert!(storage::save(&p, &d).is_err());
    assert_eq!(std::fs::read(&p).unwrap(), original);
}
#[test]
fn included_freeplane_maps_roundtrip() {
    let folder = std::path::Path::new("tests/fixtures/freeplane");
    for name in ["standard-1.6.mm", "conceptMap.mm", "BigMap.mm"] {
        let path = folder.join(name);
        let bytes = std::fs::read(&path).unwrap();
        let d = mm::read_at(&bytes, &path).unwrap();
        let out = mm::write(&d).unwrap();
        let r = mm::read_at(&out, &path).unwrap();
        assert_eq!(d.nodes.len(), r.nodes.len(), "{name}");
        assert_eq!(d.root, r.root);
    }
}

fn test_dir() -> tempfile::TempDir {
    let dir = std::path::Path::new("artifacts/tmp");
    std::fs::create_dir_all(dir).unwrap();
    tempfile::tempdir_in(dir).unwrap()
}

#[test]
fn recovery_remains_dirty_after_edit_then_undo() {
    let (saved, a, _) = sample();
    let mut recovered = saved.clone();
    recovered.nodes.get_mut(&a).unwrap().text.edit("復元内容");
    let mut history = History::restored(recovered.clone(), Some(saved));
    history
        .edit("追加編集", |d| {
            d.nodes.get_mut(&a).unwrap().text.edit("追加");
            Ok(())
        })
        .unwrap();
    history.undo();
    assert_eq!(history.doc, recovered);
    assert!(history.dirty);
    history.saved();
    assert!(!history.dirty);
}

#[test]
fn selecting_ancestor_and_grandchild_copies_only_top_branch() {
    let (mut doc, a, b) = sample();
    let c = doc.add(&b, "grandchild").unwrap();
    let selection = std::collections::BTreeSet::from([a.clone(), c]);
    assert_eq!(doc.top_selected(&selection), vec![a]);
}

#[test]
fn descendants_of_a_clone_are_also_edit_protected() {
    let (mut doc, a, b) = sample();
    doc.nodes.get_mut(&a).unwrap().protected = true;
    assert!(doc.ensure_editable(&b).is_err());
    assert!(doc.add(&b, "blocked").is_err());
}

#[test]
fn external_native_condition_edit_takes_precedence_over_extension() {
    let (mut doc, _, _) = sample();
    doc.styles.insert("red".into(), Style::default());
    doc.rules.push(Rule {
        enabled: true,
        stop: false,
        style: "red".into(),
        condition: cond(Field::Text, Operator::Contains, "needle"),
    });
    let text = String::from_utf8(mm::write(&doc).unwrap())
        .unwrap()
        .replace("VALUE=\"needle\"", "VALUE=\"changed\"");
    let modified = mm::read(text.as_bytes()).unwrap();
    assert_eq!(
        modified.rules[0].condition,
        cond(Field::Text, Operator::Contains, "changed")
    );
}

#[test]
fn new_map_shapes_icons_and_branch_endpoints_share_export_geometry() {
    let mut doc = Document::default();
    let root = doc.root.clone();
    let child = doc.add(&root, "日本語").unwrap();
    assert_eq!(doc.nodes[&root].style.shape, "oval");
    assert_eq!(doc.nodes[&child].style.shape, "fork");
    let mut cache = LayoutCache::default();
    let before = cache.build(&doc, None, |_, _| vec2(60., 20.));
    let plain = before.nodes.iter().find(|p| p.id == child).unwrap().rect;
    doc.nodes.get_mut(&child).unwrap().icons = vec!["button_ok".into(), "idea".into()];
    doc.nodes.get_mut(&child).unwrap().progress = Some(50);
    let scene = cache.build(&doc, None, |_, _| vec2(60., 20.));
    let placed = scene.nodes.iter().find(|p| p.id == child).unwrap();
    assert_eq!(
        placed.rect.height(),
        plain.height(),
        "icons must not add a footer row"
    );
    assert_eq!(
        placed.rect.width() - plain.width(),
        mindglider::layout::icon_width(3)
    );
    assert_eq!(
        mindglider::layout::branch_anchor(placed, -1.).y,
        placed.rect.bottom()
    );
    let svg = export::svg(&doc, &scene, None).unwrap();
    assert!(svg.contains(&mindglider::icons::body("button_ok")));
    assert!(svg.contains(&mindglider::icons::body("idea")));
    assert!(svg.contains(&mindglider::icons::body("50%")));
    assert!(!export::raster(&svg, "png").unwrap().is_empty());
    let restored = mm::read(&mm::write(&doc).unwrap()).unwrap();
    assert_eq!(restored.nodes[&child].progress, Some(50));
    assert_eq!(restored.nodes[&root].style.shape, "oval");
    assert_eq!(restored.nodes[&child].style.shape, "fork");
    assert_eq!(restored.nodes[&child].icons, doc.nodes[&child].icons);
}
#[test]
fn imported_icons_toggle_roundtrip_and_undo_without_losing_builtin_ids() {
    let mut d = Document::default();
    let root = d.root.clone();
    let child = d.add(&root, "icons").unwrap();
    let mut h = History::new(d);
    let targets = std::collections::BTreeSet::from([child.clone()]);
    for icon in mindglider::icons::CATALOG {
        h.edit("toggle", |d| {
            mindglider::icons::toggle(d, &targets, icon.id)
        })
        .unwrap();
        let parsed = mm::read(&mm::write(&h.doc).unwrap()).unwrap();
        assert_eq!(
            parsed.nodes[&child].icons, h.doc.nodes[&child].icons,
            "{}",
            icon.id
        );
        assert_eq!(
            parsed.nodes[&child].progress, h.doc.nodes[&child].progress,
            "{}",
            icon.id
        );
        let before_off = h.doc.clone();
        h.edit("toggle off", |d| {
            mindglider::icons::toggle(d, &targets, icon.id)
        })
        .unwrap();
        assert!(h.doc.nodes[&child].icons.is_empty());
        assert!(h.doc.nodes[&child].progress.is_none());
        h.undo();
        assert_eq!(h.doc, before_off);
        h.redo();
    }
}
#[test]
fn compact_layout_keeps_many_siblings_readable_without_overlap() {
    let mut d = Document::default();
    let root = d.root.clone();
    for _ in 0..20 {
        d.add(&root, "node").unwrap();
    }
    let scene = LayoutCache::default().build(&d, None, |_, _| vec2(50., 20.));
    let mut leaves: Vec<_> = scene.nodes.iter().filter(|p| p.id != root).collect();
    leaves.sort_by(|a, b| a.rect.top().total_cmp(&b.rect.top()));
    for p in &leaves {
        assert_eq!(p.rect.height(), 24.);
    }
    for pair in leaves.windows(2) {
        assert!((pair[1].rect.top() - pair[0].rect.bottom() - 4.).abs() < 0.01);
    }
    assert!(scene.bounds.height() < 650.);
}
#[test]
fn fork_background_covers_core_text_without_filling_image_or_icon_space() {
    let mut d = Document::default();
    let root = d.root.clone();
    let id = d.add(&root, "highlight").unwrap();
    let n = d.nodes.get_mut(&id).unwrap();
    n.style.background = "#ffe080".into();
    n.image = Some(Picture {
        uri: "assets/icons/mindglider.png".into(),
        width: 300.,
        height: 80.,
    });
    n.icons.push("idea".into());
    n.details = RichText::plain("details");
    let scene = LayoutCache::default().build(&d, None, |_, _| vec2(80., 20.));
    let p = scene.nodes.iter().find(|p| p.id == id).unwrap();
    let core = mindglider::layout::core_text_rect(&d.nodes[&id], p);
    assert_eq!(core.width(), 80.);
    assert_eq!(core.height(), 20.);
    assert!(core.left() > p.rect.left() + 16.);
    assert!(core.bottom() < p.rect.bottom());
    let svg = export::svg(&d, &scene, None).unwrap();
    assert!(svg.contains("fill=\"#ffe080\""));
    let loaded = mm::read(&mm::write(&d).unwrap()).unwrap();
    assert_eq!(loaded.nodes[&id].style.background, "#ffe080");
}
#[test]
fn mm_starts_with_known_freeplane_header_and_numeric_unicode_references() {
    let mut d = Document::default();
    let root = d.root.clone();
    d.nodes.get_mut(&root).unwrap().text = RichText::plain("日本語\n改行 & < > \" café 😀");
    let n = d.nodes.get_mut(&root).unwrap();
    n.note = RichText::plain("ノート 😀");
    n.details = RichText::plain("詳細");
    d.raw = xmltree::Element::parse(std::io::Cursor::new(
        br#"<map CUSTOM="keep" version="freeplane 1.12.1"/>"#,
    ))
    .unwrap();
    let out = mm::write(&d).unwrap();
    let text = String::from_utf8(out.clone()).unwrap();
    assert!(text.starts_with("<map version=\"freeplane 1.12.1\""));
    assert!(!text.starts_with("<?xml"));
    assert!(out.is_ascii());
    assert!(text.contains("&#x65e5;"));
    assert!(text.contains("&#x1f600;"));
    assert!(text.contains("generator=\"MindGlider"));
    let read = mm::read(&out).unwrap();
    assert_eq!(read.nodes[&root].text.text(), d.nodes[&root].text.text());
    assert_eq!(read.nodes[&root].note.text(), "ノート 😀");
    assert_eq!(read.nodes[&root].details.text(), "詳細");
    assert_eq!(read.raw.attributes["CUSTOM"], "keep");
}
#[test]
fn non_ascii_cdata_unknown_xml_and_metadata_keep_their_meaning() {
    let xml = "<map version=\"freeplane 1.12.1\"><!--日本語の注釈--><node ID=\"root\" TEXT=\"本文\"><未知 属性=\"値\"><![CDATA[日本語 < & 😀]]></未知></node></map>";
    let mut d = mm::read(xml.as_bytes()).unwrap();
    d.styles
        .insert("日本語スタイル 😀".into(), Style::default());
    let out = mm::write(&d).unwrap();
    let parsed = mm::read(&out).unwrap();
    let unknown = parsed.nodes["root"]
        .raw
        .as_ref()
        .unwrap()
        .get_child("未知")
        .unwrap();
    assert_eq!(unknown.attributes["属性"], "値");
    assert_eq!(unknown.get_text().unwrap(), "日本語 < & 😀");
    assert_eq!(parsed.styles, d.styles);
    assert!(String::from_utf8(out).unwrap().contains("日本語の注釈"));
}
#[test]
fn legacy_private_map_data_is_read_and_saved_with_new_identifiers() {
    let xml=br#"<map version="freeplane 1.12.1"><node ID="root" TEXT="legacy"><hook NAME="ExternalObject" URI="image.png" SPEEDPLANE_WIDTH="222" SPEEDPLANE_HEIGHT="111" SIZE="1"/></node></map>"#;
    let d = mm::read(xml).unwrap();
    assert_eq!(d.nodes["root"].image.as_ref().unwrap().width, 222.);
    let out = String::from_utf8(mm::write(&d).unwrap()).unwrap();
    assert!(out.contains("MINDGLIDER_WIDTH=\"222\""));
    assert!(!out.contains("SPEEDPLANE_WIDTH"));
    let read = mm::read(out.as_bytes()).unwrap();
    assert_eq!(read.nodes["root"].image, d.nodes["root"].image);
}
#[test]
fn legacy_metadata_hook_is_upgraded_without_losing_named_styles() {
    let mut doc = Document::default();
    doc.styles.insert("旧書式".into(), Style::default());
    let current = String::from_utf8(mm::write(&doc).unwrap()).unwrap();
    let legacy = current.replace("NAME=\"MindGlider\"", "NAME=\"Speedplane\"");
    let loaded = mm::read(legacy.as_bytes()).unwrap();
    assert_eq!(loaded.styles, doc.styles);
    let upgraded = String::from_utf8(mm::write(&loaded).unwrap()).unwrap();
    assert!(upgraded.contains("NAME=\"MindGlider\""));
    assert!(!upgraded.contains("NAME=\"Speedplane\""));
}
