# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.36.0] - 2026-05-05

### Features

**schema**

- Resolve template v3 variants ([`bb9f3bb`](https://github.com/citum/citum-core/commit/bb9f3bbd2668c0bb96144cac49d82d77e4b95676))


## [0.34.0] - 2026-05-03

### Features

**schema**

- Rename template reuse ([`c95ddba`](https://github.com/citum/citum-core/commit/c95ddba8b5463f5aa8109b7dfb56dbc2b8ca992f))


## [0.30.2] - 2026-04-30

### Bug Fixes

**clippy**

- Resolve used underscore bindings and collapsible ifs in discovery tool ([`d1bb1e5`](https://github.com/citum/citum-core/commit/d1bb1e5e7c17dfc67db1a710992d4bba17376c0d))


**migrate**

- Avoid locator fallback allocation ([`19a1e8e`](https://github.com/citum/citum-core/commit/19a1e8e3a8efa4fd0a7e79d34b269c21c7736eb3))



### Features

**analyze**

- Add automated profile candidate discovery ([`309e7ac`](https://github.com/citum/citum-core/commit/309e7ac2f6f93f427a1b74a364b7e23f1ede76b8))

- Add automated profile candidate discovery ([`9e13a17`](https://github.com/citum/citum-core/commit/9e13a17bb9cf88cd228b2ebdf2d457cc0de9b97c))


**migrate**

- Add taxonomy-aware wrapper lineage ([`2fbe8a3`](https://github.com/citum/citum-core/commit/2fbe8a37876a65ec9cfdf1e5931e6b6a4935960f))



### Refactor

## [0.22.0] - 2026-04-21

### Features

**schema**

- Implement config-only profile overrides ([`037c7c7`](https://github.com/citum/citum-core/commit/037c7c784006fc27ee2f11c9cd6d18dcc52ea1b0))


## [0.21.0] - 2026-04-21

### Bug Fixes

**migrate**

- Drop redundant no-date fallback ([`f94d0ba`](https://github.com/citum/citum-core/commit/f94d0ba472da2e1e977151652c66eb03926c3f16))

- Extract locator strip periods ([`8477585`](https://github.com/citum/citum-core/commit/84775850413486094948c03fe60bd2526fe72ed8))

- Group inline journal issue dates ([`68108fb`](https://github.com/citum/citum-core/commit/68108fb91110efeccf4ea3792a3c474685d8f091))

- Unblock note wave migration path ([`ad9c84f`](https://github.com/citum/citum-core/commit/ad9c84f7a00f4f5bb035be92ebefdb36af7d61fe))

- Repair inferred bib variants ([`9a759ec`](https://github.com/citum/citum-core/commit/9a759ec852bceffc3a6b7b387c472b4beb28f067))

- Substitute fallback + patent order ([`85de5a3`](https://github.com/citum/citum-core/commit/85de5a3ce996f4e375b008281c2e4b47c54a3422))

- Normalize legal_case type templates ([`90939a0`](https://github.com/citum/citum-core/commit/90939a0fb2a0403a2f6280a670b855234fc97a8f))

- Suppress patent-number omission ([`67d6e1a`](https://github.com/citum/citum-core/commit/67d6e1afa0897f83880159c2bff9976fb29ef2f1))

- Chicago rich-input pass ([`9aebee5`](https://github.com/citum/citum-core/commit/9aebee5ef14cd13f0d9ceed3a4791bfc03996c6c))

- Address clippy and match guards ([`00daf59`](https://github.com/citum/citum-core/commit/00daf59903cd1502dcc8de0672a4096eac08611c))

- Map name-as-sort-order to name_order ([`d499a17`](https://github.com/citum/citum-core/commit/d499a17054ef0d4753dfb2d7d0e45a87f3cc464c))

- Preserve wrap-bearing groups ([`f905edf`](https://github.com/citum/citum-core/commit/f905edf9d5dcdf2004471cd18e143013d0e56f4a))

- Wrap parity + schema normalize ([`a2a6f3b`](https://github.com/citum/citum-core/commit/a2a6f3b171f82133682d80fd70739bf9d61942c8))


**styles**

- Simplify wrappers and add preset hubs ([`42b57ea`](https://github.com/citum/citum-core/commit/42b57ea0469980033727edec0eb9f84e6dd33f17))



### Features

**locale**

- Add gender-aware term resolution ([`5af327d`](https://github.com/citum/citum-core/commit/5af327d53ea0fe94bd4e0e682c4e34bc0ac65ee1))


**migrate**

- Embed live JS inference ([`d2c243e`](https://github.com/citum/citum-core/commit/d2c243e668d7b6d22f9b96c7e2e4ca50099ddfaa))


**schema**

- Split nested option scopes ([`84dc668`](https://github.com/citum/citum-core/commit/84dc668cac4f9b66f8145d57d3f6392327da8da4))

- Nest inner affixes under WrapConfig ([`04c2b5f`](https://github.com/citum/citum-core/commit/04c2b5f507de01e5e7b2cb2c6cd97a040a6e4b6b))



### Refactor

**schema**

- Rename StylePreset->StyleBase, preset->extends ([`fe59c08`](https://github.com/citum/citum-core/commit/fe59c08807762a1f399ce059c5452e87fc2ea636))


## [0.19.0] - 2026-03-25

### Bug Fixes

**migrate**

- Co-emit name_form on initialize_with ([`e5b906b`](https://github.com/citum/citum-core/commit/e5b906be96bc9f2df5f8c71c5ea333a186f692d0))

- Drop bare uncertain-date markers ([`f579634`](https://github.com/citum/citum-core/commit/f57963450ce0bfed0ca1870b0b434e2057bf47d3))



### Features

**migrate**

- Extract citation-number collapse ([`9447002`](https://github.com/citum/citum-core/commit/944700270bd2b935f8e126fb8c036293ccfe453f))


## [0.18.0] - 2026-03-25

### Bug Fixes

**bibliography**

- Add journal doi fallback policy ([`8853aa8`](https://github.com/citum/citum-core/commit/8853aa86fcb31ba3d592a1676e87178986996b3f))


**delimiter**

- Normalize enum parsing across engine and migrate ([`697d083`](https://github.com/citum/citum-core/commit/697d0838a44d998cd68a9a17eef77b2b6c7bafc3))


**engine**

- Split name-form from initialize-with ([`95875c8`](https://github.com/citum/citum-core/commit/95875c8c3509afc9d2b877d97d4ec08b31e722a4))


**lint**

- Pedantic autofixes ([`b52e2b0`](https://github.com/citum/citum-core/commit/b52e2b094f2628cb9df5d9bd3f34155f58f70a22))


**migrate**

- Fix subsequent et-al propagation ([`1867c27`](https://github.com/citum/citum-core/commit/1867c27ef3f2f0395db38a6f3095a11ff1e6ab03))

- Normalize locator labels ([`46e2cd8`](https://github.com/citum/citum-core/commit/46e2cd8f69f11b4cb6ba2bacc99a725890cb3120))

- Preserve strip-periods in migration ([`66685d9`](https://github.com/citum/citum-core/commit/66685d9cca2196243ccafce2d997363dd13f501b))

- Support complex position trees ([`50b0c99`](https://github.com/citum/citum-core/commit/50b0c9925fbf995cebb9fa86b84233731f0ff017))


**oracle**

- Make scoring case-aware ([`b9dccaa`](https://github.com/citum/citum-core/commit/b9dccaab9c91cce36b4e98587c8aa320055e0763))


**schema**

- Drop is_base from StyleInfo and csl-legacy Info ([`b64a557`](https://github.com/citum/citum-core/commit/b64a557046205000fb1bc61fa4957e1f67a71644))



### Documentation

**migrate**

- Add /// to options_extractor fns ([`00ab2ef`](https://github.com/citum/citum-core/commit/00ab2ef1c944ad828d534dfa751de292dd3abb47))


**schema,engine,migrate**

- Add public API doc comments ([`7162bf7`](https://github.com/citum/citum-core/commit/7162bf712ef22adad2e9a7e4ccdbd250861d1588))



### Features

**citation**

- Align repeated-note position semantics ([`88a33e6`](https://github.com/citum/citum-core/commit/88a33e623400028bf6d8cd625675b657be7e9685))


**engine**

- Implement title text-case semantics ([`6d13aa5`](https://github.com/citum/citum-core/commit/6d13aa5b08e63727a3d0c7f60b9e2f3102a7daaf))


**lint**

- Clippy all=deny, pedantic suppressions ([`9170a21`](https://github.com/citum/citum-core/commit/9170a2197df4c289e1202cfed840c8450e2b927c))


**migrate**

- Complete variable-once cross-list deduplication ([`f2229dc`](https://github.com/citum/citum-core/commit/f2229dc7b0d7d95b2774bfe07a0757be391b3174))

- Improve inferred parity and migrate-only oracle tooling ([`0aa012e`](https://github.com/citum/citum-core/commit/0aa012e80dcd415961eb79626bbe946ccf195b5b))

- Tighten inferred bibliography parity heuristics ([`a502c61`](https://github.com/citum/citum-core/commit/a502c61c8843cd14d9fd8a0de2a8093553c3b459))

- Normalize legal-case fields by style id ([`859f304`](https://github.com/citum/citum-core/commit/859f3045f6f4191421da563a34ba87441a91007a))

- Support mixed note position trees ([`26025bc`](https://github.com/citum/citum-core/commit/26025bcb3417a99c4dd20f796a9af4dfb9ab4d25))


**report**

- Add migration behavior coverage ([`53fc74c`](https://github.com/citum/citum-core/commit/53fc74c839b129d807c8db89bf441620d578809a))


**schema**

- Add SortPreset; use in chicago ([`dd14350`](https://github.com/citum/citum-core/commit/dd143502b78999e61547785a42c139b4c89f190a))

- Add CitationField, StyleSource provenance to StyleInfo ([`e4f0105`](https://github.com/citum/citum-core/commit/e4f01053160c7a64cef813d28b264b33c257ae11))

- Short_name + edition on StyleInfo ([`bce8be2`](https://github.com/citum/citum-core/commit/bce8be2660818fdcef4716f447693140f5791e92))


**schema,engine**

- Add subsequent et-al controls ([`d35f7ca`](https://github.com/citum/citum-core/commit/d35f7ca717195cf5fb54e26e60c23c7a30069957))


**template-v2**

- Implement template schema v2 ([`cab0f41`](https://github.com/citum/citum-core/commit/cab0f41bbdd1300b351093356e536ca5bd234f5f))



### Refactor

**citum-migrate**

- Simplify upsampler ([`d37bfd5`](https://github.com/citum/citum-core/commit/d37bfd519c56097b76520c1d29150232d3593c29))


**engine,migrate**

- Rust-simplify pass ([`a2b65f7`](https://github.com/citum/citum-core/commit/a2b65f7aa237ce1159bd619178b8241a4d13989b))


**lint**

- Enforce too_many_lines and cognitive_complexity ([`443fcc6`](https://github.com/citum/citum-core/commit/443fcc62801f4518f09f65b21cda02493f19076a))


**migrate**

- Trim redundant bibliography sorts ([`b5d0be4`](https://github.com/citum/citum-core/commit/b5d0be427bd5aaf8ad14a93c5df15ea343a578bd))

- Modularize template_compiler ([`8e484ef`](https://github.com/citum/citum-core/commit/8e484ef8d38f7ff5bf71555045c4f2ca4f5b8d69))

- Extract fixups module ([`0ef1c15`](https://github.com/citum/citum-core/commit/0ef1c159ba6edda83021000fa255244636c3551f))

- Remove only_used_in_recursion allow ([`59207f5`](https://github.com/citum/citum-core/commit/59207f50c494472d5227b6d87b8f8ac715619fd6))

- Split fixups modules ([`f2df944`](https://github.com/citum/citum-core/commit/f2df944efbf6a978c32fe47e8d1c23e41254d7be))

- Drop locator label fields ([`c205cc2`](https://github.com/citum/citum-core/commit/c205cc28b451132c87d71276c25aa9ac02329ef3))


**workspace**

- Migrate csln namespace to citum ([`1e22764`](https://github.com/citum/citum-core/commit/1e227643afb5a01a3aa7ccb4d70dbd086b478b69))



### Testing

**migration**

- Expand csl-to-citum reporting ([`29d89ae`](https://github.com/citum/citum-core/commit/29d89ae622059695d6dc1ad32cabf8fe34664e2b))


