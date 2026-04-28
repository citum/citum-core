const fs = require('fs');
const path = require('path');
const { marked } = require('marked');

const MD_PATH = path.join(__dirname, '../docs/guides/style-author-guide.md');
const TEMPLATE_PATH = path.join(__dirname, '../docs/guides/style-author-guide.template.html');
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

// Simple YAML syntax highlighter using regex and project Tailwind colors
function highlightYaml(code) {
    const escaped = code
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;');

    return escaped
        // 1. Comments (slate-500)
        .replace(/(#.*$)/gm, '<span class="text-slate-500">$1</span>')
        // 2. Keys (indigo-400 for top-level, primary/sky-400 for nested)
        // Now handles both "key:" and "- key:"
        .replace(/^(\s*)(-?\s*)([a-z0-9_-]+)(:)/gm, (match, indent, dash, key, colon) => {
            const colorClass = indent.length === 0 && dash.length === 0 ? 'text-indigo-400' : 'text-primary';
            const dashHtml = dash.length > 0 ? `<span class="text-slate-400">${dash}</span>` : '';
            return `${indent}${dashHtml}<span class="${colorClass}">${key}</span>${colon}`;
        })
        // 3. String values (emerald-400)
        .replace(/(: )("[^"]*"|'[^']*'|[a-z0-9_.-]+)(?=\s|$|<)/gi, (match, separator, value) => {
            if (value === '~' || value === 'null') return `${separator}<span class="text-slate-400">${value}</span>`;
            return `${separator}<span class="text-emerald-400">${value}</span>`;
        });
}

let firstHeading = true;

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
    
    if (iconMatch && level === 2) {
        const icon = iconMatch[1];
        const title = iconMatch[2];
        
        let output = '';
        if (!firstHeading) {
            output += '</section>\n';
        }
        firstHeading = false;
        
        output += `
            <section id="${id}" class="scroll-mt-24">
                <h2 class="text-3xl font-bold text-slate-900 mb-6 flex items-center gap-3">
                    <span class="material-icons text-primary">${icon}</span>
                    ${title}
                </h2>
        `;
        return output;
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
            <div class="mb-8 bg-blue-50 border border-blue-200 rounded-lg p-5 not-prose citum-callout">
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
            <div class="border border-amber-500/30 bg-amber-50 p-4 rounded-lg mb-6 not-prose">
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

renderer.code = function(arg1, arg2) {
    let code, lang;
    if (typeof arg1 === 'object') {
        code = arg1.text;
        lang = arg1.lang;
    } else {
        code = arg1;
        lang = arg2;
    }

    const highlighted = (lang === 'yaml' || lang === 'yml')
        ? highlightYaml(code)
        : code.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');

    return `
        <div class="bg-slate-900 rounded-lg p-6 overflow-x-auto mb-8 not-prose workshop-block">
            <pre class="font-mono text-sm text-slate-300 leading-relaxed"><code class="language-${lang}">${highlighted}</code></pre>
        </div>
    `;
};

// Handle tables with Tailwind styles
renderer.table = function(token) {
    let header = '';
    let body = '';

    for (let i = 0; i < token.header.length; i++) {
        header += `<th class="px-4 py-3 text-left font-semibold text-slate-900">${marked.parseInline(token.header[i].text)}</th>`;
    }

    for (let i = 0; i < token.rows.length; i++) {
        body += '<tr class="hover:bg-slate-50">';
        for (let j = 0; j < token.rows[i].length; j++) {
            body += `<td class="px-4 py-3 text-slate-600">${marked.parseInline(token.rows[i][j].text)}</td>`;
        }
        body += '</tr>';
    }

    return `
        <div class="overflow-x-auto rounded-lg border border-slate-200 mb-8 not-prose citum-table-shell">
            <table class="w-full text-sm">
                <thead class="bg-slate-50">
                    <tr>${header}</tr>
                </thead>
                <tbody class="divide-y divide-slate-200">
                    ${body}
                </tbody>
            </table>
        </div>
    `;
};

marked.setOptions({ renderer });

function build() {
    console.log('Building Style Author Guide...');
    firstHeading = true;
    
    if (!fs.existsSync(MD_PATH)) {
        console.error(`Markdown file not found: ${MD_PATH}`);
        process.exit(1);
    }
    if (!fs.existsSync(TEMPLATE_PATH)) {
        console.error(`Template file not found: ${TEMPLATE_PATH}`);
        process.exit(1);
    }

    const mdContent = fs.readFileSync(MD_PATH, 'utf8');
    const templateContent = fs.readFileSync(TEMPLATE_PATH, 'utf8');
    
    let bodyHtml = marked.parse(mdContent);
    
    // Close the last section
    if (!firstHeading) {
        bodyHtml += '</section>';
    }
    
    const finalHtml = templateContent.replace('{{CONTENT}}', bodyHtml);
    
    fs.writeFileSync(HTML_PATH, finalHtml);
    console.log('Done!');
}

build();
