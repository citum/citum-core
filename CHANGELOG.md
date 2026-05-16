# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.47.0] - 2026-05-16

### Bug Fixes

**ci**

- Action versions and clippy hygiene ([`544ecd6`](https://github.com/citum/citum-core/commit/544ecd6d02454a885845c6dcf1f78dc0d5c186e8))


**security**

- Harden publish readiness ([`e73f246`](https://github.com/citum/citum-core/commit/e73f246414d71bf708a56ebe762d5396c6471c16))


**store**

- Wire config.yaml into resolution ([`7bebf17`](https://github.com/citum/citum-core/commit/7bebf170f1fd681dc19744198a399e914835026d))


**styles**

- Refine HTTP catalog UX ([`d9b6c86`](https://github.com/citum/citum-core/commit/d9b6c86e0351866a14de46702356852f51768385))



### Features

**cli**

- Unified table rendering for styles ([`640b98d`](https://github.com/citum/citum-core/commit/640b98d66e1fc5aadca8badd6f32b954ba47ed69))

- Unify style registry workflows ([`b3c283f`](https://github.com/citum/citum-core/commit/b3c283faf328f5e270e0158d6cea8ad85d1f2eb4))

- Add ratatui style browser ([`60da4b1`](https://github.com/citum/citum-core/commit/60da4b1e343ce2a5a689db224c727cf618f29244))

- Add style cid pin and validate commands ([`675a4f7`](https://github.com/citum/citum-core/commit/675a4f70e343d0dcd0ae4220853a8e67b8f3cf31))

- Validate style version in check ([`7219d98`](https://github.com/citum/citum-core/commit/7219d989b0bb12b5fbb281969f91e2f298fe5128))


**engine**

- Add document-level abbreviation-map ([`9f6e1fa`](https://github.com/citum/citum-core/commit/9f6e1fa3e6a3fb5b65c8dd31a42b9f808060b5f6))


**io**

- Move biblatex and i/o to dedicated crate ([`c0016ce`](https://github.com/citum/citum-core/commit/c0016cec2137e11ec4dbb498d9aa33e00e57d9c6))


**schema**

- Resolve template v3 variants ([`bb9f3bb`](https://github.com/citum/citum-core/commit/bb9f3bbd2668c0bb96144cac49d82d77e4b95676))


**server**

- Schema discovery + /schemas/ page ([`6a80e8a`](https://github.com/citum/citum-core/commit/6a80e8a9ab0342bb104939144d08e02be75e41a3))


**store**

- Migrate GitResolver to gix ([`0889e21`](https://github.com/citum/citum-core/commit/0889e21d993c06819cfe145a44b8d730417011fe))


**styles**

- Add HTTP core catalog ([`a91e968`](https://github.com/citum/citum-core/commit/a91e9689159e8d11fac2a0801ed4c923fbfbb1bd))



### Refactor

**cli**

- Delegate style loading to store ([`683c86f`](https://github.com/citum/citum-core/commit/683c86fb07043a22475a7427b213dc811c361cda))


**store**

- Unify style resolution ([`49b1337`](https://github.com/citum/citum-core/commit/49b133755146746d7d39610c0bda967061fdfae0))


## [0.35.0] - 2026-05-03

### Features

**cli**

- Distributed architecture phase 1 ([`7ae000d`](https://github.com/citum/citum-core/commit/7ae000d5d4ed3764e52634f954e4633f7f58d3bb))



### Refactor

## [0.33.0] - 2026-05-03

### Bug Fixes

**engine**

- Propagate bibliography annotations ([`a9e4b93`](https://github.com/citum/citum-core/commit/a9e4b93f1f8fe63f5480c1b4affe7ce9b6a51d78))


## [0.32.1] - 2026-05-01

### Features

**engine**

- Native bib partitioning and grouping ([`be574e9`](https://github.com/citum/citum-core/commit/be574e913c273e8506befc093373402fdbc4760f))


## [0.30.2] - 2026-04-30

### Refactor

## [0.28.0] - 2026-04-28

### Bug Fixes

**cli**

- Avoid suggestion formatting allocation ([`05f8c01`](https://github.com/citum/citum-core/commit/05f8c01aeda8181406d821cafaa5348b5f408a74))



### Refactor

**cli**

- Split app shell from core logic ([`474c93d`](https://github.com/citum/citum-core/commit/474c93d94f01e82d5af647dcc84bef30ec7df167))


## [0.26.2] - 2026-04-26

### Features

**locale**

- Support gender-aware MF2 labels ([`5994096`](https://github.com/citum/citum-core/commit/5994096cb282ac74dd9c263e23c8734d4357c833))


## [0.26.0] - 2026-04-26

### Refactor

**schema**

- Richtext enum for note/abstract ([`f426887`](https://github.com/citum/citum-core/commit/f4268878437fffcb2c5c293038922d89a9ab1055))


## [0.25.1] - 2026-04-25

### Features

**cli**

- Add locale override to render refs ([`77216eb`](https://github.com/citum/citum-core/commit/77216ebf9088dedc428d7451cc3a7344a8a72270))


## [0.25.0] - 2026-04-23

### Features

**cli**

- Add comfy-table for registry and style lists ([`8d0a868`](https://github.com/citum/citum-core/commit/8d0a868b46cbdc47ce2faf0fa8aad9b5f38176be))


**locale**

- Add gender-aware term resolution ([`5af327d`](https://github.com/citum/citum-core/commit/5af327d53ea0fe94bd4e0e682c4e34bc0ac65ee1))


**schema**

- Support custom numbering + locators ([`74920ed`](https://github.com/citum/citum-core/commit/74920ed05cc9268e99bd1403ac7b636940fbca14))



### Refactor

**schema**

- Strengthen ref and language ids ([`08f7fdc`](https://github.com/citum/citum-core/commit/08f7fdc0929d92fd7e997ecb6aaeb4f5785de11e))


## [0.20.0] - 2026-04-01

### Bug Fixes

**bib**

- Align schemas and edited-book coverage ([`a14a9f4`](https://github.com/citum/citum-core/commit/a14a9f4aba044e11b6f16fcda198974f5161438f))


**cli**

- Batch citations only for numeric ([`8f22dd6`](https://github.com/citum/citum-core/commit/8f22dd67e12e7e26750dcf903eb6785cb3c62b76))

- Keep citation-file error marker ([`1741561`](https://github.com/citum/citum-core/commit/17415612ebfa461b3c8e5d87500ccb15b9c86cc0))

- Document input formats in help; clean biblatex pivot ([`a6c949e`](https://github.com/citum/citum-core/commit/a6c949e6a3bedb8cb54c09212420f8ec1cd731e5))


**convert**

- Preserve refs fidelity across csl-json and ris ([`67d85d8`](https://github.com/citum/citum-core/commit/67d85d8cc967b9ab6a3414df63a75c1a5753002b))


**engine**

- Finish compound numeric rendering ([`ba5f832`](https://github.com/citum/citum-core/commit/ba5f83226e4a9ae5d8677592e02ed3970fc2b897))


**lint**

- Pedantic autofixes ([`b52e2b0`](https://github.com/citum/citum-core/commit/b52e2b094f2628cb9df5d9bd3f34155f58f70a22))


**schema**

- Restore schema publishing outputs ([`4b3bf27`](https://github.com/citum/citum-core/commit/4b3bf274e63945e45da6ae4a8030e0459852fa2b))

- Defaults + drop semantic_classes ([`4dd6909`](https://github.com/citum/citum-core/commit/4dd6909f539d6144b05c5216161c8523f587c946))

- Null-aware preset overlay merging ([`080df92`](https://github.com/citum/citum-core/commit/080df92f2efa1e980935bbb2ae92f2ffffeb8267))



### Documentation

**cli**

- Unify help output with a summary-and-detail model ([`ae95b1d`](https://github.com/citum/citum-core/commit/ae95b1dc98a6d70de476c42269a1335666e83cf7))


**csl-legacy, citum-cli**

- Add doc and test coverage ([`172e74c`](https://github.com/citum/citum-core/commit/172e74cbbd47d2f96303062ff2370506f6b60773))



### Features

**bindings**

- Promote wasm api + specta types ([`50376ae`](https://github.com/citum/citum-core/commit/50376ae1a244db83af72596bd240ce72435c2712))


**citation**

- Unify locator model ([`8ca1646`](https://github.com/citum/citum-core/commit/8ca1646363a1915b4f589d739bc0b5e5f8f6d0d0))


**citations**

- Add integral name memory ([`73417da`](https://github.com/citum/citum-core/commit/73417dae9ac30b8441edb12af86aeb9d6a03fff1))


**cli**

- Default output format to html ([`1c7763d`](https://github.com/citum/citum-core/commit/1c7763d615b2bb72471fc002c09e174dd2eca6d2))

- Add metadata and styles to help output ([`ce34090`](https://github.com/citum/citum-core/commit/ce34090c33ebcfcc06b99a997a06eff261d1e5a6))

- Add detailed examples to convert command help ([`b3cf5bf`](https://github.com/citum/citum-core/commit/b3cf5bf4b288323ff150bded29186d1ae96dcd96))


**cli,store**

- Integrate citum_store with CLI and resolve user styles ([`ad22314`](https://github.com/citum/citum-core/commit/ad22314a0a8d634a84bb33e14c82749863ff92b1))


**compound-sets**

- Implement sets and subentry ([`2877767`](https://github.com/citum/citum-core/commit/287776717d5ec07c4c2e560e657966bf789d14a8))


**core**

- Split schema and convert namespace ([`c44d279`](https://github.com/citum/citum-core/commit/c44d27978b5ddad97cb47147162452b1751e2d93))


**doc**

- Add pandoc markdown citations ([`a95b6ba`](https://github.com/citum/citum-core/commit/a95b6baac4e3ae8a2bbda8afaf28e84f7b28b850))


**engine**

- Djot inline rendering for annotations ([`0329ee9`](https://github.com/citum/citum-core/commit/0329ee9467eb5dc5cae7127a3db7bae3a8572409))

- Implement title text-case semantics ([`6d13aa5`](https://github.com/citum/citum-core/commit/6d13aa5b08e63727a3d0c7f60b9e2f3102a7daaf))

- Add secondary role label presets ([`9de0e74`](https://github.com/citum/citum-core/commit/9de0e74894a9c5c30c1e7ac816cede991be9884b))


**engine,cli**

- Annotated bibliography support ([`9367000`](https://github.com/citum/citum-core/commit/9367000d723380afd9df8bd946639a521c60ea49))


**lint**

- Clippy all=deny, pedantic suppressions ([`9170a21`](https://github.com/citum/citum-core/commit/9170a2197df4c289e1202cfed840c8450e2b927c))


**locale**

- ICU MF1 locale system ([`6c5c98e`](https://github.com/citum/citum-core/commit/6c5c98e02c7b7cde261e0de8a4d74ffc81967604))

- Pivot to MF2 message syntax ([`98debff`](https://github.com/citum/citum-core/commit/98debff90ed3373297cf6be69173e253c0f8be13))


**schema**

- Add StyleRegistry type, registry, and CLI ([`a895272`](https://github.com/citum/citum-core/commit/a8952727fa8f32124a7f76ef6ef474c45c274a8b))

- Split nested option scopes ([`84dc668`](https://github.com/citum/citum-core/commit/84dc668cac4f9b66f8145d57d3f6392327da8da4))

- Nest inner affixes under WrapConfig ([`04c2b5f`](https://github.com/citum/citum-core/commit/04c2b5f507de01e5e7b2cb2c6cd97a040a6e4b6b))

- Implement generalized work relation ([`c3b30e6`](https://github.com/citum/citum-core/commit/c3b30e638990045f4ec8ba696e02adbba78194f1))


**template-v2**

- Implement template schema v2 ([`cab0f41`](https://github.com/citum/citum-core/commit/cab0f41bbdd1300b351093356e536ca5bd234f5f))


**typst**

- Add native rendering and pdf output ([`c4dbe6f`](https://github.com/citum/citum-core/commit/c4dbe6f96ba5f369513b964618a8fa2fe4d0cf4d))



### Refactor

**cli**

- Make citum the only public binary name ([`1489183`](https://github.com/citum/citum-core/commit/148918328829eaf6cea835d21bff140ce036e134))

- Consolidate convert-refs path ([`199c2dc`](https://github.com/citum/citum-core/commit/199c2dcfa1dc97963a67a1d8b071a6f208c63309))

- Remove duplicate refs module ([`7aaf910`](https://github.com/citum/citum-core/commit/7aaf910ce8e7823fc13f89c4917d9a6ee43f410b))

- Bundle render params into context ([`f7746d4`](https://github.com/citum/citum-core/commit/f7746d46d05f5c321afcbb151c74658b2d4f1218))

- Extract citum-pdf crate ([`d49a6e1`](https://github.com/citum/citum-core/commit/d49a6e1944b64c2c3d04ab8d9598c8e6e59ce22f))


**lint**

- Enforce too_many_lines and cognitive_complexity ([`443fcc6`](https://github.com/citum/citum-core/commit/443fcc62801f4518f09f65b21cda02493f19076a))


**workspace**

- Migrate csln namespace to citum ([`1e22764`](https://github.com/citum/citum-core/commit/1e227643afb5a01a3aa7ccb4d70dbd086b478b69))


