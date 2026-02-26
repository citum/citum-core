# Citum Domains and GitHub Pages Rollout

## Target URL model

- `https://citum.org` -> public entry point
- `https://docs.citum.org` -> Citum core docs
- `https://labs.citum.org` -> labs and experiments
- `https://hub.citum.org` -> product app (deferred)

## Repo to domain mapping

- `citum-org` repo publishes `citum.org` (static public entry point)
- `citum-core` repo publishes `docs.citum.org` from `docs/`
- `citum-labs` repo publishes `labs.citum.org` from `site/`
- `citum-hub` repo can publish a temporary static placeholder for `hub.citum.org` until app cutover

## GitHub Pages implementation details

### citum-core

- Workflow: `.github/workflows/deploy_docs.yml`
- Publish directory: `docs/`
- Domain file: `docs/CNAME` with `docs.citum.org`

### citum-labs

- Workflow: `.github/workflows/deploy_pages.yml`
- Publish directory: `site/`
- Domain file: `site/CNAME` with `labs.citum.org`

### citum-org

- Workflow: `.github/workflows/deploy_pages.yml`
- Publish directory: `site/`
- Domain file: `site/CNAME` with `citum.org`

### citum-hub (temporary placeholder, optional)

- Workflow: `.github/workflows/deploy_pages.yml`
- Publish directory: `site/`
- Domain file: `site/CNAME` with `hub.citum.org`

## DNS records you must create

Use your DNS provider UI and create these records.

### Apex (`citum.org`) for GitHub Pages

- `A @ 185.199.108.153`
- `A @ 185.199.109.153`
- `A @ 185.199.110.153`
- `A @ 185.199.111.153`
- `AAAA @ 2606:50c0:8000::153`
- `AAAA @ 2606:50c0:8001::153`
- `AAAA @ 2606:50c0:8002::153`
- `AAAA @ 2606:50c0:8003::153`

### Subdomains

- `CNAME docs <org-or-user>.github.io`
- `CNAME labs <org-or-user>.github.io`
- `CNAME hub <org-or-user>.github.io` (temporary placeholder)

When the app is ready, replace the `hub` record with the real app host target.

For now you can point `hub` at a temporary placeholder target or leave it unset until app rollout.

## GitHub settings you must apply

For each repo (`citum-org`, `citum-core`, `citum-labs`, and optional `citum-hub` placeholder):

1. Go to `Settings -> Pages`
2. Set source to `GitHub Actions`
3. After first workflow deploy, set custom domain:
   - `citum-core` -> `docs.citum.org`
   - `citum-labs` -> `labs.citum.org`
   - `citum-org` -> `citum.org`
   - `citum-hub` -> `hub.citum.org` (optional while app is deferred)
4. Enable `Enforce HTTPS`

## Cutover sequence

1. Merge/deploy workflows and site content.
2. Confirm each workflow finishes and emits a Pages URL.
3. Configure custom domains in each repo.
4. Add DNS records.
5. Wait for cert issuance and DNS propagation.
6. Verify:
   - `https://citum.org`
   - `https://docs.citum.org`
   - `https://labs.citum.org`
   - `https://hub.citum.org` (if placeholder enabled)

## Cross-domain navigation contract

All three public surfaces should expose shared top links:

- Home (`https://citum.org`)
- Docs (`https://docs.citum.org`)
- Hub (`https://hub.citum.org`)
- Labs (`https://labs.citum.org`)

This preserves consistent navigation regardless of which subdomain a user enters first.
