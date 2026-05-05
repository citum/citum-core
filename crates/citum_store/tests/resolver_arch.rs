//! Tests for the resolver architecture.

#![allow(
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

use citum_schema::{RegistryEntry, StyleRegistry};
use citum_store::resolver::{
    ChainResolver, EmbeddedResolver, FileResolver, RegistryResolver, ResolverError, StyleResolver,
};
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use tempfile::TempDir;

#[test]
fn test_file_resolver_uri_handling() {
    let temp = TempDir::new().unwrap();
    let style_path = temp.path().join("test.yaml");
    fs::write(&style_path, "info:\n  title: Test Style\n").unwrap();

    let resolver = FileResolver;

    // Test with absolute file path
    let uri = style_path.to_str().unwrap();
    let style = resolver.resolve_style(uri).unwrap();
    assert_eq!(style.info.title.as_deref(), Some("Test Style"));
}

#[test]
fn test_file_resolver_locale_resolution() {
    // FileResolver checks locales/{id}.{ext} relative to CWD.
    // We only verify the NotFound path here to avoid polluting the workspace.
    let resolver = FileResolver;
    match resolver.resolve_locale("nonexistent-test-locale-1234") {
        Err(ResolverError::LocaleNotFound(_)) => (),
        _ => panic!("Expected LocaleNotFound"),
    }
}

#[test]
fn test_chain_resolver_fallback() {
    let temp = TempDir::new().unwrap();
    let style_path = temp.path().join("test.yaml");
    fs::write(&style_path, "info:\n  title: Test Style\n").unwrap();

    let file_resolver = Box::new(FileResolver);
    let embedded_resolver = Box::new(EmbeddedResolver);

    let chain = ChainResolver::new(vec![file_resolver, embedded_resolver]);

    // Should find test.yaml
    let style_uri = style_path.to_str().unwrap();
    let style = chain.resolve_style(style_uri).unwrap();
    assert_eq!(style.info.title.as_deref(), Some("Test Style"));

    // Should find embedded style
    let style = chain.resolve_style("apa").unwrap();
    assert_eq!(
        style.info.title.as_deref(),
        Some("American Psychological Association 7th edition")
    );

    // Should return NotFound for missing style
    match chain.resolve_style("nonexistent") {
        Err(ResolverError::StyleNotFound(_)) => (),
        _ => panic!("Expected StyleNotFound error"),
    }
}

#[test]
fn test_chain_resolver_error_propagation() {
    let temp = TempDir::new().unwrap();
    let style_path = temp.path().join("invalid.yaml");
    fs::write(&style_path, "invalid yaml content: [").unwrap();

    let file_resolver = Box::new(FileResolver);
    let embedded_resolver = Box::new(EmbeddedResolver);

    let chain = ChainResolver::new(vec![file_resolver, embedded_resolver]);

    // Should propagate YamlError rather than falling back to next resolver
    let style_uri = style_path.to_str().unwrap();
    match chain.resolve_style(style_uri) {
        Err(ResolverError::YamlError(_)) => (),
        err => panic!("Expected YamlError, got {:?}", err),
    }
}

#[test]
fn test_registry_resolver_loads_builtin_entry() {
    let registry = StyleRegistry {
        version: "1".to_string(),
        styles: vec![RegistryEntry {
            id: "apa-test".to_string(),
            aliases: vec!["apa-alias".to_string()],
            builtin: Some("apa-7th".to_string()),
            path: None,
            url: None,
            title: None,
            description: None,
            fields: vec![],
            kind: None,
        }],
    };
    let resolver = RegistryResolver::new(registry);
    let style = resolver.resolve_style("apa-alias").unwrap();
    assert_eq!(
        style.info.title.as_deref(),
        Some("American Psychological Association 7th edition")
    );
}

#[cfg(feature = "http")]
fn spawn_style_server(
    body: &'static str,
    status: &'static str,
) -> (String, Arc<AtomicUsize>, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let requests = Arc::new(AtomicUsize::new(0));
    let request_counter = Arc::clone(&requests);
    let handle = thread::spawn(move || {
        for stream in listener.incoming().take(2) {
            let mut stream = stream.unwrap();
            let mut buffer = [0; 1024];
            let _ = stream.read(&mut buffer);
            request_counter.fetch_add(1, Ordering::SeqCst);
            let response = format!(
                "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            stream.write_all(response.as_bytes()).unwrap();
        }
    });
    (format!("http://{address}/style.yaml"), requests, handle)
}

#[cfg(feature = "http")]
#[test]
fn test_http_resolver_fetches_and_reuses_cache() {
    use citum_store::resolver::HttpResolver;

    let (url, requests, handle) = spawn_style_server("info:\n  title: HTTP Style\n", "200 OK");
    let temp = TempDir::new().unwrap();
    let resolver = HttpResolver::new(temp.path().to_path_buf());

    let style = resolver.resolve_style(&url).unwrap();
    assert_eq!(style.info.title.as_deref(), Some("HTTP Style"));

    let cached = resolver.resolve_style(&url).unwrap();
    assert_eq!(cached.info.title.as_deref(), Some("HTTP Style"));
    assert_eq!(requests.load(Ordering::SeqCst), 1);
    drop(handle);
}

#[cfg(feature = "http")]
#[test]
fn test_http_resolver_reports_not_found() {
    use citum_store::resolver::HttpResolver;

    let (url, _requests, handle) = spawn_style_server("missing", "404 Not Found");
    let temp = TempDir::new().unwrap();
    let resolver = HttpResolver::new(temp.path().to_path_buf());

    match resolver.resolve_style(&url) {
        Err(ResolverError::StyleNotFound(_)) => {}
        err => panic!("expected StyleNotFound, got {err:?}"),
    }
    drop(handle);
}

#[cfg(feature = "http")]
#[test]
fn test_http_resolver_reports_malformed_yaml() {
    use citum_store::resolver::HttpResolver;

    let (url, _requests, handle) = spawn_style_server("info: [", "200 OK");
    let temp = TempDir::new().unwrap();
    let resolver = HttpResolver::new(temp.path().to_path_buf());

    match resolver.resolve_style(&url) {
        Err(ResolverError::YamlError(_)) => {}
        err => panic!("expected YamlError, got {err:?}"),
    }
    let cache_dir = temp.path().join("styles").join("http");
    let cache_entries = fs::read_dir(cache_dir)
        .map(|entries| entries.count())
        .unwrap_or_default();
    assert_eq!(cache_entries, 0);
    drop(handle);
}

#[cfg(feature = "http")]
#[test]
fn test_registry_resolver_loads_url_entry() {
    use citum_store::resolver::HttpResolver;

    let (url, _requests, handle) =
        spawn_style_server("info:\n  title: Registry HTTP Style\n", "200 OK");
    let registry = StyleRegistry {
        version: "1".to_string(),
        styles: vec![RegistryEntry {
            id: "http-style".to_string(),
            aliases: vec![],
            builtin: None,
            path: None,
            url: Some(url),
            title: None,
            description: None,
            fields: vec![],
            kind: None,
        }],
    };
    let temp = TempDir::new().unwrap();
    let resolver = RegistryResolver::new(registry).with_http(HttpResolver::new(temp.path().into()));

    let style = resolver.resolve_style("http-style").unwrap();
    assert_eq!(style.info.title.as_deref(), Some("Registry HTTP Style"));
    drop(handle);
}
