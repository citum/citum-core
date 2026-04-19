#!/usr/bin/env node
/**
 * Alias Candidate Discovery via CSL Behavioral Fingerprinting
 *
 * Discovers CSL styles behaviorally identical to registry builtins
 * by rendering the 12-scenario fixture through citeproc-js and comparing outputs.
 *
 * Usage:
 *   node scripts/find-alias-candidates.js
 *   node scripts/find-alias-candidates.js --concurrency 4
 *   node scripts/find-alias-candidates.js --threshold 0.90
 *   node scripts/find-alias-candidates.js --limit 100
 *   node scripts/find-alias-candidates.js --out /tmp/aliases.tsv
 *
 * Output:
 *   TSV file (stdout and --out PATH) with columns:
 *   candidate_id  best_target  similarity  citation_match  bib_match
 *
 *   Filtered to similarity >= --threshold (default 0.85).
 *   Sorted by similarity descending.
 */

'use strict';

const CSL = require('citeproc');
const fs = require('fs');
const path = require('path');
const yaml = require('js-yaml');
const { spawn } = require('child_process');
const {
  normalizeText,
  textSimilarity,
  loadLocale,
} = require('./oracle-utils');
const { toCiteprocItem } = require('./lib/citeproc-locators');

const WORKSPACE_ROOT = path.resolve(__dirname, '..');
const DEFAULT_REFS_FIXTURE = path.join(WORKSPACE_ROOT, 'tests', 'fixtures', 'references-expanded.json');
const DEFAULT_CITATIONS_FIXTURE = path.join(WORKSPACE_ROOT, 'tests', 'fixtures', 'citations-expanded.json');
const DEFAULT_REGISTRY = path.join(WORKSPACE_ROOT, 'registry', 'default.yaml');
const DEFAULT_STYLES_DIR = path.join(WORKSPACE_ROOT, 'styles-legacy');
const DEFAULT_OUT = path.join(WORKSPACE_ROOT, 'scripts', 'report-data', `alias-candidates-${todayDate()}.tsv`);

function todayDate() {
  const now = new Date();
  return now.toISOString().split('T')[0];
}

function parseArgs() {
  const args = process.argv.slice(2);
  const options = {
    concurrency: 8,
    threshold: 0.85,
    limit: null,
    out: DEFAULT_OUT,
    refsFixture: DEFAULT_REFS_FIXTURE,
    citationsFixture: DEFAULT_CITATIONS_FIXTURE,
    registryPath: DEFAULT_REGISTRY,
    stylesDir: DEFAULT_STYLES_DIR,
    confirmWeb: false,
    updateMetadata: false,
  };

  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    if (arg === '--concurrency') {
      options.concurrency = parseInt(args[++i], 10);
    } else if (arg === '--threshold') {
      options.threshold = parseFloat(args[++i]);
    } else if (arg === '--limit') {
      options.limit = parseInt(args[++i], 10);
    } else if (arg === '--confirm-web') {
      options.confirmWeb = true;
    } else if (arg === '--update-metadata') {
      options.updateMetadata = true;
    } else if (arg === '--out') {
      options.out = args[++i];
    } else if (arg === '--refs-fixture') {
      options.refsFixture = path.resolve(args[++i]);
    } else if (arg === '--citations-fixture') {
      options.citationsFixture = path.resolve(args[++i]);
    } else if (arg === '--registry') {
      options.registryPath = path.resolve(args[++i]);
    } else if (arg === '--styles-dir') {
      options.stylesDir = path.resolve(args[++i]);
    }
  }

  return options;
}

function normalizeFixtureItems(fixturesData) {
  if (Array.isArray(fixturesData)) {
    return Object.fromEntries(fixturesData.map((item) => [item.id, item]));
  }

  if (fixturesData && Array.isArray(fixturesData.items)) {
    return Object.fromEntries(fixturesData.items.map((item) => [item.id, item]));
  }

  if (fixturesData && Array.isArray(fixturesData.references)) {
    return Object.fromEntries(fixturesData.references.map((item) => [item.id, item]));
  }

  return Object.fromEntries(
    Object.entries(fixturesData).filter(([key, value]) => key !== 'comment' && value && typeof value === 'object')
  );
}

// Extra scenarios defined inline — NOT in the shared fixture — to avoid invalidating
// oracle snapshots. The `clusters` format (for subsequent-cite) is also not understood
// by the Citum engine's JSON deserializer.
const EXTRA_SCENARIOS = [
  {
    id: 'subsequent-same-item',
    clusters: [
      { id: 'subsequent-same-item-first', items: [{ id: 'ITEM-1' }] },
      { id: 'subsequent-same-item-second', items: [{ id: 'ITEM-1' }] },
    ],
  },
  {
    id: 'archive-single',
    items: [{ id: 'ITEM-34' }],
  },
];

function loadFixtures(refsFixture, citationsFixture) {
  const fixturesData = JSON.parse(fs.readFileSync(refsFixture, 'utf8'));
  const testItems = normalizeFixtureItems(fixturesData);
  const testCitations = [
    ...(citationsFixture ? JSON.parse(fs.readFileSync(citationsFixture, 'utf8')) : []),
    ...EXTRA_SCENARIOS,
  ];
  return { testItems, testCitations };
}

/**
 * Render citations and bibliography using citeproc-js.
 * Supports both simple scenarios and clustered scenarios with position tracking.
 */
function renderWithCiteprocJs(stylePath, testItems, testCitations) {
  let cslXml;

  if (!fs.existsSync(stylePath)) {
    return null;
  }

  cslXml = fs.readFileSync(stylePath, 'utf8');

  const sys = {
    retrieveLocale: (lang) => loadLocale(lang),
    retrieveItem: (id) => testItems[id],
  };

  try {
    const citeproc = new CSL.Engine(sys, cslXml);
    citeproc.updateItems(Object.keys(testItems));

    const citations = [];
    let citationOrder = 0;

    for (const cite of testCitations) {
      // Handle clustered scenarios (e.g., subsequent-same-item)
      if (cite.clusters && Array.isArray(cite.clusters)) {
        const clusterResults = [];
        const citationsPre = [];

        for (const cluster of cite.clusters) {
          const suppressAuthor = cluster['suppress-author'] === true;
          const citeprocItems = cluster.items.map((item) => toCiteprocItem(item, suppressAuthor));

          try {
            const citObj = {
              citationID: cluster.id || `cluster-${citationOrder}`,
              citationItems: citeprocItems,
              properties: { noteIndex: citationOrder },
            };

            const result = citeproc.processCitationCluster(citObj, citationsPre, []);
            if (result && result[1] && result[1].length > 0) {
              clusterResults.push(result[1][result[1].length - 1][1]);
              citationsPre.push([citObj.citationID, citationOrder]);
            } else {
              clusterResults.push('');
            }
            citationOrder++;
          } catch (e) {
            // Fallback to makeCitationCluster
            try {
              const citeprocItems = cluster.items.map((item) => toCiteprocItem(item, cluster['suppress-author'] === true));
              const result = citeproc.makeCitationCluster(citeprocItems);
              clusterResults.push(result);
            } catch {
              clusterResults.push('');
            }
          }
        }

        // Join cluster results with " | " separator
        citations.push(clusterResults.join(' | '));
      } else {
        // Handle simple scenarios (original behavior)
        const suppressAuthor = cite['suppress-author'] === true;
        const citeprocItems = cite.items.map((item) => toCiteprocItem(item, suppressAuthor));

        try {
          const citObj = {
            citationID: cite.id || `citation-${citationOrder}`,
            citationItems: citeprocItems,
            properties: { noteIndex: citationOrder },
          };

          const result = citeproc.processCitationCluster(citObj, [], []);
          if (result && result[1] && result[1].length > 0) {
            citations.push(result[1][result[1].length - 1][1]);
          } else {
            citations.push('');
          }
          citationOrder++;
        } catch (e) {
          // Fallback to makeCitationCluster
          try {
            const result = citeproc.makeCitationCluster(citeprocItems);
            citations.push(result);
          } catch {
            citations.push('');
          }
        }
      }
    }

    const bibResult = citeproc.makeBibliography();
    const bibliography = bibResult ? bibResult[1] : [];

    return { citations, bibliography };
  } catch (e) {
    return null;
  }
}

/**
 * Compute fingerprint: normalized citation and bibliography strings.
 */
function makeFingerprint(rendered) {
  if (!rendered) return null;
  return {
    citations: rendered.citations.map(c => normalizeText(String(c))),
    bibliography: rendered.bibliography.map(b => normalizeText(String(b))),
  };
}

/**
 * Compute similarity between two fingerprints.
 * Returns { similarity, citation_match, bib_match }.
 *
 * citation_match and bib_match use EXACT string equality (after normalizeText).
 * similarity uses bag-of-words textSimilarity for finer-grained matching.
 */
function scoreFingerprints(fp1, fp2) {
  if (!fp1 || !fp2) return { similarity: 0, citation_match: 0, bib_match: 0 };

  const citationSimilarities = [];
  const citationMatches = [];

  for (let i = 0; i < Math.max(fp1.citations.length, fp2.citations.length); i++) {
    const c1 = fp1.citations[i] || '';
    const c2 = fp2.citations[i] || '';
    citationSimilarities.push(textSimilarity(c1, c2));
    // Exact match: strings must be identical after normalizeText
    citationMatches.push(c1 === c2 ? 1 : 0);
  }

  const bibSimilarities = [];
  const bibMatches = [];

  for (let i = 0; i < Math.max(fp1.bibliography.length, fp2.bibliography.length); i++) {
    const b1 = fp1.bibliography[i] || '';
    const b2 = fp2.bibliography[i] || '';
    bibSimilarities.push(textSimilarity(b1, b2));
    // Exact match: strings must be identical after normalizeText
    bibMatches.push(b1 === b2 ? 1 : 0);
  }

  // Mean similarity across all strings (bag-of-words)
  const allSims = [...citationSimilarities, ...bibSimilarities];
  const similarity = allSims.length > 0
    ? allSims.reduce((a, b) => a + b, 0) / allSims.length
    : 0;

  // Exact match rates (string equality)
  const citation_match = citationMatches.length > 0
    ? citationMatches.reduce((a, b) => a + b, 0) / citationMatches.length
    : 0;

  const bib_match = bibMatches.length > 0
    ? bibMatches.reduce((a, b) => a + b, 0) / bibMatches.length
    : 0;

  return { similarity, citation_match, bib_match };
}

/**
 * Check if CSL has independent-parent link.
 */
function isIndependentStyle(stylePath) {
  try {
    const content = fs.readFileSync(stylePath, 'utf8');
    return !/rel="independent-parent"/.test(content);
  } catch {
    return false;
  }
}

/**
 * Load registry and extract builtins.
 */
function loadRegistry(registryPath) {
  const data = yaml.load(fs.readFileSync(registryPath, 'utf8'));
  const builtins = [];
  const knownAliases = new Set();

  if (data.styles && Array.isArray(data.styles)) {
    for (const style of data.styles) {
      builtins.push(style.id);
      if (Array.isArray(style.aliases)) {
        for (const alias of style.aliases) {
          knownAliases.add(alias);
        }
      }
    }
  }

  return { builtins, knownAliases };
}

/**
 * Enumerate independent styles in stylesDir, excluding builtins and aliases.
 */
function enumerateCandidates(stylesDir, builtins, knownAliases) {
  const exclude = new Set([...builtins, ...knownAliases]);
  const candidates = [];

  const files = fs.readdirSync(stylesDir);
  for (const file of files) {
    if (!file.endsWith('.csl')) continue;
    const id = file.replace(/\.csl$/, '');
    if (exclude.has(id)) continue;

    const stylePath = path.join(stylesDir, file);
    if (!isIndependentStyle(stylePath)) continue;

    candidates.push({ id, path: stylePath });
  }

  return candidates;
}

/**
 * Extract documentation links from CSL XML content.
 */
function extractCslMetadata(content) {
  const docLinks = [];
  const docRegex = /<link\s+[^>]*href=["']([^"']+)["']\s+rel=["']documentation["']/g;
  let match;
  while ((match = docRegex.exec(content)) !== null) {
    docLinks.push(match[1]);
  }
  // Also check reversed order of attributes
  const docRegexRev = /<link\s+[^>]*rel=["']documentation["']\s+href=["']([^"']+)["']/g;
  while ((match = docRegexRev.exec(content)) !== null) {
    docLinks.push(match[1]);
  }
  return {
    url: docLinks[0] || '',
    note: docLinks.length > 0 ? 'found in CSL metadata' : '',
  };
}

/**
 * Process a batch of candidates concurrently.
 */
async function processBatch(candidates, targetFingerprints, testItems, testCitations, onProgress) {
  const results = [];

  for (const candidate of candidates) {
    try {
      const content = fs.readFileSync(candidate.path, 'utf8');
      const rendered = renderWithCiteprocJs(candidate.path, testItems, testCitations);
      if (!rendered) {
        continue; // Skip failed renders
      }

      const fp = makeFingerprint(rendered);
      if (!fp) continue;

      let bestTarget = null;
      let bestScore = { similarity: 0, citation_match: 0, bib_match: 0 };

      for (const [targetId, targetFp] of Object.entries(targetFingerprints)) {
        const score = scoreFingerprints(fp, targetFp);
        if (score.similarity > bestScore.similarity) {
          bestTarget = targetId;
          bestScore = score;
        }
      }

      if (bestTarget) {
        // Extract documentation links from CSL
        const meta = extractCslMetadata(content);

        results.push({
          candidate_id: candidate.id,
          best_target: bestTarget,
          similarity: bestScore.similarity,
          citation_match: bestScore.citation_match,
          bib_match: bestScore.bib_match,
          evidence_url: meta.url,
          confidence_note: meta.note,
        });
      }
    } catch (e) {
      // Skip candidate on error
    }
    onProgress();
  }

  return results;
}

/**
 * Run parallel batches.
 */
async function runParallel(candidates, concurrency, targetFingerprints, testItems, testCitations) {
  const results = [];
  let processed = 0;

  const progressInterval = setInterval(() => {
    if (processed % 100 === 0 && processed > 0) {
      console.error(`Progress: ${processed}/${candidates.length} candidates processed`);
    }
  }, 500);

  const batches = [];
  for (let i = 0; i < candidates.length; i += concurrency) {
    const batch = candidates.slice(i, i + concurrency);
    batches.push(
      processBatch(batch, targetFingerprints, testItems, testCitations, () => {
        processed++;
      })
    );
  }

  const batchResults = await Promise.all(batches);
  clearInterval(progressInterval);

  for (const batchResult of batchResults) {
    results.push(...batchResult);
  }

  return results;
}

/**
 * Web confirmation for candidates using Perplexity or DuckDuckGo.
 */
async function confirmWebCandidates(results, threshold) {
  const filtered = results.filter(r => r.similarity >= threshold);
  console.error(`\nRunning web confirmation for ${filtered.length} candidates...`);

  for (let i = 0; i < filtered.length; i++) {
    const row = filtered[i];
    
    // Skip if we already have evidence from CSL metadata
    if (row.evidence_url) {
      console.error(`[${i + 1}/${filtered.length}] Skipping: ${row.candidate_id} (already has metadata evidence)`);
      continue;
    }

    const query = `"${row.candidate_id.replace(/-/g, ' ')}" citation style author guidelines`;
    
    console.error(`[${i + 1}/${filtered.length}] Checking: ${row.candidate_id}...`);
    
    let confirmation = null;
    if (process.env.PERPLEXITY_API_KEY) {
      confirmation = await checkPerplexity(query, row.best_target);
    }
    
    if (!confirmation) {
      // Fallback to DuckDuckGo
      confirmation = await checkDuckDuckGo(query, row.best_target);
      // Rate limiting for DDG
      await new Promise(resolve => setTimeout(resolve, 1000));
    }

    row.evidence_url = confirmation ? confirmation.url : '';
    row.confidence_note = confirmation ? confirmation.note : 'no web evidence found';
  }
}

async function checkPerplexity(query, targetId) {
  try {
    const response = await fetch('https://api.perplexity.ai/chat/completions', {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${process.env.PERPLEXITY_API_KEY}`,
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        model: 'llama-3.1-sonar-small-128k-online',
        messages: [
          {
            role: 'system',
            content: 'You are a research assistant confirming CSL citation style aliases. Return JSON: { "url": "...", "note": "..." }'
          },
          {
            role: 'user',
            content: `Does the journal for "${query}" explicitly state they use the "${targetId}" citation style? Provide the best URL as evidence and a brief note on confidence.`
          }
        ],
        response_format: { type: 'json_object' }
      })
    });

    if (!response.ok) return null;
    const data = await response.json();
    return JSON.parse(data.choices[0].message.content);
  } catch (e) {
    return null;
  }
}

async function checkDuckDuckGo(query, targetId) {
  try {
    const response = await fetch('https://html.duckduckgo.com/html/', {
      method: 'POST',
      headers: { 
        'Content-Type': 'application/x-www-form-urlencoded',
        'User-Agent': 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36'
      },
      body: `q=${encodeURIComponent(query)}`
    });

    if (!response.ok) return null;
    const html = await response.text();

    // Regex for html.duckduckgo.com results
    const urlMatch = html.match(/class=['"]result__a['"] href=['"](http[^'"]+)['"]/);
    const snippetMatch = html.match(/class=['"]result__snippet['"]>([^<]+)/);

    if (urlMatch) {
      const url = urlMatch[1];
      const snippet = snippetMatch ? snippetMatch[1].toLowerCase() : '';
      const targetLower = targetId.toLowerCase().replace(/-/g, ' ');
      
      let note = 'found via DDG';
      if (snippet.includes(targetLower)) {
        note = `high confidence: snippet mentions "${targetId}"`;
      } else if (snippet.includes('citation') || snippet.includes('style')) {
        note = 'medium confidence: snippet mentions citation/style';
      }

      return { url, note };
    }
  } catch (e) {
    // Ignore fetch errors
  }
  return null;
}

/**
 * Write results as TSV.
 */
function writeTSV(results, filePath, threshold) {
  // Filter by threshold
  const filtered = results.filter(r => r.similarity >= threshold);
  // Sort by similarity descending
  filtered.sort((a, b) => b.similarity - a.similarity);

  const header = [
    'candidate_id', 
    'best_target', 
    'similarity', 
    'citation_match', 
    'bib_match', 
    'evidence_url', 
    'confidence_note'
  ];
  const lines = [header.join('\t')];

  for (const row of filtered) {
    lines.push([
      row.candidate_id,
      row.best_target,
      row.similarity.toFixed(4),
      row.citation_match.toFixed(4),
      row.bib_match.toFixed(4),
      row.evidence_url || '',
      row.confidence_note || '',
    ].join('\t'));
  }

  const content = lines.join('\n') + '\n';
  fs.writeFileSync(filePath, content);
  return filtered;
}

async function handleUpdateMetadata(options) {
  if (!fs.existsSync(options.out)) {
    throw new Error(`File not found for --update-metadata: ${options.out}`);
  }

  console.error(`Updating metadata for: ${options.out}...`);
  const content = fs.readFileSync(options.out, 'utf8');
  const lines = content.split('\n').filter(l => l.trim().length > 0);
  const header = lines[0].split('\t');
  
  const results = [];
  for (let i = 1; i < lines.length; i++) {
    const cols = lines[i].split('\t');
    const row = Object.fromEntries(header.map((h, j) => [h, cols[j]]));
    
    // Convert strings to floats for similarity
    row.similarity = parseFloat(row.similarity);
    row.citation_match = parseFloat(row.citation_match);
    row.bib_match = parseFloat(row.bib_match);

    const stylePath = path.join(options.stylesDir, `${row.candidate_id}.csl`);
    if (fs.existsSync(stylePath)) {
      const cslContent = fs.readFileSync(stylePath, 'utf8');
      const meta = extractCslMetadata(cslContent);
      row.evidence_url = meta.url;
      row.confidence_note = meta.note;
    }
    results.push(row);
  }

  if (options.confirmWeb) {
    await confirmWebCandidates(results, options.threshold);
  }

  writeTSV(results, options.out, options.threshold);
  return results;
}

async function main() {
  const options = parseArgs();

  try {
    if (options.updateMetadata) {
      const filtered = await handleUpdateMetadata(options);
      
      const header = ['candidate_id', 'best_target', 'similarity', 'citation_match', 'bib_match', 'evidence_url', 'confidence_note'];
      console.log(header.join('\t'));
      for (const row of filtered) {
        console.log([
          row.candidate_id,
          row.best_target,
          row.similarity.toFixed(4),
          row.citation_match.toFixed(4),
          row.bib_match.toFixed(4),
          row.evidence_url || '',
          row.confidence_note || '',
        ].join('\t'));
      }
      return;
    }

    // Load fixtures
    const { testItems, testCitations } = loadFixtures(options.refsFixture, options.citationsFixture);

    // Load registry
    const { builtins, knownAliases } = loadRegistry(options.registryPath);

    // Pre-render target fingerprints
    console.error('Pre-rendering target styles...');
    const targetFingerprints = {};
    let failedTargets = 0;

    for (const targetId of builtins) {
      const targetPath = path.join(options.stylesDir, `${targetId}.csl`);
      const rendered = renderWithCiteprocJs(targetPath, testItems, testCitations);
      if (rendered) {
        const fp = makeFingerprint(rendered);
        if (fp) {
          targetFingerprints[targetId] = fp;
        }
      } else {
        failedTargets++;
      }
    }

    if (failedTargets > 0) {
      console.error(`Warning: ${failedTargets} target styles failed to render`);
    }

    console.error(`Target styles ready: ${Object.keys(targetFingerprints).length}/${builtins.length}`);

    // Enumerate candidates
    const allCandidates = enumerateCandidates(options.stylesDir, builtins, knownAliases);
    const candidates = options.limit
      ? allCandidates.slice(0, options.limit)
      : allCandidates;

    console.error(`Found ${candidates.length} independent candidates to evaluate`);

    // Process candidates in parallel
    const results = await runParallel(candidates, options.concurrency, targetFingerprints, testItems, testCitations);

    console.error(`Scored ${results.length} candidates`);

    // Web confirmation if requested
    if (options.confirmWeb) {
      await confirmWebCandidates(results, options.threshold);
    }

    // Write TSV
    const filtered = writeTSV(results, options.out, options.threshold);

    // Output to stdout
    const header = [
      'candidate_id', 
      'best_target', 
      'similarity', 
      'citation_match', 
      'bib_match', 
      'evidence_url', 
      'confidence_note'
    ];
    console.log(header.join('\t'));
    for (const row of filtered) {
      console.log([
        row.candidate_id,
        row.best_target,
        row.similarity.toFixed(4),
        row.citation_match.toFixed(4),
        row.bib_match.toFixed(4),
        row.evidence_url || '',
        row.confidence_note || '',
      ].join('\t'));
    }

    console.error(`\nResults written to: ${options.out}`);
    console.error(`Total candidates meeting threshold (>= ${options.threshold}): ${filtered.length}`);
  } catch (e) {
    console.error('Fatal error:', e.message);
    process.exit(2);
  }
}

main().catch((e) => {
  console.error('Unhandled error:', e);
  process.exit(2);
});
