/*
 * Citum Interactive Behaviors
 * Powered by Citum WASM Engine
 * SPDX-License-Identifier: MPL-2.0
 */

import init, { renderCitation, renderBibliography } from './wasm/citum_bindings.js';

(function() {
  'use strict';

  const CITUM = {
    refsJson: null,
    currentStyleYaml: null,
    styles: {},

    init: async function() {
      console.log('Citum: Initializing WASM engine...');
      try {
        await init();
        await this.loadData();
        this.setupStyleSwitcher();
        await this.updateContent();
        this.setupInteractivity();
      } catch (err) {
        console.error('Citum initialization failed:', err);
      }
    },

    loadData: async function() {
      // Load references
      const refsRes = await fetch('assets/data/refs.json');
      this.refsJson = await refsRes.text();

      // Load styles
      const styleFiles = [
        'apa-7th.yaml',
        'ieee.yaml',
        'nature.yaml',
        'chicago-author-date-18th.yaml'
      ];

      for (const styleFile of styleFiles) {
        const res = await fetch(`assets/data/${styleFile}`);
        this.styles[styleFile] = await res.text();
      }

      this.currentStyleYaml = this.styles['apa-7th.yaml'];
    },

    setupStyleSwitcher: function() {
      const select = document.getElementById('style-select');
      if (select) {
        select.addEventListener('change', async (e) => {
          this.currentStyleYaml = this.styles[e.target.value];
          await this.updateContent();
          this.setupInteractivity(); // Re-bind events to new elements
        });
      }
    },

    updateContent: async function() {
      if (!this.currentStyleYaml || !this.refsJson) return;

      console.log('Citum: Re-rendering citations and bibliography...');

      // Render all citations
      const citationElements = document.querySelectorAll('.citum-citation');
      for (const el of citationElements) {
        const ids = el.getAttribute('data-ref').split(' ');
        const mode = el.getAttribute('data-mode') || 'NonIntegral';
        
        const citationObj = {
          id: ids.join('-'),
          items: ids.map(id => ({ id }))
        };

        try {
          const html = renderCitation(
            this.currentStyleYaml,
            this.refsJson,
            JSON.stringify(citationObj),
            mode
          );
          // Replace the element entirely to avoid nested .citum-citation spans
          const temp = document.createElement('div');
          temp.innerHTML = html;
          const rendered = temp.firstElementChild;
          if (rendered) {
            // Restore metadata attributes for future re-renders
            rendered.setAttribute('data-ref', ids.join(' '));
            rendered.setAttribute('data-mode', mode);
            el.replaceWith(rendered);
          }
        } catch (e) {
          console.error(`Failed to render citation for [${ids}]:`, e);
        }
      }

      // Render bibliography
      const bibOutput = document.getElementById('bibliography-output');
      if (bibOutput) {
        try {
          const html = renderBibliography(this.currentStyleYaml, this.refsJson);
          bibOutput.innerHTML = html;
        } catch (e) {
          console.error('Failed to render bibliography:', e);
        }
      }
    },

    setupInteractivity: function() {
      this.setupCitations();
      this.setupBibliography();
      this.setupTooltips();
    },

    setupCitations: function() {
      const citations = document.querySelectorAll('.citum-citation');
      citations.forEach(citation => {
        // Remove existing listeners by cloning (simple way to clear)
        // Note: replaceWith above already creates fresh elements, but we clone for safety
        const newCitation = citation.cloneNode(true);
        citation.parentNode.replaceChild(newCitation, citation);

        newCitation.addEventListener('click', (e) => {
          const refs = newCitation.getAttribute('data-ref').split(' ');
          if (refs.length > 0) {
            this.scrollToEntry(refs[0]);
            this.highlightEntry(refs[0]);
          }
        });

        newCitation.addEventListener('mouseenter', () => {
          const refs = newCitation.getAttribute('data-ref').split(' ');
          refs.forEach(ref => this.highlightEntry(ref, true));
        });

        newCitation.addEventListener('mouseleave', () => {
          const refs = newCitation.getAttribute('data-ref').split(' ');
          refs.forEach(ref => this.highlightEntry(ref, false));
        });
      });
    },

    setupBibliography: function() {
      const entries = document.querySelectorAll('.citum-entry');
      entries.forEach(entry => {
        entry.addEventListener('mouseenter', () => {
          const id = entry.id.replace('ref-', '');
          this.highlightCitations(id, true);
        });

        entry.addEventListener('mouseleave', () => {
          const id = entry.id.replace('ref-', '');
          this.highlightCitations(id, false);
        });
      });
    },

    scrollToEntry: function(id) {
      const target = document.getElementById('ref-' + id);
      if (target) {
        target.scrollIntoView({ behavior: 'smooth', block: 'center' });
        history.replaceState(null, null, '#ref-' + id);
      }
    },

    highlightEntry: function(id, active = true) {
      const entry = document.getElementById('ref-' + id);
      if (entry) {
        if (active) {
          entry.classList.add('is-highlighted');
        } else {
          entry.classList.remove('is-highlighted');
        }
      }
    },

    highlightCitations: function(refId, active = true) {
      const citations = document.querySelectorAll(`.citum-citation[data-ref~="${refId}"]`);
      citations.forEach(citation => {
        if (active) {
          citation.classList.add('is-highlighted');
        } else {
          citation.classList.remove('is-highlighted');
        }
      });
    },

    setupTooltips: function() {
      // Remove existing tooltip if any
      const existing = document.querySelector('.citum-tooltip');
      if (existing) existing.remove();

      const tooltip = document.createElement('div');
      tooltip.className = 'citum-tooltip';
      document.body.appendChild(tooltip);

      const citations = document.querySelectorAll('.citum-citation');
      citations.forEach(citation => {
        citation.addEventListener('mousemove', (e) => {
          const refs = citation.getAttribute('data-ref').split(' ');
          if (refs.length === 0) return;

          const entry = document.getElementById('ref-' + refs[0]);
          if (!entry) return;

          // Try to extract info from the rendered entry if data attributes aren't present
          const author = entry.getAttribute('data-author') || entry.querySelector('.citum-author')?.textContent || '';
          const title = entry.getAttribute('data-title') || entry.querySelector('.citum-title')?.textContent || '';

          tooltip.innerHTML = '';
          if (author) {
            const authorEl = document.createElement('span');
            authorEl.className = 'citum-tooltip-author';
            authorEl.textContent = author;
            tooltip.appendChild(authorEl);
          }
          if (title) {
            const titleEl = document.createElement('span');
            titleEl.className = 'citum-tooltip-title';
            titleEl.textContent = title;
            tooltip.appendChild(titleEl);
          }

          tooltip.style.left = (e.pageX + 15) + 'px';
          tooltip.style.top = (e.pageY + 15) + 'px';
          tooltip.classList.add('is-visible');
        });

        citation.addEventListener('mouseleave', () => {
          tooltip.classList.remove('is-visible');
        });
      });
    }
  };

  // Initialize
  CITUM.init();

  // Export to window
  window.CITUM = CITUM;
})();
