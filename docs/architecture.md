# 設計

- `model`：安定ID、親子関係、原文メタデータ。編集はコピー上で検証し、成功した変更のみHistoryに追加。失敗時は無変更。
- `richtext`：文字列と装飾run。編集前後の共通部分を利用して装飾を維持。HTML原文はノードのXMLにも保持。
- `conditions`／`native`：共通条件木と順序付きスタイル評価、Freeplane条件XMLの変換。
- `mm`：未知要素を持つXMLツリーを保持し、変更された既知プロパティのみ更新。FreeplaneのMapStyleと独自拡張の両方を扱う。
- `layout`：文字計測・枝の高さをキャッシュし、共通Sceneを生成。UIでは文書変更時に再配置し、画面外ノードを描画しない。
- `export`：Scene→SVG→PNG／JPEG／PDF。テキスト形式は文書モデルから出力。PDFはXObjectの参照を振り直して複数ページへ組み立てる。
- `storage`：同一フォルダー内の一時ファイルをflushし、WindowsのMoveFileExで置換。バックアップ・復元データは元ファイルと別。
- `ai`：CLIごとの起動引数と出力エンベロープを吸収。スナップショットの範囲・指紋を検証して操作列を一括適用。
- `i18n`：外部JSON辞書と内蔵の既定辞書、言語切替・再読み込み。
- `input`：IME CommitとEnterの順序・フレーム差を吸収。
- `fonts`：OSのシステム／ユーザーフォントを読み込む。UIはSegoe UI Variableを優先し、フォントファイルは同梱しない。
- `app`／`ui_helpers`：画面、タブ、選択、マップ操作、設定、復元。

## AI入出力

入力は`instruction`、選択した`root`、`nodes`（ID、親、子の順序、本文、ノート、詳細）。外側の親IDは含めない。
出力は`operations`配列。`add`、`update`、`delete`、`move`を許可し、JSON SchemaとRust型で検証する。
`update`のnote／detailsがnullなら維持し、空文字列なら削除。既存ノードの装飾・画像・未知XMLは指定操作の対象外なら維持。
Windows Job Objectで子孫プロセスも管理し、キャンセル・タイムアウト時に終了。出力上限は16MiB、操作数は5,000。

## 将来の追加

機能IDは固定し、未採用機能はFEATURES.mdで管理する。F06／F07等の追加時はノード拡張、条件、XML入出力、UIを一緒に追加する。
汎用プラグインホストやJava実行環境は初期版に含めない。

## ノード上の編集

F2の下書きはキャンバス上に描画し、確定時に共通Historyへ1操作として渡す。IMEの確定Enterはキーボードコマンドとテキスト編集の前に除去する。
CLI用の固定プロンプトは設定に保存し、ユーザー指示・ツリーJSONを別の区切り内に渡す。プロバイダー設定はAI編集ダイアログから分離する。

## 0.3.0

`prompt_history`はユーザーの指示をUTF-8 JSON配列としてディスクへ原子的に保存する。先頭を最新とし、完全一致だけを重複として扱い、10件を超えた末尾を除く。ダイアログを開くたび読み直す。
`cli_catalog`は設定・モデルキャッシュを読み、バックグラウンドでCodex app-serverのmodel/list、Claude Codeのinitialize制御要求を問い合わせる。推論用のturnやuserメッセージは送らない。各問い合わせは15秒で打ち切り、Windows Job Objectで子孫プロセスも終了する。
認証ファイルの内容をアプリが読み出すことはない。モデル候補の取得に失敗しても既存の設定を保持し、手入力を許可する。
プロバイダーごとのモデル・effort・任意オプションは別々に保存する。旧共有model設定は選択中のCLIへ移行する。

## 0.4.0 の表示層

`theme`は共通パレット、コントロール寸法、タイポグラフィー、フライアウトとダイアログの枠を管理する。UI生成の前に適用し、eframeの初期テーマや過去のスタイルが見た目を上書きしないようにする。適用処理自体は再描画を要求しない。
`ui_helpers::install_fonts`はOSのSegoe UI Variable・Segoe UI・Yu Gothic UI・メイリオ・Segoe Fluent Iconsを優先順に登録する。既存マップの名前付きフォントの読み込みも維持する。新規ノードはSegoe UI Variableを既定とし、既存文書の書式を一括変更しない。
設定画面はページ別に表示し、固定プロンプトや印刷設定、ショートカット等の編集・既定値へのリセットを維持する。
GUIの永続化先は`data/ui`。マップ・設定・復元データと同じポータブル用データフォルダーに集約する。

## Windowsアイコン

`assets/icons/mindglider.svg`を原稿とし、`examples/generate_icon.rs`でPNGと7サイズのICOを生成する。
起動時のViewportBuilderへPNGから作ったIconDataを渡し、ウィンドウとタスクバーに使用する。
`build.rs`はWindows SDKのrc.exeで`assets/windows.rc`をコンパイルし、mindgliderバイナリーだけにアイコンとバージョンのリソースをリンクする。`RC`環境変数でコンパイラーのパスを上書きできる。
`python scripts/verify_windows_icon.py target/release/mindglider.exe`で、PE内の全7サイズがICOと一致し、256pxの画像が実行時PNGとも一致することを検査できる。

## Freeplane互換ヘッダーと文字（0.9.0）

Freeplane MFileManager.loadTreeImplは先頭1000バイトをUTF-8で読み、MapVersionInterpreter.knowsで `<map version="…` のstartsWith判定を行う。XML宣言やBOM、先頭の別属性はDEFAULT方言を選び、unknownApp／unknownURLの警告へ進む。MindGliderのwriteはXML宣言・BOMを省略し、version属性を先頭へ移す。versionは互換方言を示し、generator属性で実際の生成アプリ名と版を記録する。

MFileManagerは読み書きともStandardCharsets.UTF_8を使用する。MapWriterのuseAsciiCharsetに対応するXMLWriter.writeEncodedと同じ方針で、値・本文の非ASCII文字を数値文字参照へ変換する。非ASCIIのCDATAは同じ内容のエスケープ済みテキストへ変換し、CDATA内部で文字参照をそのまま文字列にしてしまう誤りを避ける。XML名・コメント・処理命令は保持する。

compatモジュールの旧識別子は既存マップ・環境変数の読み込み専用。書き出しは新識別子へ統一する。scripts/verify_freeplane_compat.pyはクローンの実際のMapVersionInterpreterをJDKでコンパイルし、警告・変換不要で判定されることと、日本語の意味保持を検証する。
