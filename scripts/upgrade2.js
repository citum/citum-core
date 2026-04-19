const fs = require("fs");
const yaml = require("js-yaml");
const path = require("path");

const report = require("/tmp/core-report3.json");
const needsUpgrade = report.styles.filter(s => s.fidelityScore < 0.95 || s.qualityScore < 0.90);

console.log(`Upgrading ${needsUpgrade.length} styles...`);

for (const s of needsUpgrade) {
    const yamlPath = path.join(__dirname, "..", "styles", `${s.name}.yaml`);
    if (!fs.existsSync(yamlPath)) continue;
    let content = fs.readFileSync(yamlPath, "utf8");
    let baseData;
    try {
        baseData = yaml.load(content);
    } catch (e) {
        continue;
    }
    let modified = false;

    // We can confidently map specific contributors blocks to strings.
    // Let's do string replacement for the most common ones.

    // APA
    let apaRegex = /contributors:\n\s+initialize-with: "\. "\n\s+editor-label-format: short-suffix\n\s+demote-non-dropping-particle: never\n\s+delimiter-precedes-last: always\n\s+and:\n\s+mode-dependent:\n\s+integral: text\n\s+non-integral: symbol\n\s+shorten:\n\s+min: 21\n\s+use-first: 19\n\s+use-last: 1/g;
    if (apaRegex.test(content)) {
        content = content.replace(apaRegex, "contributors: apa");
        modified = true;
    } else if (content.includes("initialize-with: \". \"") && content.includes("min: 21") && content.includes("use-first: 19")) {
        // more fuzzy apa
        // content = content.replace(/contributors:[\s\S]*?use-last: 1/, "contributors: apa");
        // modified = true;
    }

    // Harvard
    if (content.match(/contributors:.*initialize-with: \..*delimiter-precedes-last: always.*and: text.*display-as-sort: all/s)) {
        // Wait, regex might be too dangerous unless precise. Let's do js-yaml modification and use js-yaml dump.
    }

    // We will do a generic read and write using string replacement for known chunks
}
