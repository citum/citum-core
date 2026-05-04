---
# csl26-stat
title: Consider optional stateful mode for citum-server
status: draft
type: research
priority: low
created_at: 2026-05-03T15:00:00Z
updated_at: 2026-05-03T15:00:00Z
---

# Background

`citum-server` currently uses a stateless JSON-RPC architecture. For methods like `render_citation` and `render_bibliography`, the client must send the entire `refs` library and `style_path` on every request.

Recent benchmarking (PR #612) verified performance with `interval=1` (simulating true real-time, per-keystroke updates):
- With 500 references, bibliography refreshes average ~21ms.
- With 5000 references, bibliography refreshes average ~210ms.

While ~210ms is highly performant given the serialization/parsing overhead, it is approaching the threshold of perceptible UI lag for real-time word processor plugins.

# Proposal

To support massive bibliographies with near-zero latency, consider adding an *optional* stateful mode to the server while preserving the stateless mode as the default.

## Architectural Requirements

1. **Persistent Application State:**
   Introduce a session manager, e.g., `Arc<RwLock<HashMap<String, Processor>>>`, to keep instantiated processors hot in memory.

2. **Session Management Methods:**
   - `init_document(doc_id, style_path, initial_refs)`: Creates the processor.
   - `close_document(doc_id)`: Frees the memory when the user closes the document.
   - `update_refs(doc_id, added_refs, removed_ref_ids)`: Delta-updates to push new references without resending the entire library.

3. **Stateful Rendering:**
   `render_citation` and `render_bibliography` could accept an optional `doc_id`. If provided, the server bypasses JSON deserialization and style parsing entirely.

4. **Handling Mutability:**
   `Processor` maintains disambiguation and numbering state. A stateful server requires the plugin to manage operations linearly or requires a mechanism to reset/recalculate state when earlier citations are edited.

## Trade-offs

- **Pros:** Zero serialization overhead; sub-millisecond response times even for massive libraries; true real-time sync.
- **Cons:** High client complexity (word processors are notoriously bad at lifecycle management, risking memory leaks if `close_document` is missed); risk of state drift between client and server.
