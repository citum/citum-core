# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.30.2] - 2026-04-30

### Bug Fixes

**engine**

- Rich-input note parser + title case ([`453b34e`](https://github.com/citum/citum-core/commit/453b34ea3defe49e73208d5ee8c79bfbbe25a26a))


**migrate**

- Continue past free-text in note ([`93963c4`](https://github.com/citum/citum-core/commit/93963c4c26ba1edf4072fb6e1a40c566da099487))

- Address clippy and match guards ([`00daf59`](https://github.com/citum/citum-core/commit/00daf59903cd1502dcc8de0672a4096eac08611c))


**schema**

- Chicago legal-material support ([`f56ff66`](https://github.com/citum/citum-core/commit/f56ff663d93cb636459103c6cbd268ed01b182a2))


**styles**

- Advance apa rich bibliography closure ([`b354609`](https://github.com/citum/citum-core/commit/b354609d269f70bfc000421ee760ab4404e80ec0))



### Features

**migrate**

- Convert zotero notes to example ([`cbaadb9`](https://github.com/citum/citum-core/commit/cbaadb98df9818e192372aa678a0198b75984154))


**schema**

- Note field variable parser ([`aeb5d63`](https://github.com/citum/citum-core/commit/aeb5d63ace9296f70b53ba30a6f3e227c871f0e2))

- Part/supplement/printing numbers ([`47a296e`](https://github.com/citum/citum-core/commit/47a296ee882e199845a276869deca48b5a7a461c))



### Refactor

## [0.19.0] - 2026-03-25

### Bug Fixes

**lint**

- Pedantic autofixes ([`b52e2b0`](https://github.com/citum/citum-core/commit/b52e2b094f2628cb9df5d9bd3f34155f58f70a22))


**schema**

- Drop is_base from StyleInfo and csl-legacy Info ([`b64a557`](https://github.com/citum/citum-core/commit/b64a557046205000fb1bc61fa4957e1f67a71644))



### Documentation

**csl-legacy, citum-cli**

- Add doc and test coverage ([`172e74c`](https://github.com/citum/citum-core/commit/172e74cbbd47d2f96303062ff2370506f6b60773))



### Features

**engine**

- Support expanded verification cases ([`9386467`](https://github.com/citum/citum-core/commit/9386467d94d2342bef929eb1f84ea8dab5a7f136))


**lint**

- Clippy all=deny, pedantic suppressions ([`9170a21`](https://github.com/citum/citum-core/commit/9170a2197df4c289e1202cfed840c8450e2b927c))


**migrate**

- Extract citation-number collapse ([`9447002`](https://github.com/citum/citum-core/commit/944700270bd2b935f8e126fb8c036293ccfe453f))


**schema**

- Add CitationField, StyleSource provenance to StyleInfo ([`e4f0105`](https://github.com/citum/citum-core/commit/e4f01053160c7a64cef813d28b264b33c257ae11))



### Performance

**ci**

- Reduce compilation time A+B ([`aac067a`](https://github.com/citum/citum-core/commit/aac067a91ecc8d6528f094eaf0170b80a776282a))



### Refactor

**workspace**

- Migrate csln namespace to citum ([`1e22764`](https://github.com/citum/citum-core/commit/1e227643afb5a01a3aa7ccb4d70dbd086b478b69))


