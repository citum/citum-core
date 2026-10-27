const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const yaml = require('js-yaml');
const Ajv2020 = require('ajv/dist/2020');
const addFormats = require('ajv-formats');

const ROOT = path.join(__dirname, '..');
const BIB_SCHEMA_PATH = path.join(ROOT, 'docs/schemas/bib.json');
const GUIDE_PATH = path.join(ROOT, 'docs/guides/BIBLIOGRAPHY_DATA.md');
const FIELD_REFERENCE_PATH = path.join(
    ROOT,
    'docs/reference/generated/DATA_MODEL_FIELDS.md'
);

function loadBibSchema() {
    return JSON.parse(fs.readFileSync(BIB_SCHEMA_PATH, 'utf8'));
}

function referencedDefinition(fieldSchema, definitions) {
    const candidates = [
        fieldSchema.$ref,
        ...(fieldSchema.anyOf || []).map((entry) => entry.$ref),
        ...(fieldSchema.oneOf || []).map((entry) => entry.$ref),
        fieldSchema.type === 'array' ? fieldSchema.items?.$ref : undefined,
    ].filter(Boolean);

    for (const ref of candidates) {
        const name = ref.replace('#/$defs/', '');
        if (definitions[name]) return definitions[name];
    }
    return null;
}

function hasEffectiveDescription(fieldSchema, definitions) {
    if (fieldSchema.description?.trim()) return true;
    return Boolean(referencedDefinition(fieldSchema, definitions)?.description?.trim());
}

function bibliographyValidator() {
    const ajv = new Ajv2020({ allErrors: true, strict: false });
    addFormats(ajv);
    ajv.addFormat('uint8', true);
    ajv.addFormat('uint32', true);
    return ajv.compile(loadBibSchema());
}

function assertValidBibliography(validate, value, label) {
    const valid = validate(value);
    assert.equal(
        valid,
        true,
        `${label} must validate against bib.json:\n${JSON.stringify(validate.errors, null, 2)}`
    );
}

test('every InputReference field has an effective public description', () => {
    const schema = loadBibSchema();
    const definitions = schema.$defs;
    const missing = [];

    for (const variant of definitions.InputReference.oneOf) {
        const className = variant.properties.class.const;
        for (const [fieldName, fieldSchema] of Object.entries(variant.properties)) {
            if (fieldName === 'class') continue;
            if (!hasEffectiveDescription(fieldSchema, definitions)) {
                missing.push(`${className}.${fieldName}`);
            }
        }
    }

    assert.deepEqual(missing, []);
});

test('relation fields describe their meaning on the owning reference', () => {
    const schema = loadBibSchema();
    const relationNames = new Set(['container', 'original', 'reviewed', 'event']);
    const missing = [];

    for (const variant of schema.$defs.InputReference.oneOf) {
        const className = variant.properties.class.const;
        for (const [fieldName, fieldSchema] of Object.entries(variant.properties)) {
            if (relationNames.has(fieldName) && !fieldSchema.description?.trim()) {
                missing.push(`${className}.${fieldName}`);
            }
        }
    }

    assert.deepEqual(missing, []);
});

test('bibliography guide YAML blocks validate against the published schema', () => {
    const markdown = fs.readFileSync(GUIDE_PATH, 'utf8');
    const blocks = [...markdown.matchAll(/```yaml\n([\s\S]*?)```/g)].map((match) => match[1]);
    assert.ok(blocks.length > 0, 'expected complete YAML examples in the bibliography guide');

    const validate = bibliographyValidator();
    blocks.forEach((block, index) => {
        assertValidBibliography(validate, yaml.load(block), `guide YAML block ${index + 1}`);
    });
});

test('data-model fixtures validate against the published schema', () => {
    const examplesDir = path.join(ROOT, 'examples/data-model');
    const files = fs.readdirSync(examplesDir).filter((file) => file.endsWith('.yaml')).sort();
    assert.ok(files.length > 0, 'expected data-model YAML fixtures');

    const validate = bibliographyValidator();
    for (const file of files) {
        const value = yaml.load(fs.readFileSync(path.join(examplesDir, file), 'utf8'));
        assertValidBibliography(validate, value, file);
    }
});

test('generated data-model field tables contain no undocumented fields', () => {
    const markdown = fs.readFileSync(FIELD_REFERENCE_PATH, 'utf8');
    const fieldTables = markdown.split('\n## Closed Vocabularies\n')[0];
    const undocumented = fieldTables
        .split('\n')
        .filter((line) => /^\| `[^`]+` \|/.test(line) && line.endsWith('| — |'));

    assert.deepEqual(undocumented, []);
});
