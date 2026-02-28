use std::fs;
use std::path::PathBuf;

#[test]
fn test_verify_comprehensive_examples() {
    // Locate the examples directory relative to this test file
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("../../examples/comprehensive.yaml");

    let content = fs::read_to_string(&path).expect("Failed to read comprehensive.yaml");

    // Attempt to deserialize into the expected InputBibliography structure
    let bib: Result<citum_schema::InputBibliography, _> = serde_yaml::from_str(&content);

    match bib {
        Ok(bib) => {
            let refs = bib.references;
            println!("Successfully parsed {} references", refs.len());
            for reference in &refs {
                let id = reference.id().expect("Reference should have an ID");
                println!("Parsed: {}", id);

                // Verify specific fields for Foucault example
                if id == "foucault_discipline" {
                    let keywords = reference.keywords().expect("Should have keywords");
                    assert!(keywords.contains(&"humanities".to_string()));
                    assert!(keywords.contains(&"translation".to_string()));

                    let orig_date = reference
                        .original_date()
                        .expect("Should have original date");
                    assert_eq!(orig_date.0, "1975");
                }
            }
            println!("Successfully verified {} references", refs.len());
        }
        Err(e) => {
            panic!("Failed to parse comprehensive.yaml: {}", e);
        }
    }
}

#[test]
fn test_verify_all_refs_examples() {
    // Locate the examples directory relative to this test file
    let mut examples_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    examples_dir.push("../../examples");

    // Find all *-refs.yaml files
    let entries = fs::read_dir(&examples_dir).expect("Failed to read examples directory");

    let mut yaml_files: Vec<_> = entries
        .filter_map(|entry| {
            entry.ok().and_then(|e| {
                let path = e.path();
                if path.is_file() {
                    let filename = path.file_name()?;
                    let name = filename.to_str()?;
                    if name.ends_with("-refs.yaml") {
                        return Some(path);
                    }
                }
                None
            })
        })
        .collect();

    yaml_files.sort();

    println!("Found {} *-refs.yaml files", yaml_files.len());
    assert!(
        !yaml_files.is_empty(),
        "Expected to find at least one *-refs.yaml file"
    );

    for path in yaml_files {
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        println!("\nTesting: {}", filename);

        let content =
            fs::read_to_string(&path).unwrap_or_else(|_| panic!("Failed to read {}", filename));

        // Attempt to deserialize into InputBibliography
        let bib: Result<citum_schema::InputBibliography, _> = serde_yaml::from_str(&content);

        match bib {
            Ok(bib) => {
                let refs = bib.references;
                println!("  ✓ Successfully parsed {} references", refs.len());
                for reference in &refs {
                    let id = reference.id().expect("Reference should have an ID");
                    println!("    - {}", id);
                }
            }
            Err(e) => {
                panic!("Failed to parse {}: {}", filename, e);
            }
        }
    }
}
