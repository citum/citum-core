const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');

const {
  applyFixes,
  convertItemsAliasInText,
  expandAnonymousAnchorsInText,
  lintAnonymousAnchors,
  lintLegacyItemsAlias,
  lintParsedStyle,
  stripAnonymousAnchorMarkersInText,
} = require('./style-structure-lint');

function writeTempStyle(content) {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), 'style-structure-lint-'));
  const filePath = path.join(tempDir, 'fixture.yaml');
  fs.writeFileSync(filePath, content);
  return filePath;
}

test('STYLE001 detects anonymous generated anchors', () => {
  const content = `version: ""
citation:
  template:
    - contributor: author
      shorten: &id001
        min: 4
        use-first: 1
    - contributor: editor
      shorten: *id001
`;

  const violations = lintAnonymousAnchors('styles/fixture.yaml', content);

  assert.equal(violations.length, 2);
  assert.equal(violations[0].ruleId, 'STYLE001');
});

test('STYLE002 flags inert substitute overrides when template is explicitly empty', () => {
  const content = `version: ""
options:
  substitute:
    template: []
    overrides:
      legal_case:
        template: [title]
`;
  const data = {
    version: '',
    options: {
      substitute: {
        template: [],
        overrides: {
          legal_case: { template: ['title'] },
        },
      },
    },
  };

  const violations = lintParsedStyle('styles/fixture.yaml', content, data);

  assert.equal(violations.some((violation) => violation.ruleId === 'STYLE002'), true);
});

test('STYLE003 flags duplicate citation shorten blocks that can be hoisted safely', () => {
  const content = `version: ""
citation:
  template:
    - contributor: author
      shorten:
        min: 4
        use-first: 1
    - contributor: editor
      shorten:
        min: 4
        use-first: 1
`;
  const data = {
    version: '',
    citation: {
      template: [
        { contributor: 'author', shorten: { min: 4, 'use-first': 1 } },
        { contributor: 'editor', shorten: { min: 4, 'use-first': 1 } },
      ],
    },
  };

  const violations = lintParsedStyle('styles/fixture.yaml', content, data);

  assert.equal(violations.some((violation) => violation.ruleId === 'STYLE003'), true);
});

test('STYLE003 does not flag when some contributor components intentionally differ', () => {
  const content = `version: ""
bibliography:
  template:
    - contributor: author
      shorten:
        min: 5
        use-first: 1
    - contributor: editor
`;
  const data = {
    version: '',
    bibliography: {
      template: [
        { contributor: 'author', shorten: { min: 5, 'use-first': 1 } },
        { contributor: 'editor' },
      ],
    },
  };

  const violations = lintParsedStyle('styles/fixture.yaml', content, data);

  assert.equal(violations.some((violation) => violation.ruleId === 'STYLE003'), false);
});

test('STYLE004 flags type variants identical to the base template', () => {
  const content = `version: ""
bibliography:
  template:
    - contributor: author
    - title: primary
  type-variants:
    article-journal:
      - contributor: author
      - title: primary
`;
  const data = {
    version: '',
    bibliography: {
      template: [
        { contributor: 'author' },
        { title: 'primary' },
      ],
      'type-variants': {
        'article-journal': [
          { contributor: 'author' },
          { title: 'primary' },
        ],
      },
    },
  };

  const violations = lintParsedStyle('styles/fixture.yaml', content, data);

  assert.equal(violations.some((violation) => violation.ruleId === 'STYLE004'), true);
});

test('applyFixes removes inert substitute overrides, hoists shorten config, and drops duplicate variants', () => {
  const style = {
    version: '',
    options: {
      substitute: {
        template: [],
        overrides: {
          legal_case: { template: ['title'] },
        },
      },
    },
    citation: {
      template: [
        { contributor: 'author', shorten: { min: 4, 'use-first': 1 } },
        { contributor: 'editor', shorten: { min: 4, 'use-first': 1 } },
      ],
      'type-variants': {
        article: [
          { contributor: 'author', shorten: { min: 4, 'use-first': 1 } },
          { contributor: 'editor', shorten: { min: 4, 'use-first': 1 } },
        ],
      },
    },
  };

  const changed = applyFixes(style);

  assert.equal(changed, true);
  assert.deepEqual(style.options.substitute, { template: [] });
  assert.deepEqual(style.citation.options.contributors.shorten, { min: 4, 'use-first': 1 });
  assert.equal(style.citation.template.every((component) => component.shorten === undefined), true);
  assert.equal(style.citation['type-variants'], undefined);
});

test('yaml round-trip autofix removes anonymous shorten anchors from authored text', () => {
  const filePath = writeTempStyle(`version: ""
citation:
  template:
    - contributor: author
      shorten: &id001
        min: 4
        use-first: 1
    - contributor: editor
      shorten: *id001
`);
  const yaml = require('js-yaml');
  const style = yaml.load(fs.readFileSync(filePath, 'utf8'));

  applyFixes(style);
  fs.writeFileSync(filePath, yaml.dump(style, { noRefs: true, lineWidth: -1 }));
  const output = fs.readFileSync(filePath, 'utf8');

  assert.equal(output.includes('&id001'), false);
  assert.equal(output.includes('*id001'), false);
});

test('STYLE001 text fixer expands aliases without reformatting unrelated YAML', () => {
  const input = `info:
  title: Example
citation:
  template:
    - contributor: author
      shorten: &id001
        min: 4
        use-first: 1
    - contributor: editor
      shorten: *id001
  delimiter: ". "
`;

  const output = expandAnonymousAnchorsInText(input);

  assert.equal(output, `info:
  title: Example
citation:
  template:
    - contributor: author
      shorten:
        min: 4
        use-first: 1
    - contributor: editor
      shorten:
        min: 4
        use-first: 1
  delimiter: ". "
`);
});

test('STYLE001 text fixer strips stray anchor markers left behind after expansion', () => {
  const input = `bibliography:
  template:
    - group:
      - contributor: editor
        shorten: &id003
          min: 4
          use-first: 3
      - title: parent-monograph
`;

  const output = stripAnonymousAnchorMarkersInText(input);

  assert.equal(output, `bibliography:
  template:
    - group:
      - contributor: editor
        shorten:
          min: 4
          use-first: 3
      - title: parent-monograph
`);
});

test('STYLE001 text fixer expands inline aliases and removes inline anchor definitions', () => {
  const input = `bibliography:
  template:
    - contributor: translator
      label: &id001 {term: translator, form: short, placement: suffix}
    - contributor: editor
      label: *id001
`;

  const output = expandAnonymousAnchorsInText(input);

  assert.equal(output, `bibliography:
  template:
    - contributor: translator
      label: {term: translator, form: short, placement: suffix}
    - contributor: editor
      label: {term: translator, form: short, placement: suffix}
`);
});

test('STYLE005 detects legacy items aliases in style templates', () => {
  const content = `bibliography:
  template:
    - items:
        - contributor: author
`;

  const violations = lintLegacyItemsAlias('styles/fixture.yaml', content);

  assert.equal(violations.length, 1);
  assert.equal(violations[0].ruleId, 'STYLE005');
});

test('STYLE005 text fixer rewrites items to group without touching unrelated items text', () => {
  const input = `bibliography:
  template:
    - items:
        - contributor: author
docs:
  note: "citation items remain separate"
`;

  const output = convertItemsAliasInText(input);

  assert.equal(output, `bibliography:
  template:
    - group:
        - contributor: author
docs:
  note: "citation items remain separate"
`);
});
