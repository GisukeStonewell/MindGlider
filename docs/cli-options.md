# CLI設定とモデル候補

2026-11-02、インストール済みCodex CLI 0.166.0の`codex exec --help`と、Claude Code 2.1.166の`claude --help`を確認。
以下はMindGlider 0.3.0の設定項目。選択中のCLIだけを表示し、値はテキスト入力、真偽はチェックボックスで編集する。モデル・effortは入力可能なコンボボックス。

| CLI | 変更可能な設定 | CLIへの渡し方 |
|---|---|---|
| 共通 | 実行ファイル、タイムアウト、固定プロンプト | 直接起動・タイマー・標準入力 |
| Codex | モデル | `--model` |
| Codex | reasoning effort | `-c model_reasoning_effort="…"` |
| Codex | サービス階層（例：fast） | `-c service_tier="…"` |
| Codex | Web検索 | `-c web_search="live"`／`"disabled"` |
| Codex | ローカルOSSモデル、ローカルプロバイダー | `--oss`、`--local-provider`（ollama／lmstudio） |
| Claude Code | モデル、effort | `--model`、`--effort` |
| Claude Code | フォールバックモデル | `--fallback-model`（カンマ区切り） |
| Claude Code | 予算上限USD | `--max-budget-usd` |
| Claude Code | 追加参照フォルダー | `--add-dir` |
| Claude Code | スキル無効化 | `--disable-slash-commands` |

空欄はオプションを渡さない。CLIが読み込む既存のユーザー設定・フック・MCPと、MindGliderが明示した設定を混同しないよう、従来どおりユーザー設定の自動読込を抑止する。既存設定のmodel・effortは候補として表示し、使用する場合は明示的に選ぶ。
`--oss`使用時のローカルモデル名は直接入力できる。クラウド用のモデルカタログはローカルランタイムにインストール済みのモデルを保証しない。

構造化出力、非対話実行、セッション保存抑止、Codexのread-only sandbox、ClaudeのRead／Glob／Grep限定など、マップをCLIに直接変更させないための必須引数は固定。任意のシェルコマンドや権限回避用の引数欄は設けない。
全引数を配列で渡し、シェルによる文字列展開は行わない。

## 実行ファイルの検出

1. `PATH`上の`codex.exe`／`claude.exe`
2. ユーザーフォルダーの`.local/bin`
3. `AppData/Local/Programs/OpenAI/Codex/bin`
4. `.cargo/bin`

検出できなければ`codex.exe`／`claude.exe`を既定にする。設定画面でパスを変更でき、既定値に戻すと再検出する。

## モデルとeffortの候補

- Codex：`CODEX_HOME`（未指定は`~/.codex`）の`config.toml`と`models_cache.json`を読む。次に起動したapp-serverへinitialize／initialized／model/listを送り、ページを最後まで取得する。
- Claude Code：`CLAUDE_CONFIG_DIR`（未指定は`~/.claude`）の`settings.json`からmodel・effortLevel・availableModelsを読み、ANTHROPIC_MODEL／CLAUDE_CODE_EFFORT_LEVELも候補に使う。次にstream-jsonでinitialize制御要求を送り、返されたmodelsとsupportedEffortLevelsを取得する。
- モデルを選ぶと、そのモデルが返したeffort候補を表示する。未知のモデルや古いCLIで情報が得られないときも直接入力できる。
- 設定画面で自動取得し、「候補を再取得」で更新できる。APIへの推論要求は行わない。取得中もUIは操作でき、取得失敗は通知してローカル候補へ戻る。
- CLIの返す候補は実行権限・契約・組織制限・課金条件を保証しない。認証情報そのものはアプリが解析・表示・保存しない。

公式の参照元：[Codex app-server](https://developers.openai.com/codex/app-server/)、[Codex設定項目](https://developers.openai.com/codex/config-reference/)、[Claude SDKのModelInfo](https://code.claude.com/docs/en/agent-sdk/typescript#modelinfo)、[Claudeモデル設定](https://code.claude.com/docs/en/model-config)。CLIの版が異なる場合は、そのPCのヘルプと問い合わせ結果を優先する。
