---
# csl26-7b28
title: 'Refine Renderer::new: bundle params into RendererResources'
status: completed
type: task
priority: normal
created_at: 2026-03-17T00:21:43Z
updated_at: 2026-04-24T12:13:54Z
---

Remove #[allow(clippy::too_many_arguments)] from Renderer::new by bundling style/bibliography/locale/config into RendererResources<'a>. Reduces from 8 params to 5. Update 6 call sites (4 production, 2 tests).
