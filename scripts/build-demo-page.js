#!/usr/bin/env node
// Build docs/demo.html from docs/demo.djot via the Citum engine.
//
// Usage:
//   node scripts/build-demo-page.js
//
// Run scripts/build-layout.js afterward to fill nav/footer markers.
// The file is intentionally re-generatable: re-running is idempotent.

'use strict';

const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

const REPO_ROOT = path.join(__dirname, '..');
const DOCS_DIR = path.join(REPO_ROOT, 'docs');

// ---------------------------------------------------------------------------
// 1. Locate the Citum binary
// ---------------------------------------------------------------------------

function findBinary() {
  const release = path.join(REPO_ROOT, 'target', 'release', 'citum');
  const debug = path.join(REPO_ROOT, 'target', 'debug', 'citum');
  if (fs.existsSync(release)) return release;
  if (fs.existsSync(debug)) return debug;
  return null; // fall back to cargo run
}

// ---------------------------------------------------------------------------
// 2. Render demo.djot to HTML via the engine
// ---------------------------------------------------------------------------

function renderBody() {
  const binary = findBinary();
  const style = path.join(REPO_ROOT, 'styles', 'embedded', 'apa-7th.yaml');
  const refs = path.join(DOCS_DIR, 'demo-refs.yaml');
  const doc = path.join(DOCS_DIR, 'demo.djot');

  let cmd;
  if (binary) {
    cmd = `"${binary}" render doc -s "${style}" -b "${refs}" "${doc}" -f html`;
  } else {
    cmd = `cargo run -q --bin citum -- render doc -s "${style}" -b "${refs}" "${doc}" -f html`;
  }

  try {
    return execSync(cmd, { cwd: REPO_ROOT, encoding: 'utf8' });
  } catch (err) {
    console.error('Engine render failed:\n', err.stderr || err.message);
    process.exit(1);
  }
}

// ---------------------------------------------------------------------------
// 3. Extract and clean the engine fragment
//
// Engine output order:
//   <p class="citum-demo-notice">…</p>
//   <hr>
//   <p>Features illustrated…</p>
//   <hr>
//   <section id="The-Infrastructure-of-Scholarly-Memory"><h1>…</h1>…</section>
//   <section id="Bibliography"><h1>Bibliography</h1>…</section>
//
// We keep only the two <section> elements. The leading meta paragraphs/hrs are
// page furniture supplied by the template below. We strip the article's <h1>
// because the page <header> already carries the title.
// ---------------------------------------------------------------------------

function extractFragment(raw) {
  // Drop everything before the first <section
  const sectionStart = raw.indexOf('<section ');
  if (sectionStart === -1) {
    console.error('Engine output did not contain any <section> elements.');
    process.exit(1);
  }
  let fragment = raw.slice(sectionStart);

  // Add class="content" to the article section so .content p+p indent rule fires
  fragment = fragment.replace(
    /^(<section\s+id="The-Infrastructure[^"]*")/,
    '$1 class="content"'
  );

  // Remove the duplicate <h1> inside the article section (page header carries the title)
  fragment = fragment.replace(/<h1>[^<]*<\/h1>\n/, '');

  return fragment.trim();
}

// ---------------------------------------------------------------------------
// 4. Page template
//
// Furniture that is NOT derived from the djot: head, nav markers, page header
// with subtitle, demo notice + feature tags, layout controls, "Reproduce"
// block, footer markers, asset links, toggle script.
// ---------------------------------------------------------------------------

const TEMPLATE = `<!-- PAGE_ID: demo -->
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Citum | Demo</title>
  <link href="https://fonts.googleapis.com/icon?family=Material+Icons" rel="stylesheet" />
  <script src="https://cdn.tailwindcss.com?plugins=forms,container-queries,typography"></script>
  <link
      href="https://fonts.googleapis.com/css2?family=Libre+Franklin:wght@300;400;500;600;700&amp;family=JetBrains+Mono:wght@400;500;700&amp;family=Newsreader:ital,opsz,wght@0,6..72,200..800;1,6..72,200..800&amp;display=swap"
      rel="stylesheet" />
  <script>
      tailwind.config = {
          darkMode: "class",
          theme: {
              extend: {
                  colors: {
                      "primary": "#2a94d6",
                      "background-light": "#fdfbf7",
                      "accent-cream": "#f5f2eb",
                  },
                  fontFamily: {
                      "display": ["Libre Franklin", "sans-serif"],
                      "mono": ["JetBrains Mono", "monospace"]
                  },
                  borderRadius: {
                      "DEFAULT": "0.25rem",
                      "lg": "0.5rem",
                      "xl": "0.75rem",
                      "full": "9999px"
                  },
              },
          },
      }
  </script>
  <style type="text/tailwindcss">
    .glass-nav {
        background: rgba(253, 251, 247, 0.85);
        backdrop-filter: blur(12px);
        border-bottom: 1px solid rgba(42, 148, 214, 0.1);
    }
    body {
      font-family: 'Libre Franklin', sans-serif;
      line-height: 1.6;
      color: var(--citum-ink);
      background: var(--citum-paper);
    }
    .font-mono {
        font-family: 'JetBrains Mono', monospace;
    }
    .container-demo {
      max-width: 1000px;
      margin: 0 auto;
      padding: 2rem;
    }
    header {
      border-bottom: 1px solid var(--citum-border);
      margin-bottom: 2rem;
      padding-bottom: 1rem;
    }
    .config-link {
      display: inline-flex;
      align-items: center;
      gap: 0.5rem;
      color: var(--citum-muted);
      text-decoration: none;
      font-size: 0.9rem;
      padding: 0.4rem 0.8rem;
      border-radius: 4px;
      background: var(--citum-blue-soft);
      transition: all 0.2s;
    }
    .config-link:hover {
      background: var(--citum-paper-deep);
      color: var(--citum-ink-strong);
    }
    .controls {
      background: var(--citum-paper-deep);
      padding: 1rem;
      border-radius: 8px;
      margin-bottom: 1.5rem;
      display: flex;
      gap: 1rem;
      align-items: center;
    }
    .demo-container {
      transition: all 0.3s ease;
    }
    h1 { margin-bottom: 0.5rem; }
    .subtitle { color: #666; font-size: 1.1rem; }

    /* Layout toggles */
    .btn {
      padding: 0.5rem 1rem;
      border: 1px solid var(--citum-border);
      background: var(--citum-surface);
      border-radius: 4px;
      cursor: pointer;
    }
    .btn.active {
      background: var(--citum-blue);
      color: oklch(0.985 0.008 238);
      border-color: var(--citum-blue);
    }

    /* Demo notice */
    .demo-notice {
      background: var(--citum-blue-soft, #eef6fc);
      border-left: 3px solid var(--citum-blue, #2a94d6);
      border-radius: 4px;
      padding: 0.75rem 1rem;
      font-size: 0.9rem;
      color: var(--citum-muted, #555);
      margin-bottom: 2rem;
    }
    .demo-notice strong {
      color: var(--citum-ink, #222);
    }
    .feature-tags {
      margin-top: 0.5rem;
      display: flex;
      flex-wrap: wrap;
      gap: 0.4rem;
    }
    .feature-tag {
      background: white;
      border: 1px solid var(--citum-border, #ddd);
      border-radius: 12px;
      padding: 0.15rem 0.6rem;
      font-size: 0.8rem;
      color: var(--citum-muted, #555);
    }
  </style>
  <link rel="stylesheet" href="assets/citum-theme.css">
  <link rel="stylesheet" href="assets/citum-interactive.css">
</head>
<body class="bg-background-light text-slate-700">

  <!-- Navigation -->
  <!-- LAYOUT_NAV_START -->
  <!-- LAYOUT_NAV_END -->

  <div class="container-demo pt-24">
    <div class="flex justify-end mb-4">
      <a href="developer.html#interactive" class="config-link">
        <span class="material-icons" style="font-size: 1rem;">settings</span>
        View Configuration
      </a>
    </div>

    <header>
      <h1>The Infrastructure of Scholarly Memory</h1>
      <p class="subtitle">How citation practices construct scholarly knowledge across languages, times, and media.</p>
    </header>

    <div class="demo-notice">
      <strong>Illustration only.</strong> This is an entirely artificial document created to
      demonstrate Citum citation rendering. All arguments, references, and authors are
      fabricated. No scholarly claims are made.
      <div class="feature-tags">
        <span class="feature-tag">integral &amp; non-integral</span>
        <span class="feature-tag">name memory</span>
        <span class="feature-tag">multilingual (ja, es)</span>
        <span class="feature-tag">EDTF dates (2022~, 1891?)</span>
        <span class="feature-tag">archival + archive-info</span>
        <span class="feature-tag">preprint + eprint</span>
        <span class="feature-tag">primary / secondary bibliography</span>
      </div>
    </div>

    <div class="controls">
      <span>Layout:</span>
      <button class="btn active" id="btn-classic">Classic (Bottom)</button>
      <button class="btn" id="btn-sidebar">Modern Sidebar</button>
    </div>

    <div id="demo-root" class="demo-container">
{{BODY}}
    </div>

    <div class="demo-reproduce">
      <h3>Reproduce This Rendering</h3>
      <pre><code>citum render doc -s styles/embedded/apa-7th.yaml -b docs/demo-refs.yaml docs/demo.djot</code></pre>
    </div>

  </div>

  <!-- Footer -->
  <footer class="py-12 px-6 border-t border-slate-200 bg-white/50">
    <!-- LAYOUT_FOOTER_START -->
    <!-- LAYOUT_FOOTER_END -->
  </footer>

  <script src="assets/citum-interactive.js"></script>
  <script>
    const root = document.getElementById('demo-root');
    const btnClassic = document.getElementById('btn-classic');
    const btnSidebar = document.getElementById('btn-sidebar');

    btnClassic.addEventListener('click', () => {
      root.classList.remove('citum-with-sidebar');
      btnClassic.classList.add('active');
      btnSidebar.classList.remove('active');
    });

    btnSidebar.addEventListener('click', () => {
      root.classList.add('citum-with-sidebar');
      btnSidebar.classList.add('active');
      btnClassic.classList.remove('active');
    });
  </script>
</body>
</html>
`;

// ---------------------------------------------------------------------------
// 5. Build and write
// ---------------------------------------------------------------------------

function build() {
  console.log('Rendering docs/demo.djot via Citum engine…');
  const raw = renderBody();
  const fragment = extractFragment(raw);

  const indented = fragment.split('\n').map(l => l ? '      ' + l : '').join('\n');
  const html = TEMPLATE.replace('{{BODY}}', indented);

  const outPath = path.join(DOCS_DIR, 'demo.html');
  fs.writeFileSync(outPath, html);
  console.log('Written: docs/demo.html');
  console.log('Run `node scripts/build-layout.js` to fill nav/footer markers.');
}

build();
