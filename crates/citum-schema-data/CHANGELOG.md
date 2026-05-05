# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.30.3] - 2026-04-30

### Features

**schema**

- Bill+authority → hearing ([`8344296`](https://github.com/citum/citum-core/commit/8344296643458059b9118181b9e6c1b91163b67b))


## [0.30.2] - 2026-04-30

### Refactor

## [0.30.0] - 2026-04-29

### Features

**schema**

- Part/supplement/printing numbers ([`47a296e`](https://github.com/citum/citum-core/commit/47a296ee882e199845a276869deca48b5a7a461c))

- Unify geographic place type ([`2788571`](https://github.com/citum/citum-core/commit/27885717b4efbf71cea0663854772c77b2bcfbcb))


## [0.26.0] - 2026-04-26

### Bug Fixes

**engine**

- Rich-input note parser + title case ([`453b34e`](https://github.com/citum/citum-core/commit/453b34ea3defe49e73208d5ee8c79bfbbe25a26a))

- Recover apa datasets ([`aec7db7`](https://github.com/citum/citum-core/commit/aec7db75483a2a394b159f9158386cb8bdcb5f99))

- Apa song routing and term text-case ([`00ac2da`](https://github.com/citum/citum-core/commit/00ac2dac9ae419399f522e84a25ede224810c0d0))

- Citation type routing improvements ([`ac2d949`](https://github.com/citum/citum-core/commit/ac2d949cef8f1dbfe944941acaf989325974803e))


**schema**

- Chicago legal-material support ([`f56ff66`](https://github.com/citum/citum-core/commit/f56ff663d93cb636459103c6cbd268ed01b182a2))

- Archive-collection & short-title ([`82feb9d`](https://github.com/citum/citum-core/commit/82feb9d567f7664d2f3b550d95ad8ee206ad3028))

- Organizer fallback + changelog ([`1479b47`](https://github.com/citum/citum-core/commit/1479b4750acf994f8d03432528265ecc3c5d4372))

- Close chicago and apa coverage gaps ([`fc9ce24`](https://github.com/citum/citum-core/commit/fc9ce24e376ffcba6e788be785c85e470948f82c))

- Preserve collection status ([`7384538`](https://github.com/citum/citum-core/commit/7384538fa99f36c6b0777f4023f289fff0acd496))

- Preserve encyclopedia entry semantics ([`043fc06`](https://github.com/citum/citum-core/commit/043fc06dd8448c7cd08e0c9fd096a9ab8a5f7a35))

- Close apa packaging gap ([`d4101e5`](https://github.com/citum/citum-core/commit/d4101e51a3f2d6dfc4c90c032972809684c31999))


**styles**

- Advance apa rich bibliography closure ([`b354609`](https://github.com/citum/citum-core/commit/b354609d269f70bfc000421ee760ab4404e80ec0))



### Features

**engine**

- Render original publisher/place for reprints ([`d03bc9b`](https://github.com/citum/citum-core/commit/d03bc9beb9d800d22cd2364d4d19494840d9287f))

- Cite-site dynamic compound grouping ([`8e38a8c`](https://github.com/citum/citum-core/commit/8e38a8ca19ee935a883ca82882e95ea2ffab8edd))


**locale**

- Add gender-aware term resolution ([`5af327d`](https://github.com/citum/citum-core/commit/5af327d53ea0fe94bd4e0e682c4e34bc0ac65ee1))


**migrate**

- Convert zotero notes to example ([`cbaadb9`](https://github.com/citum/citum-core/commit/cbaadb98df9818e192372aa678a0198b75984154))


**schema**

- Refine numbering semantics ([`c407f7d`](https://github.com/citum/citum-core/commit/c407f7d34ac9bc1bc569981dcc2f582f6dce070c))

- Support custom numbering + locators ([`74920ed`](https://github.com/citum/citum-core/commit/74920ed05cc9268e99bd1403ac7b636940fbca14))

- Chicago/apa coverage batch 1-6 ([`84d9645`](https://github.com/citum/citum-core/commit/84d964559427a8dd604d057badb1f6ec569a2c59))

- Event field for paper-conference ([`f1d9ee0`](https://github.com/citum/citum-core/commit/f1d9ee08579c946b45ac98cdf927c3e226645447))

- Original publication support ([`72a8a47`](https://github.com/citum/citum-core/commit/72a8a47a98614cabbca97fb8b829afed8334e513))


**styles**

- Close structural fidelity follow-up ([`f6105bf`](https://github.com/citum/citum-core/commit/f6105bf99d91ffa1ab4c61652cb8c3c489a51b2b))



### Refactor

**schema**

- Strengthen ref and language ids ([`08f7fdc`](https://github.com/citum/citum-core/commit/08f7fdc0929d92fd7e997ecb6aaeb4f5785de11e))

- Richtext enum for note/abstract ([`f426887`](https://github.com/citum/citum-core/commit/f4268878437fffcb2c5c293038922d89a9ab1055))



### Testing

**schema**

- Cover note parser conversion ([`1b64862`](https://github.com/citum/citum-core/commit/1b64862b2c9ad436f3474ab7d8e362f81243d174))

- Add migration regressions ([`51a62c0`](https://github.com/citum/citum-core/commit/51a62c0804a2a090c95051bacdeb93cbfd3cd5d2))


## [0.20.0] - 2026-04-01

### Bug Fixes

**bib**

- Align schemas and edited-book coverage ([`a14a9f4`](https://github.com/citum/citum-core/commit/a14a9f4aba044e11b6f16fcda198974f5161438f))


**schema**

- Prefer archive-info location ([`d02e46e`](https://github.com/citum/citum-core/commit/d02e46ec3660a5001fff5f72149351d5611495c1))



### Documentation

**schema**

- Type modeling policy + doc audit ([`945b5a7`](https://github.com/citum/citum-core/commit/945b5a75e1a02cc1b41a2c5a87b1a38de359e645))



### Features

**locale**

- Wire guest role MF2 plural dispatch ([`32b6272`](https://github.com/citum/citum-core/commit/32b62725bb8278fb1fc9e214ac5a1b93cc55fc8a))


**schema**

- Archival and unpublished support ([`076fa11`](https://github.com/citum/citum-core/commit/076fa1192fd2c97c55375c815e61d4d8e778fad1))

- Implement generalized work relation ([`c3b30e6`](https://github.com/citum/citum-core/commit/c3b30e638990045f4ec8ba696e02adbba78194f1))


**schema-data**

- Normalize genre/medium values ([`6ddf2cf`](https://github.com/citum/citum-core/commit/6ddf2cfa140c9d9d80553c14591d1eae424a1f19))



### Refactor

**schema**

- Split types.rs (csl26-0ueb) ([`13bf9d4`](https://github.com/citum/citum-core/commit/13bf9d4d326973a12704a918842e14ae9a0cd1f2))


## [0.18.0] - 2026-03-25

### Features

**bindings**

- Promote wasm api + specta types ([`50376ae`](https://github.com/citum/citum-core/commit/50376ae1a244db83af72596bd240ce72435c2712))


**template-v2**

- Implement template schema v2 ([`cab0f41`](https://github.com/citum/citum-core/commit/cab0f41bbdd1300b351093356e536ca5bd234f5f))


## [0.14.0] - 2026-03-19

### Features

**core**

- Split schema and convert namespace ([`c44d279`](https://github.com/citum/citum-core/commit/c44d27978b5ddad97cb47147162452b1751e2d93))


**schema**

- Layered style preset overrides ([`eb8533a`](https://github.com/citum/citum-core/commit/eb8533a4e1aec6fd79f6cffb0453223fa4cd53fe))



### Refactor

