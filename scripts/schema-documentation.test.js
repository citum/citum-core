const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const yaml = require('js-yaml');
const Ajv2020 = require('ajv/dist/2020');
const addFormats = require('ajv-formats');

const ROOT = path.join(__dirname, '..');
const BIB_SCHEMA_PATH = path.join(ROOT, 'docs/schemas/bib.json');
const CITATION_SCHEMA_PATH = path.join(ROOT, 'docs/schemas/citation.json');
const SERVER_SCHEMA_PATH = path.join(ROOT, 'docs/schemas/server.json');
const REGISTRY_SCHEMA_PATH = path.join(ROOT, 'docs/schemas/registry.json');
const ABBREV_SCHEMA_PATH = path.join(ROOT, 'docs/schemas/abbrev-map.json');
const GUIDE_PATH = path.join(ROOT, 'docs/guides/BIBLIOGRAPHY_DATA.md');
const CITATION_SERVER_GUIDE_PATH = path.join(
    ROOT,
    'docs/guides/CITATION_AND_SERVER.md'
);
const AUXILIARY_GUIDE_PATH = path.join(
    ROOT,
    'docs/guides/REGISTRY_AND_AUXILIARY_SCHEMAS.md'
);
const FIELD_REFERENCE_PATH = path.join(
    ROOT,
    'docs/reference/generated/DATA_MODEL_FIELDS.md'
);

function loadBibSchema() {
    return JSON.parse(fs.readFileSync(BIB_SCHEMA_PATH, 'utf8'));
}

function loadSchema(schemaPath) {
    return JSON.parse(fs.readFileSync(schemaPath, 'utf8'));
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

function undocumentedProperties(schema) {
    const definitions = schema.$defs || {};
    const missing = [];

    function visit(node, path) {
        if (!node || typeof node !== 'object') return;

        for (const [fieldName, fieldSchema] of Object.entries(node.properties || {})) {
            const fieldPath = `${path}.${fieldName}`;
            if (!hasEffectiveDescription(fieldSchema, definitions)) {
                missing.push(fieldPath);
            }
            for (const unionKey of ['oneOf', 'anyOf', 'allOf']) {
                (fieldSchema[unionKey] || []).forEach((branch, index) => {
                    if (branch.properties) visit(branch, `${fieldPath}.${unionKey}[${index}]`);
                });
            }
        }

        for (const unionKey of ['oneOf', 'anyOf', 'allOf']) {
            (node[unionKey] || []).forEach((branch, index) => {
                if (branch.properties) visit(branch, `${path}.${unionKey}[${index}]`);
            });
        }
    }

    for (const [definitionName, definition] of Object.entries(definitions)) {
        visit(definition, `$defs.${definitionName}`);
    }
    return missing;
}

function bibliographyValidator() {
    return schemaValidator(loadBibSchema());
}

function schemaValidator(schema) {
    const ajv = new Ajv2020({ allErrors: true, strict: false });
    addFormats(ajv);
    ajv.addFormat('uint8', true);
    ajv.addFormat('uint32', true);
    return ajv.compile(schema);
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

test('citation and server integration examples validate against published schemas', () => {
    const markdown = fs.readFileSync(CITATION_SERVER_GUIDE_PATH, 'utf8');
    const citationBlocks = [...markdown.matchAll(/```yaml\n([\s\S]*?)```/g)]
        .map((match) => match[1])
        .filter((block) => block.includes('# schema: citation'));
    const requestBlocks = [...markdown.matchAll(/```json server:([a-z_]+)\n([\s\S]*?)```/g)];
    const responseBlocks = [...markdown.matchAll(/```json response\n([\s\S]*?)```/g)].map(
        (match) => match[1]
    );

    assert.equal(citationBlocks.length, 1, 'expected one complete citation example');
    assert.deepEqual(requestBlocks.map(([, method]) => method).sort(), [
        'format_document',
        'render_citation',
    ]);
    assert.equal(responseBlocks.length, 2, 'expected one response for each RPC example');

    const validateCitation = schemaValidator(loadSchema(CITATION_SCHEMA_PATH));
    assertValidBibliography(
        validateCitation,
        yaml.load(citationBlocks[0]),
        'citation guide YAML block'
    );

    const server = loadSchema(SERVER_SCHEMA_PATH);
    for (const [, method, block] of requestBlocks) {
        const request = JSON.parse(block);
        assert.equal(request.method, method);
        assert.ok(request.params, `${method} example must include params`);
        const methodSchema = {
            $schema: server.$schema,
            $defs: server.$defs,
            ...server[method],
        };
        assertValidBibliography(
            schemaValidator(methodSchema),
            request.params,
            `${method} params example`
        );
    }

    for (const [index, block] of responseBlocks.entries()) {
        const response = JSON.parse(block);
        assert.ok(Object.hasOwn(response, 'id'), `response ${index + 1} must echo id`);
        assert.ok(Object.hasOwn(response, 'result'), `response ${index + 1} must include result`);
    }
});

test('citation and server schemas expose effective public descriptions', () => {
    assert.deepEqual(undocumentedProperties(loadSchema(CITATION_SCHEMA_PATH)), []);
    assert.deepEqual(undocumentedProperties(loadSchema(SERVER_SCHEMA_PATH)), []);
});

test('citation and server guide links generated schemas and references', () => {
    const markdown = fs.readFileSync(CITATION_SERVER_GUIDE_PATH, 'utf8');
    assert.match(markdown, /\.\.\/schemas\/citation\.json/);
    assert.match(markdown, /\.\.\/schemas\/server\.json/);
    assert.match(markdown, /DATA_MODEL_FIELDS\.md/);
    assert.match(markdown, /NATIVE_FORMAT\.md/);
    assert.match(markdown, /format_document/);
    assert.match(markdown, /render_citation/);
});

test('registry and abbreviation-map guide examples validate against published schemas', () => {
    const markdown = fs.readFileSync(AUXILIARY_GUIDE_PATH, 'utf8');
    const blocks = [...markdown.matchAll(/```yaml\n([\s\S]*?)```/g)].map(
        (match) => match[1]
    );
    const registryBlocks = blocks.filter((block) => block.includes('# schema: registry'));
    const abbreviationBlocks = blocks.filter((block) => block.includes('# schema: abbrev-map'));

    assert.equal(registryBlocks.length, 1, 'expected one complete registry example');
    assert.equal(abbreviationBlocks.length, 1, 'expected one complete abbreviation-map example');

    const validateRegistry = schemaValidator(loadSchema(REGISTRY_SCHEMA_PATH));
    const validateAbbreviations = schemaValidator(loadSchema(ABBREV_SCHEMA_PATH));

    assertValidBibliography(
        validateRegistry,
        yaml.load(registryBlocks[0]),
        'registry guide YAML block'
    );
    assertValidBibliography(
        validateAbbreviations,
        yaml.load(abbreviationBlocks[0]),
        'abbreviation-map guide YAML block'
    );
});

test('registry and abbreviation-map schemas expose effective public descriptions', () => {
    const registry = loadSchema(REGISTRY_SCHEMA_PATH);
    const registryFields = [
        ...Object.entries(registry.properties),
        ...Object.entries(registry.$defs.RegistryEntry.properties),
    ];
    const missingRegistryDescriptions = registryFields
        .filter(([, field]) => !field.description?.trim())
        .map(([name]) => name);

    assert.deepEqual(missingRegistryDescriptions, []);
    assert.equal(typeof registry.$defs.StyleKind.description, 'string');
    assert.ok(registry.$defs.StyleKind.description.trim());
    for (const variant of registry.$defs.StyleKind.oneOf) {
        assert.ok(variant.description?.trim(), 'every StyleKind value needs a description');
    }

    const abbreviationMap = loadSchema(ABBREV_SCHEMA_PATH);
    assert.ok(abbreviationMap.description?.trim());
    assert.match(abbreviationMap.description, /exact, case-sensitive/);
});

test('auxiliary guide identifies type-map as generated data and links its references', () => {
    const markdown = fs.readFileSync(AUXILIARY_GUIDE_PATH, 'utf8');
    const typeMap = loadSchema(path.join(ROOT, 'docs/schemas/type-map.json'));

    assert.equal(Object.hasOwn(typeMap, '$schema'), false);
    assert.deepEqual(Object.keys(typeMap).sort(), [
        'biblatex_entry_types',
        'biblatex_fields',
        'csl_type_map',
    ]);
    assert.match(markdown, /not a JSON Schema/);
    assert.match(markdown, /BIBLATEX_MAPPING\.md/);
    assert.match(markdown, /CSL_JSON_MAPPING\.md/);
    assert.match(markdown, /just schema-gen/);
});
