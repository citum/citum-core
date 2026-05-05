# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.30.2] - 2026-04-30

### Bug Fixes

**clippy**

- Resolve used underscore bindings and collapsible ifs in discovery tool ([`d1bb1e5`](https://github.com/citum/citum-core/commit/d1bb1e5e7c17dfc67db1a710992d4bba17376c0d))


**migrate**

- Address clippy and match guards ([`00daf59`](https://github.com/citum/citum-core/commit/00daf59903cd1502dcc8de0672a4096eac08611c))



### Features

**analyze**

- Add automated profile candidate discovery ([`309e7ac`](https://github.com/citum/citum-core/commit/309e7ac2f6f93f427a1b74a364b7e23f1ede76b8))

- Add automated profile candidate discovery ([`9e13a17`](https://github.com/citum/citum-core/commit/9e13a17bb9cf88cd228b2ebdf2d457cc0de9b97c))


**styles**

- Convert audited journal wrappers ([`a775f27`](https://github.com/citum/citum-core/commit/a775f2753b2c66a1bdf1ca603268dc2166903a8e))



### Refactor

**rust**

- Dedup helpers, trim allocs ([`6298d68`](https://github.com/citum/citum-core/commit/6298d6894743820d4375ec71566f92d08c9a7f41))


## [0.14.0] - 2026-03-19

### Bug Fixes

**lint**

- Pedantic autofixes ([`b52e2b0`](https://github.com/citum/citum-core/commit/b52e2b094f2628cb9df5d9bd3f34155f58f70a22))



### Features

**citum-analyze**

- Preset migration savings ([`1eac12a`](https://github.com/citum/citum-core/commit/1eac12a986966a8111a12393a58092d7edd73b16))


**lint**

- Clippy all=deny, pedantic suppressions ([`9170a21`](https://github.com/citum/citum-core/commit/9170a2197df4c289e1202cfed840c8450e2b927c))



### Refactor

**lint**

- Enforce too_many_lines and cognitive_complexity ([`443fcc6`](https://github.com/citum/citum-core/commit/443fcc62801f4518f09f65b21cda02493f19076a))


**workspace**

- Migrate csln namespace to citum ([`1e22764`](https://github.com/citum/citum-core/commit/1e227643afb5a01a3aa7ccb4d70dbd086b478b69))


