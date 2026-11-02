use mindglider::{
    ai::{CliSettings, Provider},
    cli_catalog,
};
fn main() -> anyhow::Result<()> {
    let work = std::path::Path::new("artifacts/cli-reference").canonicalize()?;
    for provider in [Provider::Codex, Provider::Claude] {
        let settings = CliSettings {
            provider,
            ..Default::default()
        };
        let catalog = cli_catalog::discover(&settings, &work);
        let value = serde_json::json!({"provider":format!("{provider:?}"),"source":catalog.source,"warning":catalog.warning,"models":catalog.models});
        let file = work.join(format!("{provider:?}-catalog.json"));
        std::fs::write(file, serde_json::to_vec_pretty(&value)?)?;
        println!("{value}");
    }
    Ok(())
}
