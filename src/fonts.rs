//! Installed fonts only: no system font data is embedded or downloaded.
pub fn system_database() -> resvg::usvg::fontdb::Database {
    let mut db = resvg::usvg::fontdb::Database::new();
    db.load_system_fonts();
    let preferred = [
        "Segoe UI Variable",
        "Segoe UI",
        "Yu Gothic UI",
        "Meiryo",
        "Arial",
    ]
    .into_iter()
    .find(|name| {
        db.query(&resvg::usvg::fontdb::Query {
            families: &[resvg::usvg::fontdb::Family::Name(name)],
            ..Default::default()
        })
        .is_some()
    })
    .unwrap_or("Arial");
    db.set_sans_serif_family(preferred);
    db
}
