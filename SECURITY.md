# Security / セキュリティ

## 日本語

初期版のため、最新公開版を対象にベストエフォートで修正します。継続的な保守や対応期限は保証していません。

脆弱性や認証情報の漏洩は、通常の公開Issueへ投稿しないでください。GitHubリポジトリの **Security → Advisories → Report a vulnerability** を利用してください。この非公開報告機能が使えない場合は、秘密情報を含めず報告窓口を問い合わせるIssueだけを作成してください。

AI編集では、選択した枝の本文・ノート・詳細とユーザーの指示が外部CLIへ渡り、CLIの接続先へ送信される場合があります。固定プロンプトや明示的な指示に応じてCLIがローカルファイルを参照することもあります。APIキーやログイン情報はMindGliderに投稿・添付しないでください。Issueにマップや設定を添付する前に個人情報を除去してください。

## English

Security fixes are best-effort for the latest published version. Ongoing maintenance and response deadlines are not guaranteed for this initial project.

Do not post vulnerabilities or leaked credentials in public Issues. Use **Security → Advisories → Report a vulnerability** in the GitHub repository. If private reporting is unavailable, open only a request for a reporting channel without disclosing sensitive details.

AI editing passes selected branch text, notes, details and instructions to an external CLI, which may transmit them to its provider. Explicit instructions or a configured fixed prompt may also let the CLI read local files. Never attach API keys, login credentials or unsanitized maps/settings to an Issue.
