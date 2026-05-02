const fs = require('fs');
const path = require('path');

const DOCS_DIR = path.join(__dirname, '../docs');

const NAV_TEMPLATE = `
        <div class="max-w-7xl mx-auto px-6 h-16 flex items-center justify-between">
            <div class="flex items-center gap-2 shrink-0">
                <a href="{{ROOT}}index.html" class="flex items-center gap-2 group">
                    <div class="w-8 h-8 bg-primary rounded flex items-center justify-center group-hover:brightness-110 transition-all">
                        <span class="text-white font-mono font-bold">C</span>
                    </div>
                    <span class="font-mono text-xl font-bold tracking-tight text-slate-900">Citum</span>
                </a>
            </div>
            <div class="hidden md:flex items-center gap-3 lg:gap-4 xl:gap-6 min-w-0 overflow-x-auto whitespace-nowrap pl-4">
                <a class="nav-link" href="https://citum.org">Home</a>
                <a class="nav-link" href="https://citum.org/news/index.html">News</a>
                <a class="nav-link {{ACTIVE_DOCS}}" href="{{ROOT}}index.html">Docs</a>
                <a class="nav-link {{ACTIVE_DEMO}}" href="{{ROOT}}interactive-demo.html">Demo</a>
                <a class="nav-link {{ACTIVE_EXAMPLES}}" href="{{ROOT}}examples.html">Examples</a>
                <a class="nav-link {{ACTIVE_STYLE}}" href="{{ROOT}}guides/style-author-guide.html">Style Guide</a>
                <a class="nav-link {{ACTIVE_REPORTS}}" href="{{ROOT}}reports.html">Reports</a>
                <a class="nav-link" href="https://github.com/citum/citum-core">GitHub</a>
                <a class="hidden xl:flex btn-primary ml-2" href="https://github.com/citum/citum-core">
                    <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 24 24">
                        <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"></path>
                    </svg>
                    Source
                </a>
            </div>
            <button type="button" class="md:hidden inline-flex items-center justify-center rounded-lg border border-slate-200 bg-[var(--citum-surface)]/80 px-3 py-2 text-slate-700" data-nav-toggle aria-expanded="false" aria-controls="mobile-nav">
                <span class="material-icons text-[20px]">menu</span>
            </button>
        </div>
        <div id="mobile-nav" class="md:hidden hidden border-t border-slate-200 bg-background-light/95 px-6 py-4" data-mobile-menu>
            <div class="flex flex-col gap-3 text-sm font-medium text-slate-700">
                <a class="hover:text-primary transition-colors" href="https://citum.org">Home</a>
                <a class="hover:text-primary transition-colors" href="https://citum.org/news/index.html">News</a>
                <a class="nav-link {{ACTIVE_DOCS}}" href="{{ROOT}}index.html">Docs</a>
                <a class="nav-link {{ACTIVE_DEMO}}" href="{{ROOT}}interactive-demo.html">Demo</a>
                <a class="nav-link {{ACTIVE_EXAMPLES}}" href="{{ROOT}}examples.html">Examples</a>
                <a class="nav-link {{ACTIVE_STYLE}}" href="{{ROOT}}guides/style-author-guide.html">Style Guide</a>
                <a class="nav-link {{ACTIVE_REPORTS}}" href="{{ROOT}}reports.html">Reports</a>
                <a class="hover:text-primary transition-colors" href="https://github.com/citum/citum-core">GitHub</a>
            </div>
        </div>`;

const FOOTER_TEMPLATE = `
            <div class="max-w-7xl mx-auto flex flex-col md:flex-row justify-between items-center gap-8">
                <div class="flex items-center gap-2">
                    <div class="w-6 h-6 bg-primary rounded flex items-center justify-center">
                        <span class="text-white font-mono text-xs font-bold">C</span>
                    </div>
                    <span class="font-mono text-lg font-bold text-slate-900">Citum</span>
                </div>
                <div class="flex gap-8 text-sm font-medium text-slate-500">
                    <a class="hover:text-primary transition-colors" href="https://github.com/citum/citum-core">GitHub</a>
                    <a class="hover:text-primary transition-colors" href="{{ROOT}}examples.html">Examples</a>
                    <a class="hover:text-primary transition-colors" href="{{ROOT}}reports.html">Reports</a>
                </div>
                <div class="text-sm text-slate-400">
                    © 2026 Citum Project.
                </div>
            </div>`;

function getFiles(dir, fileList = []) {
    const files = fs.readdirSync(dir);
    for (const file of files) {
        const filepath = path.join(dir, file);
        if (fs.statSync(filepath).isDirectory()) {
            getFiles(filepath, fileList);
        } else if (filepath.endsWith('.html') && !filepath.includes('/news/posts/')) {
            fileList.push(filepath);
        }
    }
    return fileList;
}

function buildLayouts() {
    console.log('Centralizing Site Layout...');
    const files = getFiles(DOCS_DIR);

    for (const file of files) {
        let content = fs.readFileSync(file, 'utf8');
        let modified = false;

        // Determine active tab via <!-- PAGE_ID: xxx -->
        const idMatch = content.match(/<!-- PAGE_ID:\s*([a-zA-Z0-9-]+)\s*-->/);
        const pageId = idMatch ? idMatch[1] : '';

        // Calculate root prefix
        const relative = path.relative(path.dirname(file), DOCS_DIR);
        const rootPrefix = relative ? relative + '/' : '';

        const activeClass = 'active bg-primary/10 text-primary font-semibold';
        
        let navHtml = NAV_TEMPLATE.replace(/{{ROOT}}/g, rootPrefix)
            .replace(/{{ACTIVE_DOCS}}/g, pageId === 'docs' ? activeClass : 'text-slate-600')
            .replace(/{{ACTIVE_DEMO}}/g, pageId === 'demo' ? activeClass : 'text-slate-600')
            .replace(/{{ACTIVE_EXAMPLES}}/g, pageId === 'examples' ? activeClass : 'text-slate-600')
            .replace(/{{ACTIVE_STYLE}}/g, pageId === 'style' ? activeClass : 'text-slate-600')
            .replace(/{{ACTIVE_REPORTS}}/g, pageId === 'reports' ? activeClass : 'text-slate-600');
            
        let footerHtml = FOOTER_TEMPLATE.replace(/{{ROOT}}/g, rootPrefix);

        // Inject Nav
        const navRegex = /(<!-- LAYOUT_NAV_START -->)([\s\S]*?)(<!-- LAYOUT_NAV_END -->)/;
        if (navRegex.test(content)) {
            content = content.replace(navRegex, '$1' + navHtml + '        $3');
            modified = true;
        }

        // Inject Footer
        const footerRegex = /(<!-- LAYOUT_FOOTER_START -->)([\s\S]*?)(<!-- LAYOUT_FOOTER_END -->)/;
        if (footerRegex.test(content)) {
            content = content.replace(footerRegex, '$1' + footerHtml + '        $3');
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
