# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.36.0] - 2026-05-05

### Features

**tooling**

- Add RPC workflow benchmark ([`5508188`](https://github.com/citum/citum-core/commit/550818805e8dfcbec056397860a277c02a423310))


## [0.30.2] - 2026-04-30

### Refactor

## [0.29.1] - 2026-04-29

### Bug Fixes

**tooling**

- Harden store and RPC error paths ([`8379713`](https://github.com/citum/citum-core/commit/8379713edf850928ff9917ff8077524cf657c82c))



### Refactor

**engine**

- Rename csln- prefix to citum- ([`b19293d`](https://github.com/citum/citum-core/commit/b19293d76fc13b01a41615db69303a1bab879b84))


**styles**

- Consolidate embedded style files ([`1f0e513`](https://github.com/citum/citum-core/commit/1f0e513c692a2a4caacc10b10520374303277b2b))


## [0.18.0] - 2026-03-25

### Features

**template-v2**

- Implement template schema v2 ([`cab0f41`](https://github.com/citum/citum-core/commit/cab0f41bbdd1300b351093356e536ca5bd234f5f))


## [0.15.0] - 2026-03-19

### Bug Fixes

**lint**

- Pedantic autofixes ([`b52e2b0`](https://github.com/citum/citum-core/commit/b52e2b094f2628cb9df5d9bd3f34155f58f70a22))


**schema**

- Null-aware preset overlay merging ([`080df92`](https://github.com/citum/citum-core/commit/080df92f2efa1e980935bbb2ae92f2ffffeb8267))


**server**

- Enforce docs and invalid formats ([`1ae2faa`](https://github.com/citum/citum-core/commit/1ae2faa851e40f0d78b6fd9db9199fe485b6ad2a))



### Documentation

**cli**

- Unify help output with a summary-and-detail model ([`ae95b1d`](https://github.com/citum/citum-core/commit/ae95b1dc98a6d70de476c42269a1335666e83cf7))


**server**

- Add HTTP curl example to README ([`aeae9e8`](https://github.com/citum/citum-core/commit/aeae9e86a7670e574efd9ddced1c38f7ae6a6515))

- Fix stdio example with valid JSON ([`ed229e2`](https://github.com/citum/citum-core/commit/ed229e2812289d7d3a54dd32009c22109c95a350))



### Features

**engine**

- Annotate preview html with template indices ([`20ac734`](https://github.com/citum/citum-core/commit/20ac73405a55cd6e3cb12308d0844a749c903b37))


**lint**

- Clippy all=deny, pedantic suppressions ([`9170a21`](https://github.com/citum/citum-core/commit/9170a2197df4c289e1202cfed840c8450e2b927c))


**server**

- Add citum-server crate ([`557f780`](https://github.com/citum/citum-core/commit/557f780e6305bc5d82c01b6d19725bd447c77884))

- Support output formats ([`f876c1a`](https://github.com/citum/citum-core/commit/f876c1a1e70e31969807b6a3d94f745a35cd14b0))

- Upgrade CLI with clap and custom styling ([`f144382`](https://github.com/citum/citum-core/commit/f1443825c08b4b7ca844b772860507a13d85e631))


**typst**

- Add native rendering and pdf output ([`c4dbe6f`](https://github.com/citum/citum-core/commit/c4dbe6f96ba5f369513b964618a8fa2fe4d0cf4d))



### Testing

**server**

- Add RPC dispatcher integration tests ([`3679b36`](https://github.com/citum/citum-core/commit/3679b36c54347ce70b0e0c08dcd6b496b7b42101))

- Cover http mode ([`f61026f`](https://github.com/citum/citum-core/commit/f61026fe6846e7c3271b6eeba2c9970afe67e15d))


