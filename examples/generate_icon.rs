//! Rebuild the PNG and multi-resolution Windows ICO from the editable vector.
fn main() -> anyhow::Result<()> {
    let path = std::path::Path::new("assets/icons");
    let tree = resvg::usvg::Tree::from_data(
        &std::fs::read(path.join("mindglider.svg"))?,
        &Default::default(),
    )?;
    let sizes = [16u32, 24, 32, 48, 64, 128, 256];
    let mut frames = Vec::new();
    for size in sizes {
        let mut pix = resvg::tiny_skia::Pixmap::new(size, size).unwrap();
        resvg::render(
            &tree,
            resvg::tiny_skia::Transform::from_scale(size as f32 / 256., size as f32 / 256.),
            &mut pix.as_mut(),
        );
        let png = pix.encode_png()?;
        if size == 256 {
            std::fs::write(path.join("mindglider.png"), &png)?;
        }
        frames.push(png);
    }
    let mut ico = Vec::new();
    ico.extend([0u8, 0, 1, 0, 7, 0]);
    let mut offset = 6 + sizes.len() as u32 * 16;
    for (size, png) in sizes.into_iter().zip(&frames) {
        ico.extend([size as u8, size as u8, 0, 0]);
        ico.extend(1u16.to_le_bytes());
        ico.extend(32u16.to_le_bytes());
        ico.extend((png.len() as u32).to_le_bytes());
        ico.extend(offset.to_le_bytes());
        offset += png.len() as u32;
    }
    for png in frames {
        ico.extend(png);
    }
    std::fs::write(path.join("mindglider.ico"), ico)?;
    Ok(())
}
