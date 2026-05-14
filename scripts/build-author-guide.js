const fs = require('fs');
const path = require('path');
const { marked } = require('marked');
const yaml = require('js-yaml');

const ROOT = path.join(__dirname, '..');
const SOURCE_DIR = path.join(ROOT, 'docs/guides/style-authoring');
const TEMPLATE_PATH = path.join(ROOT, 'docs/guides/style-author-guide.template.html');
const LEGACY_HTML_PATH = path.join(ROOT, 'docs/guides/style-author-guide.html');
const FEATURES_PATH = path.join(ROOT, 'docs/reference/features.yaml');

const GUIDE_PAGES = [
    'start',
    'style-anatomy',
    'templates',
    'options',
    'locales',
    'inheritance-and-registries',
    'validation',
];

function parseFrontmatter(raw) {
    if (!raw.startsWith('---\n')) return [{}, raw];
    const end = raw.indexOf('\n---\n', 4);
    if (end === -1) return [{}, raw];
    const data = yaml.load(raw.slice(4, end)) || {};
    return [data, raw.slice(end + 5)];
}

function slugify(text) {
    return text
        .toLowerCase()
        .replace(/\[[a-z0-9_]+\]/, '')
        .trim()
        .replace(/[^\w]+/g, '-')
        .replace(/^-+|-+$/g, '');
}

function escapeHtml(text) {
    return String(text)
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;');
}

function highlightYaml(code) {
    return escapeHtml(code)
        .replace(/(#.*$)/gm, '<span class="text-slate-500">$1</span>')
        .replace(/^(\s*)(-?\s*)([a-z0-9_-]+)(:)/gm, (match, indent, dash, key, colon) => {
            const colorClass = indent.length === 0 && dash.length === 0 ? 'text-indigo-400' : 'text-primary';
            const dashHtml = dash.length > 0 ? `<span class="text-slate-400">${dash}</span>` : '';
            return `${indent}${dashHtml}<span class="${colorClass}">${key}</span>${colon}`;
        })
        .replace(/(: )("[^"]*"|'[^']*'|[a-z0-9_.-]+)(?=\s|$|<)/gi, (match, separator, value) => {
            if (value === '~' || value === 'null') return `${separator}<span class="text-slate-400">${value}</span>`;
            return `${separator}<span class="text-emerald-400">${value}</span>`;
        });
}

function createRenderer() {
    const renderer = new marked.Renderer();

    renderer.heading = function(arg1, arg2) {
        const text = typeof arg1 === 'object' ? arg1.text : arg1;
        const level = typeof arg1 === 'object' ? arg1.depth : arg2;
        const iconMatch = text.match(/^\[([a-z0-9_]+)\]\s*(.*)/);
        const cleanTitle = iconMatch ? iconMatch[2] : text;
        const id = slugify(text);
        const icon = iconMatch ? `<span class="material-icons text-primary" aria-hidden="true">${iconMatch[1]}</span>` : '';
        return `<h${level} id="${id}">${icon}${cleanTitle}</h${level}>`;
    };

    renderer.blockquote = function(arg1) {
        const text = typeof arg1 === 'object' ? arg1.text : arg1;
        const kind = text.includes('[!WARNING]') ? 'warning' : 'tip';
        const label = kind === 'warning' ? 'warning' : 'tips_and_updates';
        const content = marked.parseInline(text.replace(/\[!(TIP|WARNING)\]\s*/, ''));
        return `
            <div class="callout ${kind}">
                <span class="material-icons" aria-hidden="true">${label}</span>
                <div>${content}</div>
            </div>
        `;
    };

    renderer.code = function(arg1, arg2) {
        const code = typeof arg1 === 'object' ? arg1.text : arg1;
        const lang = typeof arg1 === 'object' ? arg1.lang : arg2;
        const highlighted = (lang === 'yaml' || lang === 'yml') ? highlightYaml(code) : escapeHtml(code);
        return `
            <div class="workshop-block">
                <pre><code class="language-${lang || 'text'}">${highlighted}</code></pre>
            </div>
        `;
    };

    renderer.table = function(token) {
        const header = token.header
            .map((cell) => `<th>${marked.parseInline(cell.text)}</th>`)
            .join('');
        const body = token.rows
            .map((row) => `<tr>${row.map((cell) => `<td>${marked.parseInline(cell.text)}</td>`).join('')}</tr>`)
            .join('');
        return `
            <div class="doc-table-shell">
                <table class="doc-table">
                    <thead><tr>${header}</tr></thead>
                    <tbody>${body}</tbody>
                </table>
            </div>
        `;
    };

    return renderer;
}

function loadFeatures() {
    if (!fs.existsSync(FEATURES_PATH)) return new Map();
    const doc = yaml.load(fs.readFileSync(FEATURES_PATH, 'utf8'));
    return new Map((doc.features || []).map((feature) => [feature.id, feature]));
}

function readPage(slug) {
    const sourcePath = path.join(SOURCE_DIR, `${slug}.md`);
    const [frontmatter, body] = parseFrontmatter(fs.readFileSync(sourcePath, 'utf8'));
    return { slug, sourcePath, frontmatter, body };
}

function renderSidebar(currentSlug, pages, rootPrefix) {
    return pages.map((page) => {
        const active = page.slug === currentSlug ? ' active' : '';
        return `<a class="doc-sidebar-link${active}" href="${rootPrefix}guides/style-authoring/${page.slug}.html">${page.frontmatter.nav || page.frontmatter.title}</a>`;
    }).join('\n');
}

function renderFeatureBadges(featureIds, featureMap) {
    if (!Array.isArray(featureIds) || featureIds.length === 0) return '';
    const badges = [];
    for (const id of featureIds) {
        const feature = featureMap.get(id);
        if (!feature) {
            badges.push(`<span class="version-badge" data-feature="${id}">unknown feature: ${id}</span>`);
            continue;
        }
        badges.push(`<span class="version-badge" data-feature="${id}" data-status="${feature.status}">${feature.status}</span>`);
        badges.push(`<span class="version-badge" data-feature="${id}">schema ${feature.since_schema}+</span>`);
        badges.push(`<span class="version-badge" data-feature="${id}">engine ${feature.since_engine}+</span>`);
    }
    return `<div class="version-badges">${badges.join('')}</div>`;
}

function renderPage(page, pages, featureMap, options = {}) {
    const renderer = createRenderer();
    marked.setOptions({ renderer });
    const rootPrefix = options.rootPrefix || '../../';
    const content = marked.parse(page.body);
    const featureBadges = renderFeatureBadges(page.frontmatter.features, featureMap);
    const template = fs.readFileSync(TEMPLATE_PATH, 'utf8');

    return template
        .replace(/{{PAGE_ID}}/g, 'style')
        .replace(/{{ROOT}}/g, rootPrefix)
        .replace(/{{TITLE}}/g, escapeHtml(page.frontmatter.title || 'Style Authoring'))
        .replace(/{{DESCRIPTION}}/g, escapeHtml(page.frontmatter.description || 'Citum style authoring documentation.'))
        .replace(/{{SIDEBAR}}/g, renderSidebar(page.slug, pages, rootPrefix))
        .replace(/{{FEATURE_BADGES}}/g, featureBadges)
        .replace(/{{CONTENT}}/g, content)
        .replace(/[ \t]+$/gm, '');
}

function build() {
    console.log('Building Style Authoring Guide...');
    const featureMap = loadFeatures();
    const pages = GUIDE_PAGES.map(readPage);

    for (const page of pages) {
        const htmlPath = path.join(SOURCE_DIR, `${page.slug}.html`);
        fs.writeFileSync(htmlPath, renderPage(page, pages, featureMap));
        console.log(`Wrote ${path.relative(ROOT, htmlPath)}`);
    }

    const start = pages[0];
    fs.writeFileSync(LEGACY_HTML_PATH, renderPage(start, pages, featureMap, { rootPrefix: '../' }));
    console.log(`Wrote ${path.relative(ROOT, LEGACY_HTML_PATH)}`);
}

build();
