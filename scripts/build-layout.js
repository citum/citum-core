const fs = require('fs');
const path = require('path');

const DOCS_DIR = path.join(__dirname, '../docs');

const NAV_TEMPLATE = `
        <div class="site-nav-inner">
            <div class="site-brand-wrap">
                <a href="{{ROOT}}index.html" class="site-brand">
                    <div class="site-brand-mark"><span>C</span></div>
                    <span class="site-brand-name">Citum</span>
                </a>
            </div>
            <div class="site-nav-links" role="navigation" aria-label="Primary docs navigation">
                <a class="site-nav-link {{ACTIVE_DOCS}}" href="{{ROOT}}index.html">Overview</a>
                <a class="site-nav-link {{ACTIVE_STYLE}}" href="{{ROOT}}guides/style-authoring/start.html">Style Authoring</a>
                <a class="site-nav-link {{ACTIVE_EXAMPLES}}" href="{{ROOT}}examples.html">Examples</a>
                <a class="site-nav-link {{ACTIVE_REFERENCE}}" href="{{ROOT}}reference.html">Reference</a>
                <a class="site-nav-link {{ACTIVE_REPORTS}}" href="{{ROOT}}reports.html">Reports</a>
                <a class="site-nav-link {{ACTIVE_OPERATING}}" href="{{ROOT}}operating.html">Operating</a>
                <a class="site-nav-link" href="https://github.com/citum/citum-core">GitHub</a>
                <a class="site-source-link" href="https://github.com/citum/citum-core">
                    <svg aria-hidden="true" class="site-source-icon" fill="currentColor" viewBox="0 0 24 24">
                        <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"></path>
                    </svg>
                    Source
                </a>
            </div>
            <button type="button" class="mobile-nav-toggle" data-nav-toggle aria-expanded="false" aria-controls="mobile-nav">
                <span class="material-icons text-[20px]">menu</span>
            </button>
        </div>
        <div id="mobile-nav" class="mobile-nav hidden" data-mobile-menu>
            <a class="site-nav-link {{ACTIVE_DOCS}}" href="{{ROOT}}index.html">Overview</a>
            <a class="site-nav-link {{ACTIVE_STYLE}}" href="{{ROOT}}guides/style-authoring/start.html">Style Authoring</a>
            <a class="site-nav-link {{ACTIVE_EXAMPLES}}" href="{{ROOT}}examples.html">Examples</a>
            <a class="site-nav-link {{ACTIVE_REFERENCE}}" href="{{ROOT}}reference.html">Reference</a>
            <a class="site-nav-link {{ACTIVE_REPORTS}}" href="{{ROOT}}reports.html">Reports</a>
            <a class="site-nav-link {{ACTIVE_OPERATING}}" href="{{ROOT}}operating.html">Operating</a>
            <a class="site-nav-link" href="https://github.com/citum/citum-core">GitHub</a>
        </div>`;

const FOOTER_TEMPLATE = `
            <div class="site-footer-inner">
                <div class="site-brand site-brand-small">
                    <div class="site-brand-mark"><span>C</span></div>
                    <span class="site-brand-name">Citum</span>
                </div>
                <div class="site-footer-links">
                    <a href="{{ROOT}}guides/style-authoring/start.html">Style Authoring</a>
                    <a href="{{ROOT}}reference.html">Reference</a>
                    <a href="{{ROOT}}reports.html">Reports</a>
                    <a href="https://github.com/citum/citum-core">GitHub</a>
                </div>
                <div class="site-footer-note">© 2026 Citum Project.</div>
            </div>`;

const ACTIVE_CLASS = 'active';

function getFiles(dir, fileList = []) {
    const files = fs.readdirSync(dir);
    for (const file of files) {
        const filepath = path.join(dir, file);
        if (fs.statSync(filepath).isDirectory()) {
            getFiles(filepath, fileList);
        } else if (filepath.endsWith('.html') && !filepath.endsWith('.template.html') && !filepath.includes('/news/posts/')) {
            fileList.push(filepath);
        }
    }
    return fileList;
}

function activeFor(pageId, target) {
    return pageId === target ? ACTIVE_CLASS : '';
}

function buildLayouts() {
    console.log('Centralizing Site Layout...');
    const files = getFiles(DOCS_DIR);

    for (const file of files) {
        let content = fs.readFileSync(file, 'utf8');
        let modified = false;

        const idMatch = content.match(/<!-- PAGE_ID:\s*([a-zA-Z0-9-]+)\s*-->/);
        const pageId = idMatch ? idMatch[1] : '';
        const relative = path.relative(path.dirname(file), DOCS_DIR);
        const rootPrefix = relative ? `${relative}/` : '';

        const navHtml = NAV_TEMPLATE.replace(/{{ROOT}}/g, rootPrefix)
            .replace(/{{ACTIVE_DOCS}}/g, activeFor(pageId, 'docs'))
            .replace(/{{ACTIVE_STYLE}}/g, activeFor(pageId, 'style'))
            .replace(/{{ACTIVE_EXAMPLES}}/g, activeFor(pageId, 'examples'))
            .replace(/{{ACTIVE_REFERENCE}}/g, activeFor(pageId, 'reference'))
            .replace(/{{ACTIVE_REPORTS}}/g, activeFor(pageId, 'reports'))
            .replace(/{{ACTIVE_OPERATING}}/g, activeFor(pageId, 'operating'));

        const footerHtml = FOOTER_TEMPLATE.replace(/{{ROOT}}/g, rootPrefix);

        const navRegex = /(<!-- LAYOUT_NAV_START -->)([\s\S]*?)(<!-- LAYOUT_NAV_END -->)/;
        if (navRegex.test(content)) {
            content = content.replace(navRegex, `$1${navHtml}        $3`);
            modified = true;
        }

        const footerRegex = /(<!-- LAYOUT_FOOTER_START -->)([\s\S]*?)(<!-- LAYOUT_FOOTER_END -->)/;
        if (footerRegex.test(content)) {
            content = content.replace(footerRegex, `$1${footerHtml}        $3`);
            modified = true;
        }

        if (modified) {
            fs.writeFileSync(file, content);
            console.log(`Updated layout in ${path.relative(DOCS_DIR, file)}`);
        }
    }
    console.log('Done!');
}

buildLayouts();
