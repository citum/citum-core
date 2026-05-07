# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.38.0] - 2026-05-06

### Bug Fixes

**migrate**

- Emit localized page labels ([`3d2b951`](https://github.com/citum/citum-core/commit/3d2b951c19b5ea98232600823ebc539688bc86f2))



### Features

**schema**

- Resolve template v3 variants ([`bb9f3bb`](https://github.com/citum/citum-core/commit/bb9f3bbd2668c0bb96144cac49d82d77e4b95676))

- Rename template reuse ([`df4c4ea`](https://github.com/citum/citum-core/commit/df4c4eaaa6e7b69486634a47a1ed355eb22a82ed))


## [0.35.0] - 2026-05-03

### Features

**cli**

- Distributed architecture phase 1 ([`7ae000d`](https://github.com/citum/citum-core/commit/7ae000d5d4ed3764e52634f954e4633f7f58d3bb))


## [0.34.0] - 2026-05-03

### Features

**schema**

- Rename template reuse ([`c95ddba`](https://github.com/citum/citum-core/commit/c95ddba8b5463f5aa8109b7dfb56dbc2b8ca992f))



### Refactor

## [0.33.0] - 2026-05-03

### Bug Fixes

**engine**

- Propagate bibliography annotations ([`a9e4b93`](https://github.com/citum/citum-core/commit/a9e4b93f1f8fe63f5480c1b4affe7ce9b6a51d78))



### Features

**engine**

- Unify Japanese script partitioning ([`feddb0c`](https://github.com/citum/citum-core/commit/feddb0c6e4ccfd87a156819c82a868d3cdab7f10))


## [0.32.1] - 2026-05-01

### Features

**engine**

- Native bib partitioning and grouping ([`be574e9`](https://github.com/citum/citum-core/commit/be574e913c273e8506befc093373402fdbc4760f))


## [0.32.0] - 2026-05-01

### Features

**engine**

- Partition bibliographies ([`a3433b5`](https://github.com/citum/citum-core/commit/a3433b5c367d93b4e3e1fbdd1ee8749f76b7a814))


## [0.31.2] - 2026-05-01

### Features

**engine**

- Harden unicode bibliography sorting ([`2d09b1e`](https://github.com/citum/citum-core/commit/2d09b1e7a4a45eb224696301f202630ad09cb290))


## [0.31.1] - 2026-05-01

### Testing

**engine**

- Bdd tests for bibliography loading ([`7c1d868`](https://github.com/citum/citum-core/commit/7c1d8680fc61475d88fb183b41c3d47600141365))


## [0.31.0] - 2026-05-01

### Features

**styles**

- Fidelity wave — locator/patent ([`ffa7e3e`](https://github.com/citum/citum-core/commit/ffa7e3e8416b2e2eab3ecefe097aaf5c25a1d78f))


## [0.30.2] - 2026-04-30

### Refactor

## [0.30.0] - 2026-04-29

### Features

**schema**

- Unify geographic place type ([`2788571`](https://github.com/citum/citum-core/commit/27885717b4efbf71cea0663854772c77b2bcfbcb))


## [0.29.1] - 2026-04-29

### Features

**schema**

- Part/supplement/printing numbers ([`47a296e`](https://github.com/citum/citum-core/commit/47a296ee882e199845a276869deca48b5a7a461c))



### Refactor

**engine**

- Rename csln- prefix to citum- ([`b19293d`](https://github.com/citum/citum-core/commit/b19293d76fc13b01a41615db69303a1bab879b84))


## [0.28.0] - 2026-04-28

### Bug Fixes

**engine**

- Harden rendered document metadata ([`1bc80df`](https://github.com/citum/citum-core/commit/1bc80df10c88a7d068d798f03a7b6710b3463025))



### Refactor

**cli**

- Split app shell from core logic ([`474c93d`](https://github.com/citum/citum-core/commit/474c93d94f01e82d5af647dcc84bef30ec7df167))



### Testing

**engine**

- Strengthen weak assertions ([`a3f3af8`](https://github.com/citum/citum-core/commit/a3f3af80f6a51e3df840aa10451187575d578e0b))

- Replace weak render assertions ([`5e88d0f`](https://github.com/citum/citum-core/commit/5e88d0f03b003de8134b9d553b86dc23125037bf))


## [0.26.3] - 2026-04-26

### Features

**locale**

- Gender-aware labels for FR and AR ([`52d6c8b`](https://github.com/citum/citum-core/commit/52d6c8ba7a05615746169ec39bc426022f01ec21))


## [0.26.1] - 2026-04-26

### Testing

**engine**

- Rehome djot note coverage ([`261dc97`](https://github.com/citum/citum-core/commit/261dc977335c3f679c24e368704fd14b9968d938))


## [0.26.0] - 2026-04-26

### Refactor

**schema**

- Richtext enum for note/abstract ([`f426887`](https://github.com/citum/citum-core/commit/f4268878437fffcb2c5c293038922d89a9ab1055))


## [0.25.0] - 2026-04-23

### Features

**cli**

- Add comfy-table for registry and style lists ([`8d0a868`](https://github.com/citum/citum-core/commit/8d0a868b46cbdc47ce2faf0fa8aad9b5f38176be))


## [0.22.0] - 2026-04-21

### Features

**schema**

- Implement config-only profile overrides ([`037c7c7`](https://github.com/citum/citum-core/commit/037c7c784006fc27ee2f11c9cd6d18dcc52ea1b0))


## [0.21.0] - 2026-04-21

### Bug Fixes

**engine**

- Complete personal comm support ([`86e3134`](https://github.com/citum/citum-core/commit/86e313403dd5d8963924ccf497b46897b719c933))

- Short title as author substitute ([`6d6cffd`](https://github.com/citum/citum-core/commit/6d6cffd21b22a567710d63fa3d662f25d194bd4d))

- Treaty/hearing citation double-title ([`fb59338`](https://github.com/citum/citum-core/commit/fb593381ec90b2fd9fd8fbe84bb0a2780723c51a))

- Co-evolution wave fidelity ([`48b6a91`](https://github.com/citum/citum-core/commit/48b6a91929cf23aecc51440245e45b2d1c5f9c98))

- Finish fidelity gate follow-up ([`5ee404e`](https://github.com/citum/citum-core/commit/5ee404efc0f55e9ca21fe56481985ec43ec54d54))

- Suppress affixes when value empty ([`4a57ce8`](https://github.com/citum/citum-core/commit/4a57ce8aca88de576f82334939a8fbaf2e8acf23))


**migrate**

- Address clippy and match guards ([`00daf59`](https://github.com/citum/citum-core/commit/00daf59903cd1502dcc8de0672a4096eac08611c))



### Documentation


### Features

**engine**

- Render original publisher/place for reprints ([`d03bc9b`](https://github.com/citum/citum-core/commit/d03bc9beb9d800d22cd2364d4d19494840d9287f))

- Cite-site dynamic compound grouping ([`8e38a8c`](https://github.com/citum/citum-core/commit/8e38a8ca19ee935a883ca82882e95ea2ffab8edd))

- Add unicode sorting ([`0050d13`](https://github.com/citum/citum-core/commit/0050d13195357117db2e14449ecb1c79c70f073e))


**locale**

- Add gender-aware term resolution ([`5af327d`](https://github.com/citum/citum-core/commit/5af327d53ea0fe94bd4e0e682c4e34bc0ac65ee1))


**schema**

- Original publication support ([`72a8a47`](https://github.com/citum/citum-core/commit/72a8a47a98614cabbca97fb8b829afed8334e513))



### Performance

**engine**

- Reuse scratch buffers in disambig ([`5c0d691`](https://github.com/citum/citum-core/commit/5c0d69104009f1f0be70b8b924a49426498960b6))



### Refactor

**schema**

- Strengthen ref and language ids ([`08f7fdc`](https://github.com/citum/citum-core/commit/08f7fdc0929d92fd7e997ecb6aaeb4f5785de11e))

- Rename StylePreset->StyleBase, preset->extends ([`fe59c08`](https://github.com/citum/citum-core/commit/fe59c08807762a1f399ce059c5452e87fc2ea636))


## [0.20.1] - 2026-04-12

### Bug Fixes

**engine**

- Rich-input note parser + title case ([`453b34e`](https://github.com/citum/citum-core/commit/453b34ea3defe49e73208d5ee8c79bfbbe25a26a))

- Chicago anon year-suffix + name-order ([`d1b8239`](https://github.com/citum/citum-core/commit/d1b82399c4e3d965cdbc41d6e992686a90b7ec4b))

- Recover apa datasets ([`aec7db7`](https://github.com/citum/citum-core/commit/aec7db75483a2a394b159f9158386cb8bdcb5f99))

- Restore apa periodical detail ([`7888915`](https://github.com/citum/citum-core/commit/7888915a256d65d5bfaeaa85cc743cd57c224453))

- Apa song routing and term text-case ([`00ac2da`](https://github.com/citum/citum-core/commit/00ac2dac9ae419399f522e84a25ede224810c0d0))

- Render date: original-published ([`f60e802`](https://github.com/citum/citum-core/commit/f60e8028cb1e488e55bd176dbb9ae9fbdf76e846))


**migrate**

- Chicago rich-input pass ([`9aebee5`](https://github.com/citum/citum-core/commit/9aebee5ef14cd13f0d9ceed3a4791bfc03996c6c))


**schema**

- Chicago legal-material support ([`f56ff66`](https://github.com/citum/citum-core/commit/f56ff663d93cb636459103c6cbd268ed01b182a2))

- Close chicago and apa coverage gaps ([`fc9ce24`](https://github.com/citum/citum-core/commit/fc9ce24e376ffcba6e788be785c85e470948f82c))

- Preserve encyclopedia entry semantics ([`043fc06`](https://github.com/citum/citum-core/commit/043fc06dd8448c7cd08e0c9fd096a9ab8a5f7a35))

- Close apa packaging gap ([`d4101e5`](https://github.com/citum/citum-core/commit/d4101e51a3f2d6dfc4c90c032972809684c31999))


**styles**

- Advance apa rich bibliography closure ([`b354609`](https://github.com/citum/citum-core/commit/b354609d269f70bfc000421ee760ab4404e80ec0))

- Restore styles and fix compat.html ([`2dfe525`](https://github.com/citum/citum-core/commit/2dfe525ed39fc34287fb806866e6f53c95ac1707))



### Features

**engine**

- Role-substitute for non-author fallback ([`a4590aa`](https://github.com/citum/citum-core/commit/a4590aa521069abe633ed676dc0db0ff344897d9))

- Apply sentence-initial label context ([`63123d5`](https://github.com/citum/citum-core/commit/63123d5c1f4c2023bf7ec761a35de4012272cbfe))

- Add family-first-only name order ([`45adcf9`](https://github.com/citum/citum-core/commit/45adcf96cf0391723eac02a882cd505b3574054a))


**schema**

- Refine numbering semantics ([`c407f7d`](https://github.com/citum/citum-core/commit/c407f7d34ac9bc1bc569981dcc2f582f6dce070c))

- Support custom numbering + locators ([`74920ed`](https://github.com/citum/citum-core/commit/74920ed05cc9268e99bd1403ac7b636940fbca14))


**styles**

- Co-evolve chicago and apa fidelity ([`fca7dfb`](https://github.com/citum/citum-core/commit/fca7dfb9f8f79ce0bc6ed6de92b8f71b410613f6))

- Close structural fidelity follow-up ([`f6105bf`](https://github.com/citum/citum-core/commit/f6105bf99d91ffa1ab4c61652cb8c3c489a51b2b))



### Refactor

**styles**

- Consolidate embedded style files ([`1f0e513`](https://github.com/citum/citum-core/commit/1f0e513c692a2a4caacc10b10520374303277b2b))


## [0.20.0] - 2026-04-01

### Bug Fixes

**bib**

- Align schemas and edited-book coverage ([`a14a9f4`](https://github.com/citum/citum-core/commit/a14a9f4aba044e11b6f16fcda198974f5161438f))


**engine**

- Suppress orphan bibliography suffix ([`c4551a2`](https://github.com/citum/citum-core/commit/c4551a229e2c9e1c95f3f6534b4550756e84e507))

- Preserve grouped semantics ([`74cb78e`](https://github.com/citum/citum-core/commit/74cb78e3b89efdd67893867588d6bb551ec412a8))

- Render html group headings ([`aa838a1`](https://github.com/citum/citum-core/commit/aa838a198b9ba95e974b9172748d806b4b2ecc72))

- Pre-merge grouped selectors ([`2ec332c`](https://github.com/citum/citum-core/commit/2ec332c000164f9a6fb2f5f008eac8fbdd9bfe27))

- Preserve manual-note ibid locators ([`294d9b2`](https://github.com/citum/citum-core/commit/294d9b253545b69010f53156eebd4b83d1cf2798))

- Show pages with locator in notes ([`0f2fa87`](https://github.com/citum/citum-core/commit/0f2fa87e72091c01b92552833b949c27f2c39ecb))

- Suppress verb labels in author substitutes ([`e265e04`](https://github.com/citum/citum-core/commit/e265e0451b4da7874c176bac30dc55bead9eb21c))


**migrate**

- Group inline journal issue dates ([`68108fb`](https://github.com/citum/citum-core/commit/68108fb91110efeccf4ea3792a3c474685d8f091))


**schema**

- Prefer archive-info location ([`d02e46e`](https://github.com/citum/citum-core/commit/d02e46ec3660a5001fff5f72149351d5611495c1))


**styles**

- Inherit scoped contributor shortening ([`f646a6c`](https://github.com/citum/citum-core/commit/f646a6cdc89237b18295631ad9aaac7fd09e1328))

- Genre capitalize-first in 3 styles ([`277bcf9`](https://github.com/citum/citum-core/commit/277bcf9a882d7f274ed93f3c14c92527ad22f37e))



### Features

**edtf**

- Render historical era suffixes ([`0bce24d`](https://github.com/citum/citum-core/commit/0bce24d8c56d38699fe1fb79e9a0100adefc897f))

- Era label profiles ([`9182543`](https://github.com/citum/citum-core/commit/918254312a3ce4da9da15afc1a46e4195870f461))


**locale**

- Wire guest role MF2 plural dispatch ([`32b6272`](https://github.com/citum/citum-core/commit/32b62725bb8278fb1fc9e214ac5a1b93cc55fc8a))

- Vocab layer for genre/medium ([`e5ee04d`](https://github.com/citum/citum-core/commit/e5ee04d45cde11f67eb02cedd999e27ee7ce002c))


**schema**

- Split nested option scopes ([`84dc668`](https://github.com/citum/citum-core/commit/84dc668cac4f9b66f8145d57d3f6392327da8da4))

- Archival and unpublished support ([`076fa11`](https://github.com/citum/citum-core/commit/076fa1192fd2c97c55375c815e61d4d8e778fad1))

- Nest inner affixes under WrapConfig ([`04c2b5f`](https://github.com/citum/citum-core/commit/04c2b5f507de01e5e7b2cb2c6cd97a040a6e4b6b))

- Implement generalized work relation ([`c3b30e6`](https://github.com/citum/citum-core/commit/c3b30e638990045f4ec8ba696e02adbba78194f1))


**schema-data**

- Normalize genre/medium values ([`6ddf2cf`](https://github.com/citum/citum-core/commit/6ddf2cfa140c9d9d80553c14591d1eae424a1f19))



### Performance

**engine**

- Reduce rendering hot-path cloning ([`6218697`](https://github.com/citum/citum-core/commit/6218697675f45866b3f4c5bef233df3693bb9a11))

- Trim disambiguation allocations ([`ee8c1c0`](https://github.com/citum/citum-core/commit/ee8c1c037512f088683e4bd5f9df6c799927a80d))



### Refactor

**engine**

- Let-else in build_group_key ([`1142209`](https://github.com/citum/citum-core/commit/1142209d672a062d9a36ced7762cd25d5a9e6742))



### Testing

**engine**

- Add behavioral coverage for logic ([`2044021`](https://github.com/citum/citum-core/commit/204402196008634dae4af2df97d8844135e8c276))


**examples**

- Use real published references ([`40f200e`](https://github.com/citum/citum-core/commit/40f200e73d19ec3e24b293eb3b6d6737d800ac81))


## [0.18.0] - 2026-03-25

### Bug Fixes

**engine**

- Split name-form from initialize-with ([`95875c8`](https://github.com/citum/citum-core/commit/95875c8c3509afc9d2b877d97d4ec08b31e722a4))



### Features

**template-v2**

- Implement template schema v2 ([`cab0f41`](https://github.com/citum/citum-core/commit/cab0f41bbdd1300b351093356e536ca5bd234f5f))


## [0.16.0] - 2026-03-22

### Bug Fixes

**citations**

- Use prose joining for integral multicites ([`88a6f62`](https://github.com/citum/citum-core/commit/88a6f629a2a017040a7283dbf1fef6d4d431b4d1))



### Features

**engine**

- Add secondary role label presets ([`9de0e74`](https://github.com/citum/citum-core/commit/9de0e74894a9c5c30c1e7ac816cede991be9884b))



### Performance

**ci**

- Reduce compilation time A+B ([`aac067a`](https://github.com/citum/citum-core/commit/aac067a91ecc8d6528f094eaf0170b80a776282a))


## [0.15.0] - 2026-03-19

### Features

**engine**

- Annotate preview html with template indices ([`20ac734`](https://github.com/citum/citum-core/commit/20ac73405a55cd6e3cb12308d0844a749c903b37))


## [0.14.0] - 2026-03-19

### Bug Fixes

**bibliography**

- Add journal doi fallback policy ([`8853aa8`](https://github.com/citum/citum-core/commit/8853aa86fcb31ba3d592a1676e87178986996b3f))


**chicago**

- Add bibliography sort for anon works ([`c36325d`](https://github.com/citum/citum-core/commit/c36325da9b16ce1e9bf379ed2f4b50c4292ccc03))


**ci**

- Stabilize local and oracle gates ([`3efa6ce`](https://github.com/citum/citum-core/commit/3efa6ce8f34af9a5cdaf9141a9a6e5371939297b))


**delimiter**

- Normalize enum parsing across engine and migrate ([`697d083`](https://github.com/citum/citum-core/commit/697d0838a44d998cd68a9a17eef77b2b6c7bafc3))


**engine**

- Normalize space-only initials formatting ([`906586c`](https://github.com/citum/citum-core/commit/906586c7793862c2b16f520eef85fee18d6919e6))

- Make bibliography sort defaults explicit ([`bfd7e5d`](https://github.com/citum/citum-core/commit/bfd7e5de15baf55c10638571aca26c3c6205f772))

- Repair bibliography block rendering ([`e91f649`](https://github.com/citum/citum-core/commit/e91f649e8c69fdb6754344adb31dd06cb1608df7))

- Add trailing newline after bibliography blocks ([`2bad291`](https://github.com/citum/citum-core/commit/2bad29109301b2f0d1d32f851a187c513898440d))

- Strip YAML frontmatter from rendered output ([`8ea72b9`](https://github.com/citum/citum-core/commit/8ea72b9782e882c03274e4f7135333de4831df3c))

- Sort undated bibliography entries last ([`ec1cfa8`](https://github.com/citum/citum-core/commit/ec1cfa8569567de16bfd3c982ae7652a1e5d63df))

- Smarten leading single quotes ([`351cf5a`](https://github.com/citum/citum-core/commit/351cf5a50e839511fbc037839d8f68395b0d2635))

- Complete high-fit wave 1 regressions ([`da851ef`](https://github.com/citum/citum-core/commit/da851ef9c2a82fdfd61c27e254e26b725b2e619c))

- Format title apostrophe logic ([`6bf61f8`](https://github.com/citum/citum-core/commit/6bf61f81b17bef515692f31a342f790b6fb1705b))

- Thread position into et-al renderer ([`1a5143c`](https://github.com/citum/citum-core/commit/1a5143c501e24ce62a0614d0c1edd0880af52e53))

- Annotation rendering for non-HTML formats ([`121faf0`](https://github.com/citum/citum-core/commit/121faf0dbd1a8115f2d677543113dd3cad8e4759))

- Implement render_org_inline properly ([`9a5a325`](https://github.com/citum/citum-core/commit/9a5a325d04079f96d2f3a00c1da5df9b1f4ae527))

- Finish compound numeric rendering ([`ba5f832`](https://github.com/citum/citum-core/commit/ba5f83226e4a9ae5d8677592e02ed3970fc2b897))

- Sort missing-name works by title ([`6442bd2`](https://github.com/citum/citum-core/commit/6442bd250d6b7d480f90281d94a3935aa721c8a9))

- Preserve harvard no-date citations ([`4039833`](https://github.com/citum/citum-core/commit/4039833ae003ee44a4211c54ecdaece08da69a3a))

- Drop cited-subset numbering ([`93b81d7`](https://github.com/citum/citum-core/commit/93b81d7793aa7e59a3f0b269f6d8d1d080d654cf))

- Correct note-style ibid rendering ([`f295585`](https://github.com/citum/citum-core/commit/f2955852d33d83308bc46348f4327ca44a73edf3))

- Integral ibid in authored notes ([`95e2577`](https://github.com/citum/citum-core/commit/95e2577e82b605022aa796714059b2e7c20f7638))

- Address review feedback ([`d8fa2bc`](https://github.com/citum/citum-core/commit/d8fa2bccd4ae06a389aec43a92a4305c9729049b))

- Integral citation rendering ([`7f03ffd`](https://github.com/citum/citum-core/commit/7f03ffd7065a79f995e70af182515cea7e6150f1))

- Per-cite suffix in grouped citations ([`12b3b0b`](https://github.com/citum/citum-core/commit/12b3b0b9461351e7df3afbfb03457608682a461b))


**labels**

- Make et-al name count configurable per preset ([`f7ea4b0`](https://github.com/citum/citum-core/commit/f7ea4b0620124cecdbdfa7cf42af2a1bf7ad74dd))


**labels,names**

- Docs and test coverage for label mode and space separator ([`e7592fb`](https://github.com/citum/citum-core/commit/e7592fbbbc7c97e8c4a335b56f1e31cb9315807a))


**lint**

- Pedantic autofixes ([`b52e2b0`](https://github.com/citum/citum-core/commit/b52e2b094f2628cb9df5d9bd3f34155f58f70a22))


**locale**

- Lowercase editor short terms (ed./eds.) ([`22bbede`](https://github.com/citum/citum-core/commit/22bbede4c3a80996bbe5d6b451d9a0aeceff10b9))


**oracle**

- Make scoring case-aware ([`b9dccaa`](https://github.com/citum/citum-core/commit/b9dccaab9c91cce36b4e98587c8aa320055e0763))


**render**

- Make HTML bibliography separators markup-aware ([`d3a1da3`](https://github.com/citum/citum-core/commit/d3a1da3fdd44e055c61533ba545896306c2be95b))


**schema**

- Defaults + drop semantic_classes ([`4dd6909`](https://github.com/citum/citum-core/commit/4dd6909f539d6144b05c5216161c8523f587c946))

- Null-aware preset overlay merging ([`080df92`](https://github.com/citum/citum-core/commit/080df92f2efa1e980935bbb2ae92f2ffffeb8267))


**styles**

- Add note repeat overrides ([`b0a735b`](https://github.com/citum/citum-core/commit/b0a735b64b7e804f2936d66b7713682aa23e91ca))



### Documentation

**engine**

- Correct review-driven docs ([`b1c9faf`](https://github.com/citum/citum-core/commit/b1c9faff66152356e860f00c633f5096b150c0cd))

- Cover public support APIs ([`5ba76e9`](https://github.com/citum/citum-core/commit/5ba76e9fc135d58fe96a8f1f3b9880a0dce6bdeb))

- Enforce missing docs coverage ([`8bc109a`](https://github.com/citum/citum-core/commit/8bc109ad5e42d4682dbd87e5a5b02ec71165129f))

- Add /// and unit tests ([`e60b1d7`](https://github.com/citum/citum-core/commit/e60b1d73cd542d4d3ce30e2dabb808fe95787c22))


**schema,engine,migrate**

- Add public API doc comments ([`7162bf7`](https://github.com/citum/citum-core/commit/7162bf712ef22adad2e9a7e4ccdbd250861d1588))



### Features

**bindings**

- Add citum-bindings crate ([`c149b68`](https://github.com/citum/citum-core/commit/c149b685a0c4390c94c02e951ea23f3f7305d1e8))


**citation**

- Unify locator model ([`8ca1646`](https://github.com/citum/citum-core/commit/8ca1646363a1915b4f589d739bc0b5e5f8f6d0d0))

- Align repeated-note position semantics ([`88a33e6`](https://github.com/citum/citum-core/commit/88a33e623400028bf6d8cd625675b657be7e9685))


**citations**

- Add integral name memory ([`73417da`](https://github.com/citum/citum-core/commit/73417dae9ac30b8441edb12af86aeb9d6a03fff1))


**compound-sets**

- Implement sets and subentry ([`2877767`](https://github.com/citum/citum-core/commit/287776717d5ec07c4c2e560e657966bf789d14a8))


**doc**

- Add pandoc markdown citations ([`a95b6ba`](https://github.com/citum/citum-core/commit/a95b6baac4e3ae8a2bbda8afaf28e84f7b28b850))


**document**

- Auto-note djot citations ([`4467bee`](https://github.com/citum/citum-core/commit/4467beefc8cf82453a80903171bf6d80ec5177bf))

- Configure note marker punctuation ([`91317b8`](https://github.com/citum/citum-core/commit/91317b825357738b58509c24ec48dd60b43b4640))


**edtf**

- Implement time component rendering ([`7143adf`](https://github.com/citum/citum-core/commit/7143adf0877cbd31c1ffac6ce6785b37a820090a))


**engine**

- Add document-level bibliography grouping via djot and YAML ([`86dca5f`](https://github.com/citum/citum-core/commit/86dca5f89f98b859f12568bf1969c248b30b9ead))

- Support container short titles ([`29794d0`](https://github.com/citum/citum-core/commit/29794d02c10cf3ef341127a219a9f947b846facd))

- Overhaul and rebrand Citum FFI bindings ([`fcd1219`](https://github.com/citum/citum-core/commit/fcd121998237c30dc7611435cdd92c25f7832ae0))

- Djot inline rendering for annotations ([`0329ee9`](https://github.com/citum/citum-core/commit/0329ee9467eb5dc5cae7127a3db7bae3a8572409))

- Add org-mode input/output ([`f3396aa`](https://github.com/citum/citum-core/commit/f3396aa07cfcca914aa0bc9c3ef01caa2676c463))

- Support expanded verification cases ([`9386467`](https://github.com/citum/citum-core/commit/9386467d94d2342bef929eb1f84ea8dab5a7f136))

- Support djot title markup ([`d2d1921`](https://github.com/citum/citum-core/commit/d2d1921ef3d29dd3cb58e9996d10064fa43d68bf))

- Implement title text-case semantics ([`6d13aa5`](https://github.com/citum/citum-core/commit/6d13aa5b08e63727a3d0c7f60b9e2f3102a7daaf))

- Render_locator subsystem ([`1a4c120`](https://github.com/citum/citum-core/commit/1a4c1205d23d28ef3973a75c6265b62bbdb91c92))

- Resolve style presets ([`be6a293`](https://github.com/citum/citum-core/commit/be6a29345dd8aaae6b23c4212065a96e2d9ff576))


**engine,cli**

- Annotated bibliography support ([`9367000`](https://github.com/citum/citum-core/commit/9367000d723380afd9df8bd946639a521c60ea49))


**lint**

- Clippy all=deny, pedantic suppressions ([`9170a21`](https://github.com/citum/citum-core/commit/9170a2197df4c289e1202cfed840c8450e2b927c))


**locale**

- ICU MF1 locale system ([`6c5c98e`](https://github.com/citum/citum-core/commit/6c5c98e02c7b7cde261e0de8a4d74ffc81967604))


**migrate**

- Tighten inferred bibliography parity heuristics ([`a502c61`](https://github.com/citum/citum-core/commit/a502c61c8843cd14d9fd8a0de2a8093553c3b459))


**multilingual**

- Support language-aware title templates ([`71fe320`](https://github.com/citum/citum-core/commit/71fe32074171e998c6b00ce49bf881e57899d853))

- Add preferred-transliteration ([`b4a8a81`](https://github.com/citum/citum-core/commit/b4a8a81165f48baa90cdd8368a767fdd2e50e644))

- Prove locale bib layouts ([`17dbe16`](https://github.com/citum/citum-core/commit/17dbe163915df806dd3931d0c60231d441c8c067))


**note**

- Audit and complete note styles ([`c28eb5b`](https://github.com/citum/citum-core/commit/c28eb5bf9b38ed1db6c55e47b13dc9d7972330e0))

- Implement note-start conformance ([`7b3e62f`](https://github.com/citum/citum-core/commit/7b3e62fefc87ce8c8903c72ed555bd37d8b9ddce))


**note-styles**

- Ibid/subsequent to chicago-notes ([`a753d2a`](https://github.com/citum/citum-core/commit/a753d2a2d02aa00c3918ad73f72e4a07d17407a4))


**notes**

- Split note-shortening audit layers ([`b199265`](https://github.com/citum/citum-core/commit/b199265438a5d204edf648b6f070eaa864d35060))


**render**

- Preserve link URL in djot rendering ([`7f54d14`](https://github.com/citum/citum-core/commit/7f54d140daced7f91202473b409955ee9d0633f3))


**schema**

- Add SortPreset; use in chicago ([`dd14350`](https://github.com/citum/citum-core/commit/dd143502b78999e61547785a42c139b4c89f190a))

- Add NameForm to ContributorConfig ([`9d394cc`](https://github.com/citum/citum-core/commit/9d394cc5329662598feae3972802035c536235c1))

- Csl support and pr schema gate ([`201bb92`](https://github.com/citum/citum-core/commit/201bb92b87245964303d8adaeac46c5a392bbcb3))

- Compound locator support ([`9b7a578`](https://github.com/citum/citum-core/commit/9b7a57868f7c20e7f0aefa6a1c924b199d198c9e))

- Locator ergonomics ([`2757aa3`](https://github.com/citum/citum-core/commit/2757aa3c78a387496aba51fbc92f63542128fc45))


**schema,engine**

- Add subsequent et-al controls ([`d35f7ca`](https://github.com/citum/citum-core/commit/d35f7ca717195cf5fb54e26e60c23c7a30069957))


**server**

- Add citum-server crate ([`557f780`](https://github.com/citum/citum-core/commit/557f780e6305bc5d82c01b6d19725bd447c77884))


**tests**

- Add csl intake audit ([`902928a`](https://github.com/citum/citum-core/commit/902928ab19a466a9bd2c0b5ec7755ef309e028ac))

- Extract CJK/Arabic CSL fixtures + native test ([`432d2e4`](https://github.com/citum/citum-core/commit/432d2e41726d0b10373d738e1b78928f0cb8146b))


**typst**

- Add native rendering and pdf output ([`c4dbe6f`](https://github.com/citum/citum-core/commit/c4dbe6f96ba5f369513b964618a8fa2fe4d0cf4d))



### Refactor

**citum-engine**

- Simplify rendering pass ([`189745b`](https://github.com/citum/citum-core/commit/189745bfb7cd033ca5770bcbf4a67d6c9ea61d3b))

- Thin processor facade ([`022adfd`](https://github.com/citum/citum-core/commit/022adfd93d97de7d390320d68264f3700d17cc3f))

- Split document module ([`b9504d0`](https://github.com/citum/citum-core/commit/b9504d0108b0dbc19a5928ec77c682c41bf54de9))

- Split rendering module ([`fd6340b`](https://github.com/citum/citum-core/commit/fd6340b98bea803209fe77d5315af148f618ef87))

- Simplify grouping helpers ([`dbd29aa`](https://github.com/citum/citum-core/commit/dbd29aa2b0d3a63af2cda90374772f2f8ae91b81))

- Simplify grouped rendering and disambiguation ([`69caa36`](https://github.com/citum/citum-core/commit/69caa36a6adfae7d918b50538018c398ee64b4cc))

- Address copilot review feedback ([`3d0f31a`](https://github.com/citum/citum-core/commit/3d0f31a377d587195fd0c945ec7519b2fc047bbf))


**djot**

- Split adapter and parsing ([`9295e72`](https://github.com/citum/citum-core/commit/9295e729a70e24cf0fa1e048f3299dc82738bd3c))


**edtf**

- Rename crate csln-edtf → citum-edtf ([`51cfc24`](https://github.com/citum/citum-core/commit/51cfc24ba677424104a8e9a2a77ad60002a8bc03))


**engine**

- Simplify hint calculation ([`283c4bf`](https://github.com/citum/citum-core/commit/283c4bf982e08bc104248fa3b651964640f7a3cb))

- Simplify rendering.rs ([`09d4b97`](https://github.com/citum/citum-core/commit/09d4b9760fddb759aeea8b5fab4c2f849bf2d6fe))

- Simplify citation helpers ([`3ff6819`](https://github.com/citum/citum-core/commit/3ff68194ba2e5f1aed7c4d5ee1cff3a225301540))

- Simplify citation rendering ([`97f4aec`](https://github.com/citum/citum-core/commit/97f4aec7b84d66a2e2673c4a53ea8f6c73887ceb))

- Simplify rendering helpers ([`85852e4`](https://github.com/citum/citum-core/commit/85852e4a1713b43273491f3e8ebfb424854f56f7))

- Simplify document and rendering flows ([`6d43e5a`](https://github.com/citum/citum-core/commit/6d43e5a8e27c7149dfc3b8cff3c9279e807e551f))

- Simplify values/* and io hotspots ([`2ef3667`](https://github.com/citum/citum-core/commit/2ef3667169616bff1620284810361d87e097204b))

- Extract title multilingual config helper ([`6f6c081`](https://github.com/citum/citum-core/commit/6f6c08142a83b51ed9901b00cc79ac8e3696b0ed))

- Extract helpers from format_names/format_single_name ([`3d7f272`](https://github.com/citum/citum-core/commit/3d7f2723bb82054af77601d7a1308d09b73ee99b))

- Split contributor module ([`76294aa`](https://github.com/citum/citum-core/commit/76294aa07312a9b6ff4246b9bedf67ec02b0fa8f))

- Extract ffi biblatex module ([`c0ff00a`](https://github.com/citum/citum-core/commit/c0ff00af596937bca6301423d4a69321326a341a))

- Split processor files ([`0e1955e`](https://github.com/citum/citum-core/commit/0e1955ed0b905163688fbeb3e12d6e0a38f65e1a))

- Remove too_many_arguments allows ([`65ed90f`](https://github.com/citum/citum-core/commit/65ed90f3a130f553c4df4c72375927adb768526d))

- Split grouped.rs into submodules ([`2765db6`](https://github.com/citum/citum-core/commit/2765db6b5195aae314f9370b95bf16cc39fd8314))

- Seal citation parser boundary ([`b5aca0f`](https://github.com/citum/citum-core/commit/b5aca0f32719a5087adc1a8cbab4013b0578abf0))

- Bundle Renderer::new params ([`e2654da`](https://github.com/citum/citum-core/commit/e2654da902da49dc629db82952d780c7bec2ab6e))


**engine,migrate**

- Rust-simplify pass ([`a2b65f7`](https://github.com/citum/citum-core/commit/a2b65f7aa237ce1159bd619178b8241a4d13989b))


**lint**

- Enforce too_many_lines and cognitive_complexity ([`443fcc6`](https://github.com/citum/citum-core/commit/443fcc62801f4518f09f65b21cda02493f19076a))


**migrate**

- Extract fixups module ([`0ef1c15`](https://github.com/citum/citum-core/commit/0ef1c159ba6edda83021000fa255244636c3551f))


**schema,engine**

- Add rendering_mut ([`c35590e`](https://github.com/citum/citum-core/commit/c35590e914f420f938c05577cfa54981482e8db1))


**workspace**

- Migrate csln namespace to citum ([`1e22764`](https://github.com/citum/citum-core/commit/1e227643afb5a01a3aa7ccb4d70dbd086b478b69))



### Testing

**citations**

- Cover empty-date citation sort ([`964ea0c`](https://github.com/citum/citum-core/commit/964ea0cab75c31ccc30b99ba96d896083355869b))


**djot**

- Add adapter pipeline tests ([`d0479d3`](https://github.com/citum/citum-core/commit/d0479d3b7015fc67aabb4c43a05d7239ede5eddf))


**engine**

- Add sort oracle tests ([`3bff72d`](https://github.com/citum/citum-core/commit/3bff72d22e5fb103681c8babc352d6303662c526))

- Cover rendered disambiguation paths ([`f8d57fc`](https://github.com/citum/citum-core/commit/f8d57fc147863b5a809078a094fa410d0dd943d7))

- Annotation rendering unit tests ([`fda0486`](https://github.com/citum/citum-core/commit/fda048626313fd7dce4be570d8bcced43a2c6af9))

- Publish behavior coverage reports ([`0cbb38c`](https://github.com/citum/citum-core/commit/0cbb38c77ee38086181823773174fd3ea3498a59))

- Expand behavior report coverage ([`8c9d572`](https://github.com/citum/citum-core/commit/8c9d57251e99a5417fcaff8a73c1e25a94454919))

- Expand behavior report coverage ([`544e850`](https://github.com/citum/citum-core/commit/544e850f8f46516bfa0a184f34957774555959a5))

- Add disambiguation benchmarks ([`0581202`](https://github.com/citum/citum-core/commit/058120215d7d44ee79e6f3038345d9fac2470824))

- Remove too_many_args suppressions ([`c725769`](https://github.com/citum/citum-core/commit/c725769f49e5bad86d53427e85467947eb399fc4))

- Convert priority-list test to rstest ([`751ddba`](https://github.com/citum/citum-core/commit/751ddbaa5cbd3cfb23d10e2b63aec90a826edb59))


**grouped**

- Add regression tests for grouped modes ([`04124c1`](https://github.com/citum/citum-core/commit/04124c1ed780af2638aaa1958dfed23cebc08b19))


