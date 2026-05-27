# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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


