#!/usr/bin/env node
/**
 * scripts/gen-biblatex-snapshot.js
 *
 * Generates biblatex-rendered bibliography snapshots for any biblatex style.
 * Runs pdflatex + biber to produce authoritative output, extracts the
 * formatted bibliography as plain text, and writes a JSON snapshot.
 *
 * Usage:
 *   node scripts/gen-biblatex-snapshot.js --style <biblatex-style>
 *   node scripts/gen-biblatex-snapshot.js --style chem-angew --bib refs.bib
 *   node scripts/gen-biblatex-snapshot.js --style apa --citum-style apa-7th
 *   node scripts/gen-biblatex-snapshot.js --style numeric-comp --force
 *
 * Options:
 *   --style <name>           biblatex style name (required)
 *   --citum-style <name>     Citum snapshot filename stem (default: same as --style)
 *   --bib <path>             Use this .bib file directly (default: auto-convert fixture)
 *   --fixture <path>         CSL JSON fixture to convert (default: references-expanded.json)
 *   --biblatex-opts <opts>   Extra biblatex package options, comma-separated
 *   --cite <id,id,...>       Only cite these keys (default: all keys in bib)
 *   --force                  Regenerate even if current
 *
 * Output: tests/snapshots/biblatex/<citum-style>.json
 *
 * Exit codes:
 *   0 — success (written or already current)
 *   1 — error
 */

'use strict';

const crypto = require('crypto');
const fs = require('fs');
const os = require('os');
const path = require('path');
const { spawnSync } = require('child_process');

const PROJECT_ROOT = path.resolve(__dirname, '..');
const SNAPSHOT_DIR = path.join(PROJECT_ROOT, 'tests', 'snapshots', 'biblatex');
const DEFAULT_FIXTURE = path.join(PROJECT_ROOT, 'tests', 'fixtures', 'references-expanded.json');

// ---------------------------------------------------------------------------
// Argument parsing
// ---------------------------------------------------------------------------

function parseArgs() {
  const args = process.argv.slice(2);
  const opts = {
    style: null,
    citumStyle: null,
    bib: null,
    fixture: DEFAULT_FIXTURE,
    biblatexOpts: '',
    cite: null,
    force: false,
  };

  for (let i = 0; i < args.length; i++) {
    const a = args[i];
    if (a === '--style')          opts.style        = args[++i];
    else if (a === '--citum-style')    opts.citumStyle   = args[++i];
    else if (a === '--bib')            opts.bib          = path.resolve(args[++i]);
    else if (a === '--fixture')        opts.fixture      = path.resolve(args[++i]);
    else if (a === '--biblatex-opts')  opts.biblatexOpts = args[++i];
    else if (a === '--cite')           opts.cite         = args[++i].split(',').map(s => s.trim());
    else if (a === '--force')          opts.force        = true;
  }

  if (!opts.citumStyle) opts.citumStyle = opts.style;
  return opts;
}

// ---------------------------------------------------------------------------
// CSL JSON → BibTeX conversion
// ---------------------------------------------------------------------------

/** Format a CSL name list into BibTeX "Family, Given and ..." notation. */
function formatBibNames(names) {
  if (!names || !names.length) return '';
  return names.map((n) => {
    if (n.literal) return `{${n.literal}}`;
    const given = n.given ? `, ${n.given}` : '';
    return `${n.family}${given}`;
  }).join(' and ');
}

/** Extract year from CSL issued field. */
function extractYear(issued) {
  if (!issued) return '';
  if (issued['date-parts'] && issued['date-parts'][0]) return String(issued['date-parts'][0][0]);
  if (issued.raw) return issued.raw.slice(0, 4);
  return '';
}

/** Extract YYYY-MM-DD urldate from accessed field. */
function extractUrldate(accessed) {
  if (!accessed || !accessed['date-parts'] || !accessed['date-parts'][0]) return '';
  const [y, m, d] = accessed['date-parts'][0];
  return [y, String(m || 1).padStart(2, '0'), String(d || 1).padStart(2, '0')].join('-');
}

const CSL_TO_BIBTEX_TYPE = {
  'article-journal':    'article',
  'article-newspaper':  'article',
  'article-magazine':   'article',
  'book':               'book',
  'chapter':            'incollection',
  'paper-conference':   'inproceedings',
  'thesis':             'thesis',
  'report':             'report',
  'webpage':            'online',
  'entry-encyclopedia': 'inreference',
  'dataset':            'dataset',
  'legal_case':         'jurisdiction',
  'patent':             'patent',
  'motion_picture':     'video',
  'broadcast':          'misc',
  'interview':          'misc',
  'personal_communication': 'misc',
};

/**
 * Convert a single CSL item to a BibTeX entry string.
 */
function cslItemToBibtex(item) {
  const type = CSL_TO_BIBTEX_TYPE[item.type] || 'misc';
  const fields = [];

  const push = (key, val) => { if (val) fields.push(`  ${key} = {${val}},`); };

  push('author',      formatBibNames(item.author));
  push('editor',      formatBibNames(item.editor));
  push('title',       item.title);
  push('year',        extractYear(item.issued));

  // Container / parent
  if (item.type === 'article-journal' || item.type === 'article-newspaper' || item.type === 'article-magazine') {
    push('journaltitle', item['container-title']);
  } else if (item.type === 'chapter' || item.type === 'paper-conference' || item.type === 'entry-encyclopedia') {
    push('booktitle', item['container-title']);
  } else {
    push('series', item['container-title']);
  }

  push('volume',      item.volume);
  push('number',      item.issue || item.number);
  push('pages',       item.page);
  push('publisher',   item.publisher);
  push('location',    item['publisher-place']);
  push('doi',         item.DOI);
  push('url',         item.URL);
  push('urldate',     extractUrldate(item.accessed));
  push('isbn',        item.ISBN);
  push('issn',        item.ISSN);

  // Type-specific extras
  if (item.type === 'thesis') push('type', item.genre || 'phdthesis');
  if (item.type === 'report') push('type', item.genre || 'techreport');
  if (item.type === 'patent') push('holder', formatBibNames(item.author));

  return `@${type}{${item.id},\n${fields.join('\n')}\n}`;
}

/** Convert a CSL JSON fixture file to a .bib string. */
function cslJsonToBibtex(fixturePath) {
  const data = JSON.parse(fs.readFileSync(fixturePath, 'utf8'));
  const entries = Object.values(data)
    .filter((v) => v && typeof v === 'object' && v.id)
    .map(cslItemToBibtex);
  return entries.join('\n\n') + '\n';
}

// ---------------------------------------------------------------------------
// LaTeX driver
// ---------------------------------------------------------------------------

function buildBiblatexOpts(style, extra) {
  const base = `style=${style},backend=biber,sorting=none`;
  return extra ? `${base},${extra}` : base;
}

/** Read all @entry keys from a .bib file. */
function bibKeys(bibPath) {
  const content = fs.readFileSync(bibPath, 'utf8');
  const keys = [];
  for (const m of content.matchAll(/^@\w+\{(\S+),/gm)) {
    if (m[1].toLowerCase() !== 'string' && m[1].toLowerCase() !== 'preamble') {
      keys.push(m[1]);
    }
  }
  return keys;
}

function generateLatexDriver(biblatexOpts, bibFilename, citeKeys) {
  const cites = citeKeys.map((k) => `\\cite{${k}}`).join('\n');
  return `\\documentclass[a4paper]{article}
\\usepackage[${biblatexOpts}]{biblatex}
\\addbibresource{${bibFilename}}
\\begin{document}
${cites}
\\printbibliography
\\end{document}
`;
}

// ---------------------------------------------------------------------------
// Bibliography text extraction
// ---------------------------------------------------------------------------

/**
 * Extract bibliography entries from pdftotext -layout output.
 *
 * With -layout, pdftotext preserves spatial layout:
 *   - Numbered entries ([1], 1.) start at column 0
 *   - Unnumbered entries (author-date) also start at column 0
 *   - Continuation lines of any entry are indented with leading spaces
 *
 * Handles both numbered ([1], 1.) and unnumbered (author-date) styles.
 */
function extractBibliography(rawText) {
  const lines = rawText.split('\n');

  // Locate bibliography section heading (trimmed match)
  let bibStart = -1;
  for (let i = 0; i < lines.length; i++) {
    if (/^(references?|bibliography)$/i.test(lines[i].trim())) {
      bibStart = i + 1;
      break;
    }
  }
  // Fallback: start after first blank line that follows the in-text citations
  if (bibStart === -1) {
    for (let i = 1; i < lines.length; i++) {
      if (!lines[i].trim() && lines[i - 1].trim()) { bibStart = i + 1; break; }
    }
  }
  if (bibStart === -1) return [];

  // Strip form-feed characters (pdftotext page breaks) — they appear at the
  // start of a line before the first entry on a new page.
  const bibLines = lines.slice(bibStart).map((l) => l.replace(/\f/g, ''));

  // Detect numbered style: allow up to 3 leading spaces (handles right-aligned
  // numbering where "[1]" may be indented one space to align with "[10]"+),
  // but not 4+ spaces (which are continuation lines in hanging-indent styles).
  const bracketNum = /^ {0,3}\[\d+\]/;
  const dotNum     = /^ {0,3}\d+\.\s/;
  const isNumbered = bibLines.some((l) => bracketNum.test(l) || dotNum.test(l));

  const entries = [];

  /** Join a continuation fragment onto current text, de-hyphenating if needed.
   *  Soft hyphen (-) at line end: remove and join without space ("Ency-" + "clopedia" → "Encyclopedia").
   *  En-dash/em-dash at line end: keep and join without space ("436–" + "444" → "436–444").
   */
  function append(current, fragment) {
    if (current.endsWith('-')) {
      return current.slice(0, -1) + fragment;
    }
    if (current.endsWith('\u2013') || current.endsWith('\u2014')) {
      return current + fragment;
    }
    return current + ' ' + fragment;
  }

  if (isNumbered) {
    // Numbered: each entry starts with [N] or N. (with at most 3 leading spaces
    // for right-aligned numbering). Continuation lines have 4+ spaces.
    //
    // pdftotext can collapse two short adjacent entries onto one line, e.g.:
    //   "[24] Bengio ... 2023. [25] Kafka ..."
    // Split on embedded [N] bracket markers only (unambiguous; avoids false splits
    // on "2023. " year fragments that also match \d+\.\s).
    function splitLine(t) {
      const parts = [];
      let last = 0;
      for (const m of t.matchAll(/\[\d+\]/g)) {
        if (m.index === 0) continue; // skip the leading marker
        parts.push(t.slice(last, m.index).trimEnd());
        last = m.index;
      }
      parts.push(t.slice(last));
      return parts.filter(Boolean);
    }

    let current = null;
    for (const line of bibLines) {
      const t = line.trim();
      if (!t || /^\d+$/.test(t)) continue; // blank or bare page number
      if (bracketNum.test(line) || dotNum.test(line)) {
        // Split in case multiple entries collapsed onto one line
        const parts = splitLine(t);
        for (const part of parts) {
          if (current !== null) entries.push(current);
          current = part;
        }
      } else if (current !== null) {
        // Continuation line — but it may contain an embedded [N] marker if
        // pdftotext placed a new entry on the same indented line (e.g. after URL wrap).
        const parts = splitLine(t);
        if (parts.length > 1) {
          // First part appends to current; remaining parts are new entries
          current = append(current, parts[0]);
          for (const part of parts.slice(1)) {
            entries.push(current);
            current = part;
          }
        } else {
          current = append(current, t);
        }
      }
    }
    if (current !== null) entries.push(current);
  } else {
    // Unnumbered: use hanging-indent detection.
    // With pdftotext -layout, entry-start lines have no leading whitespace;
    // continuation lines are indented. Blank lines and bare page numbers are skipped.
    let current = null;
    for (const line of bibLines) {
      const t = line.trim();
      if (!t || /^\d+$/.test(t)) {
        // Blank or bare page number — section may end; flush entry on blank
        if (!t && current !== null) {
          entries.push(current);
          current = null;
        }
        continue;
      }
      const isIndented = /^\s/.test(line);
      if (!isIndented) {
        // New entry starts at column 0
        if (current !== null) entries.push(current);
        current = t;
      } else if (current !== null) {
        // Continuation line — de-hyphenate if needed
        current = append(current, t);
      } else {
        // Indented line before any entry start — treat as a new entry
        current = t;
      }
    }
    if (current !== null) entries.push(current);
  }

  return entries
    .map((e) => e.replace(/\s+/g, ' ').trim())
    // Remove spaces inside URLs introduced by pdftotext line-wrapping,
    // e.g. "https://stateofjs. com/2023" → "https://stateofjs.com/2023".
    // Repeatedly join (url_prefix) + (space) + (lowercase/digit continuation)
    // until no more matches (handles multi-segment wrapping).
    .map((e) => {
      let s = e, prev;
      do { prev = s; s = s.replace(/(https?:\/\/\S+) ([a-z0-9/])/g, '$1$2'); } while (s !== prev);
      return s;
    })
    .filter(Boolean);
}

// ---------------------------------------------------------------------------
// Version probing
// ---------------------------------------------------------------------------

function probeVersion(cmd, args, pattern) {
  const r = spawnSync(cmd, args, { encoding: 'utf8', timeout: 5000 });
  const out = (r.stdout || '') + (r.stderr || '');
  const m = out.match(pattern);
  return m ? m[1] : 'unknown';
}

function getBiblatexVersion() {
  const styPath = spawnSync('kpsewhich', ['biblatex.sty'], { encoding: 'utf8' }).stdout.trim();
  if (!styPath) return 'unknown';
  try {
    const content = fs.readFileSync(styPath, 'utf8').slice(0, 4000);
    // Modern TeX Live: \def\abx@version{3.21}
    const m = content.match(/\\def\\abx@version\{([^}]+)\}/);
    if (m) return m[1];
    // Legacy: \ProvidesPackage{biblatex}[... 3.x ...]
    const m2 = content.match(/\\ProvidesPackage\{biblatex\}\[.*?(\d+\.\d+[a-z]?)/);
    return m2 ? m2[1] : 'unknown';
  } catch { return 'unknown'; }
}

// ---------------------------------------------------------------------------
// Hashing
// ---------------------------------------------------------------------------

function fileHash(p) {
  return crypto.createHash('sha256').update(fs.readFileSync(p)).digest('hex').slice(0, 16);
}

// ---------------------------------------------------------------------------
// Snapshot I/O
// ---------------------------------------------------------------------------

function snapshotPath(citumStyle) {
  return path.join(SNAPSHOT_DIR, `${citumStyle}.json`);
}

function isSnapshotCurrent(snapPath, sourceHash) {
  if (!fs.existsSync(snapPath)) return false;
  try {
    const s = JSON.parse(fs.readFileSync(snapPath, 'utf8'));
    return s.source_hash === sourceHash;
  } catch { return false; }
}

// ---------------------------------------------------------------------------
// Compilation pipeline
// ---------------------------------------------------------------------------

function run(cmd, args, cwd) {
  return spawnSync(cmd, args, {
    cwd,
    encoding: 'utf8',
    timeout: 90000,
    stdio: ['pipe', 'pipe', 'pipe'],
  });
}

function wantsRerun(logPath) {
  try {
    return fs.readFileSync(logPath, 'utf8').includes('Please rerun LaTeX');
  } catch { return false; }
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

function generate(opts) {
  if (!opts.style) {
    process.stderr.write('Error: --style is required\n');
    return 1;
  }

  // Determine source .bib
  let bibPath;
  let sourceHash;

  if (opts.bib) {
    bibPath = opts.bib;
    sourceHash = fileHash(bibPath);
  } else {
    // Convert CSL JSON fixture → temp .bib
    const tmpBib = path.join(os.tmpdir(), `citum-biblatex-${opts.citumStyle}-${Date.now()}.bib`);
    const bibContent = cslJsonToBibtex(opts.fixture);
    fs.writeFileSync(tmpBib, bibContent, 'utf8');
    bibPath = tmpBib;
    sourceHash = fileHash(opts.fixture);
  }

  const snapPath = snapshotPath(opts.citumStyle);
  if (!opts.force && isSnapshotCurrent(snapPath, sourceHash)) {
    process.stderr.write(`— Already current: ${path.relative(PROJECT_ROOT, snapPath)}\n`);
    return 0;
  }

  const workDir = fs.mkdtempSync(path.join(os.tmpdir(), `citum-biblatex-`));

  try {
    // Copy .bib into workspace
    const bibFilename = 'refs.bib';
    fs.copyFileSync(bibPath, path.join(workDir, bibFilename));

    // Determine keys to cite
    const citeKeys = opts.cite ?? bibKeys(bibPath);
    if (!citeKeys.length) {
      process.stderr.write('Error: no keys found in .bib file\n');
      return 1;
    }

    // Write driver
    const biblatexOpts = buildBiblatexOpts(opts.style, opts.biblatexOpts);
    const tex = generateLatexDriver(biblatexOpts, bibFilename, citeKeys);
    fs.writeFileSync(path.join(workDir, 'main.tex'), tex, 'utf8');

    // Pass 1
    const r1 = run('pdflatex', ['-interaction=nonstopmode', 'main.tex'], workDir);
    if (!fs.existsSync(path.join(workDir, 'main.aux'))) {
      process.stderr.write(`pdflatex pass 1 failed:\n${r1.stderr || r1.stdout}\n`);
      return 1;
    }

    // biber
    const rb = run('biber', ['main'], workDir);
    if (rb.status !== 0 && !fs.existsSync(path.join(workDir, 'main.bbl'))) {
      process.stderr.write(`biber failed:\n${rb.stderr || rb.stdout}\n`);
      return 1;
    }

    // Pass 2
    run('pdflatex', ['-interaction=nonstopmode', 'main.tex'], workDir);

    // Pass 3 if requested
    if (wantsRerun(path.join(workDir, 'main.log'))) {
      run('pdflatex', ['-interaction=nonstopmode', 'main.tex'], workDir);
    }

    const pdfPath = path.join(workDir, 'main.pdf');
    if (!fs.existsSync(pdfPath)) {
      process.stderr.write('Error: no PDF produced\n');
      return 1;
    }

    // Extract text — use -layout to preserve hanging-indent structure
    const txtPath = path.join(workDir, 'main.txt');
    const rp = run('pdftotext', ['-layout', pdfPath, txtPath], workDir);
    if (rp.status !== 0) {
      process.stderr.write(`pdftotext failed: ${rp.stderr}\n`);
      return 1;
    }

    const rawText = fs.readFileSync(txtPath, 'utf8');
    const bibliography = extractBibliography(rawText);

    if (!bibliography.length) {
      // Save debug artefacts
      const debugDir = path.join(PROJECT_ROOT, 'tests', 'snapshots', 'biblatex');
      fs.mkdirSync(debugDir, { recursive: true });
      fs.writeFileSync(path.join(debugDir, `${opts.citumStyle}.debug.txt`), rawText, 'utf8');
      process.stderr.write(
        `Error: no bibliography entries extracted.\n` +
        `Debug text saved to tests/snapshots/biblatex/${opts.citumStyle}.debug.txt\n`
      );
      return 1;
    }

    // Write snapshot
    fs.mkdirSync(path.dirname(snapPath), { recursive: true });
    const snapshot = {
      version: 1,
      generated_by: `biblatex@${getBiblatexVersion()}+biber@${probeVersion('biber', ['--version'], /biber version:\s*(\S+)/i)}`,
      generated_at: new Date().toISOString(),
      biblatex_style: opts.style,
      citum_style: opts.citumStyle,
      source_hash: sourceHash,
      bibliography,
    };
    fs.writeFileSync(snapPath, JSON.stringify(snapshot, null, 2) + '\n', 'utf8');
    process.stderr.write(`✓ Written: ${path.relative(PROJECT_ROOT, snapPath)} (${bibliography.length} entries)\n`);
    return 0;

  } finally {
    fs.rmSync(workDir, { recursive: true, force: true });
    // Clean up temp .bib if we created one
    if (!opts.bib && fs.existsSync(bibPath)) fs.unlinkSync(bibPath);
  }
}

process.exit(generate(parseArgs()));
