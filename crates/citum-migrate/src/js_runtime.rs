use deno_core::{JsRuntime, RuntimeOptions, serde_v8, v8};
use serde::Serialize;
use std::fs;
use std::path::Path;

const EMBEDDED_RUNTIME_BUNDLE: &str = include_str!("../js/embedded-template-runtime.js");
const EMBEDDED_FIXTURES_GLOBAL: &str = "__CITUM_TEMPLATE_TEST_ITEMS";
const EMBEDDED_LOCALE_GLOBAL: &str = "__CITUM_TEMPLATE_LOCALE_XML";

#[derive(Serialize)]
struct EmbeddedCallInput<'a> {
    #[serde(rename = "styleName")]
    style_name: &'a str,
    #[serde(rename = "styleXml")]
    style_xml: &'a str,
    section: &'a str,
}

pub(crate) struct EmbeddedTemplateRuntime {
    runtime: JsRuntime,
}

impl EmbeddedTemplateRuntime {
    pub(crate) fn new(workspace_root: &Path) -> Result<Self, String> {
        let mut runtime = JsRuntime::new(RuntimeOptions::default());
        runtime
            .execute_script("<embedded-template-runtime>", EMBEDDED_RUNTIME_BUNDLE)
            .map_err(|err| format!("failed to initialize embedded runtime bundle: {err:#}"))?;

        let fixtures = load_fixtures(workspace_root)?;
        let locale_xml = load_locale_xml(workspace_root)?;
        let fixtures_json = serde_json::to_string(&fixtures)
            .map_err(|err| format!("failed to serialize embedded fixtures: {err}"))?;
        let locale_json = serde_json::to_string(&locale_xml)
            .map_err(|err| format!("failed to serialize embedded locale XML: {err}"))?;

        let bootstrap = format!(
            "globalThis.{EMBEDDED_FIXTURES_GLOBAL} = {fixtures_json};\n\
             globalThis.{EMBEDDED_LOCALE_GLOBAL} = {locale_json};"
        );
        runtime
            .execute_script("<embedded-template-runtime-bootstrap>", bootstrap)
            .map_err(|err| format!("failed to bootstrap embedded runtime data: {err:#}"))?;

        Ok(Self { runtime })
    }

    pub(crate) fn infer_fragment(
        &mut self,
        style_name: &str,
        style_xml: &str,
        section: &str,
    ) -> Result<String, String> {
        let input = EmbeddedCallInput {
            style_name,
            style_xml,
            section,
        };
        let value = {
            deno_core::scope!(scope, &mut self.runtime);
            let context = scope.get_current_context();
            let global = context.global(scope);

            let infer_key = v8::String::new(scope, "infer_template_fragment")
                .ok_or_else(|| "failed to create infer_template_fragment key".to_string())?;
            let infer_value = global
                .get(scope, infer_key.into())
                .ok_or_else(|| "failed to read globalThis.infer_template_fragment".to_string())?;
            let infer_function = v8::Local::<v8::Function>::try_from(infer_value)
                .map_err(|_| "globalThis.infer_template_fragment is not a function".to_string())?;

            let input_value = serde_v8::to_v8(scope, &input)
                .map_err(|err| format!("failed to serialize embedded inference input: {err}"))?;
            let input_object = v8::Local::<v8::Object>::try_from(input_value)
                .map_err(|_| "failed to create embedded inference input object".to_string())?;

            let fixtures_key = v8::String::new(scope, EMBEDDED_FIXTURES_GLOBAL)
                .ok_or_else(|| "failed to create embedded fixtures key".to_string())?;
            let fixtures_value = global
                .get(scope, fixtures_key.into())
                .ok_or_else(|| format!("failed to read globalThis.{EMBEDDED_FIXTURES_GLOBAL}"))?;
            let test_items_key = v8::String::new(scope, "testItems")
                .ok_or_else(|| "failed to create testItems key".to_string())?;
            if input_object
                .set(scope, test_items_key.into(), fixtures_value)
                .is_none()
            {
                return Err("failed to set testItems on embedded input".to_string());
            }

            let locale_key = v8::String::new(scope, EMBEDDED_LOCALE_GLOBAL)
                .ok_or_else(|| "failed to create embedded locale key".to_string())?;
            let locale_value = global
                .get(scope, locale_key.into())
                .ok_or_else(|| format!("failed to read globalThis.{EMBEDDED_LOCALE_GLOBAL}"))?;
            let locale_xml_key = v8::String::new(scope, "localeXml")
                .ok_or_else(|| "failed to create localeXml key".to_string())?;
            if input_object
                .set(scope, locale_xml_key.into(), locale_value)
                .is_none()
            {
                return Err("failed to set localeXml on embedded input".to_string());
            }

            let output = infer_function
                .call(scope, global.into(), &[input_object.into()])
                .ok_or_else(|| "embedded inference function call failed".to_string())?;

            v8::Global::new(scope, output)
        };

        deno_core::scope!(scope, &mut self.runtime);
        let local = v8::Local::new(scope, value);
        let output = serde_v8::from_v8::<Option<String>>(scope, local)
            .map_err(|err| format!("failed to deserialize embedded inference output: {err}"))?;

        output.ok_or_else(|| "embedded inference returned null".to_string())
    }
}

fn load_fixtures(workspace_root: &Path) -> Result<serde_json::Value, String> {
    let path = workspace_root
        .join("tests")
        .join("fixtures")
        .join("references-expanded.json");
    let text = fs::read_to_string(&path).map_err(|err| {
        format!(
            "failed to read embedded fixture file {}: {err}",
            path.display()
        )
    })?;
    let mut value = serde_json::from_str::<serde_json::Value>(&text).map_err(|err| {
        format!(
            "failed to parse embedded fixture file {}: {err}",
            path.display()
        )
    })?;

    if let Some(map) = value.as_object_mut() {
        map.remove("comment");
    }

    Ok(value)
}

fn load_locale_xml(workspace_root: &Path) -> Result<String, String> {
    let path = workspace_root.join("scripts").join("locales-en-US.xml");
    fs::read_to_string(&path).map_err(|err| {
        format!(
            "failed to read embedded locale file {}: {err}",
            path.display()
        )
    })
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::todo,
    clippy::unimplemented,
    clippy::unreachable,
    clippy::get_unwrap,
    reason = "Panicking is acceptable and often desired in tests."
)]
mod tests {
    use super::EmbeddedTemplateRuntime;
    use std::path::{Path, PathBuf};
    use std::process::Command;

    #[test]
    fn embedded_runtime_matches_node_fragments_for_representative_styles() {
        let workspace_root = workspace_root();
        if !workspace_root
            .join("styles-legacy")
            .join("apa.csl")
            .exists()
        {
            eprintln!("skipping embedded/node parity test because styles-legacy is unavailable");
            return;
        }
        if Command::new("node").arg("--version").output().is_err() {
            eprintln!("skipping embedded/node parity test because node is unavailable");
            return;
        }

        let cases = [
            ("apa", "bibliography"),
            ("apa", "citation"),
            ("ieee", "bibliography"),
        ];

        let mut runtime = EmbeddedTemplateRuntime::new(&workspace_root).unwrap();
        for (style_name, section) in cases {
            let style_path = workspace_root
                .join("styles-legacy")
                .join(format!("{style_name}.csl"));
            let style_xml = std::fs::read_to_string(&style_path).unwrap();
            let embedded = runtime
                .infer_fragment(style_name, &style_xml, section)
                .unwrap();
            let node = run_node_fragment(&workspace_root, &style_path, section);

            let mut embedded_json: serde_json::Value = serde_json::from_str(&embedded).unwrap();
            let mut node_json: serde_json::Value = serde_json::from_str(&node).unwrap();

            strip_fragile_heuristics(&mut embedded_json);
            strip_fragile_heuristics(&mut node_json);

            assert_eq!(
                embedded_json, node_json,
                "embedded and node fragment outputs diverged for {style_name}/{section}"
            );
        }
    }

    /// Recursively strips `confidence`, a floating-point heuristic score that can drift
    /// between the Node.js and Deno runtimes near threshold boundaries.
    ///
    /// Structural fields like `wrap` (a discrete string enum) are intentionally preserved —
    /// divergence there indicates a real inference disagreement, not runtime noise.
    fn strip_fragile_heuristics(val: &mut serde_json::Value) {
        match val {
            serde_json::Value::Object(map) => {
                map.remove("confidence");
                for v in map.values_mut() {
                    strip_fragile_heuristics(v);
                }
            }
            serde_json::Value::Array(arr) => {
                for v in arr.iter_mut() {
                    strip_fragile_heuristics(v);
                }
            }
            _ => {}
        }
    }

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap()
    }

    fn run_node_fragment(workspace_root: &Path, style_path: &Path, section: &str) -> String {
        let output = Command::new("node")
            .arg(workspace_root.join("scripts").join("infer-template.js"))
            .arg(style_path)
            .arg(format!("--section={section}"))
            .arg("--fragment")
            .current_dir(workspace_root)
            .output()
            .unwrap();

        assert!(
            output.status.success(),
            "node fragment inference failed for {}: {}",
            style_path.display(),
            String::from_utf8_lossy(&output.stderr)
        );

        String::from_utf8(output.stdout).unwrap()
    }
}
