const fs = require('fs');
const path = require('path');
const yaml = require('js-yaml');

const ROOT = path.join(__dirname, '..');
const FEATURES_PATH = path.join(ROOT, 'docs/reference/features.yaml');
const DOCS_DIR = path.join(ROOT, 'docs');
const VALID_STATUS = new Set(['active', 'preview', 'experimental', 'planned']);
const SEMVER = /^\d+\.\d+\.\d+$/;

function warn(message) {
    console.warn(`docs-metadata warning: ${message}`);
}

function readFeatures() {
    const doc = yaml.load(fs.readFileSync(FEATURES_PATH, 'utf8'));
    const features = Array.isArray(doc?.features) ? doc.features : [];
    const ids = new Set();

    for (const feature of features) {
        if (!feature || typeof feature !== 'object') {
            warn('feature entry must be an object');
            continue;
        }
        if (!feature.id) warn('feature entry missing id');
        if (feature.id && ids.has(feature.id)) warn(`duplicate feature id '${feature.id}'`);
        if (feature.id) ids.add(feature.id);
        if (!feature.title) warn(`feature '${feature.id}' missing title`);
        if (!VALID_STATUS.has(feature.status)) {
            warn(`feature '${feature.id}' has invalid status '${feature.status}'`);
        }
        if (!SEMVER.test(String(feature.since_schema || ''))) {
            warn(`feature '${feature.id}' missing semver since_schema`);
        }
        if (!SEMVER.test(String(feature.since_engine || ''))) {
            warn(`feature '${feature.id}' missing semver since_engine`);
        }
        if (feature.spec) {
            const specPath = path.join(ROOT, feature.spec);
            if (!fs.existsSync(specPath)) warn(`feature '${feature.id}' spec does not exist: ${feature.spec}`);
        }
    }

    return ids;
}

function walk(dir, files = []) {
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
        if (entry.name === 'node_modules') continue;
        const target = path.join(dir, entry.name);
        if (entry.isDirectory()) walk(target, files);
        if (entry.isFile() && /\.(md|html)$/.test(entry.name)) files.push(target);
    }
    return files;
}

function checkFeatureReferences(featureIds) {
    for (const file of walk(DOCS_DIR)) {
        const rel = path.relative(ROOT, file);
        const content = fs.readFileSync(file, 'utf8');
        const refs = [...content.matchAll(/data-feature="([^"]+)"/g)].map((match) => match[1]);
        for (const ref of refs) {
            if (!featureIds.has(ref)) warn(`${rel} references unknown feature '${ref}'`);
        }

        if (/docs\/guides\/style-authoring\/.+\.md$/.test(rel) && !content.includes('features:')) {
            warn(`${rel} has no feature metadata`);
        }
    }
}

function main() {
    if (!fs.existsSync(FEATURES_PATH)) {
        warn(`missing ${path.relative(ROOT, FEATURES_PATH)}`);
        return;
    }
    const featureIds = readFeatures();
    checkFeatureReferences(featureIds);
}

main();
