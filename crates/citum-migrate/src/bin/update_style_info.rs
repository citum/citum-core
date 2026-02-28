use citum_migrate::InfoExtractor;
use csl_legacy::parser::parse_style;
use roxmltree::Document;
use std::fs;
use std::path::Path;

fn main() {
    let styles_dir = "styles";
    let csl_dir = "styles-legacy";

    let entries = match fs::read_dir(styles_dir) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Error reading styles directory: {}", e);
            return;
        }
    };

    let mut total = 0;
    let mut success = 0;
    let mut failures = 0;

    println!("Starting bulk update of style info metadata...");

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("yaml") {
            total += 1;

            let filename = path
                .file_name()
                .and_then(|f| f.to_str())
                .unwrap_or("unknown");

            // Find corresponding CSL file
            let yaml_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            let csl_path = Path::new(csl_dir).join(format!("{}.csl", yaml_stem));

            if !csl_path.exists() {
                eprintln!("Warning: No CSL file found for {}", filename);
                failures += 1;
                continue;
            }

            // Read and parse CSL
            let csl_text = match fs::read_to_string(&csl_path) {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("Error reading {}: {}", csl_path.display(), e);
                    failures += 1;
                    continue;
                }
            };

            let doc = match Document::parse(&csl_text) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("Error parsing {}: {}", csl_path.display(), e);
                    failures += 1;
                    continue;
                }
            };

            let legacy_style = match parse_style(doc.root_element()) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error extracting style from {}: {}", csl_path.display(), e);
                    failures += 1;
                    continue;
                }
            };

            // Extract info
            let style_info = InfoExtractor::extract(&legacy_style.info);

            // Read YAML
            let yaml_text = match fs::read_to_string(&path) {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("Error reading {}: {}", path.display(), e);
                    failures += 1;
                    continue;
                }
            };

            // Inject info block
            let updated_yaml = inject_info_block(&yaml_text, &style_info);

            // Write back
            if let Err(e) = fs::write(&path, updated_yaml) {
                eprintln!("Error writing {}: {}", path.display(), e);
                failures += 1;
            } else {
                success += 1;
            }

            if total % 50 == 0 {
                print!(".");
                use std::io::Write;
                std::io::stdout().flush().unwrap();
            }
        }
    }

    println!("\n\n=== UPDATE STATS ===");
    println!("Total Styles: {}", total);
    println!(
        "Success:      {} ({:.1}%)",
        success,
        (success as f64 / total as f64) * 100.0
    );
    println!(
        "Failures:     {} ({:.1}%)",
        failures,
        (failures as f64 / total as f64) * 100.0
    );
}

fn inject_info_block(yaml_text: &str, style_info: &citum_schema::StyleInfo) -> String {
    // Serialize the StyleInfo to YAML
    let info_yaml = serde_yaml::to_string(style_info).unwrap_or_default();

    // Find the end of the info block in the original YAML
    let lines: Vec<&str> = yaml_text.lines().collect();
    let mut info_end = 0;

    for (i, line) in lines.iter().enumerate() {
        if line.starts_with("info:") {
            // Find the next non-indented line (end of info block)
            for j in (i + 1)..lines.len() {
                if !lines[j].is_empty()
                    && !lines[j].starts_with("  ")
                    && !lines[j].starts_with("\t")
                {
                    info_end = j;
                    break;
                }
                if j == lines.len() - 1 {
                    info_end = lines.len();
                    break;
                }
            }
            break;
        }
    }

    if info_end == 0 {
        // No info block found, prepend it
        return format!("info:\n{}\n{}", indent_yaml(&info_yaml), yaml_text);
    }

    // Replace the info block
    let after: String = lines[info_end..].join("\n");

    let indented_info = indent_yaml(&info_yaml);
    format!("info:\n{}\n{}", indented_info, after)
}

fn indent_yaml(yaml: &str) -> String {
    yaml.lines()
        .map(|line| {
            if line.is_empty() {
                line.to_string()
            } else {
                format!("  {}", line)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}
