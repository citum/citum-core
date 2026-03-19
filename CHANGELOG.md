# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [unreleased]

## [0.14.0] - 2026-03-19

### Bug Fixes

**schema**

- Null-aware preset overlay merging ([`080df92`](https://github.com/citum/citum-core/commit/080df92f2efa1e980935bbb2ae92f2ffffeb8267))


### Features

**analyze**

- Preset migration savings ([`1eac12a`](https://github.com/citum/citum-core/commit/1eac12a4ea7d4c561d642201ce65217dc2158fb1))


**engine**

- Resolve style presets ([`be6a293`](https://github.com/citum/citum-core/commit/be6a293a7f970ff434770a0e66d1a39e649fa2de))


**locale**

- ICU MF1 locale system ([`6c5c98e`](https://github.com/citum/citum-core/commit/6c5c98efb4ad0ecfa462b3d94b2e38ac44c5cbe1))


**schema**

- Layered style preset overrides ([`eb8533a`](https://github.com/citum/citum-core/commit/eb8533ad8fa4d8252746f783f244e7f7c5874d15))

- Default style schema version bumped to `0.10.0` for the preset architecture additions

### Features

**engine**

- Render_locator subsystem ([`1a4c120`](https://github.com/citum/citum-core/commit/1a4c1205d23d28ef3973a75c6265b62bbdb91c92))


**schema**

- Short_name + edition on StyleInfo ([`bce8be2`](https://github.com/citum/citum-core/commit/bce8be2660818fdcef4716f447693140f5791e92))

- Style-level locator rendering config ([`a30b39c`](https://github.com/citum/citum-core/commit/a30b39c18c265e7f11ebf283d99fe48cb8894151))



### Refactor

**engine**

- Bundle Renderer::new params ([`e2654da`](https://github.com/citum/citum-core/commit/e2654da902da49dc629db82952d780c7bec2ab6e))


**migrate**

- Drop locator label fields ([`c205cc2`](https://github.com/citum/citum-core/commit/c205cc28b451132c87d71276c25aa9ac02329ef3))


## [0.12.0] - 2026-03-17

### Bug Fixes

**bibliography**

- Add journal doi fallback policy ([`8853aa8`](https://github.com/citum/citum-core/commit/8853aa86fcb31ba3d592a1676e87178986996b3f))


**engine**

- Integral citation rendering ([`7f03ffd`](https://github.com/citum/citum-core/commit/7f03ffd7065a79f995e70af182515cea7e6150f1))

- Per-cite suffix in grouped citations ([`12b3b0b`](https://github.com/citum/citum-core/commit/12b3b0b9461351e7df3afbfb03457608682a461b))


**lint**

- Pedantic autofixes ([`b52e2b0`](https://github.com/citum/citum-core/commit/b52e2b094f2628cb9df5d9bd3f34155f58f70a22))


**schema**

- Defaults + drop semantic_classes ([`4dd6909`](https://github.com/citum/citum-core/commit/4dd6909f539d6144b05c5216161c8523f587c946))


**styles**

- OSCOLA citation/bibliography fixes ([`cb7f9a0`](https://github.com/citum/citum-core/commit/cb7f9a0c86f9a7cd3968698eec5bf52dc15e4749))

- RSC type-templates and suppressions ([`bc5b18e`](https://github.com/citum/citum-core/commit/bc5b18efc76b838aea031bd1489f8f4b42b3a5ce))

- AGU type-templates ([`2472f34`](https://github.com/citum/citum-core/commit/2472f34966eeef93bb1a22a61a998636f5b7d2b5))



### Documentation

**architecture**

- Define extension governance ([`cf26cec`](https://github.com/citum/citum-core/commit/cf26cec35ce1f6cf486d4fb5e1d987f13a8bd273))


**rules**

- Harden pre-push gate rules ([`064bc07`](https://github.com/citum/citum-core/commit/064bc07f9c9532c26346ca00ffc7fd64f0f68224))


**skills**

- Style-evolve co-evolution update ([`dbc49b3`](https://github.com/citum/citum-core/commit/dbc49b36020a4edf182b9d06ea3d55a03efcec5b))


**standards**

- Add test style rule ([`a213654`](https://github.com/citum/citum-core/commit/a213654a854072ad9c1e1e73c3e23c4d65b9c79e))



### Features

**beans**

- Track external style authoring ([`762da74`](https://github.com/citum/citum-core/commit/762da744b6bd6e977b3d676ac13aea46743047a4))


**lint**

- Clippy all=deny, pedantic suppressions ([`9170a21`](https://github.com/citum/citum-core/commit/9170a2197df4c289e1202cfed840c8450e2b927c))



### Refactor

**citum-engine**

- Thin processor facade ([`022adfd`](https://github.com/citum/citum-core/commit/022adfd93d97de7d390320d68264f3700d17cc3f))

- Split document module ([`b9504d0`](https://github.com/citum/citum-core/commit/b9504d0108b0dbc19a5928ec77c682c41bf54de9))

- Split rendering module ([`fd6340b`](https://github.com/citum/citum-core/commit/fd6340b98bea803209fe77d5315af148f618ef87))

- Simplify grouping helpers ([`dbd29aa`](https://github.com/citum/citum-core/commit/dbd29aa2b0d3a63af2cda90374772f2f8ae91b81))

- Simplify grouped rendering and disambiguation ([`69caa36`](https://github.com/citum/citum-core/commit/69caa36a6adfae7d918b50538018c398ee64b4cc))

- Address copilot review feedback ([`3d0f31a`](https://github.com/citum/citum-core/commit/3d0f31a377d587195fd0c945ec7519b2fc047bbf))


**citum-migrate**

- Simplify upsampler ([`d37bfd5`](https://github.com/citum/citum-core/commit/d37bfd519c56097b76520c1d29150232d3593c29))


**cli**

- Remove duplicate refs module ([`7aaf910`](https://github.com/citum/citum-core/commit/7aaf910ce8e7823fc13f89c4917d9a6ee43f410b))

- Bundle render params into context ([`f7746d4`](https://github.com/citum/citum-core/commit/f7746d46d05f5c321afcbb151c74658b2d4f1218))


**djot**

- Split adapter and parsing ([`9295e72`](https://github.com/citum/citum-core/commit/9295e729a70e24cf0fa1e048f3299dc82738bd3c))


**engine**

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


**engine,migrate**

- Rust-simplify pass ([`a2b65f7`](https://github.com/citum/citum-core/commit/a2b65f7aa237ce1159bd619178b8241a4d13989b))


**lint**

- Enforce too_many_lines and cognitive_complexity ([`443fcc6`](https://github.com/citum/citum-core/commit/443fcc62801f4518f09f65b21cda02493f19076a))


**migrate**

- Extract fixups module ([`0ef1c15`](https://github.com/citum/citum-core/commit/0ef1c159ba6edda83021000fa255244636c3551f))

- Remove only_used_in_recursion allow ([`59207f5`](https://github.com/citum/citum-core/commit/59207f50c494472d5227b6d87b8f8ac715619fd6))

- Split fixups modules ([`f2df944`](https://github.com/citum/citum-core/commit/f2df944efbf6a978c32fe47e8d1c23e41254d7be))


**schema,engine**

- Add rendering_mut ([`c35590e`](https://github.com/citum/citum-core/commit/c35590e914f420f938c05577cfa54981482e8db1))



### Testing

**djot**

- Add adapter pipeline tests ([`d0479d3`](https://github.com/citum/citum-core/commit/d0479d3b7015fc67aabb4c43a05d7239ede5eddf))


**engine**

- Remove too_many_args suppressions ([`c725769`](https://github.com/citum/citum-core/commit/c725769f49e5bad86d53427e85467947eb399fc4))

- Convert priority-list test to rstest ([`751ddba`](https://github.com/citum/citum-core/commit/751ddbaa5cbd3cfb23d10e2b63aec90a826edb59))


**grouped**

- Add regression tests for grouped modes ([`04124c1`](https://github.com/citum/citum-core/commit/04124c1ed780af2638aaa1958dfed23cebc08b19))



### Track

**beans**

- Log rust-refine pass (csl26-l13e) ([`b6d8da1`](https://github.com/citum/citum-core/commit/b6d8da1f1154ccce863b339f5b1a4fa3f5b50a03))


## [0.11.0] - 2026-03-13

### Bug Fixes

**engine**

- Address review feedback ([`d8fa2bc`](https://github.com/citum/citum-core/commit/d8fa2bccd4ae06a389aec43a92a4305c9729049b))


**migrate**

- Support complex position trees ([`50b0c99`](https://github.com/citum/citum-core/commit/50b0c9925fbf995cebb9fa86b84233731f0ff017))


**notes**

- Harden conformance audit reporting ([`e3b56c9`](https://github.com/citum/citum-core/commit/e3b56c98b9645a183dfdda5daf413d305a6d053b))


**oracle**

- Make scoring case-aware ([`b9dccaa`](https://github.com/citum/citum-core/commit/b9dccaab9c91cce36b4e98587c8aa320055e0763))


**release**

- Sync workspace release detection ([`3a3e9aa`](https://github.com/citum/citum-core/commit/3a3e9aa7122d748df4b4c45352292c3ac6fdc7b9))

- Surface release-pr failures ([`314c528`](https://github.com/citum/citum-core/commit/314c528c64c4ea6eeab1be327451b404d490765b))

- Use tagged manifest baseline ([`5b85a73`](https://github.com/citum/citum-core/commit/5b85a7330ce53b5ca618030f47d384d62e3ec6c4))

- Clarify code-track release PR body ([`86f097f`](https://github.com/citum/citum-core/commit/86f097f34d67b1079a585069f4a58d971b31a476))

- Remove unsupported release-plz flag ([`b503567`](https://github.com/citum/citum-core/commit/b503567e6c1d6fda43bd5abb387e7ac87bd5ed60))


**reports**

- Trim section boilerplate ([`700c0f1`](https://github.com/citum/citum-core/commit/700c0f1d0288af1aec62f57ad5a0523878d75e1d))

- Turn overview into toc ([`9942782`](https://github.com/citum/citum-core/commit/9942782cb5df4fde0f6fa7dfd2a6787f136af531))


**styles**

- Add note repeat overrides ([`b0a735b`](https://github.com/citum/citum-core/commit/b0a735b64b7e804f2936d66b7713682aa23e91ca))



### Documentation

**notes**

- Add note-start follow-up spec ([`17acc7a`](https://github.com/citum/citum-core/commit/17acc7abdbe7e6bc716e662be093a48a0cd95c06))


**skills**

- Add engine behavior reporting skill ([`c1dbd63`](https://github.com/citum/citum-core/commit/c1dbd63a7539881d27d74f8232bed81d2b367ac3))


**spec**

- Settle note-start policy ([`3ab84ed`](https://github.com/citum/citum-core/commit/3ab84ede2b1d4d0e01298d368756e1b9ebfd9032))

- Define title text-case semantics ([`537da5b`](https://github.com/citum/citum-core/commit/537da5bd104317333523a444cdad2521ccb22c7f))

- Fix title yaml examples ([`2a27ede`](https://github.com/citum/citum-core/commit/2a27ede4781092e828475c9e9d1220956feef60d))



### Features

**beans**

- Add title case impl task ([`21f2300`](https://github.com/citum/citum-core/commit/21f23000b6aead0c353ba32229a0b42739a5e538))


**citation**

- Align repeated-note position semantics ([`88a33e6`](https://github.com/citum/citum-core/commit/88a33e623400028bf6d8cd625675b657be7e9685))


**engine**

- Implement title text-case semantics ([`6d13aa5`](https://github.com/citum/citum-core/commit/6d13aa5b08e63727a3d0c7f60b9e2f3102a7daaf))


**migrate**

- Support mixed note position trees ([`26025bc`](https://github.com/citum/citum-core/commit/26025bcb3417a99c4dd20f796a9af4dfb9ab4d25))


**note**

- Audit and complete note styles ([`c28eb5b`](https://github.com/citum/citum-core/commit/c28eb5bf9b38ed1db6c55e47b13dc9d7972330e0))

- Implement note-start conformance ([`7b3e62f`](https://github.com/citum/citum-core/commit/7b3e62fefc87ce8c8903c72ed555bd37d8b9ddce))


**notes**

- Split note-shortening audit layers ([`b199265`](https://github.com/citum/citum-core/commit/b199265438a5d204edf648b6f070eaa864d35060))


**report**

- Add migration behavior coverage ([`53fc74c`](https://github.com/citum/citum-core/commit/53fc74c839b129d807c8db89bf441620d578809a))



### Refactor

**citum-engine**

- Simplify rendering pass ([`189745b`](https://github.com/citum/citum-core/commit/189745bfb7cd033ca5770bcbf4a67d6c9ea61d3b))


**engine**

- Simplify hint calculation ([`283c4bf`](https://github.com/citum/citum-core/commit/283c4bf982e08bc104248fa3b651964640f7a3cb))


**schema**

- Dedupe facade crate ([`cf79130`](https://github.com/citum/citum-core/commit/cf79130f85d34d2b383280107e814a2d32a24b4c))



### Testing

**engine**

- Publish behavior coverage reports ([`0cbb38c`](https://github.com/citum/citum-core/commit/0cbb38c77ee38086181823773174fd3ea3498a59))

- Expand behavior report coverage ([`8c9d572`](https://github.com/citum/citum-core/commit/8c9d57251e99a5417fcaff8a73c1e25a94454919))

- Expand behavior report coverage ([`544e850`](https://github.com/citum/citum-core/commit/544e850f8f46516bfa0a184f34957774555959a5))

- Expand behavior report coverage ([`08b4e3e`](https://github.com/citum/citum-core/commit/08b4e3e7ca794d9ed413eb520b8aaa434027eca3))

- Add disambiguation benchmarks ([`0581202`](https://github.com/citum/citum-core/commit/058120215d7d44ee79e6f3038345d9fac2470824))


**migration**

- Expand csl-to-citum reporting ([`29d89ae`](https://github.com/citum/citum-core/commit/29d89ae622059695d6dc1ad32cabf8fe34664e2b))


## [0.10.0] - 2026-03-10

### Bug Fixes

**convert**

- Preserve refs fidelity across csl-json and ris ([`67d85d8`](https://github.com/citum/citum-core/commit/67d85d8cc967b9ab6a3414df63a75c1a5753002b))


**engine**

- Correct note-style ibid rendering ([`f295585`](https://github.com/citum/citum-core/commit/f2955852d33d83308bc46348f4327ca44a73edf3))

- Integral ibid in authored notes ([`95e2577`](https://github.com/citum/citum-core/commit/95e2577e82b605022aa796714059b2e7c20f7638))



### Documentation

**guide**

- Fix schema and add missing features ([`5d8c5d7`](https://github.com/citum/citum-core/commit/5d8c5d794e57db354eac30fc9ddcb4b4dc543974))

- Broaden author-date mode description ([`336e79d`](https://github.com/citum/citum-core/commit/336e79d461a8733c2ac20b783dd3bd7f01812069))


**i18n**

- Reframe gendered terms spec ([`2e8a30c`](https://github.com/citum/citum-core/commit/2e8a30c65639b48a9a13d2456198478b1279d580))


**spec**

- Add note-context spec draft ([`ad2ac10`](https://github.com/citum/citum-core/commit/ad2ac104d710aabadb598d317b3360aa1b6f4ede))

- Refine authored-note ibid rules ([`43923f9`](https://github.com/citum/citum-core/commit/43923f9312f219ef7999de83b91ce9ebf930dc8d))

- Draft schema split+convert ([`51cd163`](https://github.com/citum/citum-core/commit/51cd163f55702fe0af4987a11edde56a8b8aa9ca))



### Features

**core**

- Split schema and convert namespace ([`c44d279`](https://github.com/citum/citum-core/commit/c44d27978b5ddad97cb47147162452b1751e2d93))


**i18n**

- Spec for gendered locale term forms ([`f54ce43`](https://github.com/citum/citum-core/commit/f54ce438face4106e99ffa2ff2e1da7c3838b88f))



### Refactor

**cli**

- Consolidate convert-refs path ([`199c2dc`](https://github.com/citum/citum-core/commit/199c2dcfa1dc97963a67a1d8b071a6f208c63309))


## [0.9.0] - 2026-03-09

### Bug Fixes

**beans**

- Close stale umbrella epics ([`aca0238`](https://github.com/citum/citum-core/commit/aca023874c223a225dbc000d75f3dea955e4758b))


**cli**

- Batch citations only for numeric ([`8f22dd6`](https://github.com/citum/citum-core/commit/8f22dd67e12e7e26750dcf903eb6785cb3c62b76))

- Keep citation-file error marker ([`1741561`](https://github.com/citum/citum-core/commit/17415612ebfa461b3c8e5d87500ccb15b9c86cc0))


**engine**

- Finish compound numeric rendering ([`ba5f832`](https://github.com/citum/citum-core/commit/ba5f83226e4a9ae5d8677592e02ed3970fc2b897))

- Sort missing-name works by title ([`6442bd2`](https://github.com/citum/citum-core/commit/6442bd250d6b7d480f90281d94a3935aa721c8a9))

- Preserve harvard no-date citations ([`4039833`](https://github.com/citum/citum-core/commit/4039833ae003ee44a4211c54ecdaece08da69a3a))

- Drop cited-subset numbering ([`93b81d7`](https://github.com/citum/citum-core/commit/93b81d7793aa7e59a3f0b269f6d8d1d080d654cf))


**migrate**

- Normalize locator labels ([`46e2cd8`](https://github.com/citum/citum-core/commit/46e2cd8f69f11b4cb6ba2bacc99a725890cb3120))

- Preserve strip-periods in migration ([`66685d9`](https://github.com/citum/citum-core/commit/66685d9cca2196243ccafce2d997363dd13f501b))


**oracle**

- Address copilot review findings ([`cdb030a`](https://github.com/citum/citum-core/commit/cdb030a8213db3be5fde948aa8b1145a438c58d9))

- Compound style coverage and SQI ([`123dba7`](https://github.com/citum/citum-core/commit/123dba72366096dee10cddefc64c41663c5bd80f))


**report**

- Clarify compat provenance ([`b68050a`](https://github.com/citum/citum-core/commit/b68050a524f4f01da7edcafc7c0ece6ed0909594))

- Improve compat accuracy ([`ae00610`](https://github.com/citum/citum-core/commit/ae006101b830f12e2247a3b8b8d7b828558c4092))

- Align family-scoped verification ([`00fd31c`](https://github.com/citum/citum-core/commit/00fd31c2873d56abbf2fc2509d7e019e57d3da56))


**snapshots**

- Fix stale note-style snapshots ([`b53ed7e`](https://github.com/citum/citum-core/commit/b53ed7ed125a5bafcb6b120749150496d4eedf53))

- Auto-detect note styles in oracle ([`eaf0858`](https://github.com/citum/citum-core/commit/eaf08588bb5043940281b9c6d117f6e1042a71ae))

- Pdftotext label-padding artifact ([`7e5c7be`](https://github.com/citum/citum-core/commit/7e5c7be0b7493f15ba8190ad85afd0c8f990204f))


**styles**

- Recover family fixture fidelity ([`2e2ce8e`](https://github.com/citum/citum-core/commit/2e2ce8ecdd6ce2159b9e97d204dd64ad50f4f554))

- Finish compound numeric fidelity ([`75bcf71`](https://github.com/citum/citum-core/commit/75bcf71c179ea06f36e4786b42e7b1862d582dde))

- Raise compatibility floor ([`105330f`](https://github.com/citum/citum-core/commit/105330f64465fe16da4e021a5127fe27709363b1))

- Upgrade angewandte and numeric-comp ([`56d026e`](https://github.com/citum/citum-core/commit/56d026e24852a2faab424acda2ceebd04112b5e2))

- Restore core style fidelity ([`09dfb73`](https://github.com/citum/citum-core/commit/09dfb7397e0bc8eac898c80d5e1ddcc9d2029207))

- Harvard-ctr citations 17/18 ([`931eb39`](https://github.com/citum/citum-core/commit/931eb395106a3e385d4f01ffafcecca04c9949e7))


**verification**

- Handle div-004 in gates ([`2450ad2`](https://github.com/citum/citum-core/commit/2450ad21c7da3c95d52313a3e087e388a40a87cf))

- Merge divergence summaries ([`56e0675`](https://github.com/citum/citum-core/commit/56e0675b559dcfb898cae03f4c9af602844bc1f8))



### Documentation

**adjudication**

- Add divergence register ([`2fa9035`](https://github.com/citum/citum-core/commit/2fa9035c5689c4757604f7ce3ea8a2d3babd8051))


**policies**

- Tighten bean lifecycle rules ([`4e1b6a4`](https://github.com/citum/citum-core/commit/4e1b6a450d036225ca8f476711b292f2b15fe6fe))


**skills**

- Improve style-evolve skill ([`dfe9e6d`](https://github.com/citum/citum-core/commit/dfe9e6d1f5d4ec126b5fbe4523af0a2bdaaf9502))

- Add beans frontmatter + commit rule ([`a64c947`](https://github.com/citum/citum-core/commit/a64c9474cb8b76d220981c11d09a9febe313457b))

- Require divergence preflight ([`751b341`](https://github.com/citum/citum-core/commit/751b341a9f4dd36f37b4f850b04fb2f3b4cd00ca))


**spec**

- Add pandoc markdown citations ([`6913759`](https://github.com/citum/citum-core/commit/6913759c442329dcb392eff84d35289b650adc12))


**styles**

- Add authority divergence guidance ([`6d5a258`](https://github.com/citum/citum-core/commit/6d5a25898a95ef82c93c9f5202bdb32ae6633bee))



### Features

**beans**

- Dependency-aware /beans next ranking ([`65842c0`](https://github.com/citum/citum-core/commit/65842c00a4f789b6519f3bf2794ca0ed877b972d))


**bindings**

- Add citum-bindings crate ([`c149b68`](https://github.com/citum/citum-core/commit/c149b685a0c4390c94c02e951ea23f3f7305d1e8))


**citation**

- Unify locator model ([`8ca1646`](https://github.com/citum/citum-core/commit/8ca1646363a1915b4f589d739bc0b5e5f8f6d0d0))


**citations**

- Add integral name memory ([`73417da`](https://github.com/citum/citum-core/commit/73417dae9ac30b8441edb12af86aeb9d6a03fff1))


**doc**

- Add pandoc markdown citations ([`a95b6ba`](https://github.com/citum/citum-core/commit/a95b6baac4e3ae8a2bbda8afaf28e84f7b28b850))


**engine**

- Support expanded verification cases ([`9386467`](https://github.com/citum/citum-core/commit/9386467d94d2342bef929eb1f84ea8dab5a7f136))

- Support djot title markup ([`d2d1921`](https://github.com/citum/citum-core/commit/d2d1921ef3d29dd3cb58e9996d10064fa43d68bf))


**multilingual**

- Prove locale bib layouts ([`17dbe16`](https://github.com/citum/citum-core/commit/17dbe163915df806dd3931d0c60231d441c8c067))


**oracle**

- Static snapshot infrastructure ([`3ef6b5d`](https://github.com/citum/citum-core/commit/3ef6b5d1b522f904836475117ec8c483af31b5d7))

- Biblatex snapshot generator ([`2689f92`](https://github.com/citum/citum-core/commit/2689f92564fb652602fa14c67dd859f901f53347))


**render**

- Preserve link URL in djot rendering ([`7f54d14`](https://github.com/citum/citum-core/commit/7f54d140daced7f91202473b409955ee9d0633f3))


**report**

- Snapshot oracle for native styles ([`4c4a533`](https://github.com/citum/citum-core/commit/4c4a533f8a01e2b2ca10aba7bd03c4558232496c))

- Add verification policy model ([`a77526e`](https://github.com/citum/citum-core/commit/a77526e9ed0b27a5ecdd8f3beb450d13c7a266bf))


**skills**

- Add jcodemunch skill ([`8fce1c3`](https://github.com/citum/citum-core/commit/8fce1c3c9749109bf295ba3074d5b1de8cec3825))


**styles**

- Add 5 numeric-compound styles ([`93c4d5e`](https://github.com/citum/citum-core/commit/93c4d5e684948c610f847a68b38e2c9faae8094c))



### Refactor

**skills**

- Tighten co-evolution discipline ([`173b97c`](https://github.com/citum/citum-core/commit/173b97cf3732a7ac36988d2a6ea031d4c7a6fadb))



### Testing

**fixtures**

- Add coverage audit and fill gaps ([`9c76957`](https://github.com/citum/citum-core/commit/9c7695720504b86233d6f407893b35051439408a))


**report**

- Add family fixture coverage ([`6662483`](https://github.com/citum/citum-core/commit/6662483f72c69eae6fbc9d8e4176c4c9d3b11977))


## [0.8.0] - 2026-03-05

### Bug Fixes

**ci**

- Remove non-oracle portfolio style ([`e04c1bc`](https://github.com/citum/citum-core/commit/e04c1bc92b10d8518b8d148a75e8bded289bc272))


**edtf**

- Validate explicit date parts ([`9c102b0`](https://github.com/citum/citum-core/commit/9c102b06a98d9334a30e1e98f51257b94484fa77))


**engine**

- Annotation rendering for non-HTML formats ([`121faf0`](https://github.com/citum/citum-core/commit/121faf0dbd1a8115f2d677543113dd3cad8e4759))

- Implement render_org_inline properly ([`9a5a325`](https://github.com/citum/citum-core/commit/9a5a325d04079f96d2f3a00c1da5df9b1f4ae527))


**scripts**

- Replace bump workflow with python tool ([`e756236`](https://github.com/citum/citum-core/commit/e756236144e44bca2d5bc289483ed9f31d6446e7))

- Make bump workflow schema-only ([`f045bbc`](https://github.com/citum/citum-core/commit/f045bbc951bd5065f4670c8aaeb5b4cdb2f3c167))


**server**

- Enforce docs and invalid formats ([`1ae2faa`](https://github.com/citum/citum-core/commit/1ae2faa851e40f0d78b6fd9db9199fe485b6ad2a))


**styles**

- Fix chicago-author-date validation ([`00665cd`](https://github.com/citum/citum-core/commit/00665cd16d56811500f8540294d5ab7f9e370301))



### Documentation

**architecture**

- Annotations on by default ([`ec34b75`](https://github.com/citum/citum-core/commit/ec34b7549db0672086dbf7f4cc5e67d8fd1a7d3b))

- Djot rich text design and implementation notes ([`7bffce8`](https://github.com/citum/citum-core/commit/7bffce84a484856f17d81d0eb0a950e9821b540c))


**cli**

- Unify help output with a summary-and-detail model ([`ae95b1d`](https://github.com/citum/citum-core/commit/ae95b1dc98a6d70de476c42269a1335666e83cf7))


**compound-sets**

- Align design and examples ([`4143dd1`](https://github.com/citum/citum-core/commit/4143dd1beb0f68feea0b1a869239e71427fc7fdd))


**csl-legacy, citum-cli**

- Add doc and test coverage ([`172e74c`](https://github.com/citum/citum-core/commit/172e74cbbd47d2f96303062ff2370506f6b60773))


**engine**

- Correct review-driven docs ([`b1c9faf`](https://github.com/citum/citum-core/commit/b1c9faff66152356e860f00c633f5096b150c0cd))

- Cover public support APIs ([`5ba76e9`](https://github.com/citum/citum-core/commit/5ba76e9fc135d58fe96a8f1f3b9880a0dce6bdeb))

- Enforce missing docs coverage ([`8bc109a`](https://github.com/citum/citum-core/commit/8bc109ad5e42d4682dbd87e5a5b02ec71165129f))

- Add /// and unit tests ([`e60b1d7`](https://github.com/citum/citum-core/commit/e60b1d73cd542d4d3ce30e2dabb808fe95787c22))


**examples**

- Mention djot markup support in annotated bibliography section ([`29258a6`](https://github.com/citum/citum-core/commit/29258a6b75427d91a94ef19342a580fb988f571c))

- Clarify smallcaps is a Citum convention on djot spans ([`76269a7`](https://github.com/citum/citum-core/commit/76269a7655c5c71186d29c4459f6a31ebc2bb34f))


**migrate**

- Add /// to options_extractor fns ([`00ab2ef`](https://github.com/citum/citum-core/commit/00ab2ef1c944ad828d534dfa751de292dd3abb47))


**schema**

- Cover root style-model docs ([`553070d`](https://github.com/citum/citum-core/commit/553070d99437e174577460405d5735c03cad94b0))

- Cover renderer docs ([`af0fb99`](https://github.com/citum/citum-core/commit/af0fb99d59c019b46b02f949cf1d5cf6ba9d4928))

- Cover citation locator docs ([`8e980c1`](https://github.com/citum/citum-core/commit/8e980c179ff06ce8b50f3ee84066b3484344092c))

- Cover locale support docs ([`f804402`](https://github.com/citum/citum-core/commit/f8044025a4f7b3e72246000e85a61aae6086f675))

- Document and test renderer ([`052bb8f`](https://github.com/citum/citum-core/commit/052bb8f227b8f2c578699e6ca191eb8d4c22a997))

- Document and test locale types ([`781f89b`](https://github.com/citum/citum-core/commit/781f89bd32a9b608f4ecb26e645949fa8a3674e9))

- Document and test processing options ([`9450c70`](https://github.com/citum/citum-core/commit/9450c70b40a7c63ac852fc5bddac30b5fa0b568b))

- Align versioning docs with bump workflow ([`d4197b6`](https://github.com/citum/citum-core/commit/d4197b64e0a85411b6650bb6b8008bc364463b93))



### Features

**cli**

- Add metadata and styles to help output ([`ce34090`](https://github.com/citum/citum-core/commit/ce34090c33ebcfcc06b99a997a06eff261d1e5a6))

- Add detailed examples to convert command help ([`b3cf5bf`](https://github.com/citum/citum-core/commit/b3cf5bf4b288323ff150bded29186d1ae96dcd96))


**cli,store**

- Integrate citum_store with CLI and resolve user styles ([`ad22314`](https://github.com/citum/citum-core/commit/ad22314a0a8d634a84bb33e14c82749863ff92b1))


**compound-sets**

- Implement sets and subentry ([`2877767`](https://github.com/citum/citum-core/commit/287776717d5ec07c4c2e560e657966bf789d14a8))


**engine**

- Overhaul and rebrand Citum FFI bindings ([`fcd1219`](https://github.com/citum/citum-core/commit/fcd121998237c30dc7611435cdd92c25f7832ae0))

- Djot inline rendering for annotations ([`0329ee9`](https://github.com/citum/citum-core/commit/0329ee9467eb5dc5cae7127a3db7bae3a8572409))

- Add org-mode input/output ([`f3396aa`](https://github.com/citum/citum-core/commit/f3396aa07cfcca914aa0bc9c3ef01caa2676c463))


**engine,cli**

- Annotated bibliography support ([`9367000`](https://github.com/citum/citum-core/commit/9367000d723380afd9df8bd946639a521c60ea49))


**migrate**

- Improve inferred parity and migrate-only oracle tooling ([`0aa012e`](https://github.com/citum/citum-core/commit/0aa012e80dcd415961eb79626bbe946ccf195b5b))

- Tighten inferred bibliography parity heuristics ([`a502c61`](https://github.com/citum/citum-core/commit/a502c61c8843cd14d9fd8a0de2a8093553c3b459))

- Normalize legal-case fields by style id ([`859f304`](https://github.com/citum/citum-core/commit/859f3045f6f4191421da563a34ba87441a91007a))


**schema**

- Add NameForm to ContributorConfig ([`9d394cc`](https://github.com/citum/citum-core/commit/9d394cc5329662598feae3972802035c536235c1))

- Remove AndOptions::ModeDependent ([`334a486`](https://github.com/citum/citum-core/commit/334a486ec1260d15e31aafbbf22b0380354ad2ea))

- Csl support and pr schema gate ([`201bb92`](https://github.com/citum/citum-core/commit/201bb92b87245964303d8adaeac46c5a392bbcb3))

- Compound locator support ([`9b7a578`](https://github.com/citum/citum-core/commit/9b7a57868f7c20e7f0aefa6a1c924b199d198c9e))

- Locator ergonomics ([`2757aa3`](https://github.com/citum/citum-core/commit/2757aa3c78a387496aba51fbc92f63542128fc45))


**scripts**

- Bump-schema.sh -> unified bump.sh ([`d6dbe9f`](https://github.com/citum/citum-core/commit/d6dbe9fb69b0020cfdcd6b9258d4ced72c92d34c))


**server**

- Upgrade CLI with clap and custom styling ([`f144382`](https://github.com/citum/citum-core/commit/f1443825c08b4b7ca844b772860507a13d85e631))



### Testing

**engine**

- Cover rendered disambiguation paths ([`f8d57fc`](https://github.com/citum/citum-core/commit/f8d57fc147863b5a809078a094fa410d0dd943d7))

- Annotation rendering unit tests ([`fda0486`](https://github.com/citum/citum-core/commit/fda048626313fd7dce4be570d8bcced43a2c6af9))


**store**

- Add resolver integration tests ([`b8262f7`](https://github.com/citum/citum-core/commit/b8262f72c433a0ae3b5cf66f6bb4b101191b670f))


## [0.7.0] - 2026-03-01

### Bug Fixes

**beans**

- Filter non-actionable statuses in next ([`e783693`](https://github.com/citum/citum-core/commit/e78369331444dc4b39cbacf92df5e0f52fec80d5))

- Prefer executable next targets ([`d045d5f`](https://github.com/citum/citum-core/commit/d045d5fb8360252971f8c6c12f98886ef91d2b90))


**chicago**

- Add bibliography sort for anon works ([`c36325d`](https://github.com/citum/citum-core/commit/c36325da9b16ce1e9bf379ed2f4b50c4292ccc03))


**chicago-author-date-classic**

- Clean up duplicate keys ([`7efaca9`](https://github.com/citum/citum-core/commit/7efaca9ea9a36582f1b824a7258b7346ee13d15f))


**chicago-shortened**

- Close patent and personal-comm bibliography gaps ([`5aefdb6`](https://github.com/citum/citum-core/commit/5aefdb625e67065ba7b99a9d16dbf2d0111c79e7))


**ci**

- Skip metadata-only compat commits ([`624282f`](https://github.com/citum/citum-core/commit/624282fd3d55cc40c053acc5428c0a90dec0680a))

- Normalize compat hash-only marker ([`82ef18c`](https://github.com/citum/citum-core/commit/82ef18c154b3261e31e25b336a26c4562ebca7a0))

- Harden core-quality style resolution ([`d0ec8b1`](https://github.com/citum/citum-core/commit/d0ec8b1e692e226a5483b641b006c2de0d7ed5e8))

- Stabilize local and oracle gates ([`3efa6ce`](https://github.com/citum/citum-core/commit/3efa6ce8f34af9a5cdaf9141a9a6e5371939297b))


**cli**

- Add --builtin support to render doc and check commands ([`6a0b527`](https://github.com/citum/citum-core/commit/6a0b527f890910fcbc568576486f277d6277c9a9))

- Wire grouped bibliography rendering to render refs ([`988e221`](https://github.com/citum/citum-core/commit/988e221bace8707a86660d0d31217d67e65ed45f))


**compat**

- Normalize oracle text and lift springer ([`1e85b52`](https://github.com/citum/citum-core/commit/1e85b52fcfd8a46c86ad56edbc1e1bf4a6ef5246))

- Raise springer-vancouver above 90 ([`35ac8e4`](https://github.com/citum/citum-core/commit/35ac8e4b38c5214b89696e21e5faecb0a8fbb1af))


**core**

- Match default overrides only as fallback ([`00841f0`](https://github.com/citum/citum-core/commit/00841f0a404542ca0c1dd044262e622d3997ad16))

- Normalize type-name casing in TypeSelector::matches ([`04f761b`](https://github.com/citum/citum-core/commit/04f761b8479b9f19b9689b7f42263413681b380e))

- Switch InputReference from untagged to class-tagged dispatch ([`19ea821`](https://github.com/citum/citum-core/commit/19ea821d1be727112b971c025d03a07cd6a9cbed))


**delimiter**

- Normalize enum parsing across engine and migrate ([`697d083`](https://github.com/citum/citum-core/commit/697d0838a44d998cd68a9a17eef77b2b6c7bafc3))


**docs**

- Prevent nav overlap ([`c241b87`](https://github.com/citum/citum-core/commit/c241b8749eb487a7a1e3b75ab0bdac91d3ebce28))

- Restore nav on md screens ([`0a6538a`](https://github.com/citum/citum-core/commit/0a6538a1cb2a814f6405f882be2609518c7089b5))

- Align compat branding to citum ([`abeb4c3`](https://github.com/citum/citum-core/commit/abeb4c3774dce49a60c8dd8ad585290cc907b09c))

- Add soon badges to Hub and Labs in nav ([`83612ed`](https://github.com/citum/citum-core/commit/83612edcdf2242de991f7d63ee8bf327fcd073d1))


**engine**

- Stabilize numeric cite ordering ([`aaf48b4`](https://github.com/citum/citum-core/commit/aaf48b49e8ccc8542aab1320e191b4d90aee7c02))

- Align note locator and serial metadata ([`a2715ab`](https://github.com/citum/citum-core/commit/a2715ab36aaf10216edcbc70f6a72dde0cabc819))

- Calculate group_length for author-format disambiguation ([`6e7e810`](https://github.com/citum/citum-core/commit/6e7e8106c95ad6e170995afd57ce07bc12509912))

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


**examples**

- Validate and fix refs YAML files ([`b7c4ef9`](https://github.com/citum/citum-core/commit/b7c4ef963c2684a4aad2a410e2f378d44638f949))


**labels**

- Make et-al name count configurable per preset ([`f7ea4b0`](https://github.com/citum/citum-core/commit/f7ea4b0620124cecdbdfa7cf42af2a1bf7ad74dd))


**labels,names**

- Docs and test coverage for label mode and space separator ([`e7592fb`](https://github.com/citum/citum-core/commit/e7592fbbbc7c97e8c4a335b56f1e31cb9315807a))


**latex**

- Escape bare & in finish() for valid LaTeX output ([`393a5e3`](https://github.com/citum/citum-core/commit/393a5e3d5154607bc6a55d75a715b3bb6ba76139))


**locale**

- Preserve locator term forms from yaml ([`995362a`](https://github.com/citum/citum-core/commit/995362a94671a3e19f0055c0566074829539a2aa))

- Lowercase editor short terms (ed./eds.) ([`22bbede`](https://github.com/citum/citum-core/commit/22bbede4c3a80996bbe5d6b451d9a0aeceff10b9))


**migrate**

- Make inferred mode cache-only ([`bf30ecf`](https://github.com/citum/citum-core/commit/bf30ecfb80ed3fad8cf1d0c54ec1302fe6eee62d))

- Guard inferred citation templates ([`4b36133`](https://github.com/citum/citum-core/commit/4b36133ab6dada009be5066215133b2bd739eafe))

- Preserve help flag in pr rewrite ([`90228f9`](https://github.com/citum/citum-core/commit/90228f90e6c47e2c95f08b7f384f446af70b64f5))

- Recover branch-specific numeric fidelity ([`1d30e5b`](https://github.com/citum/citum-core/commit/1d30e5b32c37944a20f88d98800378293f8f0b90))

- Scope inferred type-template merges ([`7e02ed7`](https://github.com/citum/citum-core/commit/7e02ed7a21b4c97c7f7bb9541d8c01932674e21c))

- Improve numeric sort fidelity ([`590c967`](https://github.com/citum/citum-core/commit/590c967a440c62b910321c9da4e8008bcd697109))

- Improve wave3 citation fidelity ([`fa6a7d0`](https://github.com/citum/citum-core/commit/fa6a7d0c8a3af79e9da46a745784d27f3c269d46))

- Improve migration logic for legal and special types ([`8e66f15`](https://github.com/citum/citum-core/commit/8e66f151621d7b4c0899d91d773e8b2a0e3f1e74))

- Fix subsequent et-al propagation ([`1867c27`](https://github.com/citum/citum-core/commit/1867c27ef3f2f0395db38a6f3095a11ff1e6ab03))


**oracle**

- Use token-similarity matching ([`65a4175`](https://github.com/citum/citum-core/commit/65a417591ce8114e15fdeb672c2fff32e765786c))

- Strip bibliography numbering after whitespace normalization ([`24bf41a`](https://github.com/citum/citum-core/commit/24bf41a89b32e32e68f19fe34b69bf962536b36f))


**render**

- Make HTML bibliography separators markup-aware ([`d3a1da3`](https://github.com/citum/citum-core/commit/d3a1da3fdd44e055c61533ba545896306c2be95b))


**report**

- Include tier-2 wave in compat ([`dfcb2d6`](https://github.com/citum/citum-core/commit/dfcb2d6db1359cec3fec363fd11cfc191dfc6bb9))

- Infer format for !custom and missing processing styles ([`0515f3c`](https://github.com/citum/citum-core/commit/0515f3c78dd6add9e591a1a0ac2ef8d7fbe5df69))


**schema**

- Drop is_base from StyleInfo and csl-legacy Info ([`b64a557`](https://github.com/citum/citum-core/commit/b64a557046205000fb1bc61fa4957e1f67a71644))

- Address missing field_languages field in ref_article_authors macro ([`acdb8ba`](https://github.com/citum/citum-core/commit/acdb8bafeef8b8a80efcd84ad83ac30c26c2bce9))

- Restore schema publishing outputs ([`4b3bf27`](https://github.com/citum/citum-core/commit/4b3bf274e63945e45da6ae4a8030e0459852fa2b))


**styles**

- Fix citation templates across 27 styles ([`4012cd9`](https://github.com/citum/citum-core/commit/4012cd993958e75ec6f2b2c821fbfc57bd7b3e69))

- Modernize MLA and refine citations ([`6a00e1c`](https://github.com/citum/citum-core/commit/6a00e1c33cdfe5f85cfbaea8a12a715b5ca0b209))

- Correct gost processing variants ([`7441184`](https://github.com/citum/citum-core/commit/74411840ce1350af2e2343d54a569400c86da8e1))

- Bring all styles to 51%+ fidelity and 80%+ SQI ([`f922e84`](https://github.com/citum/citum-core/commit/f922e847c4bfea20f5922c460f775108fde57ce2))

- Fix 3 springer-physics bib failures ([`07652fd`](https://github.com/citum/citum-core/commit/07652fd72a37dc31c0770cf0504379418cd43cd2))

- Close top-10 bibliography deltas ([`05875e6`](https://github.com/citum/citum-core/commit/05875e6a4dd0f0ba0c05ac62fed09fa4d5e1d4ba))

- Rename CSLN to Citum in apa-7th title ([`c1ce553`](https://github.com/citum/citum-core/commit/c1ce553782f26d872fe3462a69e410cecdd62df1))

- Remove parenthetical from apa-7th title ([`47e50df`](https://github.com/citum/citum-core/commit/47e50df12be1aa7dac52449a0a83e2ce911b722f))


**tooling**

- Align schema validator with repo inputs ([`26c9f63`](https://github.com/citum/citum-core/commit/26c9f6396debc1a2e60e05291337f91c9cfa180a))



### Documentation

**architecture**

- Add migrate wave handoff docs ([`8ca9696`](https://github.com/citum/citum-core/commit/8ca9696337f90aa38004e4b2f6a5f778c3213899))

- Consolidate wave runbook ([`b181578`](https://github.com/citum/citum-core/commit/b181578d5fcf6458cf9eb8a4cb6343102a85682b))

- Define style target roadmap ([`adc4dc2`](https://github.com/citum/citum-core/commit/adc4dc215fad8371c7a818a61edcaad25729bf9e))

- Refresh sqi plan status ([`506f5cf`](https://github.com/citum/citum-core/commit/506f5cf3eb28fbbb6364771ddcf951f39078be3e))

- Update Section 3 to reflect deny_unknown_fields trade-off ([`0e21b08`](https://github.com/citum/citum-core/commit/0e21b08f2426ca8f9ef93da6f7a7ef610799280c))

- Add architectural soundness assessment and gap beans ([`3d0a3b9`](https://github.com/citum/citum-core/commit/3d0a3b91c7da4602c8e5fbf3dee3435e8305f540))

- Close csl26-o18j with verification ([`1bd1c65`](https://github.com/citum/citum-core/commit/1bd1c6594a41ced02965fa06ec67b897488ef56a))

- Use bun for csl intake examples ([`43de28c`](https://github.com/citum/citum-core/commit/43de28ce32395b311657ec0cd990c60bb199ab69))


**examples**

- Add multilingual and bib-grouping runnable examples ([`a916eda`](https://github.com/citum/citum-core/commit/a916eda05e73109df21747a8a62a9c908f392316))


**guide**

- Add IDE editor integration callout ([`dac7985`](https://github.com/citum/citum-core/commit/dac7985110994ae159565fc6df468070b6310a53))

- Fix enum values from schema review ([`bdded11`](https://github.com/citum/citum-core/commit/bdded113f8b31013f94e8571f654cb31ec98e274))

- Expand preset catalog with all 14 contributor presets ([`f9cf508`](https://github.com/citum/citum-core/commit/f9cf508edc7f464a8ce10b28ae5ac51e8c0d1116))


**migrate**

- Add csln-migrate guide ([`f38d38f`](https://github.com/citum/citum-core/commit/f38d38fd620fa3c57e5be4c89a9f07882e220b68))


**multilingual**

- Clarify field-scoped language metadata ([`eecbb25`](https://github.com/citum/citum-core/commit/eecbb25f2269fd05ad05bb2d95dfc6977be28ef9))


**schema,engine,migrate**

- Add public API doc comments ([`7162bf7`](https://github.com/citum/citum-core/commit/7162bf712ef22adad2e9a7e4ccdbd250861d1588))


**server**

- Clarify dependency graph in plan ([`3e89502`](https://github.com/citum/citum-core/commit/3e89502acedfde148a5dc69e5a1db1dc5bfe7800))

- Add HTTP curl example to README ([`aeae9e8`](https://github.com/citum/citum-core/commit/aeae9e86a7670e574efd9ddced1c38f7ae6a6515))

- Fix stdio example with valid JSON ([`ed229e2`](https://github.com/citum/citum-core/commit/ed229e2812289d7d3a54dd32009c22109c95a350))


**tests**

- Record deferred wave 1 follow-up ([`1a4a1de`](https://github.com/citum/citum-core/commit/1a4a1de47533aac33f71792bad3c8d162d569256))



### Features

**beans**

- Add citum wrapper and smart next ([`fcd9670`](https://github.com/citum/citum-core/commit/fcd9670f8e6ef127a44e50754279b64a6664e6ba))


**bindings/latex**

- Add LuaLaTeX integration ([`5fa112c`](https://github.com/citum/citum-core/commit/5fa112c8f5994aea0c31ac0ee164b14d22cc884d))


**ci**

- Add oracle regression baseline gate ([`006c285`](https://github.com/citum/citum-core/commit/006c2853693cd7f04acb9e81aba359823b223f4e))


**cli**

- Comprehensive UX overhaul ([`7882e85`](https://github.com/citum/citum-core/commit/7882e85381b28daf1607aae10d502bb2693d4ec8))

- Default output format to html ([`1c7763d`](https://github.com/citum/citum-core/commit/1c7763d615b2bb72471fc002c09e174dd2eca6d2))


**core**

- Migrate and refine next five styles ([`c3a9ad4`](https://github.com/citum/citum-core/commit/c3a9ad461d7f613d19a3fdf92757e509e7868c81))

- Migrate workspace to Rust 2024 edition ([`2bb274c`](https://github.com/citum/citum-core/commit/2bb274c5978b226cda9c5187299ae11c9ea677ea))

- Add FamilyOnly and DayMonthAbbrYear to schema ([`633db5e`](https://github.com/citum/citum-core/commit/633db5e6f3c35419ef8aa6c80e82f3e99603845a))


**document**

- Auto-note djot citations ([`4467bee`](https://github.com/citum/citum-core/commit/4467beefc8cf82453a80903171bf6d80ec5177bf))

- Configure note marker punctuation ([`91317b8`](https://github.com/citum/citum-core/commit/91317b825357738b58509c24ec48dd60b43b4640))


**edtf**

- Implement time component rendering ([`7143adf`](https://github.com/citum/citum-core/commit/7143adf0877cbd31c1ffac6ce6785b37a820090a))


**embedded**

- Add 12 priority styles with builtin CLI support ([`cface2d`](https://github.com/citum/citum-core/commit/cface2dcb6087bc144b9ec54b1ff4f192752f01c))


**engine**

- Normalize note citations and locator rules ([`3378a8f`](https://github.com/citum/citum-core/commit/3378a8fb3e92b5eed6d9357b16cd9b8cca299865))

- Add locator label controls ([`9f79f43`](https://github.com/citum/citum-core/commit/9f79f4398d2b626e5b03b3fe677748f424f40938))

- Sort numeric label assignment ([`52fa7e9`](https://github.com/citum/citum-core/commit/52fa7e926f6d139672163430bae947a1ae6927a8))

- Support FamilyOnly and DayMonthAbbrYear rendering ([`12b83ac`](https://github.com/citum/citum-core/commit/12b83ac6bdd40576d759c176d16067e2ffaf1130))

- Add position-based citation variants (ibid, subsequent) ([`22dff4c`](https://github.com/citum/citum-core/commit/22dff4c20c6452a9b516eac43cefda243b141542))

- Add document-level bibliography grouping via djot and YAML ([`86dca5f`](https://github.com/citum/citum-core/commit/86dca5f89f98b859f12568bf1969c248b30b9ead))

- Support container short titles ([`29794d0`](https://github.com/citum/citum-core/commit/29794d02c10cf3ef341127a219a9f947b846facd))


**grouping**

- Add style targets and document tests ([`8ee163b`](https://github.com/citum/citum-core/commit/8ee163bf5fafc5ab8723d72210bf11e0d1d2c5b8))


**label**

- Implement label citation mode ([`7aa285a`](https://github.com/citum/citum-core/commit/7aa285a1e0dfd04e951973b27a61da60fee9715d))


**migrate**

- Automate output-driven templates ([`3c0f54b`](https://github.com/citum/citum-core/commit/3c0f54bb0867a69618e10ec0aafb01d0ab8a6cba))

- Score inferred template confidence ([`ceb8de8`](https://github.com/citum/citum-core/commit/ceb8de82697361ea3ba3d0960bb3e17cb8d82c4d))

- Carry scope-aware name/disambig config ([`9ec3bdd`](https://github.com/citum/citum-core/commit/9ec3bdd49d71d54714b0ac2c93b73fbf2b6cb3be))

- Finish phase3/4 and wave4 ([`d6017c2`](https://github.com/citum/citum-core/commit/d6017c21176622356463de580d52cebc9f37ae86))

- Complete variable-once cross-list deduplication ([`f2229dc`](https://github.com/citum/citum-core/commit/f2229dc7b0d7d95b2774bfe07a0757be391b3174))


**migration**

- Add component coverage gate ([`fcbcd48`](https://github.com/citum/citum-core/commit/fcbcd48230402e4fe1c6b572b30a03bffef5ad81))


**mla**

- Add disambiguate_only title field and MLA style ([`3b97714`](https://github.com/citum/citum-core/commit/3b9771408958c692cdfb9a2f8460d09b81703cff))


**multilingual**

- Support language-aware title templates ([`71fe320`](https://github.com/citum/citum-core/commit/71fe32074171e998c6b00ce49bf881e57899d853))

- Add preferred-transliteration ([`b4a8a81`](https://github.com/citum/citum-core/commit/b4a8a81165f48baa90cdd8368a767fdd2e50e644))


**note-styles**

- Ibid/subsequent to chicago-notes ([`a753d2a`](https://github.com/citum/citum-core/commit/a753d2a2d02aa00c3918ad73f72e4a07d17407a4))


**presets**

- Expand sqi preset extraction ([`8c02d19`](https://github.com/citum/citum-core/commit/8c02d19bbe6f7b060e9248d9d9d04908409a9c7b))

- Add 6 new ContributorPreset variants ([`9a187b8`](https://github.com/citum/citum-core/commit/9a187b8f0646ec1cd52b9ce0682d400399bd824d))

- Add numeric contributor variants ([`b2007d1`](https://github.com/citum/citum-core/commit/b2007d1ecebf9f66d365d8f07d2a884e862e78a1))


**report**

- Add note citation type coverage ([`06263bd`](https://github.com/citum/citum-core/commit/06263bd75f1a1962e443a6a3335d6a0d9df40be5))

- Add SQI and sortable compat table ([`95938ac`](https://github.com/citum/citum-core/commit/95938ac0bfb87e463ee9e8396a6bc3e876427265))

- Replace status column with sqi tier ([`eea9a62`](https://github.com/citum/citum-core/commit/eea9a6294d125495ccbaefe1cbb33a21433af060))

- Auto-discover styles for compat ([`8ac465f`](https://github.com/citum/citum-core/commit/8ac465fcece68d988ed9fa8c1029d6edd390e492))

- Add style search to compat table ([`7ca8b10`](https://github.com/citum/citum-core/commit/7ca8b107922274fc34d04562b4094bace6b88a37))


**schema**

- Add SortPreset; use in chicago ([`dd14350`](https://github.com/citum/citum-core/commit/dd143502b78999e61547785a42c139b4c89f190a))

- Add CitationField, StyleSource provenance to StyleInfo ([`e4f0105`](https://github.com/citum/citum-core/commit/e4f01053160c7a64cef813d28b264b33c257ae11))


**schema,engine**

- Add subsequent et-al controls ([`d35f7ca`](https://github.com/citum/citum-core/commit/d35f7ca717195cf5fb54e26e60c23c7a30069957))


**scripts**

- Add csl intake progress summary ([`5745441`](https://github.com/citum/citum-core/commit/5745441493e3e1d69412f14781b7f84b9e42c44f))


**server**

- Add citum-server crate ([`557f780`](https://github.com/citum/citum-core/commit/557f780e6305bc5d82c01b6d19725bd447c77884))

- Support output formats ([`f876c1a`](https://github.com/citum/citum-core/commit/f876c1a1e70e31969807b6a3d94f745a35cd14b0))


**styles**

- Complete core fidelity and probes ([`3873fc4`](https://github.com/citum/citum-core/commit/3873fc47a4a3f9cef4225356561009fc3908a85c))

- Add 58 priority styles with presets ([`f1bb4a3`](https://github.com/citum/citum-core/commit/f1bb4a337d2555c822dcefe6729a8e4b0b92b894))

- Migrate wave1 and wave2 batches ([`87ad046`](https://github.com/citum/citum-core/commit/87ad04653247e837f6a883a11833b791e20bfceb))

- Add ams-label and alpha label styles ([`d04de63`](https://github.com/citum/citum-core/commit/d04de6344d7e1a85fcfc07443a3192e3daaeab15))

- Add basic multilingual config to apa and mla ([`c094276`](https://github.com/citum/citum-core/commit/c0942766fe4387cdb7149b33cd099b89b373eedb))

- Localize sectional group headings ([`d604bc5`](https://github.com/citum/citum-core/commit/d604bc57c479be41d1f1e67e7c94c8fc5b58fb22))

- Add gost-r-7-0-5-2008 grouping styles ([`5347a7c`](https://github.com/citum/citum-core/commit/5347a7c88451e296daa26431641ce679005a2d94))

- Multilingual YAML styles csl26-mls1 ([`677ce78`](https://github.com/citum/citum-core/commit/677ce78faf058f57b536c9782477a3ce8946ac6e))


**tests**

- Add interactive fixture generator ([`c37ab43`](https://github.com/citum/citum-core/commit/c37ab430d1e3ab419423a45820cca53584c91c2f))

- Add csl intake audit ([`902928a`](https://github.com/citum/citum-core/commit/902928ab19a466a9bd2c0b5ec7755ef309e028ac))

- Extract CJK/Arabic CSL fixtures + native test ([`432d2e4`](https://github.com/citum/citum-core/commit/432d2e41726d0b10373d738e1b78928f0cb8146b))


**tooling**

- Add --dry-run flag to release.sh ([`e9013cc`](https://github.com/citum/citum-core/commit/e9013cceabcd0b94a92c5778d7b3f24cb6d3c848))


**typst**

- Add native rendering and pdf output ([`c4dbe6f`](https://github.com/citum/citum-core/commit/c4dbe6f96ba5f369513b964618a8fa2fe4d0cf4d))


**wave3**

- Seed baseline styles and metrics ([`1181cc8`](https://github.com/citum/citum-core/commit/1181cc857748f3f0f4a7f627fea31d1eb7d49483))



### Refactor

**beans**

- Clean up /beans next human output ([`eefd082`](https://github.com/citum/citum-core/commit/eefd0829676b7ee1e3c39496918ea6eca4b411bb))


**citation**

- Move suppress-author to citation level ([`3d0893e`](https://github.com/citum/citum-core/commit/3d0893ef04edc5e9e40f17accdbcd9d1b35d3f08))


**cli**

- Make citum the only public binary name ([`1489183`](https://github.com/citum/citum-core/commit/148918328829eaf6cea835d21bff140ce036e134))


**core**

- Decouple csln_core from csl_legacy and biblatex ([`26426e1`](https://github.com/citum/citum-core/commit/26426e1a64b2c437e939ed92ee57c02c03dc183d))


**edtf**

- Rename crate csln-edtf → citum-edtf ([`51cfc24`](https://github.com/citum/citum-core/commit/51cfc24ba677424104a8e9a2a77ad60002a8bc03))


**engine**

- Remove unused clap dependency ([`35bae75`](https://github.com/citum/citum-core/commit/35bae7523d0d4f20144463fc2378978f3b206db3))


**migrate**

- Harden inferred template merge ([`fedba76`](https://github.com/citum/citum-core/commit/fedba76587fb073d59b263c44af5effdab2c4e12))

- Trim redundant bibliography sorts ([`b5d0be4`](https://github.com/citum/citum-core/commit/b5d0be427bd5aaf8ad14a93c5df15ea343a578bd))

- Modularize template_compiler ([`8e484ef`](https://github.com/citum/citum-core/commit/8e484ef8d38f7ff5bf71555045c4f2ca4f5b8d69))


**sqi**

- Align core scoring across styles ([`6765a75`](https://github.com/citum/citum-core/commit/6765a7502a841e6c4dffac4b9d8c85d9e4a42e24))


**tests**

- Add reference-builder macros and migrate test boilerplate ([`4b5b737`](https://github.com/citum/citum-core/commit/4b5b73776e19dc2b8918d11888ca1993ee185cf8))


**workflow**

- Unify style-evolve workflow ([`3403e13`](https://github.com/citum/citum-core/commit/3403e133281ec0062dbc7fef91b6f9bf9a9f8ade))


**workspace**

- Migrate csln namespace to citum ([`1e22764`](https://github.com/citum/citum-core/commit/1e227643afb5a01a3aa7ccb4d70dbd086b478b69))



### Styling

**american-medical-association**

- Expand bib types ([`68994ed`](https://github.com/citum/citum-core/commit/68994ed5b0b363c4d7e4dd020cfb77b463a63b5c))


**batch**

- Migrate and enhance next 10 ([`b115b85`](https://github.com/citum/citum-core/commit/b115b851715d35ce272d01b39b45c737b981a7db))

- Raise wave-100 fidelity floor ([`e42eddf`](https://github.com/citum/citum-core/commit/e42eddf0b6fdb9212993ed93af1dbf4e643b1713))


**chicago-notes**

- Reach oracle citation parity ([`8382695`](https://github.com/citum/citum-core/commit/83826956bba6967f76a1aeb6fa786dd781622425))

- Cover note reference types ([`bd67504`](https://github.com/citum/citum-core/commit/bd675042693b8de7becbd5a22665362fd662315d))


**core**

- Improve citation fidelity across core set ([`92897ee`](https://github.com/citum/citum-core/commit/92897ee3a2d33c97d544a30ba0b4b373562da934))

- Remove annals from repository ([`e408f6f`](https://github.com/citum/citum-core/commit/e408f6f68eaf978f501c5f5af5a46a4ad6599f1e))

- Use locator label controls ([`c69e15b`](https://github.com/citum/citum-core/commit/c69e15b2d6e51723a413a512f8fc32091ad1db31))

- Lift SQI via numeric citation presetization ([`a73f4ed`](https://github.com/citum/citum-core/commit/a73f4edb9da11537d5de0433f5a220aa17a743dc))


**migrate**

- Update generated styles and work logs ([`d260665`](https://github.com/citum/citum-core/commit/d2606655dd72efc3d3cc2d96fd77cbb9076f1253))


**mla**

- Apply new template components to MLA ([`e618ff2`](https://github.com/citum/citum-core/commit/e618ff233554f2d558d0e6434119f12826d756ac))


**priority**

- Complete next-10 wave ([`d717f8d`](https://github.com/citum/citum-core/commit/d717f8d61fb2b3b01cb9bda6df102fc62d63fedc))

- Migrate and enhance next 20 ([`af46a97`](https://github.com/citum/citum-core/commit/af46a97e0ba824dcca2027a71cd42d99c6172445))


**tfca**

- Raise fidelity above 25 percent ([`715ddba`](https://github.com/citum/citum-core/commit/715ddbae2c66d396d43daade94287df7c61cade3))


**top10**

- Add springer-socpsych-author-date ([`a571170`](https://github.com/citum/citum-core/commit/a571170c44a899592b2e444c6facf151df6edbac))


**vancouver**

- Raise elsevier-vancouver match ([`9fb95bd`](https://github.com/citum/citum-core/commit/9fb95bd64dd1421d0421265faa23ea1113e76d3b))



### Testing

**citations**

- Cover empty-date citation sort ([`964ea0c`](https://github.com/citum/citum-core/commit/964ea0cab75c31ccc30b99ba96d896083355869b))


**engine**

- Add sort oracle tests ([`3bff72d`](https://github.com/citum/citum-core/commit/3bff72d22e5fb103681c8babc352d6303662c526))


**fixtures**

- Add legal hierarchy grouping fixture ([`ff8cce0`](https://github.com/citum/citum-core/commit/ff8cce01d51f33f4612ac4cf47e382b03067e065))


**grouping**

- Cover jm legal heading order ([`617c4ca`](https://github.com/citum/citum-core/commit/617c4ca1f5b9f8a2233df41310cdaa9192ea2206))

- Cover localized heading fallback ([`364de50`](https://github.com/citum/citum-core/commit/364de5031f7f83a4b4a79768445ad8a3006d55a4))


**metadata**

- Add test coverage for new MLA forms ([`7fb9425`](https://github.com/citum/citum-core/commit/7fb9425de85e50b427660a7f384591d67b135637))


**server**

- Add RPC dispatcher integration tests ([`3679b36`](https://github.com/citum/citum-core/commit/3679b36c54347ce70b0e0c08dcd6b496b7b42101))

- Cover http mode ([`f61026f`](https://github.com/citum/citum-core/commit/f61026fe6846e7c3271b6eeba2c9970afe67e15d))


## [0.6.0] - 2026-02-19

### Bug Fixes

**apa**

- Handle legal/personal citation edges ([`91ed20f`](https://github.com/citum/citum-core/commit/91ed20f554421dcc6f5ee356bd71203a7389200d))


**ci**

- Remove npm install step — node_modules is committed to repo ([`c9dadf2`](https://github.com/citum/citum-core/commit/c9dadf21854a48941aeea47a5b34b1a7c666df17))

- Add workflow_dispatch to compat-report ([`aa3d2ba`](https://github.com/citum/citum-core/commit/aa3d2bac7299211022f49be0fe4ad91e2d4096d8))

- Trigger Pages deploy after compat report commit ([`056d80b`](https://github.com/citum/citum-core/commit/056d80b73f9340b5ca259dd790fb83c3aa3a77bc))

- Track package.json and restore npm install step ([`6900a78`](https://github.com/citum/citum-core/commit/6900a78620d70cc8b31ba2e6f57022569d87e38a))

- Add concurrency group to prevent parallel runs ([`6f94e83`](https://github.com/citum/citum-core/commit/6f94e83c21b71eb5261c237c8dcc682a0c104530))


**engine**

- Align process alias with render refs flags ([`d64b926`](https://github.com/citum/citum-core/commit/d64b926fa170f0277714b57ef5f0ebca1abc010b))


**examples**

- Improve example bibliography and citation files ([`aaf6faf`](https://github.com/citum/citum-core/commit/aaf6faf44e3da5540d89c22f9a501ea3bdba9290))


**lua**

- Portable FFI loading and lifecycle ([`66ab921`](https://github.com/citum/citum-core/commit/66ab921a6ada42ccb2848236acaa91c7b3eebccd))


**proc**

- Improve grouped cites and quotes ([`9951f00`](https://github.com/citum/citum-core/commit/9951f00a85b016e36448d20ea79529e077494934))


**schema**

- Allow string presets for contributors, dates, and titles ([`3a50b63`](https://github.com/citum/citum-core/commit/3a50b638ed5f8d94febb2a8f60d4b1ea64943789))


**scripts**

- Harden compatibility reporting and fix ama style ([`e492289`](https://github.com/citum/citum-core/commit/e492289348d7adc087c49aeea40f2140ab70e4c5))


**springer**

- Raise full-oracle bibliography fidelity above 90% ([`1627705`](https://github.com/citum/citum-core/commit/162770577d482da8e80a1009d05865768d23a79e))



### Documentation

**bean**

- Refine boltffi plan with phased binding strategy ([`a9b1fae`](https://github.com/citum/citum-core/commit/a9b1faec44e026d16db2b7110b07779e692e24ca))

- Update apa-7th fidelity progress ([`2b71009`](https://github.com/citum/citum-core/commit/2b7100972978093dfd1b400d7ade16f99a41c5e2))


**bindings**

- Remove unneeded ref ([`10961c1`](https://github.com/citum/citum-core/commit/10961c164db8f6548b243b7eb8c6fb5c3e51db5b))


**engine**

- Fix table formatting ([`a9967af`](https://github.com/citum/citum-core/commit/a9967af083d2f6ae3f4e40038449e851b8883f07))



### Features

**apa-7th**

- Push bibliography fidelity above 20 matches ([`b75a69a`](https://github.com/citum/citum-core/commit/b75a69ac87937d9c8cf60353c9af70db07f6dd3a))


**bean**

- Add top-10 style reporting task ([`9c416d7`](https://github.com/citum/citum-core/commit/9c416d73a54cb0e54b01917247c2059d947f4caa))


**beans**

- Document-level bibliography grouping ([`bacf69c`](https://github.com/citum/citum-core/commit/bacf69c47f1e120f41d2aa4166f8687d2d8fd095))


**bindings**

- Add LuaJIT FFI binding for LuaLaTeX ([`feea618`](https://github.com/citum/citum-core/commit/feea6186a357a702fe2bbae830357c1ea898f282))


**cli**

- Improve error handling for input files ([`30db2b1`](https://github.com/citum/citum-core/commit/30db2b1385fef33c2863baed78bd7d27b232a894))


**core**

- Introduce ComponentOverride enum ([`7dbae9b`](https://github.com/citum/citum-core/commit/7dbae9be5a5dbc969060d96a48b42a309120acdc))

- Achieve strict template validation ([`f862b66`](https://github.com/citum/citum-core/commit/f862b6693a33c6554ceb5e4cec9cc83277469994))

- Add TypeSelector and semantic number fields ([`3b4e100`](https://github.com/citum/citum-core/commit/3b4e10017dfd077afcc8f54fd7de725482ee1d42))


**engine**

- Add native LaTeX renderer ([`a042f25`](https://github.com/citum/citum-core/commit/a042f25be4317ad7791cab57316f7196957f6ded))

- Add universal C-FFI bridge ([`d248107`](https://github.com/citum/citum-core/commit/d248107ebc7300cc7be05d209abc1272f70ad64f))

- Fix djot citation parsing and rendering ([`8bd2193`](https://github.com/citum/citum-core/commit/8bd2193ce7d6fd94efd6f30251b24e498af29bb7))

- Simplify citation model and djot support ([`0443a01`](https://github.com/citum/citum-core/commit/0443a012a3302e677da56f0f78496e003a388a35))

- Implement citation sorting and improved grouping ([`28abe12`](https://github.com/citum/citum-core/commit/28abe12797be47ae2a2f21ed9e3a337f9a508e36))


**render**

- Full latex support in csln process ([`a44fac1`](https://github.com/citum/citum-core/commit/a44fac1021258264e4408a471c4763ceb1d94b7f))


**report**

- Add top-10 style compatibility report ([`c262031`](https://github.com/citum/citum-core/commit/c26203177710e2669944a46ad76ffb771916b598))

- Richer compatibility metrics and detail view ([`9e9fe20`](https://github.com/citum/citum-core/commit/9e9fe20fa87defcb949ba7bc5f45d165f4c8802e))


**styles**

- Modernize apa-7th conjunctions ([`912b576`](https://github.com/citum/citum-core/commit/912b576729bafb1b514831624d755bc8dea84368))

- Improve apa-7th fidelity and add documentation ([`32364c2`](https://github.com/citum/citum-core/commit/32364c2b584de06d5e495fb7ea8744e2243cf716))



### Refactor

**engine**

- Format-aware value extraction pipeline ([`5f8b499`](https://github.com/citum/citum-core/commit/5f8b499cbfcbe6ef7e3b43d7ed8909eafd2dabdc))

- Unify cli ux around render/check ([`328ed3b`](https://github.com/citum/citum-core/commit/328ed3b0ac755900caa41bc8c4a1a41a8378da9a))

- Remove process command alias ([`9d8e7a9`](https://github.com/citum/citum-core/commit/9d8e7a9053cd730a12922b36b2352a56317cf7b3))


**i18n**

- Localize bib group headings ([`d5c3919`](https://github.com/citum/citum-core/commit/d5c391964fe92307a9216235e6ab109552a8b3c6))



### Styling

**apa-7th**

- Reach full oracle fidelity ([`fe443f3`](https://github.com/citum/citum-core/commit/fe443f33d123cb54050f83d22d1be09e0f860601))


**elsevier-harvard**

- Raise fidelity >90% ([`4fb5a82`](https://github.com/citum/citum-core/commit/4fb5a82cc590255facac989655a5eac9d2265cbe))



### Testing

**engine**

- Wire domain fixtures into CI runs ([`05fdca4`](https://github.com/citum/citum-core/commit/05fdca489600bcb0689fcc917921a569f4b85f0d))


**i18n**

- Align multilingual tests ([`b17e042`](https://github.com/citum/citum-core/commit/b17e04207c72c0322bcc2b42eaefff28d943f4bc))


## [0.5.0] - 2026-02-16

### Bug Fixes

**ci**

- Remove nextest to avoid yanked deps ([`4b2a345`](https://github.com/citum/citum-core/commit/4b2a345d9aa0250135b1c3e3181b9f866e9b8337))


**engine**

- Resolve clippy warnings in document tests ([`156202c`](https://github.com/citum/citum-core/commit/156202c01b788e3069c326909b87961b67162387))


**nextest**

- Correct config field types ([`81b1cbb`](https://github.com/citum/citum-core/commit/81b1cbb2b748a08057bd51dd3deead2a679efd84))


**release**

- Remove unsupported update_all_packages field ([`f994a67`](https://github.com/citum/citum-core/commit/f994a677464983cecace658be82ba4236937193f))

- Consolidate to root changelog and align versioning ([`0935e6e`](https://github.com/citum/citum-core/commit/0935e6ec38a6fc651662978d3ed719de6cda40bf))



### Documentation

**grouping**

- Add primary/secondary sources ([`f350a07`](https://github.com/citum/citum-core/commit/f350a0787941ddadef6d5016fd9d8ff2040ee218))



### Features

**beans**

- Add typst output format ([`eb44dd2`](https://github.com/citum/citum-core/commit/eb44dd26c2cfea27ec598a39452aaf712f47c618))

- Add interactive html css/js ([`b643c31`](https://github.com/citum/citum-core/commit/b643c31ad30feef59bbd8acaab2431d7b335ee83))

- Add deno evaluation task ([`4ffe228`](https://github.com/citum/citum-core/commit/4ffe2284f96829b0051ee0592eb85cb36b1869c0))


**core**

- Support legal reference conversion ([`1725f49`](https://github.com/citum/citum-core/commit/1725f49045c4964acb0898f49585b27177323b48))


**djot**

- Implement citation visibility modifiers and grouping  ([`ca1da33`](https://github.com/citum/citum-core/commit/ca1da33df2c61afcdb63fb1fa66878a1d2ff44d4))


**dx**

- Optimize binary size and automate schema publishing ([`224a5f6`](https://github.com/citum/citum-core/commit/224a5f6cf8b4c5193cc8fea60384b38c471fcac4))

- Export and publish all top-level schemas ([`55b8957`](https://github.com/citum/citum-core/commit/55b895793adeae1bdcfc45476afdab7c12e55b0e))


**grouping**

- Implement group disambiguation ([`caf7f50`](https://github.com/citum/citum-core/commit/caf7f504082d79c56d263ca2cb8cde734ddeda29))


**styles**

- Support integral citations in Springer Vancouver ([`43a6af2`](https://github.com/citum/citum-core/commit/43a6af2290418b209fa7a3f7f6b9ed440e5cae9b))


**test**

- Add CSL test suite for disambiguation ([`6f42d0c`](https://github.com/citum/citum-core/commit/6f42d0c64db471b907902f8a43f0acc0e6e15b2d))


**web**

- Implement interactive HTML renderer ([`943a0fb`](https://github.com/citum/citum-core/commit/943a0fb2a9edab4f23bfbf7586c1b5f9c3ffab60))



### Refactor

**test**

- Use pure CSLN types ([`b78febd`](https://github.com/citum/citum-core/commit/b78febdd8dec29e8c5c7b297b4c228d2e8e7aa6e))



### Testing

**engine**

- Expand native test suite and refactor existing tests ([`3a0eb43`](https://github.com/citum/citum-core/commit/3a0eb4397c1c5f79d58915b417e4701da7219d49))

- Reorganize integration tests into functional targets ([`dfe52be`](https://github.com/citum/citum-core/commit/dfe52bef72521ebb7b39972e330776c83c2a0d5c))


## [0.3.0] - 2026-02-15

### Bug Fixes

**bibliography**

- Preserve component suffixes in separator deduplication ([`b943d22`](https://github.com/citum/citum-core/commit/b943d2204ad218aae2e0278ec6f593c130f7ecb4))


**core**

- Enable initialize-with override on contributor components ([`22c3e5f`](https://github.com/citum/citum-core/commit/22c3e5f0955db8aa1c30c0c48419ec6e6439e929))

- Alias DOI/URL/ISBN/ISSN for CSL-JSON ([`e9d207e`](https://github.com/citum/citum-core/commit/e9d207ec82390bbda2cd453d67e2f8d500903e27))


**csln_migrate**

- Improve substitute extraction for real styles ([`b767d88`](https://github.com/citum/citum-core/commit/b767d886d656da0874fe8ff6089960a30af62548))


**engine**

- Use container_title for chapter book titles ([`56a70ec`](https://github.com/citum/citum-core/commit/56a70ecb51dca1921dec31c7929d80b37b2abba8))

- Correctly map ParentSerial/ParentMonograph to container_title ([`43b0785`](https://github.com/citum/citum-core/commit/43b07851a5aa983703c39f1403c0785a24099b29))

- Implement contributor verb and label forms ([`4bdadb3`](https://github.com/citum/citum-core/commit/4bdadb34e51bb75a98b4a98990992af077c6ac6a))

- Add context-aware delimiter for two-author bibliographies ([`65a2e15`](https://github.com/citum/citum-core/commit/65a2e1595a1aed52892e8cf604e867e5ceaa6df3))

- Implement variable-once rule for substituted titles ([`75efee2`](https://github.com/citum/citum-core/commit/75efee2d13f431f1a467ce2a825f99342cb659de))

- Improve bibliography sorting with proper key chaining ([`5620190`](https://github.com/citum/citum-core/commit/56201900255388d2568445f2ecf1b2cf48138a8d))

- Add contributor labels and sorting fixes ([`31a96aa`](https://github.com/citum/citum-core/commit/31a96aa20c6d49e6eadb612ba605e40d3165c141))

- Improve delimiter detection and URL suffix handling ([`872906f`](https://github.com/citum/citum-core/commit/872906f45292808fc525aeb6ead76fbdf07faf8a))

- Resolve mode-dependent conjunctions and implement deep config merging ([`6acd4b8`](https://github.com/citum/citum-core/commit/6acd4b890ae1947ce122b20d4a7f90c3c0d1bad5))

- Allow variable repetition with different context ([`34670a4`](https://github.com/citum/citum-core/commit/34670a46e7a43243e1f2934b530cebda734489cd))

- Author substitution and grouping bugs ([`ef4e075`](https://github.com/citum/citum-core/commit/ef4e0755c2f296ec0c47aed1a48e6a5cf51ebab2))

- Use correct jotdown API ([`b195afc`](https://github.com/citum/citum-core/commit/b195afcf39971dd5e4ad233d975d873013addb48))

- Prevent HTML escaping in docs ([`c57ee77`](https://github.com/citum/citum-core/commit/c57ee7716a5ab5b7198d7b8f41f5b7b4a9f4e44f))


**gitignore**

- Improve baselines directory exclusion pattern ([`f236856`](https://github.com/citum/citum-core/commit/f236856ca146c5c7eb268a874fa203bbe79f373c))


**locale**

- Handle nested Forms in role term extraction ([`a34bafc`](https://github.com/citum/citum-core/commit/a34bafcd2a15b935279f4b727beb7b921ed3eabd))


**migrate**

- Improve template compilation ([`4f1a57c`](https://github.com/citum/citum-core/commit/4f1a57c29e5efd681041a461741f63eccceef295))

- Disable auto chapter type_template generation ([`faf0193`](https://github.com/citum/citum-core/commit/faf019310aadb0a36343175b0cc4bf52ba279ede))

- Improve citation delimiter extraction ([`e729495`](https://github.com/citum/citum-core/commit/e729495c7891c4cffb2b7f62bdc8e27982186e7c))

- Extract date wrapping from original CSL style ([`20de07f`](https://github.com/citum/citum-core/commit/20de07fd21a3e6f11a8c255cd1d1d8c15357cb1a))

- Add editor/container-title for chapters, suppress journal publisher ([`e2b7e79`](https://github.com/citum/citum-core/commit/e2b7e790f771a67814e7f5c007a67579dcaf5a8a))

- Add page formatting overrides for journals and chapters ([`e5e7cda`](https://github.com/citum/citum-core/commit/e5e7cda647336e2cd663e220718ce755d557126b))

- Resolve template nesting regression with recursive deduplication ([`5e322b2`](https://github.com/citum/citum-core/commit/5e322b29501ae9cb26b68dc7cb1d16b23bdc2e0b))

- Context-aware contributor option extraction ([`811efc7`](https://github.com/citum/citum-core/commit/811efc7cbe4459931462a878d3068b1b664e6199))

- Recursive type overrides for nested components ([`0835b7f`](https://github.com/citum/citum-core/commit/0835b7f6c1e4ea18f25ac9807f30a60f79d2424f))

- Improve CSL extraction and template generation ([`e1046f1`](https://github.com/citum/citum-core/commit/e1046f1adee1cc46c0de324d4541d31d33bae19b))

- Suppress pages for chicago chapters ([`4d4fe3d`](https://github.com/citum/citum-core/commit/4d4fe3dff85c46e4257d01c59219fdf5af252a6b))

- Chicago publisher-place visibility rules ([`ca99571`](https://github.com/citum/citum-core/commit/ca99571baca0ae58d491824ddf030285d33882fa))

- Remove comma before volume for chicago journals ([`767473e`](https://github.com/citum/citum-core/commit/767473eff7e32e84f759c1a9691a01cdc1972c72))

- Use space suffix for chicago journal titles ([`d7fbe74`](https://github.com/citum/citum-core/commit/d7fbe74b5a5d1455b10559683fb7fe014fdaf45e))

- Extract 'and' configuration from citation macros ([`cf29fe1`](https://github.com/citum/citum-core/commit/cf29fe14b44491aa7774fab1acce00749d9945cb))

- Use full names in bibliography for styles without style-level initialize-with (#56) ([`73935e0`](https://github.com/citum/citum-core/commit/73935e0ab005da00854fa2cf84de9d218a24d558))

- Improve bibliography template extraction ([`49d5129`](https://github.com/citum/citum-core/commit/49d512927952e3b69d136a5302923e28b351b3f4))

- Deduplicate nested lists and fix volume-issue grouping ([`3166f87`](https://github.com/citum/citum-core/commit/3166f875af4e693b3e9ad350b6eae8f2e321de8a))

- Extract author from substitute when primary is rare role ([`6f6d115`](https://github.com/citum/citum-core/commit/6f6d11555584e9ffbd2fca91a853270c199512fd))

- Detect numeric styles and position year at end ([`2df15b5`](https://github.com/citum/citum-core/commit/2df15b568640639b741cf6196b47e756fa80d042))

- Add space prefix to volume after journal name ([`7231c8b`](https://github.com/citum/citum-core/commit/7231c8b4f98a56fe1c38b278fc6e40ca1b9404be))

- Extract correct citation delimiter from innermost group ([`65f2341`](https://github.com/citum/citum-core/commit/65f23410139f86d495863baa8728bc395e417dd7))

- Handle Choose blocks in delimiter extraction ([`102f8e7`](https://github.com/citum/citum-core/commit/102f8e7272b31d99b0ba59d6a69cdd73f0f2fe25))

- Extract bibliography delimiter from nested groups ([`22ae5de`](https://github.com/citum/citum-core/commit/22ae5dea588c7b22ff6801c8a7d733786bad419d))

- Improve bibliography component extraction for nested variables ([`04b95e5`](https://github.com/citum/citum-core/commit/04b95e57d4d63dda8332ff785a15aee55b62bd73))

- Prevent duplicate list variables ([`bc7c1e8`](https://github.com/citum/citum-core/commit/bc7c1e8b742b4fb25e6d3c6c3a52b17ec8b97255))

- Improve contributor and bibliography migration ([`5e84131`](https://github.com/citum/citum-core/commit/5e84131e305a93c74194adac4f988481040b3b5b))

- Add text-case support for term nodes and deduplicate numbers ([`218c3e1`](https://github.com/citum/citum-core/commit/218c3e1cecfc7e6e84cd34c046f476b0c17e576c))

- Use IndexMap to preserve component ordering ([`7e2781e`](https://github.com/citum/citum-core/commit/7e2781eb03985fc6402c585aab2f24897a724bef))

- Disable hardcoded component sorting ([`42b5af8`](https://github.com/citum/citum-core/commit/42b5af8f2bfb45a0403e7c7ba0cde3e0c8e8a1a9))

- Add date deduplication in lists ([`0ebeb83`](https://github.com/citum/citum-core/commit/0ebeb83977c922d776be85b7b8113fa95d49b275))

- Preserve label_form from CSL 1.0 Label nodes ([`d0bc155`](https://github.com/citum/citum-core/commit/d0bc15525739ee34b3797653d1710c7cd01328e9))

- Preserve macro call order across choose branches ([`b4b96ae`](https://github.com/citum/citum-core/commit/b4b96aece112d5dc7093b5b31cbcb7f04fe4248d))

- Correct contributor name order logic ([`d2dba73`](https://github.com/citum/citum-core/commit/d2dba732dfa92f5fa5b9d8bc3c4ffa8e68593137))


**reference**

- Extract actual day from EDTF dates ([`f013d0b`](https://github.com/citum/citum-core/commit/f013d0be704691aa9df107b5d12f2f0b97b54b01))


**render**

- Suppress trailing period after URLs in nested lists ([`ee7989f`](https://github.com/citum/citum-core/commit/ee7989fa3e26187e738055193e17f560e00b1ed7))


**scripts**

- Attach overrides to template objects for JSON output ([`2b665b9`](https://github.com/citum/citum-core/commit/2b665b945b60dc6d2bb85366f734384d8c48fa8b))

- Per-type confidence metric for template inferrer ([`91607bb`](https://github.com/citum/citum-core/commit/91607bbfbbab38fd54bfe77e3eba44e794ae5f6c))

- Detect prefixes and emit delimiter in inferrer output ([`e330f9e`](https://github.com/citum/citum-core/commit/e330f9e8f0a6361b14d444044b3040d70c28784c))


**sort**

- Strip leading articles and fix anonymous work formatting ([`82acff5`](https://github.com/citum/citum-core/commit/82acff5ee631ee54f3013350118ec4c7ac5af746))


**styles**

- Update metadata to CSLN conventions ([`1fac31e`](https://github.com/citum/citum-core/commit/1fac31efa6d0d1820bb5704175b8805388740bf4))

- APA integral and config ([`c0010cc`](https://github.com/citum/citum-core/commit/c0010cc6c10ecf608fb15ea7a77d53b896785e4d))


**web**

- Add scroll margin to example anchors ([`662bc03`](https://github.com/citum/citum-core/commit/662bc03e82a57f0d356ac44155786a38cd06a6cf))


**workflow**

- Implement opus review critical fixes and strategy updates ([`491732e`](https://github.com/citum/citum-core/commit/491732e09ce9e4258a16feb4469b16a16f92e1d1))



### Documentation

**agent**

- Add prior art research and roadmap ([`d50a132`](https://github.com/citum/citum-core/commit/d50a132ea250b2333fd4c38811b1e2ec0e1917f4))

- Add style editor vision document ([`7d43e31`](https://github.com/citum/citum-core/commit/7d43e31089eec347be47440ebe1c550f1fcfea8d))


**architecture**

- Add migration strategy analysis ([`5614ce4`](https://github.com/citum/citum-core/commit/5614ce4dea7852ba2e9af0dc703b5ee2c5798ac7))

- Revise migration strategy analysis ([`e80da58`](https://github.com/citum/citum-core/commit/e80da5865b182650e97666ca014e24bfcc47daac))

- Add validation results to migration strategy analysis ([`a446a0c`](https://github.com/citum/citum-core/commit/a446a0c10d357dfca473c923505e0830d2f1367f))


**bench**

- Add benchmark requirements policy ([`8834b91`](https://github.com/citum/citum-core/commit/8834b91a1c06bf16d12977f098b08541fe22678e))


**design**

- Update style aliasing decisions ([`89784ef`](https://github.com/citum/citum-core/commit/89784ef55a54845e7fbb604bc81b26799d1e26d8))


**examples**

- Clarify EDTF uses locale terms not hardcoded values ([`62359ae`](https://github.com/citum/citum-core/commit/62359ae2612cf03b17748402ed71f562723edaa6))

- Add info field and restructure bibliography files ([`1974cc5`](https://github.com/citum/citum-core/commit/1974cc541f7fdbb1bdf654eace0b38a0122e16f6))

- Add chaucer with edtf approximate date ([`a4906dd`](https://github.com/citum/citum-core/commit/a4906dd73bdd8162d7b37f40f5520fcb49157610))


**instructions**

- Add humanizer skill integration ([`488d246`](https://github.com/citum/citum-core/commit/488d246aada1d204a00c36cc5f8135b5d3d8f999))


**migrate**

- Convert remaining TODOs to issues ([`a77c738`](https://github.com/citum/citum-core/commit/a77c738021d132b49009f9aa7ded798900401626))


**multilingual**

- Add architectural design for multilingual support ([`ac1a0b1`](https://github.com/citum/citum-core/commit/ac1a0b112e7a910dd8d31aad0d0ed8797796d5b9))


**reference**

- Convert parent-by-id TODO ([`821405c`](https://github.com/citum/citum-core/commit/821405c395b85716089ea088b3f30f6088594ebf))


**skills**

- Prefer wrap and delimiters for semantic joining ([`81e6194`](https://github.com/citum/citum-core/commit/81e6194836b545009a7126974d0ef9e5df845415))


**state**

- Update state.json with delimiter fix progress ([`fcf46b7`](https://github.com/citum/citum-core/commit/fcf46b748a0c6cb41573d72a07ab2d02ab1c509b))



### Features

**analyze**

- Add parent style ranking for dependent styles ([`68a9f43`](https://github.com/citum/citum-core/commit/68a9f431af9e1e749723677769f0dc0e27dbbc7d))


**bib**

- Implement subsequent author substitution ([`089483d`](https://github.com/citum/citum-core/commit/089483dabdb719ad4e837994f1d61b09e0a2f9ad))


**citations**

- Add infix support for integrals ([`556c160`](https://github.com/citum/citum-core/commit/556c1609e29dc3f2cac9b11205973890722f797b))


**cli**

- Merge process and validate into csln ([`04ba582`](https://github.com/citum/citum-core/commit/04ba5827e1e932cfd61444704de92c7d8c36ebdf))

- Add --show-keys flag to process command ([`e1cdaf1`](https://github.com/citum/citum-core/commit/e1cdaf1578613f0cc918b9f6c38c5e9d0e7c4be7))

- Support complex citation models as input ([`a466f20`](https://github.com/citum/citum-core/commit/a466f2087efcbf738c9e0d7a47ef6a21e0f0eb88))


**contributor**

- Implement et-al-use-last truncation ([`f02fb2f`](https://github.com/citum/citum-core/commit/f02fb2f6928fed3c5ad043a326027f53cc0d8f36))


**core**

- Implement style versioning and forward compatibility ([`09c8b8f`](https://github.com/citum/citum-core/commit/09c8b8f280adca84189a2c93220f38fd8d76f6d0))

- Add json schema generation support and docs ([`398c3f1`](https://github.com/citum/citum-core/commit/398c3f12e7ae1e835ae5f4177da3720988963f0a))

- Add multi-language locale support ([`eb3b1c1`](https://github.com/citum/citum-core/commit/eb3b1c177be7fd69ee92ecc9a33d738a9d6515c6))

- Add overrides support to contributor and date components ([`f28277d`](https://github.com/citum/citum-core/commit/f28277da98bf7370237376b3b56f88a898638731))

- Add else-if branches and type-specific bibliography templates ([`a61ff66`](https://github.com/citum/citum-core/commit/a61ff66b1c4e0d8b349698b856551bad88d9b648))

- Add style preset vocabulary for Phase 1 ([`704acc5`](https://github.com/citum/citum-core/commit/704acc5eb27b48be7c72a1c1ed1fd72e4b728f9e))

- Add embedded priority templates for Phase 2 ([`97bee83`](https://github.com/citum/citum-core/commit/97bee83e3a348ecc299c586a98fdaf043ad21a2f))

- Expose embedded templates via use-preset ([`90df37a`](https://github.com/citum/citum-core/commit/90df37a629a6c944e75485a61483a8fa18fa4633))

- Enhance citation model and add bibliography separator config ([`6067be9`](https://github.com/citum/citum-core/commit/6067be91f1eaa1018ad723634cd4b6701c14aca6))

- Implement editor label format standardization ([`66e680f`](https://github.com/citum/citum-core/commit/66e680f2c8f7cc7f4fdb2299da43ce8522acc3e5))

- Add prefix_inside_wrap for flexible wrap ordering ([`fcbb2c8`](https://github.com/citum/citum-core/commit/fcbb2c84a32c8b6d835f6cf953c66c9a345b0a08))

- Add InputBibliography and TemplateDate fallback support ([`2ab55b4`](https://github.com/citum/citum-core/commit/2ab55b40b8f67695a85a156992d0797a8468f656))

- Implement declarative hyperlink configuration ([`acef5f8`](https://github.com/citum/citum-core/commit/acef5f81dc416a4d3fc62440f651a0348da3f00d))

- Add Tier 1 legal reference types ([`e17ca43`](https://github.com/citum/citum-core/commit/e17ca4368bb5257bf779abdc856e0ebe554e19e1))

- Add Patent and Dataset reference types ([`c55140c`](https://github.com/citum/citum-core/commit/c55140cc9310b096c510f033e94bf8bb691faee2))

- Add Standard and Software types ([`57df1d9`](https://github.com/citum/citum-core/commit/57df1d9ed949913a6b775d4cb098f8cab4ec44fe))

- Add locale term role labels ([`48001bb`](https://github.com/citum/citum-core/commit/48001bb7b0a26ae57cfb9938dff40f7c24e4fbac))


**core,processor**

- Implement curly quote rendering ([`3c90fd2`](https://github.com/citum/citum-core/commit/3c90fd236cb39050453c2ce4e3bd576217007bba))

- Add locator support and refine punctuation rendering ([`bb2a485`](https://github.com/citum/citum-core/commit/bb2a48593a27febd54ef8f6d551c9adb5d54dcd8))

- Add locator support, mode-dependent logic, and integral citation templates ([`d522b5b`](https://github.com/citum/citum-core/commit/d522b5bca3a1b9bc28c4ad4b239ec20546fa8f9f))


**csl-tasks**

- Implement task management CLI ([`0707df8`](https://github.com/citum/citum-core/commit/0707df856ef8b522f633201f758e2df07f480da7))

- Add ux improvements for local-first workflow ([`dc11e5f`](https://github.com/citum/citum-core/commit/dc11e5f5d2d4269eef0fbdeb812b2432651daa03))

- Implement GitHub issue number alignment for task IDs ([`c894387`](https://github.com/citum/citum-core/commit/c894387c7e60da54db9c116f48a3c083e84876aa))

- Improve GitHub sync error handling ([`2f59c7d`](https://github.com/citum/citum-core/commit/2f59c7d517943a82f4729bbb43de00999e38d637))

- Add duplicate detection for github sync ([`c11ad50`](https://github.com/citum/citum-core/commit/c11ad50189a69198061075f349f9d6967780e46c))


**csln_core, csln_migrate**

- Add CSLN schema and OptionsExtractor ([`0501dc0`](https://github.com/citum/citum-core/commit/0501dc0b9b3f43b3bac9b5bd9e93dff9309baf5b))


**csln_migrate**

- Integrate OptionsExtractor into migration CLI ([`db842a1`](https://github.com/citum/citum-core/commit/db842a135e08694f301f928c354bb6585f53b05f))

- Add TemplateCompiler for clean CSLN output ([`e919140`](https://github.com/citum/citum-core/commit/e9191407a75b5807b85a9f781a597ac76269e010))

- Improve template ordering and author-date citation ([`a80aeee`](https://github.com/citum/citum-core/commit/a80aeeefc6555965855209fe2d8ba43778b3f795))


**dates**

- Implement EDTF uncertainty, approximation, and range rendering ([`fff0065`](https://github.com/citum/citum-core/commit/fff006506a6c81257642b0fb7c559855003a589e))


**edtf**

- Implement modern winnow-based parser ([`cfa8732`](https://github.com/citum/citum-core/commit/cfa873250ad0699d9643232b6304ef31bb146b04))


**engine**

- Add citation layout support ([`910f3d5`](https://github.com/citum/citum-core/commit/910f3d5a2b0b93b50e872af5ac117e9d477db03d))

- Add bibliography entry numbering for numeric styles ([`3b2faa3`](https://github.com/citum/citum-core/commit/3b2faa3933c2766218219941185ea867c57c634e))

- Fix name initials formatting and extraction ([`e9247bf`](https://github.com/citum/citum-core/commit/e9247bffd630d6f36ecddcf961ad8b6db046907c))

- Support per-component name conjunction override ([`03b1d35`](https://github.com/citum/citum-core/commit/03b1d35dd1c2ba0fc5501adb33d5bf0227b40db4))

- Implement declarative title and contributor rendering logic ([`1fd8a5f`](https://github.com/citum/citum-core/commit/1fd8a5f5e92e889a673c41a92b8e9d172b2ed655))

- Achieve 15/15 oracle parity for Chicago and APA (#54) ([`4892bd9`](https://github.com/citum/citum-core/commit/4892bd96de7e25d6b7acc21fa1b2df6453ca5fa2))

- Add citation grouping and year suffix ordering ([`175df7d`](https://github.com/citum/citum-core/commit/175df7df5dc96cf20f2d67566b8dd7a4e107d020))

- Improve bibliography separator handling ([`ffc718a`](https://github.com/citum/citum-core/commit/ffc718a5dca9dba2550b84a27b74a85de2ee2e78))

- Implement inner/outer affixes ([`5fd7f66`](https://github.com/citum/citum-core/commit/5fd7f6613152097fd1fc30026287e7ccfa8acb23))

- Improve rendering engine and test dataset ([`1a74457`](https://github.com/citum/citum-core/commit/1a7445778746bb1b43e96e040c1871b358426bbf))

- Add integral citation mode to CLI output ([`66db9b0`](https://github.com/citum/citum-core/commit/66db9b0299d3316a0b57f2efcc05c8f9f4fee4ad))

- Implement multilingual BCP 47 resolution ([`828a7ff`](https://github.com/citum/citum-core/commit/828a7ff00d00d0a1ec7b9e833a11b641a6bf97de))

- Implement strip-periods in term and number labels ([`a58ee37`](https://github.com/citum/citum-core/commit/a58ee37d00bfdc2e6083cd8b93071a14cc1079dd))

- Add document-level processing prototype ([`02dba48`](https://github.com/citum/citum-core/commit/02dba48cda1ea5ba7f9f207e3566d4d7f12e0af1))

- Implement WinnowCitationParser for Djot syntax ([`9c7e787`](https://github.com/citum/citum-core/commit/9c7e7876ddc252c860100e53f6a61ce94a5fd49b))

- Simplify Djot citation syntax by removing mandatory attribute ([`b2f487c`](https://github.com/citum/citum-core/commit/b2f487cb503dc3bf8f8b63c965d24e0466193086))

- Support hybrid and structured locators in Djot parser ([`08f24d9`](https://github.com/citum/citum-core/commit/08f24d9c755ecd7a00d34b792570c165638090ec))

- Implement djot document processing and structured locators ([`a28b741`](https://github.com/citum/citum-core/commit/a28b7410165829a5b336c4f08e44222116414c9b))

- Add HTML output for Djot document processing ([`183314a`](https://github.com/citum/citum-core/commit/183314a4383d4a3f93610e717b6d9286c85f5c55))

- Support infix variable in integral citations ([`2fa7f7b`](https://github.com/citum/citum-core/commit/2fa7f7b46918c0ffc3f3f9f4c6c5f2dfe3fa724c))


**fixtures**

- Expand test references to 28 items across 17 types ([`18e915a`](https://github.com/citum/citum-core/commit/18e915a8e0a652e3e85f37bfe08259e183d5eb95))


**github**

- Add style request issue template ([`e5be139`](https://github.com/citum/citum-core/commit/e5be139bdb3a869e8e13c84a2cc3a0926468f39d))


**locale**

- Implement punctuation-in-quote as locale option ([`7011486`](https://github.com/citum/citum-core/commit/7011486d9802a31ccfa5377ccf38caa40e92d3f1))

- Expose locator terms for page labels ([`064caa4`](https://github.com/citum/citum-core/commit/064caa49845e765644244cc72ad293d47baaee37))


**migrate**

- Extract bibliography sort and fix citation delimiter ([`f96d4d8`](https://github.com/citum/citum-core/commit/f96d4d86acb96342e26300c56efb8e70d0779f23))

- Add type-specific template extraction (disabled) ([`a74aba3`](https://github.com/citum/citum-core/commit/a74aba306b45f0da24996361c9e18745bb3629a5))

- Add chapter type_template for author-date styles ([`38116f0`](https://github.com/citum/citum-core/commit/38116f027c509a865575eda388a05e8123686bb4))

- Extract volume-pages delimiter from CSL styles ([`44dce9e`](https://github.com/citum/citum-core/commit/44dce9ee7979e1aa9f2a1dfd0b12ebeeba5868cb))

- Extract bibliography entry suffix from CSL layout ([`e37a9c1`](https://github.com/citum/citum-core/commit/e37a9c1fb61c248ef79699be49538bf4fa1cdce2))

- Add preset detection for extracted configs ([`a198754`](https://github.com/citum/citum-core/commit/a198754cb98bb4999431180180e28a7a36965c1a))

- Infer month format from CSL date-parts ([`08182f5`](https://github.com/citum/citum-core/commit/08182f53668a333ad3f799ffd5128919e46a491f))

- Support type-conditional substitution extraction ([`d225b1f`](https://github.com/citum/citum-core/commit/d225b1f86410a6cdca9f8548ef04328298f2315e))

- Improve migration fidelity and deduplication ([`4559a73`](https://github.com/citum/citum-core/commit/4559a73c92bb56f9f34050f7bf3483c22be31505))

- Implement publisher-place visibility rules ([`c41a3ff`](https://github.com/citum/citum-core/commit/c41a3ffed2c95eb124bb5f38d4c9cf4b27fefc7e))

- Add variable provenance debugger ([`c719c51`](https://github.com/citum/citum-core/commit/c719c51f52747a447d80bf0455a92fdc8e052c3a))

- Add custom delimiter support for CSL 1.0 compatibility ([`43a4d02`](https://github.com/citum/citum-core/commit/43a4d022df928f35ffda91884c1790885a389f26))

- Implement complete source_order tracking system ([`5729ac5`](https://github.com/citum/citum-core/commit/5729ac5d72b91cffb5e513b007af8d45441b9d98))

- Integrate template resolution cascade with per-component delimiters ([`ecd46b5`](https://github.com/citum/citum-core/commit/ecd46b5f9720cf60f0c512745619a7b6e8092dce))


**migration**

- Add styleauthor migration pathway ([`de689c1`](https://github.com/citum/citum-core/commit/de689c1a1ed0c71a3ca5d1893ea6f39aeaa4bdb6))


**multilingual**

- Implement holistic parallel metadata for names and titles ([`c8d6260`](https://github.com/citum/citum-core/commit/c8d6260d9c68d28bdfe6a187cdd1cd2300bb92e6))

- Implement multilingual support ([`d98b401`](https://github.com/citum/citum-core/commit/d98b401fd481e05fd89fdac5e6f41aa09ffefbe6))


**options**

- Add configurable URL trailing period ([`77ae1af`](https://github.com/citum/citum-core/commit/77ae1af52dc0baee9bd88b5c3092085c9deacd25))

- Add substitute presets and style-aware contributor matching ([`3c4db2b`](https://github.com/citum/citum-core/commit/3c4db2bc96c26cb75fb7e85b909edec090b4f8aa))


**presets**

- Add options-level preset support ([`8824ff5`](https://github.com/citum/citum-core/commit/8824ff5e536f60650e9ff92c32c8e852b1213dd4))


**reference**

- Support parent reference by ID ([`13f93c2`](https://github.com/citum/citum-core/commit/13f93c24c99a479348e42502d4ed95221737f76a))


**render**

- Add title quotes and fix period-inside-quotes ([`86b2f37`](https://github.com/citum/citum-core/commit/86b2f37fd0a75e0fb4d3d0b815545720edf50a74))

- Implement structured hyperlinking in templates ([`34395d9`](https://github.com/citum/citum-core/commit/34395d9edb42e2bc89cba53612849a65509a0113))


**scripts**

- Add citeproc-js oracle for verification ([`0459ae1`](https://github.com/citum/citum-core/commit/0459ae1bec90849cb6757252821a84a50fefb377))

- Add structured diff oracle for component-level comparison ([`e97a0aa`](https://github.com/citum/citum-core/commit/e97a0aa82c555746b30ec1e9d274c1a55719b0ae))

- Add batch oracle aggregator for pattern detection ([`b19d3ab`](https://github.com/citum/citum-core/commit/b19d3ab537fcf414b0123a3a19ef0f81b11e8b48))

- Add parallel execution and --all flag for corpus analysis ([`b8ec99e`](https://github.com/citum/citum-core/commit/b8ec99e485121077e0b4a0555af2ed25266e96f5))

- Add output-driven template inference engine ([`b617ac2`](https://github.com/citum/citum-core/commit/b617ac208745eb95d75cac342b11e1484b782b13))

- Add prefix, wrap, and items grouping to inferrer ([`37420bc`](https://github.com/citum/citum-core/commit/37420bc74896005cc570b65ffbed3387dd1894f3))

- Show per-type confidence in verbose output ([`63ce8b0`](https://github.com/citum/citum-core/commit/63ce8b0deda269b80d3b4aa1b84b0f76992ca7de))

- Add formatting inference and parent-monograph detection ([`37763d2`](https://github.com/citum/citum-core/commit/37763d2f15795453f397c5186a147929778fc271))


**skills**

- Add styleauthor skill and agent for LLM-driven style creation ([`166c563`](https://github.com/citum/citum-core/commit/166c5637d10d9d9e79659a4c35b0c036fc902c00))

- Add update to styleauthor ([`5ebe12d`](https://github.com/citum/citum-core/commit/5ebe12d19111c574fee219d737f900e07ff3a3ab))


**styleauthor**

- Add workflow optimizations ([`49a8fbb`](https://github.com/citum/citum-core/commit/49a8fbbed76876bc1c0ee4993ec02701c1d8910f))

- Add autonomous command whitelist ([`00be0a1`](https://github.com/citum/citum-core/commit/00be0a121dfcafac292233d9d917d8973bfce7ca))


**styles**

- Add APA 7th edition CSLN style ([`cceab8c`](https://github.com/citum/citum-core/commit/cceab8ca39a905dfde124c06650d7060fe92880e))

- Add APA 7th edition CSLN style with integral/narrative support ([`32341ed`](https://github.com/citum/citum-core/commit/32341ed12f0603d00a9d163dc78fafa75449f2df))

- Add elsevier-with-titles style ([`3d795d4`](https://github.com/citum/citum-core/commit/3d795d404502917fbff17c6926c00fbd602eae10))

- Add chicago manual of style 18th edition (author-date) ([`69fc345`](https://github.com/citum/citum-core/commit/69fc345d01aa9869efed705b6327761da3044fef))

- Add elsevier-harvard author-date style ([`f21012e`](https://github.com/citum/citum-core/commit/f21012eb341c6ffb9d3b88b5d85c11da5410e666))

- Add elsevier-vancouver ([`681707a`](https://github.com/citum/citum-core/commit/681707ae1245b6babfc71b55984d9fc97a08adf8))

- Implement springer-vancouver-brackets.yaml ([`923512c`](https://github.com/citum/citum-core/commit/923512cd9a1875e1bd7a7c83ec74815ded66ea09))

- Add springer-vancouver-brackets style ([`13a6ed1`](https://github.com/citum/citum-core/commit/13a6ed13e1a93e7618af7e504289a279c4e0eb32))

- Add strip-periods to springer-basic ([`f91fde8`](https://github.com/citum/citum-core/commit/f91fde8552d2da97c41718dcee8e0ab679b355c8))

- Add taylor-and-francis-chicago-author-date ([`51179de`](https://github.com/citum/citum-core/commit/51179deec9b517a5f89738fa0ac8c308452ebc14))

- Add legal-case override to APA 7th ([`58cf56c`](https://github.com/citum/citum-core/commit/58cf56cbffaf07dd114437e55adcc4f60a15842f))

- Add label config to AMA ([`8e261be`](https://github.com/citum/citum-core/commit/8e261becb4dad42388044e8b5ada2f5b0ede220b))


**task-cli**

- Fill out task management skill ([`950bd0e`](https://github.com/citum/citum-core/commit/950bd0e3cba8c9668019019b0bac64741c32b4e0))


**test**

- Expand test data to 15 reference items (#53) ([`896ba86`](https://github.com/citum/citum-core/commit/896ba86b85bd022000dd669aa694bcb214f050bb))


**workflow**

- Add regression detection with baseline tracking ([`f905f2d`](https://github.com/citum/citum-core/commit/f905f2d8a92dc2f4ed17796ecf3a76299b9107d7))

- Optimize styleauthor migration workflow ([`6e5d7b5`](https://github.com/citum/citum-core/commit/6e5d7b521cb8443fccee0b2ccbc3a0e0488b47cc))

- Migration workflow optimizations ([`69ccbfe`](https://github.com/citum/citum-core/commit/69ccbfe38e5a494a8dbeea33827d0f51807ef71e))



### Refactor

**beans**

- Reorganize tasks into epic structure ([`1da7954`](https://github.com/citum/citum-core/commit/1da7954af57bfa81181c03bf10e7dd08cddcf5ec))


**cli**

- Csln-processor -> csln process ([`96d8e24`](https://github.com/citum/citum-core/commit/96d8e2495c53a07fe52f54bc66c5c6512c1df30a))


**core**

- Use DelimiterPunctuation enum for volume_pages_delimiter ([`0282483`](https://github.com/citum/citum-core/commit/028248334603f51b9ddd1f887b365391ee4b377f))

- Remove feature gate from embedded templates ([`7abf381`](https://github.com/citum/citum-core/commit/7abf38127ff677f5d087a5d98bc481d198aef5f1))

- Strict typing with custom fields ([`19a5f18`](https://github.com/citum/citum-core/commit/19a5f18f6f846e4c848c8c7fea29210d7c919d8f))


**engine**

- Modularize document processing ([`0fe8d54`](https://github.com/citum/citum-core/commit/0fe8d54d8eb7311183dfd1d2ed88dc990760994d))


**migrate**

- Implement occurrence-based template compilation ([`f40fb9a`](https://github.com/citum/citum-core/commit/f40fb9aea6cc41dfdf1b0f2b044647f9c8a841ab))


**scripts**

- Harden oracle component parser ([`ddaad0d`](https://github.com/citum/citum-core/commit/ddaad0debd34b3c88822a7bbd5a294625db41aea))


**styleauthor**

- Use Sonnet + checkpoints ([`ee8e538`](https://github.com/citum/citum-core/commit/ee8e53810228fe5421ce2bc36d855033e8f46ad1))



### Styling

**migrate**

- Fix doc comment indentation for clippy ([`b78c1f3`](https://github.com/citum/citum-core/commit/b78c1f3a7a2248a3fbe01c751bb8e6c42bac3b5d))

