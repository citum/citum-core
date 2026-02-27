#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const readline = require('readline');
const { spawnSync } = require('child_process');

const PROJECT_ROOT = path.resolve(__dirname, '..');
const DEFAULT_REFS_FILE = path.join(PROJECT_ROOT, 'tests', 'fixtures', 'references-expanded.json');
const DEFAULT_CITATIONS_FILE = path.join(PROJECT_ROOT, 'tests', 'fixtures', 'citations-expanded.json');
const DEFAULT_STYLE = 'styles-legacy/apa.csl';

const SUPPORTED_LOCATOR_LABELS = [
  'page',
  'section',
  'chapter',
  'figure',
  'table',
  'paragraph',
  'volume',
];

const TEMPLATE_LIST = [
  {
    key: 'article-journal',
    label: 'Journal article',
    className: 'serial-component',
    type: 'article-journal',
    requiredFields: ['title', 'issued', 'container-title'],
    optionalFields: ['author', 'translator', 'volume', 'issue', 'page', 'DOI', 'URL', 'accessed', 'language', 'note'],
  },
  {
    key: 'article-magazine',
    label: 'Magazine article',
    className: 'serial-component',
    type: 'article-magazine',
    requiredFields: ['title', 'issued', 'container-title'],
    optionalFields: ['author', 'translator', 'volume', 'issue', 'page', 'section', 'URL', 'accessed', 'language', 'note'],
  },
  {
    key: 'article-newspaper',
    label: 'Newspaper article',
    className: 'serial-component',
    type: 'article-newspaper',
    requiredFields: ['title', 'issued', 'container-title'],
    optionalFields: ['author', 'translator', 'section', 'page', 'URL', 'accessed', 'language', 'note'],
  },
  {
    key: 'book',
    label: 'Book',
    className: 'monograph',
    type: 'book',
    requiredFields: ['title', 'issued'],
    optionalFields: ['author', 'editor', 'translator', 'publisher', 'publisher-place', 'edition', 'ISBN', 'URL', 'accessed', 'language', 'note'],
  },
  {
    key: 'chapter',
    label: 'Book chapter',
    className: 'collection-component',
    type: 'chapter',
    requiredFields: ['title', 'issued', 'container-title'],
    optionalFields: ['author', 'editor', 'translator', 'page', 'publisher', 'publisher-place', 'collection-number', 'URL', 'accessed', 'language', 'note'],
  },
  {
    key: 'paper-conference',
    label: 'Conference paper',
    className: 'collection-component',
    type: 'paper-conference',
    requiredFields: ['title', 'issued', 'container-title'],
    optionalFields: ['author', 'editor', 'translator', 'event', 'page', 'publisher', 'publisher-place', 'URL', 'accessed', 'language', 'note'],
  },
  {
    key: 'report',
    label: 'Report',
    className: 'monograph',
    type: 'report',
    requiredFields: ['title', 'issued'],
    optionalFields: ['author', 'editor', 'publisher', 'publisher-place', 'genre', 'number', 'URL', 'accessed', 'language', 'note'],
  },
  {
    key: 'thesis',
    label: 'Thesis',
    className: 'monograph',
    type: 'thesis',
    requiredFields: ['title', 'issued'],
    optionalFields: ['author', 'publisher', 'publisher-place', 'genre', 'URL', 'accessed', 'language', 'note'],
  },
  {
    key: 'webpage',
    label: 'Web page',
    className: 'monograph',
    type: 'webpage',
    requiredFields: ['title', 'issued', 'URL'],
    optionalFields: ['author', 'publisher', 'accessed', 'language', 'note'],
  },
  {
    key: 'dataset',
    label: 'Dataset',
    className: 'dataset',
    type: 'dataset',
    requiredFields: ['title', 'issued'],
    optionalFields: ['author', 'publisher', 'version', 'DOI', 'URL', 'accessed', 'language', 'note'],
  },
  {
    key: 'legal_case',
    label: 'Legal case',
    className: 'legal-case',
    type: 'legal_case',
    requiredFields: ['title', 'issued', 'authority'],
    optionalFields: ['container-title', 'volume', 'page', 'number', 'URL', 'accessed', 'language', 'note'],
  },
  {
    key: 'patent',
    label: 'Patent',
    className: 'patent',
    type: 'patent',
    requiredFields: ['title', 'issued', 'number'],
    optionalFields: ['author', 'country', 'authority', 'URL', 'accessed', 'language', 'note'],
  },
  {
    key: 'software',
    label: 'Software',
    className: 'software',
    type: 'software',
    requiredFields: ['title', 'issued'],
    optionalFields: ['author', 'publisher', 'version', 'medium', 'URL', 'accessed', 'language', 'note'],
  },
];

const TEMPLATE_MAP = new Map(TEMPLATE_LIST.map((template) => [template.key, template]));

function parseArgs(argv) {
  const options = {
    style: DEFAULT_STYLE,
    oracleMode: 'single',
    withCitation: false,
    refsFile: DEFAULT_REFS_FILE,
    citationsFile: DEFAULT_CITATIONS_FILE,
  };

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === '--style') {
      options.style = argv[++i];
    } else if (arg === '--no-oracle') {
      options.oracleMode = 'skip';
    } else if (arg === '--with-citation') {
      options.withCitation = true;
    } else if (arg === '--oracle-mode') {
      const mode = argv[++i];
      if (!['single', 'skip'].includes(mode)) {
        throw new Error(`Unsupported oracle mode "${mode}". Use "single" or "skip".`);
      }
      options.oracleMode = mode;
    } else if (arg === '--refs-file') {
      options.refsFile = path.resolve(argv[++i]);
    } else if (arg === '--citations-file') {
      options.citationsFile = path.resolve(argv[++i]);
    } else if (arg === '--help' || arg === '-h') {
      printHelp();
      process.exit(0);
    } else {
      throw new Error(`Unknown argument: ${arg}`);
    }
  }

  return options;
}

function printHelp() {
  console.log(`Usage: node scripts/generate-test-item.js [options]

Options:
  --style <path>            Default oracle style path (default: ${DEFAULT_STYLE})
  --no-oracle               Skip oracle execution
  --with-citation           Scaffold a citation scenario
  --oracle-mode <mode>      Oracle mode: single or skip
  --refs-file <path>        Override references fixture path
  --citations-file <path>   Override citations fixture path
  --help, -h                Show this help message`);
}

function createPrompter() {
  const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout,
  });

  function askRaw(question) {
    return new Promise((resolve) => {
      rl.question(question, (answer) => resolve(answer.replace(/\r$/, '')));
    });
  }

  async function ask(question) {
    const answer = await askRaw(question);
    return answer.trim();
  }

  async function askWithDefault(label, defaultValue) {
    const suffix = defaultValue ? ` [${defaultValue}]` : '';
    const answer = await askRaw(`${label}${suffix}: `);
    if (!answer && defaultValue !== undefined) {
      return defaultValue;
    }
    return answer.trim();
  }

  async function confirm(label, defaultValue = true) {
    const defaultToken = defaultValue ? 'Y/n' : 'y/N';
    while (true) {
      const answer = (await ask(`${label} (${defaultToken}): `)).toLowerCase();
      if (!answer) {
        return defaultValue;
      }
      if (['y', 'yes'].includes(answer)) {
        return true;
      }
      if (['n', 'no'].includes(answer)) {
        return false;
      }
      console.log('Enter "y" or "n".');
    }
  }

  async function choose(label, options) {
    console.log(label);
    options.forEach((option, index) => {
      console.log(`  ${index + 1}. ${option.label}`);
    });

    while (true) {
      const raw = await ask('Select an option by number: ');
      const choice = Number.parseInt(raw, 10);
      if (choice >= 1 && choice <= options.length) {
        return options[choice - 1];
      }
      console.log(`Enter a number between 1 and ${options.length}.`);
    }
  }

  return {
    ask,
    askRaw,
    askWithDefault,
    choose,
    confirm,
    close: () => rl.close(),
  };
}

function readJson(filePath) {
  return JSON.parse(fs.readFileSync(filePath, 'utf8'));
}

function writeJson(filePath, value, spaces) {
  fs.writeFileSync(filePath, `${JSON.stringify(value, null, spaces)}\n`);
}

function nextItemId(refs) {
  let max = 0;
  for (const key of Object.keys(refs)) {
    const match = /^ITEM-(\d+)$/.exec(key);
    if (match) {
      max = Math.max(max, Number.parseInt(match[1], 10));
    }
  }
  return `ITEM-${max + 1}`;
}

function parseDateInput(raw) {
  const trimmed = raw.trim();
  if (!trimmed) {
    throw new Error('Date is required.');
  }

  if (!/^\d{4}(-\d{1,2}){0,2}$/.test(trimmed)) {
    throw new Error('Use YYYY, YYYY-MM, or YYYY-MM-DD.');
  }

  const parts = trimmed.split('-').map((part) => Number.parseInt(part, 10));
  const [year, month, day] = parts;
  if (month !== undefined && (month < 1 || month > 12)) {
    throw new Error('Month must be between 1 and 12.');
  }
  if (day !== undefined && (day < 1 || day > 31)) {
    throw new Error('Day must be between 1 and 31.');
  }

  return { 'date-parts': [parts] };
}

function maybeSet(target, key, value) {
  if (value === undefined || value === null || value === '') {
    return;
  }
  target[key] = value;
}

async function promptNonEmpty(prompter, label, defaultValue) {
  while (true) {
    const answer = await prompter.askWithDefault(label, defaultValue);
    if (answer) {
      return answer;
    }
    console.log('This field is required.');
  }
}

async function promptOptional(prompter, label, defaultValue = '', preserveWhitespace = false) {
  if (preserveWhitespace) {
    const suffix = defaultValue ? ` [${defaultValue}]` : '';
    const answer = await prompter.askRaw(`${label}${suffix}: `);
    if (!answer && defaultValue !== undefined) {
      return defaultValue;
    }
    return answer;
  }

  return prompter.askWithDefault(label, defaultValue);
}

async function promptDate(prompter, label, required) {
  while (true) {
    const raw = await prompter.ask(`${label}${required ? '' : ' (optional)'}: `);
    if (!raw) {
      if (required) {
        console.log('This field is required.');
        continue;
      }
      return undefined;
    }

    try {
      return parseDateInput(raw);
    } catch (error) {
      console.log(error.message);
    }
  }
}

async function promptNames(prompter, label) {
  const addNames = await prompter.confirm(`Add ${label}?`, false);
  if (!addNames) {
    return undefined;
  }

  const entries = [];
  while (true) {
    const nameType = await prompter.choose(`Select ${label} entry type:`, [
      { label: 'Structured name' },
      { label: 'Literal name' },
    ]);

    if (nameType.label === 'Structured name') {
      const family = await promptNonEmpty(prompter, `${label} family name`, undefined);
      const given = await promptOptional(prompter, `${label} given name`);
      const suffix = await promptOptional(prompter, `${label} suffix`);
      const entry = { family };
      maybeSet(entry, 'given', given);
      maybeSet(entry, 'suffix', suffix);
      entries.push(entry);
    } else {
      const literal = await promptNonEmpty(prompter, `${label} literal name`, undefined);
      entries.push({ literal });
    }

    const addAnother = await prompter.confirm(`Add another ${label}?`, false);
    if (!addAnother) {
      break;
    }
  }

  return entries.length > 0 ? entries : undefined;
}

async function promptCommonFields(prompter, item, template) {
  item.title = await promptNonEmpty(prompter, 'Title', undefined);
  item.issued = await promptDate(prompter, 'Issued date (YYYY, YYYY-MM, or YYYY-MM-DD)', true);

  if (template.optionalFields.includes('author')) {
    maybeSet(item, 'author', await promptNames(prompter, 'author'));
  }
  if (template.optionalFields.includes('editor')) {
    maybeSet(item, 'editor', await promptNames(prompter, 'editor'));
  }
  if (template.optionalFields.includes('translator')) {
    maybeSet(item, 'translator', await promptNames(prompter, 'translator'));
  }
}

async function promptFieldSet(prompter, item, template) {
  if (template.requiredFields.includes('container-title')) {
    item['container-title'] = await promptNonEmpty(prompter, 'Container title', undefined);
  } else if (template.optionalFields.includes('container-title')) {
    maybeSet(item, 'container-title', await promptOptional(prompter, 'Container title'));
  }

  if (template.optionalFields.includes('event')) {
    maybeSet(item, 'event', await promptOptional(prompter, 'Event'));
  }
  if (template.optionalFields.includes('volume')) {
    maybeSet(item, 'volume', await promptOptional(prompter, 'Volume'));
  }
  if (template.optionalFields.includes('issue')) {
    maybeSet(item, 'issue', await promptOptional(prompter, 'Issue'));
  }
  if (template.optionalFields.includes('page')) {
    maybeSet(item, 'page', await promptOptional(prompter, 'Page or page range'));
  }
  if (template.optionalFields.includes('section')) {
    maybeSet(item, 'section', await promptOptional(prompter, 'Section'));
  }
  if (template.optionalFields.includes('publisher')) {
    maybeSet(item, 'publisher', await promptOptional(prompter, 'Publisher'));
  }
  if (template.optionalFields.includes('publisher-place')) {
    maybeSet(item, 'publisher-place', await promptOptional(prompter, 'Publisher place'));
  }
  if (template.optionalFields.includes('edition')) {
    maybeSet(item, 'edition', await promptOptional(prompter, 'Edition'));
  }
  if (template.optionalFields.includes('genre')) {
    maybeSet(item, 'genre', await promptOptional(prompter, 'Genre'));
  }
  if (template.requiredFields.includes('authority')) {
    item.authority = await promptNonEmpty(prompter, 'Authority', undefined);
  } else if (template.optionalFields.includes('authority')) {
    maybeSet(item, 'authority', await promptOptional(prompter, 'Authority'));
  }
  if (template.requiredFields.includes('number')) {
    item.number = await promptNonEmpty(prompter, 'Number', undefined);
  } else if (template.optionalFields.includes('number')) {
    maybeSet(item, 'number', await promptOptional(prompter, 'Number'));
  }
  if (template.optionalFields.includes('country')) {
    maybeSet(item, 'country', await promptOptional(prompter, 'Country code or name'));
  }
  if (template.optionalFields.includes('collection-number')) {
    maybeSet(item, 'collection-number', await promptOptional(prompter, 'Collection number'));
  }
  if (template.optionalFields.includes('version')) {
    maybeSet(item, 'version', await promptOptional(prompter, 'Version'));
  }
  if (template.optionalFields.includes('medium')) {
    maybeSet(item, 'medium', await promptOptional(prompter, 'Medium'));
  }
  if (template.optionalFields.includes('DOI')) {
    maybeSet(item, 'DOI', await promptOptional(prompter, 'DOI'));
  }
  if (template.requiredFields.includes('URL')) {
    item.URL = await promptNonEmpty(prompter, 'URL', undefined);
  } else if (template.optionalFields.includes('URL')) {
    maybeSet(item, 'URL', await promptOptional(prompter, 'URL'));
  }
  if (template.optionalFields.includes('ISBN')) {
    maybeSet(item, 'ISBN', await promptOptional(prompter, 'ISBN'));
  }
  if (template.optionalFields.includes('accessed')) {
    maybeSet(item, 'accessed', await promptDate(prompter, 'Accessed date (YYYY, YYYY-MM, or YYYY-MM-DD)', false));
  }
  if (template.optionalFields.includes('language')) {
    maybeSet(item, 'language', await promptOptional(prompter, 'Language (BCP 47)'));
  }
  if (template.optionalFields.includes('note')) {
    maybeSet(item, 'note', await promptOptional(prompter, 'Note'));
  }
}

async function promptTemplate(prompter) {
  const template = await prompter.choose(
    'Select the reference template to create:',
    TEMPLATE_LIST.map((entry) => ({ label: `${entry.key} (${entry.label})`, value: entry.key }))
  );

  return TEMPLATE_MAP.get(template.value);
}

function validateItem(item, template) {
  const missing = [];
  for (const field of template.requiredFields) {
    if (!item[field]) {
      missing.push(field);
    }
  }
  if (missing.length > 0) {
    throw new Error(`Missing required fields: ${missing.join(', ')}`);
  }
}

function defaultCitationId(itemId, templateKey) {
  return `${templateKey.replace(/_/g, '-')}-${itemId.toLowerCase()}`;
}

async function promptCitation(prompter, itemId, template, citations) {
  const defaultId = defaultCitationId(itemId, template.key);
  let citationId;

  while (true) {
    citationId = await promptNonEmpty(prompter, 'Citation scenario id', defaultId);
    if (citations.some((entry) => entry.id === citationId)) {
      console.log(`Citation id "${citationId}" already exists.`);
      continue;
    }
    break;
  }

  const citation = {
    id: citationId,
    items: [{ id: itemId }],
  };

  const locator = await promptOptional(prompter, 'Locator');
  if (locator) {
    citation.items[0].locator = locator;
    const labelChoice = await prompter.choose('Select locator label:', SUPPORTED_LOCATOR_LABELS.map((label) => ({ label })));
    citation.items[0].label = labelChoice.label;
  }

  const prefix = await promptOptional(prompter, 'Prefix', '', true);
  maybeSet(citation.items[0], 'prefix', prefix);

  const suffix = await promptOptional(prompter, 'Suffix', '', true);
  maybeSet(citation.items[0], 'suffix', suffix);

  const suppressAuthor = await prompter.confirm('Suppress author?', false);
  if (suppressAuthor) {
    citation['suppress-author'] = true;
  }

  return citation;
}

function relativeToRoot(targetPath) {
  const absolute = path.resolve(targetPath);
  const relative = path.relative(PROJECT_ROOT, absolute);
  if (!relative || relative === '') {
    return '.';
  }
  if (relative.startsWith('..')) {
    return absolute;
  }
  return relative;
}

function runOracle(stylePath, refsFile, citationsFile) {
  const scriptPath = path.join('scripts', 'oracle.js');
  const args = [
    scriptPath,
    stylePath,
    '--verbose',
  ];

  if (path.resolve(refsFile) !== DEFAULT_REFS_FILE) {
    args.push('--refs-fixture', refsFile);
  }
  if (path.resolve(citationsFile) !== DEFAULT_CITATIONS_FILE) {
    args.push('--citations-fixture', citationsFile);
  }

  console.log(`\nRunning oracle: node ${args.join(' ')}`);
  const result = spawnSync('node', args, {
    cwd: PROJECT_ROOT,
    stdio: 'inherit',
  });

  if (result.error) {
    return { ok: false, fatal: true, message: result.error.message };
  }
  if (result.status === 0) {
    return { ok: true, fatal: false };
  }
  if (result.status === 1) {
    return { ok: true, fatal: false, message: 'Oracle reported mismatches.' };
  }
  return { ok: false, fatal: true, message: `Oracle failed with exit code ${result.status}.` };
}

async function main() {
  const options = parseArgs(process.argv.slice(2));
  const prompter = createPrompter();

  try {
    const refs = readJson(options.refsFile);
    const citations = readJson(options.citationsFile);
    const template = await promptTemplate(prompter);
    const itemId = nextItemId(refs);
    const item = {
      id: itemId,
      class: template.className,
      type: template.type,
    };

    console.log(`\nCreating ${template.key} as ${itemId}`);
    await promptCommonFields(prompter, item, template);
    await promptFieldSet(prompter, item, template);
    validateItem(item, template);

    refs[itemId] = item;
    writeJson(options.refsFile, refs, 2);
    console.log(`\nAdded ${itemId} to ${relativeToRoot(options.refsFile)}.`);

    let wroteCitation = false;
    const shouldAddCitation = options.withCitation || await prompter.confirm('Add a citation scenario?', false);
    if (shouldAddCitation) {
      const citation = await promptCitation(prompter, itemId, template, citations);
      citations.push(citation);
      writeJson(options.citationsFile, citations, 4);
      wroteCitation = true;
      console.log(`Added citation ${citation.id} to ${relativeToRoot(options.citationsFile)}.`);
    }

    let oracleExitCode = 0;
    if (options.oracleMode === 'single') {
      const stylePath = await promptNonEmpty(prompter, 'Oracle style path', options.style);
      const oracle = runOracle(stylePath, options.refsFile, options.citationsFile);
      if (oracle.message) {
        console.log(oracle.message);
      }
      if (oracle.fatal) {
        oracleExitCode = 1;
      }
    } else {
      console.log('Oracle execution skipped.');
    }

    console.log('\nSummary:');
    console.log(`  Item: ${itemId}`);
    console.log(`  Template: ${template.key}`);
    console.log(`  References file: ${relativeToRoot(options.refsFile)}`);
    if (wroteCitation) {
      console.log(`  Citations file: ${relativeToRoot(options.citationsFile)}`);
    }
    console.log(`  Citation added: ${wroteCitation ? 'yes' : 'no'}`);

    process.exitCode = oracleExitCode;
  } finally {
    prompter.close();
  }
}

main().catch((error) => {
  console.error(`Error: ${error.message}`);
  process.exit(1);
});
