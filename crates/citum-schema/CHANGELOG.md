# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.30.2] - 2026-04-30

### Refactor

## [0.29.0] - 2026-04-29

### Bug Fixes

**schema**

- Move lint module to schema-style ([`28538d3`](https://github.com/citum/citum-core/commit/28538d3cbe1df24f33463ec43fd74d02327d30da))


## [0.28.0] - 2026-04-28

### Features

**schema**

- Refine numbering semantics ([`c407f7d`](https://github.com/citum/citum-core/commit/c407f7d34ac9bc1bc569981dcc2f582f6dce070c))

- Support custom numbering + locators ([`74920ed`](https://github.com/citum/citum-core/commit/74920ed05cc9268e99bd1403ac7b636940fbca14))



### Refactor

**cli**

- Split app shell from core logic ([`474c93d`](https://github.com/citum/citum-core/commit/474c93d94f01e82d5af647dcc84bef30ec7df167))


**schema**

- Strengthen ref and language ids ([`08f7fdc`](https://github.com/citum/citum-core/commit/08f7fdc0929d92fd7e997ecb6aaeb4f5785de11e))


**styles**

- Consolidate embedded style files ([`1f0e513`](https://github.com/citum/citum-core/commit/1f0e513c692a2a4caacc10b10520374303277b2b))



### Testing

**schema**

- Avoid owned keyword probes ([`c4b14ec`](https://github.com/citum/citum-core/commit/c4b14ecf59509a6352b5af3551757bfc05f4ca53))


## [0.20.0] - 2026-04-01

### Features

**schema**

- Archival and unpublished support ([`076fa11`](https://github.com/citum/citum-core/commit/076fa1192fd2c97c55375c815e61d4d8e778fad1))

- Implement generalized work relation ([`c3b30e6`](https://github.com/citum/citum-core/commit/c3b30e638990045f4ec8ba696e02adbba78194f1))


## [0.18.0] - 2026-03-25

### Bug Fixes

**lint**

- Pedantic autofixes ([`b52e2b0`](https://github.com/citum/citum-core/commit/b52e2b094f2628cb9df5d9bd3f34155f58f70a22))



### Features

**bindings**

- Promote wasm api + specta types ([`50376ae`](https://github.com/citum/citum-core/commit/50376ae1a244db83af72596bd240ce72435c2712))


**lint**

- Clippy all=deny, pedantic suppressions ([`9170a21`](https://github.com/citum/citum-core/commit/9170a2197df4c289e1202cfed840c8450e2b927c))



### Refactor

**schema**

- Dedupe facade crate ([`cf79130`](https://github.com/citum/citum-core/commit/cf79130f85d34d2b383280107e814a2d32a24b4c))


## [0.10.0] - 2026-03-10

### Bug Fixes

**delimiter**

- Normalize enum parsing across engine and migrate ([`697d083`](https://github.com/citum/citum-core/commit/697d0838a44d998cd68a9a17eef77b2b6c7bafc3))


**engine**

- Make bibliography sort defaults explicit ([`bfd7e5d`](https://github.com/citum/citum-core/commit/bfd7e5de15baf55c10638571aca26c3c6205f772))

- Finish compound numeric rendering ([`ba5f832`](https://github.com/citum/citum-core/commit/ba5f83226e4a9ae5d8677592e02ed3970fc2b897))

- Preserve harvard no-date citations ([`4039833`](https://github.com/citum/citum-core/commit/4039833ae003ee44a4211c54ecdaece08da69a3a))


**examples**

- Validate and fix refs YAML files ([`b7c4ef9`](https://github.com/citum/citum-core/commit/b7c4ef963c2684a4aad2a410e2f378d44638f949))


**labels**

- Make et-al name count configurable per preset ([`f7ea4b0`](https://github.com/citum/citum-core/commit/f7ea4b0620124cecdbdfa7cf42af2a1bf7ad74dd))


**locale**

- Lowercase editor short terms (ed./eds.) ([`22bbede`](https://github.com/citum/citum-core/commit/22bbede4c3a80996bbe5d6b451d9a0aeceff10b9))


**schema**

- Drop is_base from StyleInfo and csl-legacy Info ([`b64a557`](https://github.com/citum/citum-core/commit/b64a557046205000fb1bc61fa4957e1f67a71644))

- Address missing field_languages field in ref_article_authors macro ([`acdb8ba`](https://github.com/citum/citum-core/commit/acdb8bafeef8b8a80efcd84ad83ac30c26c2bce9))


**scripts**

- Replace bump workflow with python tool ([`e756236`](https://github.com/citum/citum-core/commit/e756236144e44bca2d5bc289483ed9f31d6446e7))

- Make bump workflow schema-only ([`f045bbc`](https://github.com/citum/citum-core/commit/f045bbc951bd5065f4670c8aaeb5b4cdb2f3c167))



### Documentation

**schema**

- Cover root style-model docs ([`553070d`](https://github.com/citum/citum-core/commit/553070d99437e174577460405d5735c03cad94b0))

- Cover renderer docs ([`af0fb99`](https://github.com/citum/citum-core/commit/af0fb99d59c019b46b02f949cf1d5cf6ba9d4928))

- Cover citation locator docs ([`8e980c1`](https://github.com/citum/citum-core/commit/8e980c179ff06ce8b50f3ee84066b3484344092c))

- Cover locale support docs ([`f804402`](https://github.com/citum/citum-core/commit/f8044025a4f7b3e72246000e85a61aae6086f675))

- Document and test renderer ([`052bb8f`](https://github.com/citum/citum-core/commit/052bb8f227b8f2c578699e6ca191eb8d4c22a997))

- Document and test locale types ([`781f89b`](https://github.com/citum/citum-core/commit/781f89bd32a9b608f4ecb26e645949fa8a3674e9))

- Document and test processing options ([`9450c70`](https://github.com/citum/citum-core/commit/9450c70b40a7c63ac852fc5bddac30b5fa0b568b))


**schema,engine,migrate**

- Add public API doc comments ([`7162bf7`](https://github.com/citum/citum-core/commit/7162bf712ef22adad2e9a7e4ccdbd250861d1588))



### Features

**citation**

- Unify locator model ([`8ca1646`](https://github.com/citum/citum-core/commit/8ca1646363a1915b4f589d739bc0b5e5f8f6d0d0))


**citations**

- Add integral name memory ([`73417da`](https://github.com/citum/citum-core/commit/73417dae9ac30b8441edb12af86aeb9d6a03fff1))


**compound-sets**

- Implement sets and subentry ([`2877767`](https://github.com/citum/citum-core/commit/287776717d5ec07c4c2e560e657966bf789d14a8))


**core**

- Split schema and convert namespace ([`c44d279`](https://github.com/citum/citum-core/commit/c44d27978b5ddad97cb47147162452b1751e2d93))


**document**

- Configure note marker punctuation ([`91317b8`](https://github.com/citum/citum-core/commit/91317b825357738b58509c24ec48dd60b43b4640))


**edtf**

- Implement time component rendering ([`7143adf`](https://github.com/citum/citum-core/commit/7143adf0877cbd31c1ffac6ce6785b37a820090a))


**engine**

- Support container short titles ([`29794d0`](https://github.com/citum/citum-core/commit/29794d02c10cf3ef341127a219a9f947b846facd))

- Support expanded verification cases ([`9386467`](https://github.com/citum/citum-core/commit/9386467d94d2342bef929eb1f84ea8dab5a7f136))


**multilingual**

- Support language-aware title templates ([`71fe320`](https://github.com/citum/citum-core/commit/71fe32074171e998c6b00ce49bf881e57899d853))

- Add preferred-transliteration ([`b4a8a81`](https://github.com/citum/citum-core/commit/b4a8a81165f48baa90cdd8368a767fdd2e50e644))


**schema**

- Add SortPreset; use in chicago ([`dd14350`](https://github.com/citum/citum-core/commit/dd143502b78999e61547785a42c139b4c89f190a))

- Add CitationField, StyleSource provenance to StyleInfo ([`e4f0105`](https://github.com/citum/citum-core/commit/e4f01053160c7a64cef813d28b264b33c257ae11))

- Add NameForm to ContributorConfig ([`9d394cc`](https://github.com/citum/citum-core/commit/9d394cc5329662598feae3972802035c536235c1))

- Csl support and pr schema gate ([`201bb92`](https://github.com/citum/citum-core/commit/201bb92b87245964303d8adaeac46c5a392bbcb3))

- Compound locator support ([`9b7a578`](https://github.com/citum/citum-core/commit/9b7a57868f7c20e7f0aefa6a1c924b199d198c9e))

- Locator ergonomics ([`2757aa3`](https://github.com/citum/citum-core/commit/2757aa3c78a387496aba51fbc92f63542128fc45))


**schema,engine**

- Add subsequent et-al controls ([`d35f7ca`](https://github.com/citum/citum-core/commit/d35f7ca717195cf5fb54e26e60c23c7a30069957))


**typst**

- Add native rendering and pdf output ([`c4dbe6f`](https://github.com/citum/citum-core/commit/c4dbe6f96ba5f369513b964618a8fa2fe4d0cf4d))



### Refactor

**edtf**

- Rename crate csln-edtf → citum-edtf ([`51cfc24`](https://github.com/citum/citum-core/commit/51cfc24ba677424104a8e9a2a77ad60002a8bc03))


**migrate**

- Trim redundant bibliography sorts ([`b5d0be4`](https://github.com/citum/citum-core/commit/b5d0be427bd5aaf8ad14a93c5df15ea343a578bd))


**workspace**

- Migrate csln namespace to citum ([`1e22764`](https://github.com/citum/citum-core/commit/1e227643afb5a01a3aa7ccb4d70dbd086b478b69))


