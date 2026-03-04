# Citum.org Website and Domain Strategy

**Date:** February 26, 2026  
**Owner:** Citum maintainers  
**Scope:** Public docs/website migration from GitHub Pages project URL to `citum.org`

## 1) Goals

1. Make `https://citum.org` the canonical public entry point.
2. Keep all existing deep links working during migration.
3. Preserve search ranking and avoid split-indexing between old/new URLs.
4. Keep deployment low-friction by continuing to use GitHub Pages first.

## 2) Hosting Model (Recommended)

Use the current GitHub Pages pipeline and attach a custom domain.

- Primary: `https://citum.org`
- Secondary redirect: `https://www.citum.org` -> `https://citum.org`
- Legacy URL retained with redirects where possible:
  - `https://citum.github.io/citum-core/`

## 3) Domain Setup Instructions (Registrar + DNS)

Configure DNS records for `citum.org` as follows.

### Apex (`citum.org`)

Add `A` records:

- `185.199.108.153`
- `185.199.109.153`
- `185.199.110.153`
- `185.199.111.153`

Add `AAAA` records:

- `2606:50c0:8000::153`
- `2606:50c0:8001::153`
- `2606:50c0:8002::153`
- `2606:50c0:8003::153`

### `www` host

Add `CNAME`:

- `www` -> `citum.github.io`

## 4) GitHub Pages Configuration

1. In repository settings (`citum/citum-core`), set Pages custom domain to `citum.org`.
2. Enforce HTTPS after certificate issuance.
3. Commit a `CNAME` file at the published site root containing only:

```text
citum.org
```

4. Verify site is reachable at both:
   - `https://citum.org`
   - `https://www.citum.org` (redirecting to apex)

## 5) Redirect and Canonical Policy

1. Set canonical URL metadata in HTML to `https://citum.org/...`.
2. Update sitemap URLs to `citum.org` only.
3. Keep `citum.github.io/citum-core` links alive for a transition window.
4. Add explicit redirects from old project branding/domain references where feasible.

## 6) Website Content Integration Plan

1. Replace old project name in all public pages (`index`, `examples`, `compat`, guides).
2. Standardize nav/footer links to:
   - Website: `https://citum.org`
   - Source: `https://github.com/citum/citum-core`
3. Add a short migration note on the homepage for one release cycle:
   - “Citum was previously published under Citum/csl26 naming.”

## 7) Release Sequence

1. Land branding + URL updates in docs source.
2. Merge to `main` and publish Pages build.
3. Apply DNS records.
4. Configure custom domain and HTTPS in GitHub Pages.
5. Smoke-test critical pages and deep links.
6. Submit sitemap in search consoles.

## 8) Validation Checklist

- `dig citum.org A +short` returns all 4 GitHub IPs.
- `dig citum.org AAAA +short` returns all 4 GitHub IPv6 values.
- `dig www.citum.org CNAME +short` returns `citum.github.io.`
- `curl -I https://citum.org` returns `200`.
- `curl -I https://www.citum.org` redirects to apex.
- Browser lock icon confirms valid TLS certificate.
- No mixed-content warnings on homepage/docs.

## 9) Risk Notes

- DNS propagation may take up to 24 hours.
- GitHub Pages certificate issuance can lag after first domain attach.
- Search engines may temporarily show both old and new URLs; canonical tags reduce this quickly.

## 10) Follow-up Work

1. Add canonical/sitemap checks to CI for docs builds.
2. Track 404s for 2 to 4 weeks after cutover.
3. Decide whether to keep or eventually retire `citum.github.io/citum-core` as a public entry point.
