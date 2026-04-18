const fs = require('fs');
const path = require('path');
const { marked } = require('marked');

const MD_PATH = path.join(__dirname, '../docs/guides/style-author-guide.md');
const HTML_PATH = path.join(__dirname, '../docs/guides/style-author-guide.html');

const renderer = new marked.Renderer();

// Helper to generate IDs consistent with the sidebar
function slugify(text) {
    return text
        .toLowerCase()
        .replace(/\[[a-z0-9_]+\]/, '') // strip icons
        .trim()
        .replace(/[^\w]+/g, '-')
        .replace(/^-+|-+$/g, '');
}

renderer.heading = function(arg1, arg2) {
    let text, level;
    if (typeof arg1 === 'object') {
        text = arg1.text;
        level = arg1.depth;
    } else {
        text = arg1;
        level = arg2;
    }
    
    const iconMatch = text.match(/^\[([a-z0-9_]+)\]\s*(.*)/);
    const id = slugify(text);
    
    if (iconMatch) {
        const icon = iconMatch[1];
        const title = iconMatch[2];
        const sizeClass = level === 2 ? 'text-3xl' : 'text-xl';
        const mtClass = level === 2 ? 'mb-6' : 'mb-4';
        
        return `
            </section>
            <section id="${id}" class="scroll-mt-24">
                <h${level} class="${sizeClass} font-bold text-slate-900 ${mtClass} flex items-center gap-3">
                    <span class="material-icons text-primary">${icon}</span>
                    ${title}
                </h${level}>
        `;
    }
    return `<h${level} id="${id}" class="font-bold text-slate-900 mt-8 mb-4">${text}</h${level}>`;
};

renderer.blockquote = function(arg1) {
    let text;
    if (typeof arg1 === 'object') {
        text = arg1.text;
    } else {
        text = arg1;
    }

    const content = marked.parseInline(text.replace(/\[!(TIP|WARNING)\]\s*/, ''));

    if (text.includes('[!TIP]')) {
        return `
            <div class="mb-8 bg-blue-50 border border-blue-200 rounded-lg p-5 not-prose">
                <div class="flex items-start gap-3">
                    <span class="material-icons text-blue-600 mt-0.5">tips_and_updates</span>
                    <div class="flex-1 text-blue-800 text-sm">
                        ${content}
                    </div>
                </div>
            </div>
        `;
    }
    if (text.includes('[!WARNING]')) {
        return `
            <div class="border-l-4 border-amber-500 bg-amber-50 p-4 rounded mb-6 not-prose">
                <div class="flex gap-3">
                    <span class="material-icons text-amber-600 flex-shrink-0">warning</span>
                    <div class="text-amber-800 text-sm">
                        ${content}
                    </div>
                </div>
            </div>
        `;
    }
    return `<blockquote>${content}</blockquote>`;
};

// Ensure code blocks look like the rest of the site
renderer.code = function(arg1, arg2) {
    let code, lang;
    if (typeof arg1 === 'object') {
        code = arg1.text;
        lang = arg1.lang;
    } else {
        code = arg1;
        lang = arg2;
    }
    return `
        <div class="bg-slate-900 rounded-lg p-6 overflow-x-auto mb-8 not-prose">
            <pre class="font-mono text-sm text-slate-300 leading-relaxed"><code class="language-${lang}">${code.replace(/</g, '&lt;')}</code></pre>
        </div>
    `;
};

marked.setOptions({ renderer });

function build() {
    console.log('Building Style Author Guide...');
    
    const mdContent = fs.readFileSync(MD_PATH, 'utf8');
    const htmlContent = fs.readFileSync(HTML_PATH, 'utf8');
    
    let bodyHtml = marked.parse(mdContent);
    
    // Cleanup first empty section tag if it exists
    bodyHtml = bodyHtml.replace(/^<\/section>/, '');
    if (!bodyHtml.endsWith('</section>')) {
        bodyHtml += '</section>';
    }

    const mainStartTag = /<main[^>]*>/;
    const mainEndTag = /<\/main>/;
    
    const startMatch = htmlContent.match(mainStartTag);
    const endMatch = htmlContent.match(mainEndTag);
    
    if (!startMatch || !endMatch) {
        console.error('Could not find <main> tags in template!');
        process.exit(1);
    }
    
    const head = htmlContent.substring(0, startMatch.index);
    const tail = htmlContent.substring(endMatch.index + endMatch[0].length);
    
    // Wrap in prose class for automatic styling of tables/paragraphs/etc
    const finalHtml = head + 
        '<main class="flex-1 min-w-0 space-y-16 pb-24 prose prose-slate prose-blue max-w-none prose-headings:m-0 prose-p:leading-relaxed prose-pre:p-0 prose-pre:bg-transparent">' + 
        '\n' + bodyHtml + '\n' + 
        '</main>' + 
        tail;
    
    fs.writeFileSync(HTML_PATH, finalHtml);
    console.log('Done!');
}

build();
