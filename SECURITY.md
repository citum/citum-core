# Security Policy

## Supported Versions

Security fixes are supported for the latest published Citum crates and the
current `main` branch. Pre-release APIs may change while the project is still
below 1.0, but security-impacting fixes should be reported and handled
privately.

## Reporting a Vulnerability

Report suspected vulnerabilities through GitHub private vulnerability reporting
for this repository. Do not open a public issue for an unconfirmed security
problem.

Include:

- affected crate, feature, command, or API;
- input needed to reproduce the issue;
- observed impact and expected impact;
- whether the issue is reachable through default features, optional features,
  CLI/server usage, or FFI/WASM bindings.

We will triage privately, coordinate a fix, and publish an advisory when the
impact justifies one.

## Security Boundaries

Citum treats citation styles, bibliographic records, registry files, and RPC
payloads as untrusted input. Network style resolution is disabled for risky
schemes by default and should be exposed only with an explicit resolver policy.

The `citum-server` HTTP mode binds to loopback and is intended for local
tooling. Do not expose it directly to untrusted networks.

The C FFI assumes callers pass valid C pointers allocated according to the
documented ownership rules. Invalid null pointers are rejected, but double-free
or foreign-allocator pointers remain caller-side undefined behavior.
