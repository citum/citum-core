#!/usr/bin/env node

const fs = require('node:fs');
const path = require('node:path');
const yaml = require('js-yaml');

const PROJECT_ROOT = path.resolve(__dirname, '..');
const DEFAULT_STYLES_DIR = path.join(PROJECT_ROOT, 'styles');
const RENDERING_KEYS = new Set([
  'text-case',
  'emph',
  'quote',
  'strong',
  'small-caps',
  'vertical-align',
  'prefix',
  'suffix',
  'wrap',
  'suppress',
  'initialize-with',
  'name-form',
  'strip-periods',
]);
const SELECTOR_KEYS = ['contributor', 'date', 'title', 'number', 'variable', 'term', 'group'];

function parseArgs(argv = process.argv.slice(2)) {
  const args = {
    paths: [],
    write: false,
    json: false,
  };

  for (const arg of argv) {
    if (arg === '--write') {
      args.write = true;
    } else if (arg === '--json') {
      args.json = true;
    } else if (arg === '-h' || arg === '--help') {
      printUsage();
      process.exit(0);
    } else {
      args.paths.push(arg);
    }
  }

  return args;
}

function printUsage() {
  console.log('Usage: node scripts/convert-template-v3.js [--write] [--json] [styles/file.yaml ...]');
}

function listStyleFiles(inputPaths) {
  if (inputPaths.length === 0) {
    return walkYaml(DEFAULT_STYLES_DIR);
  }

  return inputPaths
    .map((inputPath) => path.resolve(inputPath))
    .flatMap((inputPath) => {
      if (fs.statSync(inputPath).isDirectory()) {
        return walkYaml(inputPath);
      }
      return inputPath.endsWith('.yaml') ? [inputPath] : [];
    })
    .sort();
}

function walkYaml(dirPath) {
  return fs.readdirSync(dirPath, { withFileTypes: true })
    .flatMap((entry) => {
      const filePath = path.join(dirPath, entry.name);
      if (entry.isDirectory()) {
        return walkYaml(filePath);
      }
      return entry.isFile() && entry.name.endsWith('.yaml') ? [filePath] : [];
    })
    .sort();
}

function cloneJson(value) {
  return value == null ? value : JSON.parse(JSON.stringify(value));
}

function componentSelector(component) {
  if (!component || typeof component !== 'object' || Array.isArray(component)) {
    return null;
  }

  for (const key of SELECTOR_KEYS) {
    if (Object.prototype.hasOwnProperty.call(component, key)) {
      return { [key]: component[key] };
    }
  }
  return null;
}

function componentKey(component) {
  const selector = componentSelector(component);
  return selector ? JSON.stringify(selector) : null;
}

function lcsPairs(left, right) {
  const lengths = Array.from({ length: left.length + 1 }, () => Array(right.length + 1).fill(0));

  for (let i = left.length - 1; i >= 0; i -= 1) {
    for (let j = right.length - 1; j >= 0; j -= 1) {
      lengths[i][j] = left[i] === right[j]
        ? lengths[i + 1][j + 1] + 1
        : Math.max(lengths[i + 1][j], lengths[i][j + 1]);
    }
  }

  const pairs = [];
  let i = 0;
  let j = 0;
  while (i < left.length && j < right.length) {
    if (left[i] === right[j]) {
      pairs.push([i, j]);
      i += 1;
      j += 1;
    } else if (lengths[i + 1][j] >= lengths[i][j + 1]) {
      i += 1;
    } else {
      j += 1;
    }
  }
  return pairs;
}

function stripRendering(component) {
  const stripped = { ...component };
  for (const key of RENDERING_KEYS) {
    delete stripped[key];
  }
  return stripped;
}

function isRenderingOnlyChange(base, target) {
  return JSON.stringify(stripRendering(base)) === JSON.stringify(stripRendering(target));
}

function selectorValueMatches(expected, actual) {
  if (Array.isArray(expected)) {
    return Array.isArray(actual)
      && expected.length === actual.length
      && expected.every((item, index) => selectorValueMatches(item, actual[index]));
  }
  if (expected && typeof expected === 'object') {
    return actual && typeof actual === 'object' && !Array.isArray(actual)
      && Object.entries(expected).every(([key, value]) => selectorValueMatches(value, actual[key]));
  }
  return JSON.stringify(actual) === JSON.stringify(expected);
}

function componentMatchesSelector(component, selector) {
  return Object.entries(selector).every(([key, expected]) => (
    selectorValueMatches(expected, component?.[key])
  ));
}

function findTemplateIndex(template, selector) {
  const matches = [];
  template.forEach((component, index) => {
    if (componentMatchesSelector(component, selector)) {
      matches.push(index);
    }
  });
  return matches.length === 1 ? matches[0] : -1;
}

function applyDiff(baseTemplate, diff) {
  const template = cloneJson(baseTemplate);

  for (const operation of diff.modify || []) {
    const index = findTemplateIndex(template, operation.match);
    if (index < 0) return null;
    const rendering = { ...operation };
    delete rendering.match;
    template[index] = { ...template[index], ...rendering };
  }

  for (const operation of diff.remove || []) {
    const index = findTemplateIndex(template, operation.match);
    if (index < 0) return null;
    template.splice(index, 1);
  }

  for (const operation of diff.add || []) {
    const anchor = operation.before || operation.after;
    const index = findTemplateIndex(template, anchor);
    if (index < 0) return null;
    template.splice(operation.before ? index : index + 1, 0, cloneJson(operation.component));
  }

  return template;
}

function deriveTemplateVariantDiff(baseTemplate, targetTemplate) {
  if (!Array.isArray(baseTemplate) || !Array.isArray(targetTemplate) || baseTemplate.length === 0) {
    return null;
  }

  const baseKeys = baseTemplate.map(componentKey);
  const targetKeys = targetTemplate.map(componentKey);
  if (baseKeys.some((key) => !key) || targetKeys.some((key) => !key)) {
    return null;
  }

  const pairs = lcsPairs(baseKeys, targetKeys);
  const diff = {};
  const modify = [];
  const remove = [];
  const add = [];

  for (let baseIndex = 0; baseIndex < baseTemplate.length; baseIndex += 1) {
    if (!pairs.some(([pairedBase]) => pairedBase === baseIndex)) {
      remove.push({ match: componentSelector(baseTemplate[baseIndex]) });
    }
  }

  for (const [baseIndex, targetIndex] of pairs) {
    const baseComponent = baseTemplate[baseIndex];
    const targetComponent = targetTemplate[targetIndex];
    if (JSON.stringify(baseComponent) === JSON.stringify(targetComponent)) continue;
    if (!isRenderingOnlyChange(baseComponent, targetComponent)) return null;

    const operation = { match: componentSelector(baseComponent) };
    for (const key of RENDERING_KEYS) {
      if (Object.prototype.hasOwnProperty.call(targetComponent, key)) {
        operation[key] = targetComponent[key];
      }
    }
    modify.push(operation);
  }

  let lastAnchor = null;
  for (let targetIndex = 0; targetIndex < targetTemplate.length; targetIndex += 1) {
    const pair = pairs.find(([, pairedTarget]) => pairedTarget === targetIndex);
    if (pair) {
      lastAnchor = componentSelector(baseTemplate[pair[0]]);
      continue;
    }

    const nextPair = pairs.find(([, pairedTarget]) => pairedTarget > targetIndex);
    const operation = { component: targetTemplate[targetIndex] };
    if (nextPair) {
      operation.before = componentSelector(baseTemplate[nextPair[0]]);
    } else if (lastAnchor) {
      operation.after = lastAnchor;
    } else {
      return null;
    }
    add.push(operation);
    lastAnchor = componentSelector(targetTemplate[targetIndex]);
  }

  if (modify.length > 0) diff.modify = modify;
  if (remove.length > 0) diff.remove = remove;
  if (add.length > 0) diff.add = add;
  if (Object.keys(diff).length === 0) return null;

  const resolved = applyDiff(baseTemplate, diff);
  return JSON.stringify(resolved) === JSON.stringify(targetTemplate) ? diff : null;
}

function diffOperationWeight(diff) {
  return (diff.modify || []).length
    + (diff.remove || []).length
    + (diff.add || []).length;
}

function deriveBestTemplateVariantDiff(baseTemplate, candidateParents, targetTemplate) {
  let bestDiff = deriveTemplateVariantDiff(baseTemplate, targetTemplate);
  let bestWeight = bestDiff ? diffOperationWeight(bestDiff) : Infinity;

  for (const [parentName, parentTemplate] of candidateParents) {
    const parentDiff = deriveTemplateVariantDiff(parentTemplate, targetTemplate);
    if (!parentDiff) continue;
    const weight = diffOperationWeight(parentDiff);
    if (weight >= bestWeight) continue;
    bestDiff = {
      extends: parentName,
      ...parentDiff,
    };
    bestWeight = weight;
  }

  return bestDiff;
}

function convertSection(section, sectionPath = [], log = []) {
  if (!section || typeof section !== 'object') {
    return { converted: 0, total: 0 };
  }

  let converted = 0;
  let total = 0;
  const baseTemplate = section.template;
  const variants = section['type-variants'];
  if (Array.isArray(baseTemplate) && variants && typeof variants === 'object' && !Array.isArray(variants)) {
    const candidateParents = [];
    for (const [variantName, variantTemplate] of Object.entries(variants)) {
      if (!Array.isArray(variantTemplate)) continue;
      total += 1;
      const diff = deriveBestTemplateVariantDiff(baseTemplate, candidateParents, variantTemplate);
      if (diff) {
        const diffYaml = yaml.dump(diff, { noRefs: true, lineWidth: -1 });
        const fullYaml = yaml.dump(variantTemplate, { noRefs: true, lineWidth: -1 });
        if (diffYaml.length < fullYaml.length) {
          variants[variantName] = diff;
          converted += 1;
          log.push({ sectionPath, variantName, diff: cloneJson(diff) });
        }
      }
      candidateParents.push([variantName, variantTemplate]);
    }
  }

  for (const childKey of ['integral', 'non-integral', 'subsequent', 'ibid']) {
    const child = convertSection(section[childKey], [...sectionPath, childKey], log);
    converted += child.converted;
    total += child.total;
  }

  return { converted, total };
}

function convertStyle(data) {
  if (data.extends != null) {
    return { converted: 0, total: 0, log: [] };
  }

  const log = [];
  const citation = convertSection(data.citation, ['citation'], log);
  const bibliography = convertSection(data.bibliography, ['bibliography'], log);
  return {
    converted: citation.converted + bibliography.converted,
    total: citation.total + bibliography.total,
    log,
  };
}

function escapeRegex(str) {
  return str.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

function findTypeVariantsLine(lines, sectionPath) {
  let searchFrom = 0;
  let lastSegIndent = -2; // tracks indent of last matched segment

  for (const segment of sectionPath) {
    const segRegex = new RegExp(`^(\\s*)${escapeRegex(segment)}:\\s*$`);
    let found = -1;
    let segIndent = -1;
    for (let i = searchFrom; i < lines.length; i += 1) {
      const m = lines[i].match(segRegex);
      if (m) {
        found = i;
        segIndent = m[1].length;
        break;
      }
    }
    if (found < 0) return -1;
    searchFrom = found + 1;
    lastSegIndent = segIndent;
  }

  // type-variants: must be a direct child of the last segment,
  // so its indent must be exactly lastSegIndent + 2.
  const expectedIndent = lastSegIndent + 2;
  const tvRegex = new RegExp(`^(\\s{${expectedIndent}})type-variants:\\s*$`);

  for (let i = searchFrom; i < lines.length; i += 1) {
    const line = lines[i];
    if (tvRegex.test(line)) return i;
    // Stop if we exit the parent block (non-blank line at or below parent indent)
    const lineIndent = line.search(/\S/);
    if (lineIndent >= 0 && lineIndent <= lastSegIndent && i > searchFrom) break;
  }
  return -1;
}

function findVariantBlock(lines, typeVariantsLine, variantName) {
  const tvIndent = lines[typeVariantsLine].search(/\S/);
  const keyIndent = tvIndent + 2;
  const keyRegex = new RegExp(`^(\\s{${keyIndent}})${escapeRegex(variantName)}:`);

  let keyLine = -1;
  for (let i = typeVariantsLine + 1; i < lines.length; i += 1) {
    if (keyRegex.test(lines[i])) {
      keyLine = i;
      break;
    }
    // Stop if we've left the type-variants block
    const lineIndent = lines[i].search(/\S/);
    if (lineIndent >= 0 && lineIndent <= tvIndent) break;
  }
  if (keyLine < 0) return null;

  // Find the end of this variant's block (content lines, not including the key line)
  // The block ends when we encounter a non-blank line at keyIndent that looks like a sibling key
  // (i.e., a mapping key at the same level, including inline-value forms like `key: []`)
  let endLine = keyLine + 1;
  while (endLine < lines.length) {
    const l = lines[endLine];
    if (l.trim() === '') {
      endLine += 1;
      continue;
    }
    const indent = l.search(/\S/);
    // Stop if we find a sibling key (indent == keyIndent and line starts with a mapping key).
    // Use `[\w-]+:` without trailing `$` so inline-value forms like `key: []` also terminate.
    if (indent === keyIndent && /^\s*[\w-]+:/.test(l)) break;
    // Also stop if indent < keyIndent (left the section)
    if (indent < keyIndent) break;
    endLine += 1;
  }

  // startLine is the first content line (keyLine + 1), endLine is where the content ends
  return { keyLine, startLine: keyLine + 1, endLine, keyIndent };
}

function locateAndBuildReplacement(lines, entry) {
  const replacement = findTypeVariantsLine(lines, entry.sectionPath);
  if (replacement < 0) return null;

  const block = findVariantBlock(lines, replacement, entry.variantName);
  if (!block) return null;

  const { startLine, endLine, keyIndent } = block;

  // Render only the diff object as YAML
  const valueYaml = yaml.dump(entry.diff, { noRefs: true, lineWidth: -1, indent: 2 });

  // The YAML dump has absolute indentation from 0. We need to shift all lines
  // to be indented by keyIndent + 2 spaces
  const targetBaseIndent = keyIndent + 2;
  const newLines = valueYaml
    .trimEnd()
    .split('\n')
    .map((l) => {
      if (l.trim() === '') return '';
      // Get the indentation from the dump
      const dumpIndent = l.search(/\S/);
      // Get the content (everything from first non-space)
      const content = l.substring(dumpIndent);
      // Preserve relative indentation: if dump had 6 spaces, that's 6 more than base 0
      // We want to keep that relative indentation but shift to targetBaseIndent
      const relativeIndent = dumpIndent;
      return ' '.repeat(targetBaseIndent + relativeIndent) + content;
    });

  // newLines will replace the old content (from startLine to endLine)
  // The key line itself is preserved above (keyLine), we only replace content lines

  return { startLine, endLine, newLines };
}

function surgicalWrite(filePath, rawText, log) {
  const lines = rawText.split('\n');

  // Process replacements bottom-to-top so earlier line numbers stay valid
  const replacements = [];

  for (const entry of log) {
    const replacement = locateAndBuildReplacement(lines, entry);
    if (!replacement) {
      // Fallback: if we can't locate the block surgically, skip (keeps original)
      continue;
    }
    replacements.push(replacement);
  }

  // Sort by startLine descending
  replacements.sort((a, b) => b.startLine - a.startLine);

  for (const { startLine, endLine, newLines } of replacements) {
    lines.splice(startLine, endLine - startLine, ...newLines);
  }

  fs.writeFileSync(filePath, lines.join('\n'));
}

function main() {
  const args = parseArgs();
  const summaries = [];

  for (const filePath of listStyleFiles(args.paths)) {
    const rawText = fs.readFileSync(filePath, 'utf8');
    const data = yaml.load(rawText) || {};
    const summary = convertStyle(data);
    if (summary.total > 0) {
      summaries.push({
        file: path.relative(PROJECT_ROOT, filePath),
        converted: summary.converted,
        total: summary.total,
      });
    }
    if (summary.converted > 0 && args.write && summary.log.length > 0) {
      try {
        surgicalWrite(filePath, rawText, summary.log);
        // Verify the result can be parsed
        yaml.load(fs.readFileSync(filePath, 'utf8'));
      } catch (err) {
        // Fallback: write the full dump if surgical write failed
        fs.writeFileSync(filePath, yaml.dump(data, { noRefs: true, lineWidth: -1 }));
      }
    }
  }

  const totals = summaries.reduce(
    (acc, summary) => ({
      files: acc.files + 1,
      converted: acc.converted + summary.converted,
      total: acc.total + summary.total,
    }),
    { files: 0, converted: 0, total: 0 }
  );

  if (args.json) {
    console.log(JSON.stringify({ totals, summaries }, null, 2));
  } else {
    console.log(`Template V3 conversion: ${totals.converted}/${totals.total} variants in ${totals.files} files`);
    for (const summary of summaries) {
      console.log(`${summary.file}: ${summary.converted}/${summary.total}`);
    }
  }
}

if (require.main === module) {
  main();
}

module.exports = {
  convertStyle,
  deriveTemplateVariantDiff,
};
