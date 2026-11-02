# MindGlider 0.4.0 のデザイン方針

2026-11-02にMicrosoftのWindows／WinUI 3公式ガイドを確認し、eguiの表示層に以下を反映した。

| 項目 | 適用内容 | 公式資料 |
|---|---|---|
| 文字 | 本文14px、補足12px、セクション20px、ダイアログ見出し24px。UIの文字は基本的に左揃え | [Typography](https://learn.microsoft.com/en-us/windows/apps/design/signature-experiences/typography) |
| フォント | OSのSegoe UI Variableを優先。日本語はOSのYu Gothic UI・メイリオで補完。OSのSegoe Fluent Iconsをコマンドバーに使用 | [Typography](https://learn.microsoft.com/en-us/windows/apps/design/signature-experiences/typography)、[Segoe Fluent Icons](https://learn.microsoft.com/en-us/windows/apps/design/iconography/segoe-fluent-icons-font) |
| 角丸 | コントロール4px、ダイアログ・フライアウト8px。接する部分の角は丸めない | [Geometry](https://learn.microsoft.com/en-us/windows/apps/design/signature-experiences/geometry) |
| 余白 | 4／8pxを基本単位とし、ダイアログ外周24px、セクション間16px、通常の操作対象は高さ32px | [Alignment, margin, padding](https://learn.microsoft.com/en-us/windows/apps/develop/ui/alignment-margin-padding) |
| 色 | ライト／ダークそれぞれの中立色を基調に、選択・主操作・フォーカスへアクセントを限定 | [Color](https://learn.microsoft.com/en-us/windows/apps/design/signature-experiences/color) |
| 階層 | アプリ背景、編集キャンバス、プロパティ面、浮いたダイアログを明度・細い境界・影で区別 | [Color](https://learn.microsoft.com/en-us/windows/apps/design/signature-experiences/color) |
| ダイアログ | 左揃えの目的を示す見出し、中央配置、簡潔な本文、閉じる操作、設定保存を主操作として強調 | [Dialogs](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/dialogs-and-flyouts/dialogs) |

上部はメニューバー、CommandBarを参考にした操作列、TabViewを参考にしたドキュメントタブの順に整理する。タブの下端を編集領域へ接続し、外周の上下余白と行間を設けず、操作対象の高さ32pxは維持する。フライアウトのコマンド名とショートカットは左右へ分けて揃える。
設定は表示と言語／AI CLI／印刷／ショートカットのページに分け、縦長の一枚のフォームを避ける。項目ごとのリセットと全体リセットは維持する。
本文・補足文字と背景、アクセントボタンの文字には最低4.5:1のコントラストがあることを自動テストする。

## 実装上の範囲

Rust＋egui／wgpuを継続し、WinUIのデザイン指針を表示層に適用した。XAMLのWinUI 3コントロールやMica／Acrylicそのものを導入したものではない。
Segoe UI VariableのファイルはOSから実行時に読み込み、コピー・埋め込み・ダウンロードを行わない。eguiは既定の可変軸位置を使用し、XAMLと同じ光学サイズの自動調整やウェイトの可変軸制御までは実装していない。
既存マップの明示的なノード書式は文書データとして保持する。ノードの形・クラウド・装飾は、UIコントロールの角丸やテーマで一括上書きしない。
AI入力はユーザー指定のボタンなし・Enter実行／Esc終了を維持したコマンド入力ウィンドウとして扱う。一般的なContentDialogのボタン配置規則より、この操作指定を優先する。

## 0.5.0 の調整

ショートカット設定は入れ子のGrid内で入力欄が狭くならないよう、ラベル220px、入力欄240px、行の操作対象32pxで配置する。ホバー時の挿入位置プレビューは無効にし、入力時の通常のカーソルは維持する。
専用アプリアイコンは、紙飛行機と接続されたノードによるスピード線を組み合わせたオリジナルSVG。Windows用ICOには16～256pxの7サイズを収録し、PNGと同じ図柄をexeのリソースにも埋め込む。

## 0.6.0 の調整

メニューの下線はアクティブな項目だけに表示する。編集可能コンボボックスは一つの枠内に入力欄とドロップダウン矢印を収める。設定のスクロール領域はカードの幅を使い、スクロールバーを右端に配置する。

Freeplaneの `IconSelectionPlugin`、`IconSelectionPopupDialog`、標準スタイルの `viewer_standard.mm` を操作と形状の参照に使用。アイコンの図柄は独自に作成した98種類の簡略SVGで、色数と線幅を抑えた。FreeplaneのBUILTIN名を保持し、画面とSVG／PNG／JPEG／PDFで共通の図柄を使う。新規のルートはoval、ルートから追加するノードはforkとし、その先は親の書式を引き継ぐ。forkは背景を塗らず、枝の端点を下線に接続する。明示的な既存形状は変更しない。

## 0.7.0 の調整

0.6.0の独自図柄を廃し、Freeplaneの実画像201点を元にしたSVGへ統一。BUILTIN IDは画像の相対パスと一致させ、不透明なグラデーションを単色化し、フィルターを除去した。透明度のグラデーションは輪郭や表情を隠さないため保持する。優先度の数字などSVG内の文字にもOSフォントを使用する。

左のアイコンパネルは初期非表示。左右端の24pxハンドルまたは表示メニューで左右パネルを開閉し、settings.jsonへ状態を保存する。アイコンは選択ノードに対しクリックで付け外しする。

下線型ノードのパディングを左右4px／上下2px、兄弟間隔4px、親子間隔32pxへ縮小。楕円等の囲み形状には必要な余白を残す。アイコンは16pxと2pxの間隔で本文左に並べ、画面・画像・PDFの配置を共通化した。

Freeplane MapViewのselectRelatedNode、selectPreferredVisibleSiblingOrAncestor、selectPreferredVisibleAncestorSibling、scrollを参照。MindGliderの上下移動は表示座標と同じマップ側を基準に近い候補を選び、階層に限定しない。候補がないときは視点を押された方向へ移動し、押下状態をフレームごとに検査してOSのキーリピートが来ない間も連続スクロールする。選択先が画面外なら自動的に画面内へ入れる。

## 0.8.0 の背景とドラッグ

下線型ノードの背景は計測済み本文の矩形へ適用し、SVG／画像／PDFでも同じ矩形を使う。ドラッグする枝は通常描画と共通の描画処理で55％の不透明度にし、掴んだ位置を保ってマウスへ追随させる。

Freeplane MainView.dragOverDirection／DragOverDirectionとMNodeDropListenerを参照。左右端で子への接続、上下で兄弟の前後へ挿入する。ルートでは左右半分で接続先の側を選ぶ。ドロップ先は方向に応じたグラデーションで示し、無効な位置は赤色と禁止カーソルで示す。プレビューと適用でdnd::destinationを共有し、循環・ルート移動・保護ノードを拒否する。左右の変更は子孫へ伝播し、親子変更とまとめて1つのHistory編集として記録する。ルートへの移動で既存の側を失うmove_nodeの不具合も修正した。
