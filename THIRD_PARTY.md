# Third-party components

Freeplane由来のアイコンと翻訳には元の権利表記を保持しています。出典と変更内容は assets/freeplane-icons/README.md を参照してください。
Freeplane icons and translated labels retain their upstream attribution. See assets/freeplane-icons/README.md.

依存ライブラリのバージョンは Cargo.lock に固定し、取得できたライセンス文書を licenses/ に収録しています。OSのフォントは再配布しません。egui内蔵フォントのライセンスは licenses/epaint_default_fonts-0.31.1/fonts/ にあります。

Versions are pinned in Cargo.lock. License texts supplied by the packages are in licenses/.

Windows system fonts are loaded from the user's Windows installation and are not redistributed.

| Package | Version | Declared license | License files |
|---|---|---|---|
| ab_glyph | 0.2.32 | Apache-2.0 | LICENSE |
| ab_glyph_rasterizer | 0.1.10 | Apache-2.0 | LICENSE |
| accesskit | 0.17.1 | MIT OR Apache-2.0 | Declared in Cargo manifest |
| accesskit_consumer | 0.26.0 | MIT OR Apache-2.0 | Declared in Cargo manifest |
| accesskit_windows | 0.24.1 | MIT OR Apache-2.0 | Declared in Cargo manifest |
| accesskit_winit | 0.23.1 | Apache-2.0 | Declared in Cargo manifest |
| adler2 | 2.0.1 | 0BSD OR MIT OR Apache-2.0 | LICENSE-0BSD, LICENSE-APACHE, LICENSE-MIT |
| ahash | 0.8.12 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| aho-corasick | 1.1.5 | Unlicense OR MIT | COPYING, LICENSE-MIT |
| anyhow | 1.0.104 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| arboard | 3.7.0 | MIT OR Apache-2.0 | LICENSE-APACHE.txt, LICENSE-MIT.txt |
| arrayref | 0.3.9 | BSD-2-Clause | LICENSE |
| arrayvec | 0.7.8 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| ash | 0.38.0+1.3.281 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| autocfg | 1.5.1 | Apache-2.0 OR MIT | LICENSE-APACHE, LICENSE-MIT |
| base64 | 0.21.7 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| base64 | 0.22.1 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| bit-set | 0.8.0 | Apache-2.0 OR MIT | LICENSE-APACHE, LICENSE-MIT |
| bit-vec | 0.8.0 | Apache-2.0 OR MIT | LICENSE-APACHE, LICENSE-MIT |
| bitflags | 1.3.2 | MIT/Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| bitflags | 2.13.2 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| bytemuck | 1.25.2 | Zlib OR Apache-2.0 OR MIT | LICENSE-APACHE, LICENSE-MIT, LICENSE-ZLIB |
| bytemuck_derive | 1.12.1 | Zlib OR Apache-2.0 OR MIT | LICENSE-APACHE, LICENSE-MIT, LICENSE-ZLIB |
| byteorder-lite | 0.1.0 | Unlicense OR MIT | LICENSE-MIT |
| cfg-if | 1.0.5 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| cfg_aliases | 0.2.2 | MIT | LICENSE, NOTICES.md |
| clipboard-win | 5.4.3 | BSL-1.0 | Declared in Cargo manifest |
| codespan-reporting | 0.11.1 | Apache-2.0 | Declared in Cargo manifest |
| color_quant | 1.1.0 | MIT | LICENSE |
| core_maths | 0.1.1 | MIT | LICENSE |
| crc32fast | 1.5.4 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| cursor-icon | 1.2.0 | MIT OR Apache-2.0 OR Zlib | LICENSE-APACHE, LICENSE-MIT, LICENSE-ZLIB |
| data-url | 0.3.2 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| displaydoc | 0.2.7 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| document-features | 0.2.12 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| dpi | 0.1.2 | Apache-2.0 AND MIT | LICENSE, LICENSE-LIBM-MIT |
| ecolor | 0.31.1 | MIT OR Apache-2.0 | Declared in Cargo manifest |
| eframe | 0.31.1 | MIT OR Apache-2.0 | Declared in Cargo manifest |
| egui | 0.31.1 | MIT OR Apache-2.0 | Declared in Cargo manifest |
| egui-wgpu | 0.31.1 | MIT OR Apache-2.0 | Declared in Cargo manifest |
| egui-winit | 0.31.1 | MIT OR Apache-2.0 | Declared in Cargo manifest |
| egui_extras | 0.31.1 | MIT OR Apache-2.0 | Declared in Cargo manifest |
| emath | 0.31.1 | MIT OR Apache-2.0 | Declared in Cargo manifest |
| enum-map | 2.7.3 | MIT OR Apache-2.0 | LICENSES/Apache-2.0.txt, LICENSES/CC0-1.0.txt, LICENSES/MIT.txt, licenses/Apache-2.0.txt, licenses/CC0-1.0.txt, licenses/MIT.txt |
| enum-map-derive | 0.17.0 | MIT OR Apache-2.0 | LICENSES/Apache-2.0.txt, LICENSES/CC0-1.0.txt, LICENSES/MIT.txt, licenses/Apache-2.0.txt, licenses/CC0-1.0.txt, licenses/MIT.txt |
| enumn | 0.1.14 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| epaint | 0.31.1 | MIT OR Apache-2.0 | Declared in Cargo manifest |
| epaint_default_fonts | 0.31.1 | (MIT OR Apache-2.0) AND OFL-1.1 AND Ubuntu-font-1.0 | fonts/emoji-icon-font-mit-license.txt, fonts/Hack-Regular.txt, fonts/OFL.txt, fonts/UFL.txt |
| equivalent | 1.0.3 | Apache-2.0 OR MIT | LICENSE-APACHE, LICENSE-MIT |
| error-code | 3.4.0 | BSL-1.0 | LICENSE |
| euclid | 0.22.14 | MIT OR Apache-2.0 | COPYRIGHT, LICENSE-APACHE, LICENSE-MIT |
| fastrand | 2.5.0 | Apache-2.0 OR MIT | LICENSE-APACHE, LICENSE-MIT |
| fax | 0.2.7 | MIT | LICENSE |
| fdeflate | 0.3.7 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| flate2 | 1.1.10 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| float-cmp | 0.9.0 | MIT | LICENSE |
| foldhash | 0.1.5 | Zlib | LICENSE |
| font-types | 0.11.3 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| fontdb | 0.23.0 | MIT | LICENSE |
| form_urlencoded | 1.2.2 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| getrandom | 0.2.17 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| getrandom | 0.4.3 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| gif | 0.13.3 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| gif | 0.14.2 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| gl_generator | 0.14.0 | Apache-2.0 | Declared in Cargo manifest |
| glow | 0.16.0 | MIT OR Apache-2.0 OR Zlib | LICENSE-APACHE, LICENSE-MIT, LICENSE-ZLIB |
| glutin_wgl_sys | 0.6.1 | Apache-2.0 | LICENSE |
| gpu-alloc | 0.6.2 | MIT OR Apache-2.0 | Declared in Cargo manifest |
| gpu-alloc-types | 0.3.1 | MIT OR Apache-2.0 | Declared in Cargo manifest |
| gpu-descriptor | 0.3.2 | MIT OR Apache-2.0 | Declared in Cargo manifest |
| gpu-descriptor-types | 0.2.0 | MIT OR Apache-2.0 | Declared in Cargo manifest |
| half | 2.7.1 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| hashbrown | 0.15.5 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| hashbrown | 0.17.1 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| heck | 0.5.0 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| hexf-parse | 0.2.1 | CC0-1.0 | Declared in Cargo manifest |
| home | 0.5.12 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| icu_collections | 2.4.0 | Unicode-3.0 | LICENSE |
| icu_locale_core | 2.4.0 | Unicode-3.0 | LICENSE |
| icu_normalizer | 2.4.0 | Unicode-3.0 | LICENSE |
| icu_normalizer_data | 2.4.0 | Unicode-3.0 | LICENSE |
| icu_properties | 2.4.0 | Unicode-3.0 | LICENSE |
| icu_properties_data | 2.4.0 | Unicode-3.0 | LICENSE |
| icu_provider | 2.4.0 | Unicode-3.0 | LICENSE |
| idna | 1.1.0 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| idna_adapter | 1.2.2 | Apache-2.0 OR MIT | LICENSE-APACHE, LICENSE-MIT |
| image | 0.25.10 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| image-webp | 0.2.4 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| imagesize | 0.13.0 | MIT | LICENSE |
| immutable-chunkmap | 2.1.5 | Apache-2.0 OR MIT | LICENSE-APACHE, LICENSE-MIT |
| indexmap | 2.14.2 | Apache-2.0 OR MIT | LICENSE-APACHE, LICENSE-MIT |
| itoa | 1.0.18 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| khronos-egl | 6.0.0 | MIT/Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| khronos_api | 3.1.0 | Apache-2.0 | Declared in Cargo manifest |
| kurbo | 0.11.3 | Apache-2.0 OR MIT | LICENSE-APACHE, LICENSE-MIT |
| kurbo | 0.13.1 | Apache-2.0 OR MIT | LICENSE-APACHE, LICENSE-MIT |
| libc | 0.2.191 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| libloading | 0.8.9 | ISC | LICENSE |
| libm | 0.2.16 | MIT | LICENSE.txt |
| litemap | 0.8.3 | Unicode-3.0 | LICENSE |
| litrs | 1.0.1 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| lock_api | 0.4.14 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| log | 0.4.35 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| memchr | 2.8.3 | Unlicense OR MIT | COPYING, LICENSE-MIT |
| memmap2 | 0.9.12 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| mime | 0.3.17 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| mime_guess2 | 2.3.1 | MIT | LICENSE |
| miniz_oxide | 0.8.9 | MIT OR Zlib OR Apache-2.0 | LICENSE, LICENSE-APACHE.md, LICENSE-MIT.md, LICENSE-ZLIB.md |
| miniz_oxide | 0.9.1 | MIT OR Zlib OR Apache-2.0 | LICENSE, LICENSE-APACHE.md, LICENSE-MIT.md, LICENSE-ZLIB.md |
| moxcms | 0.8.1 | BSD-3-Clause OR Apache-2.0 | LICENSE-APACHE.md, LICENSE.md |
| naga | 24.0.0 | MIT OR Apache-2.0 | Declared in Cargo manifest |
| nohash-hasher | 0.2.0 | Apache-2.0 OR MIT | LICENSE-APACHE, LICENSE-MIT |
| num-traits | 0.2.19 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| once_cell | 1.21.4 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| ordered-float | 4.6.0 | MIT | LICENSE-MIT |
| owned_ttf_parser | 0.25.1 | Apache-2.0 | LICENSE |
| parking_lot | 0.12.5 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| parking_lot_core | 0.9.12 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| paste | 1.0.15 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| pdf-writer | 0.12.1 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| percent-encoding | 2.3.2 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| phf | 0.11.3 | MIT | LICENSE |
| phf_generator | 0.11.3 | MIT | LICENSE |
| phf_macros | 0.11.3 | MIT | LICENSE |
| phf_shared | 0.11.3 | MIT | LICENSE |
| pico-args | 0.5.0 | MIT | LICENSE |
| pin-project-lite | 0.2.17 | Apache-2.0 OR MIT | LICENSE-APACHE, LICENSE-MIT |
| pkg-config | 0.3.34 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| png | 0.17.16 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| png | 0.18.1 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| pollster | 0.4.0 | Apache-2.0/MIT | LICENSE-APACHE, LICENSE-MIT |
| polycool | 0.4.0 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| potential_utf | 0.1.6 | Unicode-3.0 | LICENSE |
| ppv-lite86 | 0.2.21 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| proc-macro2 | 1.0.107 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| profiling | 1.0.18 | MIT OR Apache-2.0 | Declared in Cargo manifest |
| pxfm | 0.1.30 | BSD-3-Clause OR Apache-2.0 | LICENSE-APACHE.md, LICENSE.md |
| quick-error | 2.0.1 | MIT/Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| quote | 1.0.47 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| rand | 0.8.8 | MIT OR Apache-2.0 | COPYRIGHT, LICENSE-APACHE, LICENSE-MIT |
| rand_chacha | 0.3.1 | MIT OR Apache-2.0 | COPYRIGHT, LICENSE-APACHE, LICENSE-MIT |
| rand_core | 0.6.4 | MIT OR Apache-2.0 | COPYRIGHT, LICENSE-APACHE, LICENSE-MIT |
| raw-window-handle | 0.6.2 | MIT OR Apache-2.0 OR Zlib | LICENSE-APACHE.md, LICENSE-MIT.md, LICENSE-ZLIB.md |
| read-fonts | 0.39.2 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| regex | 1.13.1 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| regex-automata | 0.4.18 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| regex-syntax | 0.8.11 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| renderdoc-sys | 1.1.0 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| resvg | 0.45.1 | Apache-2.0 OR MIT | LICENSE-APACHE, LICENSE-MIT |
| rfd | 0.15.4 | MIT | LICENSE |
| rgb | 0.8.53 | MIT | LICENSE |
| ron | 0.8.1 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| roxmltree | 0.20.0 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| rustc-hash | 1.1.0 | Apache-2.0/MIT | LICENSE-APACHE, LICENSE-MIT |
| rustc-hash | 2.1.3 | Apache-2.0 OR MIT | LICENSE-APACHE, LICENSE-MIT |
| rustversion | 1.0.23 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| rustybuzz | 0.20.1 | MIT | LICENSE |
| ryu | 1.0.23 | Apache-2.0 OR BSL-1.0 | LICENSE-APACHE, LICENSE-BOOST |
| scopeguard | 1.2.0 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| serde | 1.0.229 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| serde_core | 1.0.229 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| serde_derive | 1.0.229 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| serde_json | 1.0.154 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| simd-adler32 | 0.3.10 | MIT | LICENSE.md |
| simplecss | 0.2.2 | Apache-2.0 OR MIT | LICENSE-APACHE, LICENSE-MIT |
| siphasher | 1.0.4 | MIT OR Apache-2.0 | COPYING, LICENSE-APACHE, LICENSE-MIT |
| skrifa | 0.42.1 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| slotmap | 1.1.1 | Zlib | LICENSE |
| smallvec | 1.16.4 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| smol_str | 0.2.2 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| spirv | 0.3.0+sdk-1.3.268.0 | Apache-2.0 | Declared in Cargo manifest |
| stable_deref_trait | 1.2.1 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| static_assertions | 1.1.0 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| strict-num | 0.1.1 | MIT | LICENSE |
| strum | 0.26.3 | MIT | LICENSE |
| strum_macros | 0.26.4 | MIT | LICENSE |
| subsetter | 0.2.6 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT, NOTICE |
| svg2pdf | 0.13.0 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT, NOTICE |
| svgtypes | 0.15.3 | Apache-2.0 OR MIT | LICENSE-APACHE, LICENSE-MIT |
| syn | 2.0.119 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| syn | 3.0.9 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| synstructure | 0.14.0 | MIT | LICENSE |
| tempfile | 3.27.0 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| termcolor | 1.4.1 | Unlicense OR MIT | COPYING, LICENSE-MIT |
| thiserror | 1.0.69 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| thiserror | 2.0.21 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| thiserror-impl | 1.0.69 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| thiserror-impl | 2.0.21 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| tiff | 0.11.3 | MIT | LICENSE |
| tiny-skia | 0.11.4 | BSD-3-Clause | LICENSE |
| tiny-skia-path | 0.11.4 | BSD-3-Clause | LICENSE |
| tinystr | 0.8.5 | Unicode-3.0 | LICENSE |
| tinyvec | 1.13.3 | Zlib OR Apache-2.0 OR MIT | LICENSE-APACHE.md, LICENSE-MIT.md, LICENSE-ZLIB.md |
| toml_datetime | 1.1.2+spec-1.1.0 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| toml_edit | 0.25.17+spec-1.1.0 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| toml_parser | 1.1.5+spec-1.1.0 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| tracing | 0.1.44 | MIT | LICENSE |
| tracing-attributes | 0.1.31 | MIT | LICENSE |
| tracing-core | 0.1.36 | MIT | LICENSE |
| ttf-parser | 0.25.1 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| type-map | 0.5.1 | MIT/Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| unicase | 2.10.0 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| unicode-bidi | 0.3.18 | MIT OR Apache-2.0 | COPYRIGHT, LICENSE-APACHE, LICENSE-MIT |
| unicode-bidi-mirroring | 0.4.0 | MIT/Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| unicode-ccc | 0.4.0 | MIT/Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| unicode-ident | 1.0.27 | (MIT OR Apache-2.0) AND Unicode-3.0 | LICENSE-APACHE, LICENSE-MIT, LICENSE-UNICODE |
| unicode-properties | 0.1.5 | (MIT OR Apache-2.0) AND Unicode-3.0 | COPYRIGHT, LICENSE-APACHE, LICENSE-MIT, LICENSE-UNICODE |
| unicode-script | 0.5.8 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| unicode-segmentation | 1.13.3 | MIT OR Apache-2.0 | COPYRIGHT, LICENSE-APACHE, LICENSE-MIT |
| unicode-vo | 0.1.0 | MIT/Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| unicode-width | 0.1.14 | MIT OR Apache-2.0 | COPYRIGHT, LICENSE-APACHE, LICENSE-MIT |
| unicode-xid | 0.2.6 | MIT OR Apache-2.0 | COPYRIGHT, LICENSE-APACHE, LICENSE-MIT |
| url | 2.5.8 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| usvg | 0.45.1 | Apache-2.0 OR MIT | LICENSE-APACHE, LICENSE-MIT |
| utf8_iter | 1.1.0 | Apache-2.0 OR MIT | COPYRIGHT, LICENSE-APACHE, LICENSE-MIT |
| utf_types | 1.0.0 | Apache-2.0 OR MIT | COPYRIGHT, LICENSE-APACHE, LICENSE-MIT |
| uuid | 1.29.0 | Apache-2.0 OR MIT | LICENSE-APACHE, LICENSE-MIT |
| version_check | 0.9.5 | MIT/Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| web-time | 1.1.0 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| webbrowser | 1.2.6 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| weezl | 0.1.12 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| wgpu | 24.0.5 | MIT OR Apache-2.0 | LICENSE.APACHE, LICENSE.MIT |
| wgpu-core | 24.0.5 | MIT OR Apache-2.0 | LICENSE.APACHE, LICENSE.MIT |
| wgpu-hal | 24.0.4 | MIT OR Apache-2.0 | LICENSE.APACHE, LICENSE.MIT |
| wgpu-types | 24.0.0 | MIT OR Apache-2.0 | LICENSE.APACHE, LICENSE.MIT |
| winapi | 0.3.9 | MIT/Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| winapi-util | 0.1.11 | Unlicense OR MIT | COPYING, LICENSE-MIT |
| windows | 0.58.0 | MIT OR Apache-2.0 | license-apache-2.0, license-mit |
| windows-core | 0.58.0 | MIT OR Apache-2.0 | license-apache-2.0, license-mit |
| windows-implement | 0.58.0 | MIT OR Apache-2.0 | license-apache-2.0, license-mit |
| windows-interface | 0.58.0 | MIT OR Apache-2.0 | license-apache-2.0, license-mit |
| windows-link | 0.2.1 | MIT OR Apache-2.0 | license-apache-2.0, license-mit |
| windows-result | 0.2.0 | MIT OR Apache-2.0 | license-apache-2.0, license-mit |
| windows-strings | 0.1.0 | MIT OR Apache-2.0 | license-apache-2.0, license-mit |
| windows-sys | 0.52.0 | MIT OR Apache-2.0 | license-apache-2.0, license-mit |
| windows-sys | 0.59.0 | MIT OR Apache-2.0 | license-apache-2.0, license-mit |
| windows-sys | 0.61.2 | MIT OR Apache-2.0 | license-apache-2.0, license-mit |
| windows-targets | 0.52.6 | MIT OR Apache-2.0 | license-apache-2.0, license-mit |
| windows_x86_64_msvc | 0.52.6 | MIT OR Apache-2.0 | license-apache-2.0, license-mit |
| winit | 0.30.13 | Apache-2.0 | LICENSE |
| winnow | 1.0.4 | MIT | LICENSE-MIT |
| write-fonts | 0.48.1 | MIT OR Apache-2.0 | LICENSE-APACHE, LICENSE-MIT |
| writeable | 0.6.4 | Unicode-3.0 | LICENSE |
| xml-rs | 0.8.29 | MIT | LICENSE |
| xmltree | 0.11.0 | MIT | LICENSE |
| xmlwriter | 0.1.0 | MIT | LICENSE |
| yoke | 0.8.4 | Unicode-3.0 | LICENSE |
| yoke-derive | 0.8.4 | Unicode-3.0 | LICENSE |
| zerocopy | 0.8.63 | BSD-2-Clause OR Apache-2.0 OR MIT | LICENSE-APACHE, LICENSE-BSD, LICENSE-MIT |
| zerocopy-derive | 0.8.63 | BSD-2-Clause OR Apache-2.0 OR MIT | LICENSE-APACHE, LICENSE-BSD, LICENSE-MIT |
| zerofrom | 0.1.8 | Unicode-3.0 | LICENSE |
| zerofrom-derive | 0.1.8 | Unicode-3.0 | LICENSE |
| zerotrie | 0.2.5 | Unicode-3.0 | LICENSE |
| zerovec | 0.11.9 | Unicode-3.0 | LICENSE |
| zerovec-derive | 0.11.7 | Unicode-3.0 | LICENSE |
| zlib-rs | 0.6.8 | Zlib | LICENSE |
| zmij | 1.0.23 | MIT | LICENSE-MIT |
| zune-core | 0.4.12 | MIT OR Apache-2.0 OR Zlib | Declared in Cargo manifest |
| zune-core | 0.5.3 | MIT OR Apache-2.0 OR Zlib | LICENSE-APACHE, LICENSE-MIT, LICENSE-ZLIB |
| zune-jpeg | 0.4.21 | MIT OR Apache-2.0 OR Zlib | Declared in Cargo manifest |
| zune-jpeg | 0.5.15 | MIT OR Apache-2.0 OR Zlib | LICENSE-APACHE, LICENSE-MIT, LICENSE-ZLIB |
