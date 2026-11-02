# 公開手順 / Publishing

## 日本語

初回公開先は https://github.com/GisukeStonewell/MindGlider です。
ソースは main ブランチ、バイナリは GitHub Releases に公開します。GitHub Pages は不要です。

1. Cargo.toml のバージョン、CHANGELOG.md、docs/releases/v<version>.md を日英で更新します。
2. Windows で `cargo fmt --all -- --check`、`cargo test --locked --tests`、`cargo clippy --locked --all-targets -- -D warnings` を実行します。
3. 依存変更時は `python scripts/notices.py` でライセンス一覧を再生成・確認します。`powershell -ExecutionPolicy Bypass -File scripts/build.ps1 -Package` で配布を確認します。
4. 内容をコミットして main を push し、CI 成功を確認します。公開用の作者メールは GitHub の noreply アドレスを利用します。
5. `git tag -a v<version> -m "MindGlider v<version>"`、`git push origin v<version>` を実行します。
6. Release ワークフローがテスト・ビルド・ZIP検証を行い、ソース入りZIPと SHA256SUMS.txt を付けた下書きプレリリースを作ります。
7. 実行結果と添付ファイルを確認し、GitHub の Releases で下書きを公開します。初期版はプレリリースの表示を維持します。

ZIPは毎回新規作業フォルダから作り、実行時の data/、認証情報、AI履歴を含めません。配布ZIPには対応ソースとライセンスを同梱します。
GitHubがタグから生成する Source code ZIP には実行ファイルは含まれません。利用者向けは MindGlider-<version>-windows-x64.zip です。
SHA-256は `Get-FileHash <zip> -Algorithm SHA256` で照合できます。現状バイナリにコード署名はありません。
公開済みのタグと配布物を差し替えず、修正時は新しいバージョンを公開します。

## English

Sources are published on main at https://github.com/GisukeStonewell/MindGlider; binaries use GitHub Releases. GitHub Pages is not required.

1. Update Cargo.toml, CHANGELOG.md and bilingual docs/releases/v<version>.md.
2. Run `cargo fmt --all -- --check`, `cargo test --locked --tests` and `cargo clippy --locked --all-targets -- -D warnings` on Windows.
3. After dependency changes regenerate and review notices using `python scripts/notices.py`. Build and validate with `powershell -ExecutionPolicy Bypass -File scripts/build.ps1 -Package`.
4. Commit, push main and check CI. Use your GitHub noreply address for public commits.
5. Create an annotated tag with `git tag -a v<version> -m "MindGlider v<version>"` and `git push origin v<version>`.
6. The Release workflow tests, builds and validates the ZIP, then creates a draft prerelease with corresponding source and SHA256SUMS.txt.
7. Inspect the workflow and assets, then publish the draft from GitHub Releases. Keep the prerelease label for preview versions.

Packaging starts from a fresh directory, excluding runtime settings, credentials and AI history. The portable ZIP includes corresponding source and license texts.
GitHub's automatic Source code archives contain no executable; users should download MindGlider-<version>-windows-x64.zip.
Verify SHA-256 using `Get-FileHash <zip> -Algorithm SHA256`. Executables are currently unsigned.
Never replace published tags or assets; publish a new version for fixes.
