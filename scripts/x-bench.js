import fs from "fs";
import CSL from "citeproc";

const styleXml = fs.readFileSync("../styles-legacy/apa.csl", "utf8");
const localeXml = fs.readFileSync("./locales-en-US.xml", "utf8");

const items = Object.fromEntries(
  Array.from({ length: 800 }, (_, i) => [
    `item-${i}`,
    {
      id: `item-${i}`,
      type: "article-journal",
      title: `Title ${i}`,
      author: [{ family: "Smith", given: "J." }],
      issued: { "date-parts": [[2020]] },
      "container-title": "Some Journal",
      volume: "1",
      page: "1-10",
    },
  ])
);

const itemIds = Object.keys(items);

const sys = {
  retrieveLocale: () => localeXml,
  retrieveItem: (id) => items[id],
};

function renderOnce() {
  const engine = new CSL.Engine(sys, styleXml);
  engine.updateItems(itemIds);
  return engine.makeBibliography();
}

// Single measured run
const t0 = Bun.nanoseconds();
const bib = renderOnce();
const t1 = Bun.nanoseconds();
const singleMs = (t1 - t0) / 1_000_000;

process.stdout.write(`Rendered ${bib[1].length} entries\n`);
process.stdout.write(`Single run (inside Bun): ${singleMs.toFixed(2)} ms\n`);

// Warmed average
const warmup = 5;
const iterations = 25;
const times = [];

for (let i = 0; i < warmup; i++) {
  renderOnce();
}

for (let i = 0; i < iterations; i++) {
  const start = Bun.nanoseconds();
  renderOnce();
  const end = Bun.nanoseconds();
  times.push((end - start) / 1_000_000);
}

const total = times.reduce((a, b) => a + b, 0);
const avg = total / times.length;
const min = Math.min(...times);
const max = Math.max(...times);

process.stdout.write(`Average over ${iterations} warm runs: ${avg.toFixed(2)} ms\n`);
process.stdout.write(`Min/Max warm runs: ${min.toFixed(2)} / ${max.toFixed(2)} ms\n`);
