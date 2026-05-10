/* tslint:disable */
/* eslint-disable */

/**
 * Format a complete document's citations and bibliography in one call.
 *
 * Takes a JSON-encoded `FormatDocumentRequest` and returns a JSON-encoded
 * `FormatDocumentResult`. In WASM, the resolver chain is unavailable —
 * `StyleInput::Id` and `StyleInput::Uri` variants return an error; use
 * `StyleInput::Yaml` (preferred) or `StyleInput::Path` (if filesystem
 * access is available in the host).
 *
 * # Errors
 *
 * Returns a string error on request JSON parse failure, style resolution failure,
 * or engine rendering error.
 */
export function formatDocument(request_json: string): string;

/**
 * Extract the `info` block from a YAML style string as JSON.
 *
 * # Errors
 *
 * Returns a string error if the YAML fails to parse or the info block cannot
 * be serialized to JSON.
 */
export function getStyleMetadata(style_yaml: string): string;

/**
 * Materialize all template presets in a style and return the updated YAML.
 *
 * # Errors
 *
 * Returns a string error if the input YAML fails to parse or the materialized
 * style cannot be serialized back to YAML.
 */
export function materializeStyle(style_yaml: string): string;

/**
 * Render a full bibliography to HTML.
 *
 * - `style_yaml` — Citum style as YAML
 * - `refs_json` — bibliography as JSON object or CSL-JSON array
 *
 * # Errors
 *
 * Returns a string error on style or reference parse failure.
 */
export function renderBibliography(style_yaml: string, refs_json: string): string;

/**
 * Render a single citation to HTML.
 *
 * - `style_yaml` — Citum style as YAML
 * - `refs_json` — bibliography as JSON object (`{id: Reference}`) or CSL-JSON array
 * - `citation_json` — a single [`Citation`] as JSON
 * - `mode` — optional mode override (e.g. `"Integral"`)
 *
 * # Errors
 *
 * Returns a string error on style/reference/citation parse failure, invalid
 * mode string, or engine rendering error.
 */
export function renderCitation(style_yaml: string, refs_json: string, citation_json: string, mode?: string | null): string;

/**
 * Validate a Citum style string.
 *
 * # Errors
 *
 * Returns a string error describing the parse or schema validation failure.
 */
export function validateStyle(style_yaml: string): void;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly formatDocument: (a: number, b: number, c: number) => void;
    readonly getStyleMetadata: (a: number, b: number, c: number) => void;
    readonly materializeStyle: (a: number, b: number, c: number) => void;
    readonly renderBibliography: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly renderCitation: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => void;
    readonly validateStyle: (a: number, b: number, c: number) => void;
    readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
    readonly __wbindgen_export: (a: number, b: number) => number;
    readonly __wbindgen_export2: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_export3: (a: number, b: number, c: number) => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
