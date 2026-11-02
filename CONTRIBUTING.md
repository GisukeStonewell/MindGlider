# Contributing / 貢献方法

## 日本語

初期段階の個人プロジェクトです。不具合報告・翻訳修正・小さな改善を歓迎します。大きな機能追加や仕様変更は先にIssueで相談してください。作者自身もすべての操作の組み合わせを検証できていません。

1. Windows 11とREADME記載の開発環境を用意します。
2. `cargo fmt --all -- --check`、`cargo test --locked --tests`、`cargo clippy --locked --all-targets -- -D warnings`を実行します。
3. 関連する画面の変更は、日英UIと125～200％の表示倍率を確認してください。
4. `.mm`の未知情報・ID・装飾・日本語が失われないことと、AI結果を1回のUndoで戻せることを重視してください。
5. `data`、`artifacts`、`.tools`、認証情報、個人のマップやスクリーンショットはコミットしないでください。

翻訳は`locales/ja.json`と`locales/en.json`です。キーと差し込み変数を揃えてください。Freeplane素材を変更した場合は出典・元のライセンス・変更内容を保持してください。

貢献は既存ファイルの権利表示と本プロジェクトのGPL-3.0-or-laterに従って取り扱います。AIを利用した変更も歓迎しますが、投稿者自身が内容・権利・テスト結果を確認し、PRで利用範囲を説明してください。実際のAPI利用料金を発生させるテストを必須にしません。

## English

This is an early personal project. Bug reports, translation fixes and focused improvements are welcome. Discuss large features or design changes in an Issue first. The author has not tested every combination of operations.

1. Use Windows 11 and the development environment in the README.
2. Run `cargo fmt --all -- --check`, `cargo test --locked --tests`, and `cargo clippy --locked --all-targets -- -D warnings`.
3. For UI changes, check Japanese/English and 125–200% scaling.
4. Preserve unknown `.mm` data, IDs, formatting and Unicode. Keep AI changes reversible with a single Undo.
5. Do not commit `data`, `artifacts`, `.tools`, credentials, personal maps or private screenshots.

Translations live in `locales/ja.json` and `locales/en.json`; preserve matching keys and interpolation placeholders. Retain provenance, original licenses and modification notices for upstream material.

Contributions follow existing third-party notices and this project's GPL-3.0-or-later terms. AI-assisted changes are welcome if you review their content, rights and tests and describe their scope in the PR. Paid/live AI-service calls are not required for tests.
