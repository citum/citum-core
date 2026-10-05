const REFS_JSON = JSON.stringify({
  smith2020: {
    id: 'smith2020',
    class: 'monograph',
    type: 'book',
    title: 'Sample Work',
    author: [{ family: 'Smith', given: 'Jane' }],
    issued: '2020',
  },
});

const CITATION_JSON = JSON.stringify({
  id: 'c1',
  items: [{ id: 'smith2020' }],
});

const MHRA_BIB_REFS_JSON = JSON.stringify({
  smith2020: {
    id: 'smith2020',
    class: 'monograph',
    type: 'book',
    title: 'Sample Work',
    author: [{ family: 'Smith', given: 'Jane' }],
    issued: '2020',
    publisher: 'Example Press',
  },
});

const APA_CITATION =
  '(<span class="citum-citation" data-ref="smith2020">Smith, <span class="citum-issued">2020</span></span>)';
const APA_BIBLIOGRAPHY = `<div class="citum-bibliography citum-bibliography--hanging-indent">
<div class="citum-entry" id="ref-smith2020" data-author="Smith" data-year="2020" data-title="Sample Work"><span class="citum-author">Smith, J. </span><span class="citum-issued">(2020)</span>. <span class="citum-title"><em>Sample Work</em></span></div>
</div>`;
const MHRA_CITATION =
  '<span class="citum-citation" data-ref="smith2020">Jane Smith, <span class="citum-title">“<em>Sample Work</em>”</span><span class="citum-issued"> (2020)</span></span>.';
const MHRA_BIBLIOGRAPHY = `<div class="citum-bibliography citum-bibliography--hanging-indent">
<div class="citum-entry" id="ref-smith2020" data-author="Smith" data-year="2020" data-title="Sample Work"><span class="citum-author">Smith, Jane</span> <span class="citum-title"><em>Sample Work</em></span><span class="citum-publisher"> (Example Press</span><span class="citum-issued">, 2020)</span></div>
</div>`;

function equal(actual, expected, label) {
  if (actual !== expected) {
    throw new Error(`${label}\nexpected: ${expected}\nactual: ${actual}`);
  }
}

function truthy(value, label) {
  if (!value) throw new Error(label);
}

export function runJsrPackageSmoke(api) {
  const {
    DocumentSession,
    formatDocument,
    getStyleMetadata,
    materializeStyle,
    renderBibliography,
    renderCitation,
    validateStyle,
  } = api;

  validateStyle('id: apa-7th');
  validateStyle('id: mhra');

  const metadata = JSON.parse(getStyleMetadata('id: apa-7th'));
  equal(metadata.title, 'American Psychological Association 7th edition', 'APA metadata title');
  const mhraMetadata = JSON.parse(getStyleMetadata('id: mhra'));
  equal(mhraMetadata.title, 'MHRA Style Guide 4th edition (notes)', 'MHRA metadata title');
  equal(mhraMetadata['default-locale'], 'en-GB', 'MHRA default locale');
  equal(renderCitation('id: apa-7th', REFS_JSON, CITATION_JSON), APA_CITATION, 'APA citation');
  equal(renderBibliography('id: apa-7th', REFS_JSON), APA_BIBLIOGRAPHY, 'APA bibliography');
  equal(renderCitation('id: mhra', REFS_JSON, CITATION_JSON), MHRA_CITATION, 'MHRA citation');
  equal(
    renderBibliography('id: mhra', MHRA_BIB_REFS_JSON),
    MHRA_BIBLIOGRAPHY,
    'MHRA bibliography'
  );

  const materialized = materializeStyle('id: apa-7th');
  truthy(
    /^version: \d+\.\d+\.\d+\ninfo:\n/.test(materialized),
    'materialized style has schema version'
  );
  truthy(!materialized.includes('\nextends:'), 'materialized style must not retain extends');
  truthy(!materialized.includes('template-ref:'), 'materialized style must not retain template refs');

  const document = JSON.parse(
    formatDocument(
      JSON.stringify({
        style: { kind: 'id', value: 'apa-7th' },
        output_format: 'html',
        refs: JSON.parse(REFS_JSON),
        citations: [{ id: 'c1', items: [{ id: 'smith2020' }] }],
      })
    )
  );
  equal(document.formatted_citations[0].text, APA_CITATION, 'formatDocument citation');
  equal(document.bibliography.content, APA_BIBLIOGRAPHY, 'formatDocument bibliography');

  const session = new DocumentSession('id: apa-7th', REFS_JSON);
  const batch = JSON.parse(session.insert_citations_batch(`[${CITATION_JSON}]`));
  equal(batch.affected_citations[0].text, APA_CITATION, 'DocumentSession citation');
  equal(batch.bibliography.content, APA_BIBLIOGRAPHY, 'DocumentSession bibliography');
  session.dispose();

  let unknownIdError = '';
  try {
    validateStyle('id: not-a-builtin');
  } catch (error) {
    unknownIdError = String(error);
  }
  equal(
    unknownIdError,
    "Style ID 'not-a-builtin' is not bundled with @citum/engine; fetch version-pinned style YAML and pass the YAML content instead",
    'unknown style ID error'
  );

  return { apa: true, mhra: true, document: true, session: true };
}
