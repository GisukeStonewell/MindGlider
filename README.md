# MindGlider

[English](README.en.md) | 日本語

**Freeplaneを、AIを活用しながら必要な機能に絞ってRustへ移植・再実装した、Windows 11向けのマインドマップアプリです。** Rust＋egui／eframe＋wgpuで開発し、Javaの実行環境を必要としません。

> **初回公開の試用版です。移植した作者自身もすべての機能・操作の組み合わせを試したわけではありません。** 不具合や互換性の不足が残る可能性があります。重要なマップは元ファイルのコピーでお試しください。

## ダウンロード

[GitHub Releases](https://github.com/GisukeStonewell/MindGlider/releases)から`MindGlider-0.11.0-windows-x64.zip`をダウンロードし、書き込み可能な場所へ展開して`mindglider.exe`を起動します。ローカル編集にJava・Rust・Microsoft 365へのサインインは不要です。ZIPのSHA-256は同じリリースの`SHA256SUMS.txt`で確認できます。

組織のSharePoint Onlineで共同編集を試す場合は、認証コンポーネントとSharePoint側の設定が必要です。[セットアップ手順](docs/sharepoint-preview.md)

「表示 → テーマ」から10色を選択できます。候補にマウスを重ねると一時的に画面へ反映され、クリックすると保存されます。対応するソースコードは同じリリースの「Source code」から取得できます。

対象は**Windows 11 x64**です。初回公開版はGitHub上でPre-release（試用版）として扱います。通常のマインドマップ編集にAI用CLIは不要です。

## 主な機能

- Freeplaneの`.mm`ファイルと互換の読み込み・保存。XMLの未対応情報や未編集HTMLの保持を重視しています。
- ノード編集、親子・兄弟の変更、左右配置、折り畳み、複数選択、ドラッグ、Undo／Redo、検索・置換。
- 書式、部分的な文字装飾、ノート・詳細、ローカル画像、Freeplane由来のアイコン、関連線・クラウド。
- 条件付きスタイル・フィルター、複数タブ、作業再開、自動保存とバックアップ。
- TXT／Markdown／OPML／PNG／JPEG／SVG／PDF／HTMLへの出力。
- 日本語・EnglishのUI。翻訳は外部JSONファイルで編集できます。

**Freeplaneの全機能を再現したものではなく、完全な相互互換性を保証するものでもありません。** クローン・暗号化などの未対応構造は編集を制限します。詳細は[機能対応表](FEATURES.md)をご確認ください。Freeplane／FreeMindプロジェクトの公式製品・公式移植ではありません。

## CLIによるAIツリー編集

ローカルにインストールした**CodexまたはClaude CodeのCLIを呼び出し、選択したノードとその子孫をAIで更新する機能を追加しています。**

1. 利用するCLIを別途インストールし、そのCLIの通常の手順でログインします。
2. MindGliderの設定でCLI、実行ファイル、モデル等を選びます。固定プロンプトも編集できます。
3. 枝を選んで**F12**を押し、指示を入力してEnterで実行します。
4. 対象範囲・構造・競合の検証に成功すると一括反映します。**1回のUndoで戻せます。**

**MindGliderがAIサービスのAPIを直接呼び出す機能は搭載していません。** APIキーをMindGliderへ登録する方式もありません。AIサービスとの通信・認証・利用料金等は各CLIとその接続先の条件に従います。

選択した枝の本文・ノート・詳細・指示がCLIに渡ります。画像本体、選択外の枝、リンク先のローカルファイル内容は既定では渡しません。ただし、ユーザーが指示したナレッジファイル等をCLIが読むことはあります。履歴・設定・復元データは`data`に保存されます。共有時には個人情報や秘密を含まないことを確認してください。

[操作説明](docs/usage.ja.md) | [CLI設定](docs/cli-options.md) | [機能対応表](FEATURES.md)

## 基本操作

| 操作 | キー／操作 |
|---|---|
| 子／兄弟を追加 | Tab／Enter |
| ノード上で本文を編集 | F2／ダブルクリック |
| 改行／確定／取り消し | Shift+Enter／Enter／Esc |
| アイコンを選ぶ | Ctrl+F2 |
| AIで選択した枝を編集 | F12 |
| Undo／Redo | Ctrl+Z／Ctrl+Y |
| 保存／名前を付けて保存 | Ctrl+S／Ctrl+Shift+S |
| 検索／コマンド検索 | Ctrl+F／Ctrl+P |
| マップを移動／拡大縮小 | 背景の左ドラッグ／ホイール |

設定・復元データは実行ファイル横の`data`に保存します。`MINDGLIDER_DATA_DIR`で保存先を指定できます。UIフォントはOSのSegoe UI Variable等を使い、システムフォントを再配布しません。

## ソースからのビルド

WindowsのVisual Studio C++ Build ToolsとWindows SDK、Rust（`rust-toolchain.toml`指定版）、Python 3.11以降が必要です。

```powershell
cargo test --locked --tests
cargo clippy --locked --all-targets -- -D warnings
./scripts/build.ps1 -Package
```

出力は`dist`です。FreeplaneのクローンやJDKは通常のビルドには不要です。Freeplane由来の素材を再取り込みする場合や、上流Javaソースを用いた任意の互換検証には別途必要です。

## ライセンスとクレジット

MindGliderは**GPL-3.0-or-later**で公開します。[LICENSE.txt](LICENSE.txt)／[NOTICE.md](NOTICE.md)を参照してください。Freeplane由来の図柄・翻訳・テスト用マップと、Rust依存ライブラリの権利表示・ライセンスは引き継ぎます。[THIRD_PARTY.md](THIRD_PARTY.md)、[アイコンの出典と変更内容](assets/freeplane-icons/README.md)もご確認ください。

Copyright © 2026 GisukeStonewell and MindGlider contributors.

## 不具合報告・貢献

再現手順、バージョン、Windowsの表示倍率などを添えて[Issues](https://github.com/GisukeStonewell/MindGlider/issues)へ報告してください。実際の個人マップ、AI指示履歴、CLI認証情報は投稿せず、再現用の最小サンプルをお使いください。[貢献方法](CONTRIBUTING.md)／[セキュリティ報告](SECURITY.md)。
