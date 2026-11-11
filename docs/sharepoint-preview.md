# SharePoint共同編集プレビュー

ローカルの `.mm` はMicrosoft 365へのサインインなしで使えます。SharePointマップを明示的に開いたときは、Windowsの職場アカウントで自動認証を試します。「設定 → 表示」でSharePoint接続をオフにできます。

## 組織でのセットアップ

1. Microsoft Entraに組織用のパブリッククライアントを登録します。「モバイルとデスクトップ アプリケーション」に `ms-appx-web://microsoft.aad.brokerplugin/<アプリケーションID>` を登録し、Microsoft Graphの委任権限 `Files.ReadWrite` を追加します。必要な同意を済ませ、利用者に試験用 `.mm` の権限を付与します。 [WAM設定手順](https://learn.microsoft.com/en-us/entra/msal/dotnet/acquiring-tokens/desktop-mobile/wam)
2. リリースのソースを展開し、.NET 10 SDKで認証コンポーネントをビルドします。WAMのネイティブ依存はNuGetから各環境で取得します。以下の出力先は `mindglider.exe` を展開した場所に合わせてください。

   ```powershell
   dotnet publish sharepoint/MindGlider.SharePoint.csproj -c Release -r win-x64 --self-contained true -o C:\Apps\MindGlider\sharepoint
   .\scripts\register-sharepoint-protocol.ps1 -Executable C:\Apps\MindGlider\mindglider.exe
   ```

   URI登録は現在のWindowsユーザーに適用されます。登録解除は `.\scripts\register-sharepoint-protocol.ps1 -Unregister` です。
3. MindGliderのデータディレクトリ（既定は実行ファイル横の `data`）に `sharepoint.json` を作成し、組織の実際のGUIDを設定します。

   ```json
   {
     "tenant_id": "52d13ec3-244a-44b6-9072-19cfa0eaac05",
     "client_id": "16bd6718-d734-4c6a-a85e-136714413385"
   }
   ```

4. SharePoint管理者がリリース添付の `mindglider-sharepoint.sppkg` をアプリカタログにアップロードし、SharePoint管理センターのAPIアクセスで拡張が要求するMicrosoft Graphの `Sites.Read.All` を承認します。この権限は、選んだリスト行をドライブとファイルのIDに変換するために使います。対象サイトにアプリを追加すると、ドキュメントライブラリで単一の `.mm` を選択したとき、そのファイルのメニューに「MindGliderで開く」が表示されます。ブラウザーがデスクトップアプリ起動の確認を表示することがあります。 [SPFxコマンドの配備](https://learn.microsoft.com/en-us/sharepoint/dev/spfx/extensions/get-started/building-simple-cmdset-with-dialog-api)
5. 拡張の配備前でも、`mindglider://sharepoint?tenant=<テナントGUID>&drive=<ドライブID>&item=<アイテムID>` を「ファイル → SharePointから開く」に貼り付けて接続を試せます。

## 自動保存と競合

- 共有タブは自動保存オンで開き、確定した編集を数秒間隔で保存します。「ファイル → SharePoint自動保存を切り替える」で一時停止でき、`Ctrl+S` で手動保存できます。自動保存オフでも他の利用者の更新を取得します。
- 同じ版から別々の枝や属性を編集した場合は自動統合します。同じ属性の編集、削除と編集の衝突、同じ兄弟順の変更などは自動保存を停止します。「名前を付けて保存」で手元の編集をローカルに残し、共有マップを開き直してください。「SharePointの版を読み直す」を選ぶと、確認後に未保存編集を破棄します。
- 他の利用者の更新を反映した際は、取り込み前のUndo/Redo履歴をクリアします。画像とローカルファイル参照は共有タブでは読み込みません。閉じる操作で未保存の共有タブを保存する場合はローカルコピーを作ります。
- 未送信の編集はデータディレクトリの `recovery-*.mm` に保管します。異常終了後は更新時刻を見て該当ファイルをローカルで開けます。通常の保存成功や明示的な破棄時に削除します。会社の端末で復元データを扱う際は、組織が許可するデータディレクトリを利用してください。

## 検証範囲

仮想SharePointで読み込み、条件付き取得、保存、古い版からの保存拒否、アップロード途中の競合を試験しました。実テナントで初めて使う場合は、バージョン履歴を有効にした試験ライブラリで、2アカウントによる読み書きと条件付き確定を確認してください。テナントの権限、条件付きアクセス、チェックアウト設定によって利用できる操作は変わります。

## English setup summary

Local maps work without Microsoft 365 sign-in. Register a single-tenant Entra public client with the WAM redirect URI and delegated Graph `Files.ReadWrite`. Build the optional WAM component from the release source using the .NET 10 SDK and place its output under `sharepoint` beside `mindglider.exe`. Register the Windows URI with the included source script and configure `tenant_id` and `client_id` in `data/sharepoint.json`. Upload the SPFx package to the app catalog, approve its Graph `Sites.Read.All` request and install the app on the target SharePoint site.

Shared maps use AutoSave by default and poll for remote revisions. Conflicting edits stay local until you save a copy or confirm reloading the shared version. The preview was tested against a simulated service. Use a versioned test library to verify conditional commits in your tenant before using production files.
