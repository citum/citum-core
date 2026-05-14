---
title: Inheritance and Registries
nav: Inheritance
description: Reuse base styles, pin remote parents, and validate distributed style dependencies.
features:
  - style-inheritance-pins
  - distributed-registries
---

## [auto_awesome] Extending a style

Use `extends` when a style is mainly a tuned wrapper around an existing base.
The parent supplies templates. The wrapper supplies metadata and scoped options.

```yaml
info:
  title: Springer - Basic (author-date)

extends: springer-basic-author-date-core
options:
  contributors: springer
bibliography:
  options:
    date-position: after-author
```

## [lock] Pinning a parent

Remote parents should be pinned by content identifier so the wrapper stays
deterministic across upgrades.

```yaml
extends: https://hub.citum.org/styles/apa-7th.yaml
extends-pin: cid:bafkreicpx6nc4rll4eahyfid2nbxjjli65tf2vjjed75xtl2ymjtjref44
```

Generate a paste-ready pair with:

```bash
citum style pin styles/apa-7th.yaml
```

## [hub] Distributed registries

Resolver chains can fetch styles from embedded styles, local files, remote
registries, and content identifiers. Validation walks the inheritance chain,
checks pins, and applies `info.citum-version` requirements.

```bash
citum style validate path/to/my-journal.yaml
```

For the full operational workflow, see
[`DISTRIBUTED_REGISTRIES.md`](../DISTRIBUTED_REGISTRIES.md).
