# Copyright and licenses / 著作権とライセンス

Copyright © 2026 GisukeStonewell and MindGlider contributors.

MindGlider is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version. It is distributed without any warranty, including merchantability or fitness for a particular purpose. See LICENSE.txt.

MindGliderはGNU GPL第3版またはそれ以降の版で再配布・変更できます。商品性・特定目的適合性を含め無保証です。正式な条文はLICENSE.txtの英語原文です。

## Freeplane

This independent AI-assisted Rust reimplementation uses Freeplane source as a behavior/file-format reference and includes modified Freeplane graphics, translated icon names and test maps. It is not endorsed by Freeplane or FreeMind. Original third-party works are not claimed as MindGlider originals or relicensed away from their original terms.

Freeplaneのソースを動作・ファイル形式の参照に使用し、変更した図柄、アイコン名の翻訳、テスト用マップを収録しています。Freeplane／FreeMindの公式製品ではなく、上流の著作物に対する権利を独占的に主張したり、元の条件を取り除いたりしません。

- Upstream / 上流: https://github.com/freeplane/freeplane
- Reference revision / 参照版: `25f7e70756c7d93b11ee9061a4134f61324be873`
- Icon provenance / 図柄: `assets/freeplane-icons/README.md`, `manifest.json`, `original/`
- Modifications / 変更: opaque gradients flattened, filters/editor metadata removed, SVG IDs namespaced, original shapes and BUILTIN identifiers retained.
- Original metadata and GPL text / 原本のメタデータとGPL条文: `assets/freeplane-icons/original/`, `assets/freeplane-icons/Freeplane-LICENSE.txt`
- Freeplane's GPLv2-or-later works are used under the GPLv3 option for this distribution. Other upstream notices remain applicable.

## Dependencies and fonts / 依存物とフォント

Rust dependency licenses and notices are listed in THIRD_PARTY.md and licenses/. Retain them when redistributing. Third-party files remain under their stated original licenses. The operating system's font files are not included. Default fonts embedded by egui retain their own upstream font licenses in the dependency notices.

Rust依存物はTHIRD_PARTY.mdとlicenses/に記載し、再配布時も権利表示を保持します。各依存物・上流ファイルはそれぞれの元のライセンスに従います。OSのフォントファイルは収録しません。eguiが内蔵する既定フォントについては、依存物に付属する元のフォントライセンスを保持します。
