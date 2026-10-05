# @citum/engine

WebAssembly and TypeScript bindings for the Citum citation renderer.

This package exposes the browser/JavaScript entry points from
`citum-bindings`. Applications can use a bundled style selector or supply full
Citum YAML, then validate styles, render citations, or format a document.

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

- Styles are full Citum YAML strings or exact selectors such as `id: apa-7th`.
- References are JSON strings containing either an object map keyed by ID or a
  CSL-JSON-style array with `id` fields.
- `renderCitation` accepts one Citum citation JSON payload.
- Functions throw JavaScript exceptions when parsing, validation, or rendering
  fails.

```ts
const style = "id: apa-7th";

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
validateStyle(style);
```

Render one citation to HTML:

```ts
const citationHtml = renderCitation(style, refsJson, citationJson);
```

Render a full bibliography to HTML:

```ts
const bibliographyHtml = renderBibliography(style, refsJson);
```

Return canonical YAML with inheritance, variants, scoped options, and template
references resolved:

```ts
const expandedStyleYaml = materializeStyle(style);
```

Read style metadata:

```ts
const metadata = JSON.parse(getStyleMetadata(style));
```

Format a document in one call:

```ts
const result = JSON.parse(
  formatDocument(
    JSON.stringify({
      style: { kind: "id", value: "apa-7th" },
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

Selectors resolve bundled IDs and aliases only. For example, `id: apa` and
`id: mhra` resolve without a network request. An unknown or catalog-only ID
throws an error that asks for version-pinned YAML. Fetch that YAML in your
application, then pass its contents directly or use
`{ "kind": "yaml", "value": "..." }` with `formatDocument`. The package does
not fetch remote styles, URIs, or paths.

`formatDocument` chooses a locale in this order: the request's `locale`, the
style's `info.default-locale`, then `en-US`. APA therefore uses `en-US`, while
the bundled MHRA style uses the embedded `en-GB` locale. Structured document
and session results report locale fallbacks in `warnings`. Direct render calls
throw an error because their string return type has no warning channel.

`renderCitation` and `renderBibliography` also throw when the style does not
define the requested section. They do not synthesize a missing template or
return an unexplained empty string.

The same selector contract applies to every style-taking function:

```ts
const mhra = "id: mhra";
const mhraMetadata = JSON.parse(getStyleMetadata(mhra));
const mhraCitation = renderCitation(mhra, refsJson, citationJson);
const mhraBibliography = renderBibliography(mhra, refsJson);
```

## Stateful Session API

Use `DocumentSession` when citations evolve incrementally, such as when an
editor inserts or deletes citations one at a time.

```ts
// Create a session with a style; optionally pass an initial refs JSON string
const session = new DocumentSession("id: apa-7th", refsJson);

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
  version: number;
  affected_citations: Array<{ id: string; text: string; ref_ids: string[] }>;
  bibliography: { format: string; content: string; entries: unknown[] };
  renumbering_occurred: boolean;
  warnings: Array<{ level: string; code: string; message: string }>;
}
```

`get_citations` returns `{ formatted_citations }` and `get_bibliography`
returns `{ bibliography }` without triggering a re-render.

## License

Package metadata is `MIT` for JSR compatibility. Citum is dual-licensed under
MIT OR Apache-2.0. See `LICENSE` and `LICENSE-APACHE`.
