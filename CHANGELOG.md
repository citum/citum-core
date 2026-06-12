# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.68.0] - 2026-06-12

### Bug Fixes

**engine**

- Skip suppressed consumption ([`a989446`](https://github.com/citum/citum-core/commit/a989446dfc45d1e48ea0fdb6283dd0e7f32f18bf))


**migrate**

- Strip suppressed variable poison ([`93f8e85`](https://github.com/citum/citum-core/commit/93f8e8526b308901b3cc18eb0d1f918de2327e4b))

- Note-class citation repeat forms ([`5f3561e`](https://github.com/citum/citum-core/commit/5f3561ec1f4fa477bb2ff97cff391aae390e54bd))

- Full variants in wrapper emission ([`8c15f9d`](https://github.com/citum/citum-core/commit/8c15f9da948e01921581ca6eb6325bf99f2485b8))

- Keep default-branch bib order ([`a7afdbb`](https://github.com/citum/citum-core/commit/a7afdbbb3fed7a916d8d9b6b339bddca0ed50e08))



### Documentation

**spec**

- Test-soundness ledger + shared bar ([`f97c022`](https://github.com/citum/citum-core/commit/f97c022db9c86a73d9b76a3e9e33d578ea5f147e))

- Clarify two sorting silences ([`5113b25`](https://github.com/citum/citum-core/commit/5113b250f87c07154e6b29b7823534f52408b38a))

- Fill two multilingual spec silences ([`c36ded1`](https://github.com/citum/citum-core/commit/c36ded1d813f09cf3d66a6b805b5f7537cfda3be))

- Define consumption semantics ([`ed746ec`](https://github.com/citum/citum-core/commit/ed746ec0e8c4804bbf4292265a8889308a31cc57))



### Features

**i18n**

- Romanized-script-translated preset ([`4c6d2e5`](https://github.com/citum/citum-core/commit/4c6d2e544eee984cf8c3ff1fafc201cd7754a5fd))


**migrate**

- Random-corpus sqi scorecard mode ([`d28c6e9`](https://github.com/citum/citum-core/commit/d28c6e915a9a16ce2d6b392b04d4bc89f9446ef6))

- Measured citation selection ([`d296408`](https://github.com/citum/citum-core/commit/d296408f5d3c2f2a024b756592c0d4532257aea2))

- Add measured candidate selection ([`666cda1`](https://github.com/citum/citum-core/commit/666cda13979ee5a745ed782a0478ffdde0dd3e30))


**tooling**

- Rework test-soundness-review skill ([`6eb53df`](https://github.com/citum/citum-core/commit/6eb53dfadb0f5f0a4e99154001fe668ad1adee21))

- Auto-proceed after soundness audit ([`9624fe3`](https://github.com/citum/citum-core/commit/9624fe3533097a57ea2873626ce3280144a951e9))



### Refactor

**i18n**

- Original-script view + drop CNE ([`d6f2faf`](https://github.com/citum/citum-core/commit/d6f2faff6c66ef6f8b75031cbb038fd755aa3afe))



### Testing

**engine**

- Apply SORTING.md soundness findings ([`d2bbac0`](https://github.com/citum/citum-core/commit/d2bbac0e952fb3e958b8693133de16d9d022f24d))

- Harden disambiguation soundness ([`de54051`](https://github.com/citum/citum-core/commit/de54051f04b0916577956f2d1d1349f7cd97f3ea))

- Harden multilingual test soundness ([`28f2335`](https://github.com/citum/citum-core/commit/28f233502edbf2242593796ef3f21314fb973b13))


## [0.67.0] - 2026-06-09

### Bug Fixes

**engine**

- Warn on unknown reference fields ([`0016e96`](https://github.com/citum/citum-core/commit/0016e965d86c27f43895326784779445d80c2fa7))

- Render cne name patterns natively ([`7e2a709`](https://github.com/citum/citum-core/commit/7e2a70915d2c961b2f5087b2588c17608cbfad26))


**examples**

- Use Murakami instead of Murasaki ([`aab7d4d`](https://github.com/citum/citum-core/commit/aab7d4da0a88c0b887d5f1eb4974c4b95384bd0f))



### Documentation

**examples**

- Promote cne refs to examples/ ([`279b990`](https://github.com/citum/citum-core/commit/279b99058ad35cb4afccbfe816b58f04243fecba))



### Features

**i18n**

- Add cne chicago fixtures ([`5e3bbe0`](https://github.com/citum/citum-core/commit/5e3bbe0e9aa8a2a6495097a696e06824db8189a4))


## [0.66.0] - 2026-06-09

### Bug Fixes

**ci**

- Scope release notes to previous tag ([`558e6c0`](https://github.com/citum/citum-core/commit/558e6c06469dc0168cc76b6c791ebbe67c9c853e))


**engine**

- Same-author collapse first-class rule ([`1272383`](https://github.com/citum/citum-core/commit/12723834a1331de7af76abd8daf18beb91b7bb5b))



### Documentation

**spec**

- Reconcile bib grouping vocabulary ([`31e6daa`](https://github.com/citum/citum-core/commit/31e6daae5885cc84d71811422662b527513ce82d))

- Expand citation cluster rendering spec ([`9c7c4b2`](https://github.com/citum/citum-core/commit/9c7c4b22e24b84a623c3eac8f85273089eb5213f))



### Features

**edtf**

- Add FromStr and ParseError ([`7fccb7a`](https://github.com/citum/citum-core/commit/7fccb7ac327dfd488cb6fc8b0a824a530237011d))


**engine**

- Nocite bibliography-only entries ([`22212e9`](https://github.com/citum/citum-core/commit/22212e9d6046a5c882c91d29b9fd2899ea630b24))



### Refactor

**engine**

- Fold bib groups onto blocks path ([`3e75f63`](https://github.com/citum/citum-core/commit/3e75f63c0ddb9d0f35c05f6d2232f5e271f10b58))

- Unify document bib rendering ([`b5b01ba`](https://github.com/citum/citum-core/commit/b5b01bad751eccc5bc67f89713d896822b33acd7))


## [0.65.0] - 2026-06-08

### Features

**engine**

- Consolidate bib block rendering ([`fd0c6ee`](https://github.com/citum/citum-core/commit/fd0c6eee114d657a701eb5b5c1a72bc194940a83))


## [0.64.0] - 2026-06-07

### Bug Fixes

**engine**

- Honor preview citation mode ([`f86e5a7`](https://github.com/citum/citum-core/commit/f86e5a740d8af9ed204ea1a2f89e04c2cea086f2))

- Format bibliography entry html ([`80caaf6`](https://github.com/citum/citum-core/commit/80caaf6e58df707bb97809cde82076ed1f26a47b))

- Compose typst strong markup ([`52906b4`](https://github.com/citum/citum-core/commit/52906b43af6fef8b3f44aa7fc20f912e81d4b1f8))



### Documentation

**server**

- Add session API docs, fix examples ([`ce91379`](https://github.com/citum/citum-core/commit/ce91379f44cf30a4f4cabe8f677248edaf494785))


**tooling**

- Formalize repo-local harness ([`ff7cd2f`](https://github.com/citum/citum-core/commit/ff7cd2f71c720e93e505e4888d115c3f618f9ede))



### Features

**engine**

- Per-document style overrides ([`8d2f61e`](https://github.com/citum/citum-core/commit/8d2f61e495484b21b2d55652c03723505474506b))

- Wire name-memory through session API ([`7689814`](https://github.com/citum/citum-core/commit/7689814d30c03b09c85ea29e2c15eb47653dfe73))


## [0.63.0] - 2026-06-04

### Documentation

**spec**

- Activate per-doc overrides spec ([`d2c6d8b`](https://github.com/citum/citum-core/commit/d2c6d8b70540afc1ac2f3dfc006635d5cc4fdb91))



### Features

**schema**

- Multilingual presets + pattern mode ([`98ca8e4`](https://github.com/citum/citum-core/commit/98ca8e4cc66132f2dfbd15ca82d906be14e5d974))

- Citation sentence_start signal ([`acf6c29`](https://github.com/citum/citum-core/commit/acf6c29ffa418fc2ff4fc0f5eb30815baeb0968e))


**server**

- Add session api ([`ecea9dd`](https://github.com/citum/citum-core/commit/ecea9ddfaf5ac6d0be991ce7a4e2cbcc868aa878))



### Refactor

**engine**

- Clarify delimiter-join loops ([`baf9953`](https://github.com/citum/citum-core/commit/baf9953464e45f24c8d65cbde476e4e1d02d7c30))


## [0.62.0] - 2026-06-02

### Bug Fixes

**engine**

- Implement by-cite givenname expansion ([`b0fb8ca`](https://github.com/citum/citum-core/commit/b0fb8cac47bedaf9d0b84f4b80e9b0cfbc4a1558))

- Primary-name falls to suffix on tie ([`daf2748`](https://github.com/citum/citum-core/commit/daf2748fd7ce0d25cfc5392e0c9bf194b9376faa))


**styles**

- Chicago em-dash + cross-entry infra ([`b91dd61`](https://github.com/citum/citum-core/commit/b91dd616c22a25eef6391595437dd132934f8b01))



### Documentation

**spec**

- Cross-entry fidelity + givenname gap ([`c9b5565`](https://github.com/citum/citum-core/commit/c9b5565ffa9ba5406523f9fd331035dfde200c62))

- Align givenname-disambiguation-rule ([`03b9463`](https://github.com/citum/citum-core/commit/03b9463200f6994b1d7558413349c633aa2a4415))



### Features

**engine**

- Add biblatex refs input variant ([`b4e088d`](https://github.com/citum/citum-core/commit/b4e088de4498ec008f892a3358bbe040accb0386))


**schema**

- Givenname-disambiguation-rule ([`327252a`](https://github.com/citum/citum-core/commit/327252a0bf3d50b26a8effeb784e735e8a0453a9))



### Testing

**report**

- Regen oracle snapshots for fixtures ([`430dc6f`](https://github.com/citum/citum-core/commit/430dc6f49ee578f1ee4d8f13ac629ec72f3d5bb5))

- Refresh oracle baseline ([`9178e9c`](https://github.com/citum/citum-core/commit/9178e9c3a1652a741d40b671be2ddc0c302c80ca))


## [0.61.1] - 2026-06-01

### Bug Fixes

**docs**

- Update jsr package name ([`aff4854`](https://github.com/citum/citum-core/commit/aff4854081083173051e1ac7114a39c7ceadae82))

- Capabilties link ([`9f91a6b`](https://github.com/citum/citum-core/commit/9f91a6bdecf4bdb8cf77bb4f2270ac6763ec40b9))

- The other capabilities link ([`78ef5cf`](https://github.com/citum/citum-core/commit/78ef5cfe58dad2935177df0df989abb39b876543))

- Point to specs/ rather than arch ([`68e5685`](https://github.com/citum/citum-core/commit/68e56855119ddd772808ceab6c13ef7f55fe2abd))


**engine**

- Document bib restricted to cited refs ([`545328c`](https://github.com/citum/citum-core/commit/545328c52669a5ca7ae6cf341315439c23ceae87))



### Documentation

## [0.61.0] - 2026-06-01

### Bug Fixes

**engine**

- Honor grouped author-date delimiter ([`03f7be0`](https://github.com/citum/citum-core/commit/03f7be0680b17e01457ec67567764de0c2730cf8))

- Dedup across fenced-div bib blocks ([`9130123`](https://github.com/citum/citum-core/commit/9130123268a649ce6f52f9d7654dac3c1118c863))


**styles**

- Chicago author-date locator ([`e46afc4`](https://github.com/citum/citum-core/commit/e46afc4a0e6eddc106e925a398874bc5bc04d293))



### Documentation

**spec**

- Update markdown pandoc workflow ([`086db76`](https://github.com/citum/citum-core/commit/086db768bd0064f70b7623428012cdc7fb0c4a8d))

- Add capability index + SORTING.md ([`e3e258e`](https://github.com/citum/citum-core/commit/e3e258eec8029fc7a42d3fbb4af223d7df9a3ded))

- Fold completed rows + fix alint ([`fcf1e73`](https://github.com/citum/citum-core/commit/fcf1e73a30c27eefaf44b8b409ab7f413c3fc9c0))

- Fix 7 status mismatches + SORTING note ([`f3ed853`](https://github.com/citum/citum-core/commit/f3ed8539f82ed303bf1881b9ce3c09f716af2d94))

- Fix WASM + TEMPLATE_V3 vs reality ([`5080d78`](https://github.com/citum/citum-core/commit/5080d782ba96102805b6ac42bf1e209cc79dd8d1))



### Features

**engine**

- Add markdown output format ([`89b5142`](https://github.com/citum/citum-core/commit/89b5142ce02b817dc57936e866e1fcbabdbd913c))

- Markdown footnote placement ([`ec5110e`](https://github.com/citum/citum-core/commit/ec5110eabf2039767a3852b925722946d56bb413))

- Multilingual disambiguation key ([`3452db0`](https://github.com/citum/citum-core/commit/3452db03c7b51acd019a7716b6cff54c80f6fc65))

- Add format_document_with_resolver ([`8e2ce23`](https://github.com/citum/citum-core/commit/8e2ce23b9633649e4b7bebe2eb55eac27b2d7eee))



### Testing

**engine**

- Harden disambiguation test soundness ([`aa7af75`](https://github.com/citum/citum-core/commit/aa7af75828cd5459a2d9e4ea8c2bffff8f707071))


**locale**

- Add MaybeGendered snapshot coverage ([`787780d`](https://github.com/citum/citum-core/commit/787780d36d9fe6c923061bc62aaec3a0fade31cd))


## [0.60.0] - 2026-05-30

### Bug Fixes

**engine**

- Convert body markup for typst/latex ([`f278031`](https://github.com/citum/citum-core/commit/f2780312a20bddc92732f85b662709e281418a23))

- Fix inline code fences and escaping ([`d095c8f`](https://github.com/citum/citum-core/commit/d095c8f6d3d8722c6c54b1e5b49841c45af61bff))

- Convert markdown body markup to html ([`31b0503`](https://github.com/citum/citum-core/commit/31b05033476e14ca30459a603aa43c1b13b03954))


## [0.59.0] - 2026-05-29

### Bug Fixes

**ci**

- Add citum-refs to publish-crates order ([`0b7b654`](https://github.com/citum/citum-core/commit/0b7b654d80a9312075501bc7a09c9f35918dd439))


**demo**

- Style bibliography section headings ([`480b6c4`](https://github.com/citum/citum-core/commit/480b6c4f050dd5705e0589ab9e444111136d0f12))

- Split bib heading styles; enlarge h1 ([`e4df8d8`](https://github.com/citum/citum-core/commit/e4df8d86b304bf96d8012fab58da7e3f23ef2abc))

- Enlarge page title h1 to 2rem ([`59d5d1f`](https://github.com/citum/citum-core/commit/59d5d1f94baddd1d64c1244f048b7f913478d2c5))


**engine**

- Apply mode options to Renderer ([`c4c78e1`](https://github.com/citum/citum-core/commit/c4c78e16313e512e00c443150860035afed3a284))

- Resolve multilingual data-* attrs ([`e50702d`](https://github.com/citum/citum-core/commit/e50702dc1e190f6fb97a9086d6131105574c1e67))



### Documentation

**demo**

- Generate page from djot via engine ([`4d367e9`](https://github.com/citum/citum-core/commit/4d367e9f35ae029d2f6619cbc82e165614ebbc6e))


**spec**

- Consolidate disambiguation spec ([`3b7ac2f`](https://github.com/citum/citum-core/commit/3b7ac2fc9865bdf55245c61aab3b5e334a986278))



### Features

**engine**

- Year-suffix issued date only ([`94cb79d`](https://github.com/citum/citum-core/commit/94cb79d45391a458a987ed188be0aa21992fa25e))

- Suppress disambig title in xref ([`84b0070`](https://github.com/citum/citum-core/commit/84b00701d3d7c864580f885432c6ff39d0852f2d))


**schema**

- Add disambiguate.ignore option ([`73f4b70`](https://github.com/citum/citum-core/commit/73f4b70463846cd9195411e29aef00e30f05c6c0))



### Performance

**schema**

- Typed merge_style_overlay ([`98495bd`](https://github.com/citum/citum-core/commit/98495bd4d78909f9ef815e2d6e78c554159bc306))



### Refactor

**schema**

- Dry structural from deser impls ([`b54ba09`](https://github.com/citum/citum-core/commit/b54ba09bb05fdbc81b9e7d8591a312d47a9b40e6))


## [0.58.0] - 2026-05-27

### Bug Fixes

**ci**

- Explicit changelog generation step ([`a824da0`](https://github.com/citum/citum-core/commit/a824da0d5dc47836a3b7d4ed318f117f4e57d180))

- Replace dorny/paths-filter with git ([`d9ba15e`](https://github.com/citum/citum-core/commit/d9ba15e8d6e1daacccbc5f66e4cd70900cf3656f))

- Legal fixture path after spec dir move ([`f4d044f`](https://github.com/citum/citum-core/commit/f4d044fcb572d800ecf7c21347246af487dfba52))


**engine**

- Romanize multilingual APA titles ([`e931850`](https://github.com/citum/citum-core/commit/e931850c537f45ccb8e72b20c6317514800db8b9))

- Parse negated djot bibliography types ([`c9f9338`](https://github.com/citum/citum-core/commit/c9f9338ff1bfd1cbfad7d81beb5ba07067f84061))


**locale**

- Wire date forms to pattern.date-* ([`b6b6e75`](https://github.com/citum/citum-core/commit/b6b6e75a7a17dfa062fcc15b36792f124b357c5d))



### Documentation

**engine**

- Specify multilingual names ([`27f628a`](https://github.com/citum/citum-core/commit/27f628a80a67ded49dd16c754507306f18a29fd6))

- Per-document config overrides spec ([`3a287e8`](https://github.com/citum/citum-core/commit/3a287e8250e3403d15b14182657a705e1d717c73))



### Features

**cli**

- Add --locale flag to render doc ([`736201b`](https://github.com/citum/citum-core/commit/736201b30e324411ed7ce4d49fcdf7b54b38f37f))


**engine**

- Support script name separators ([`26f0159`](https://github.com/citum/citum-core/commit/26f01593210781a87088f8614e38739dfa427c78))

- Split name-memory configs ([`3d1495b`](https://github.com/citum/citum-core/commit/3d1495b77d3f5df0aa9c98e31a5e3a1d5d52602d))

- Document options override structs ([`4d2ebf7`](https://github.com/citum/citum-core/commit/4d2ebf7e967942a6ac74eea25c121ca2f2729711))

- Wire document options into pipeline ([`004bd37`](https://github.com/citum/citum-core/commit/004bd374118f16412e40b280ab24b088f4c6021b))



### Build

**tooling**

- Add wasm-release cargo profile ([`73ffad8`](https://github.com/citum/citum-core/commit/73ffad8b16838eb08f79ee40d49f522d26366b98))


## [0.57.0] - 2026-05-25

### Bug Fixes

**release**

- Align schema changelog ([`3ad75c3`](https://github.com/citum/citum-core/commit/3ad75c3504ca616029ebb1dfce8e107ac8617533))



### Documentation

**release**

- Add jsr package readme ([`e72f343`](https://github.com/citum/citum-core/commit/e72f343100b02be74ad7b2f9dd51a96d123bcbd1))


## [0.42.0] - 2026-05-13

### Bug Fixes

**ci**

- Action versions and clippy hygiene ([`544ecd6`](https://github.com/citum/citum-core/commit/544ecd6d02454a885845c6dcf1f78dc0d5c186e8))


## [0.39.0] - 2026-05-11

### Features

**bindings**

- Add WASM performance benchmark ([`e187def`](https://github.com/citum/citum-core/commit/e187def85a248521a1ea6a041c43083aeb9aa5ea))


**schema**

- Rename template reuse ([`df4c4ea`](https://github.com/citum/citum-core/commit/df4c4eaaa6e7b69486634a47a1ed355eb22a82ed))


**server**

- Wire format_document arm + wasm ([`4b7dc22`](https://github.com/citum/citum-core/commit/4b7dc22834eda499ad8e9920de3c68bee4ae5a35))


## [0.35.0] - 2026-05-03

### Features

**cli**

- Distributed architecture phase 1 ([`7ae000d`](https://github.com/citum/citum-core/commit/7ae000d5d4ed3764e52634f954e4633f7f58d3bb))


## [0.34.0] - 2026-05-03

### Features

**schema**

- Rename template reuse ([`c95ddba`](https://github.com/citum/citum-core/commit/c95ddba8b5463f5aa8109b7dfb56dbc2b8ca992f))


## [0.30.2] - 2026-04-30

### Refactor

**schema**

- Strengthen ref and language ids ([`08f7fdc`](https://github.com/citum/citum-core/commit/08f7fdc0929d92fd7e997ecb6aaeb4f5785de11e))


**styles**

- Consolidate embedded style files ([`1f0e513`](https://github.com/citum/citum-core/commit/1f0e513c692a2a4caacc10b10520374303277b2b))


## [0.18.0] - 2026-03-25

### Bug Fixes

**lint**

- Pedantic autofixes ([`b52e2b0`](https://github.com/citum/citum-core/commit/b52e2b094f2628cb9df5d9bd3f34155f58f70a22))


**schema**

- Null-aware preset overlay merging ([`080df92`](https://github.com/citum/citum-core/commit/080df92f2efa1e980935bbb2ae92f2ffffeb8267))



### Features

**bindings**

- Add citum-bindings crate ([`c149b68`](https://github.com/citum/citum-core/commit/c149b685a0c4390c94c02e951ea23f3f7305d1e8))

- Promote wasm api + specta types ([`50376ae`](https://github.com/citum/citum-core/commit/50376ae1a244db83af72596bd240ce72435c2712))


**lint**

- Clippy all=deny, pedantic suppressions ([`9170a21`](https://github.com/citum/citum-core/commit/9170a2197df4c289e1202cfed840c8450e2b927c))


