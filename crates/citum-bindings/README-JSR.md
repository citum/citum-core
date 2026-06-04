# @citum/engine

WebAssembly and TypeScript bindings for the Citum citation renderer.

This package exposes the browser/JavaScript entry points from
`citum-bindings`. It is meant for applications that already have a Citum YAML
style and bibliography data, and need to validate styles, render citations, or
format a document in JavaScript.

## Install

```bash
deno add jsr:@citum/engine
```

```ts
import init, {
  DocumentSession,
  formatDocument,
  getStyleMetadata,
  materializeStyle,
  renderBibliography,
  renderCitation,
  validateStyle,
} from "jsr:@citum/engine";

await init();
```

## Inputs

- Styles are Citum YAML strings.
- References are JSON strings containing either an object map keyed by ID or a
  CSL-JSON-style array with `id` fields.
- `renderCitation` accepts one Citum citation JSON payload.
- Functions throw JavaScript exceptions when parsing, validation, or rendering
  fails.

```ts
const styleYaml = await Deno.readTextFile(
  "./styles/american-sociological-association.yaml",
);

const refsJson = JSON.stringify({
  smith2020: {
    class: "monograph",
    type: "book",
    title: "Sample Work",
    issued: "2020",
  },
});

const citationJson = JSON.stringify({
  items: [{ id: "smith2020" }],
});
```

## API

Validate a style:

```ts
validateStyle(styleYaml);
```

Render one citation to HTML:

```ts
const citationHtml = renderCitation(styleYaml, refsJson, citationJson);
```

Render a full bibliography to HTML:

```ts
const bibliographyHtml = renderBibliography(styleYaml, refsJson);
```

Materialize template presets in a style:

```ts
const expandedStyleYaml = materializeStyle(styleYaml);
```

Read style metadata:

```ts
const metadata = JSON.parse(getStyleMetadata(styleYaml));
```

Format a document in one call:

```ts
const result = JSON.parse(
  formatDocument(
    JSON.stringify({
      style: { kind: "yaml", value: styleYaml },
      output_format: "html",
      refs: JSON.parse(refsJson),
      citations: [
        {
          id: "cite-1",
          items: [{ id: "smith2020" }],
        },
      ],
    }),
  ),
);
```

In WASM, Citum does not have the resolver chain used by the CLI and server.
For `formatDocument`, pass an inline YAML style with
`{ "kind": "yaml", "value": "..." }`. Style IDs and remote URIs require an
external resolver before calling this package.

## Stateful Session API

Use `DocumentSession` when citations evolve incrementally — an editor inserting
or deleting citations one at a time — rather than re-sending the entire document
on every change.

```ts
// Create a session with a style; optionally pass an initial refs JSON string
const session = new DocumentSession(styleYaml, refsJson);

// Replace the full reference set at any time
session.put_references(refsJson);

// Replace the entire ordered citation list and get updated output
const batchResult = JSON.parse(session.insert_citations_batch(citationsJson));

// Insert one citation (optionally with a neighbour-ID position hint)
const insertResult = JSON.parse(session.insert_citation(citationJson));

// Update an existing citation
const updateResult = JSON.parse(
  session.update_citation("cite-1", citationJson),
);

// Delete a citation
const deleteResult = JSON.parse(session.delete_citation("cite-1"));

// Preview a citation without mutating session state
const preview = JSON.parse(session.preview_citation(itemsJson));

// Read current state
const citations = JSON.parse(session.get_citations());
const bibliography = JSON.parse(session.get_bibliography());

// Dispose when done
session.dispose();
```

Mutation methods (`insert_citations_batch`, `insert_citation`,
`update_citation`, `delete_citation`) return:

```ts
{
  formatted_citations: Array<{ id: string; text: string; ref_ids: string[] }>;
  bibliography: { format: string; content: string; entries: unknown[] };
  warnings: string[];
}
```

`get_citations` returns `{ formatted_citations }` and `get_bibliography`
returns `{ bibliography }` without triggering a re-render.

## License

Package metadata is `MIT` for JSR compatibility. Citum is dual-licensed under
MIT OR Apache-2.0. See `LICENSE` and `LICENSE-APACHE`.
