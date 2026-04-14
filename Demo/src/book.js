import initLexEpub, { WasmEpubExtractor } from "./wasm/lexepub.js";
import { showLoading, showError, hideLoading } from "./main";
import { toggleLibrary } from "./library";

/***** Book Variables *****/
let extractor = null;
let metadata = null;
let chapters = [];
let currentChapterIndex = 0;
let wasmInitPromise = null;
let chapterCache = new Map();
let coverObjectUrl = null;
let renderRequestId = 0;
let tocEntries = [];
let chapterPathIndex = new Map();

/***** DOM Elements *****/
const tocButton = document.getElementById('toc-button');
const prevButton = document.getElementById('prev-button');
const nextButton = document.getElementById('next-button');
const currentPageInput = document.getElementById('current-page');
const overlay = document.getElementById('overlay');
const totalPagesSpan = document.getElementById('total-pages');
const bookTitleSpan = document.getElementById('book-title');
const tocContainer = document.getElementById('toc-container');
const tocContent = document.getElementById('toc-content');
const viewer = document.getElementById('viewer');

/**
 * Open an EPUB file selected via a file input and load it with lexepub WASM.
 *
 * @param {Event} e - Change event from a file input; the function reads e.target.files[0].
 */
export async function openBook(e) {
  const file = e.target.files[0];
  if (!file) return;
  if (file.type !== 'application/epub+zip' && !file.name.endsWith('.epub')) {
    showError('The selected file is not a valid EPUB file.');
    return;
  }

  showLoading();
  try {
    const bookData = await file.arrayBuffer();
    await loadBook(bookData);
  } catch (err) {
    showError('Error loading book: ' + err.message);
  } finally {
    hideLoading();
  }
}

/**
 * Load one of the built-in demo EPUBs via fetch and parse it with lexepub WASM.
 *
 * @param {string} bookPath - Public URL path to an EPUB file.
 */
export async function openBuiltinBook(bookPath) {
  if (!bookPath) return;

  showLoading();
  try {
    const response = await fetch(bookPath);
    if (!response.ok) {
      throw new Error(`HTTP ${response.status} while fetching ${bookPath}`);
    }
    const bookData = await response.arrayBuffer();
    await loadBook(bookData);
  } catch (err) {
    showError('Error loading built-in demo: ' + err.message);
  } finally {
    hideLoading();
  }
}

// Immediately close library on click so the user sees the main viewer
/**
 * Open and load an EPUB from a library entry, managing the library UI and loading spinner.
 *
 * Reads the file from the given library entry (object with an async `getFile()` method), converts it to an ArrayBuffer,
 * and delegates to loadBook to render the book. Closes the library and shows a loading indicator while loading.
 * If an error occurs, the library is reopened and an error message is shown; the function always hides the loading indicator before returning.
 *
 * @param {Object} entry - Library entry providing an async `getFile()` method that returns a File/Blob.
 * @return {Promise<void>} Resolves once loading has finished or an error has been handled.
 */
export async function openBookFromEntry(entry) {
  toggleLibrary(false);
  showLoading();
  try {
    const file = (typeof entry?.getFile === 'function') ? await entry.getFile() : entry;
    const arrayBuffer = await file.arrayBuffer();
    await loadBook(arrayBuffer);
  } catch (err) {
    toggleLibrary(true);
    showError('Error opening book: ' + err.message);
  } finally {
    hideLoading();
  }
}

/**
 * Initialize the lexepub WASM module once.
 */
async function ensureWasmReady() {
  if (!wasmInitPromise) {
    wasmInitPromise = initLexEpub();
  }
  await wasmInitPromise;
}

/**
 * Parse an EPUB ArrayBuffer via lexepub and hydrate reader state.
 *
 * @param {ArrayBuffer} bookData - Raw EPUB bytes.
 */
async function loadBook(bookData) {
  await ensureWasmReady();

  if (coverObjectUrl) {
    URL.revokeObjectURL(coverObjectUrl);
    coverObjectUrl = null;
  }

  viewer.innerHTML = '';
  chapterCache = new Map();
  extractor = new WasmEpubExtractor();
  await extractor.load_from_bytes(new Uint8Array(bookData));
  metadata = await extractor.get_metadata();
  chapters = await extractor.get_chapters_text();
  tocEntries = await extractor.get_toc();
  chapterPathIndex = new Map(
    tocEntries
      .map(entry => [normalizeChapterPath(entry.chapter_href), entry.chapter_index])
      .filter(([path]) => Boolean(path))
  );

  try {
    if (await extractor.has_cover()) {
      const coverBytes = await extractor.get_cover_image();
      const coverFormat = (await extractor.get_cover_image_format()) || 'image/jpeg';
      coverObjectUrl = URL.createObjectURL(new Blob([coverBytes], { type: coverFormat }));
    }
  } catch {
    coverObjectUrl = null;
  }

  if (!Array.isArray(chapters) || chapters.length === 0) {
    throw new Error('This EPUB has no readable chapters.');
  }

  currentChapterIndex = 0;
  void renderCurrentChapter();
  generateToc();

  totalPagesSpan.textContent = String(chapters.length);
  currentPageInput.value = '1';

  prevButton.disabled = false;
  nextButton.disabled = false;
  tocButton.disabled = false;

  window.removeEventListener('keyup', handleKeyEvents);
  window.addEventListener('keyup', handleKeyEvents);

  bookTitleSpan.textContent = metadata?.title || 'Untitled EPUB';
}

/**
 * Render current chapter using AST first, then plain-text fallback.
 */
async function renderCurrentChapter() {
  if (!chapters.length) return;

  const requestId = ++renderRequestId;
  const chapterText = chapters[currentChapterIndex] || '';
  viewer.innerHTML = '';

  const chapterContainer = document.createElement('article');
  chapterContainer.className = 'chapter-view';

  if (currentChapterIndex === 0 && coverObjectUrl) {
    const cover = document.createElement('img');
    cover.className = 'book-cover';
    cover.src = coverObjectUrl;
    cover.alt = `${metadata?.title || 'Book'} cover`;
    chapterContainer.appendChild(cover);
  }

  const parsedChapter = await getParsedChapter(currentChapterIndex);
  if (requestId !== renderRequestId) return;

  const renderedAst = appendAstChapter(chapterContainer, parsedChapter);
  if (!renderedAst) {
    renderTextFallback(chapterContainer, chapterText);
  }

  viewer.appendChild(chapterContainer);
  currentPageInput.value = String(currentChapterIndex + 1);
  totalPagesSpan.textContent = String(chapters.length);
  prevButton.disabled = currentChapterIndex === 0;
  nextButton.disabled = currentChapterIndex >= chapters.length - 1;
}

async function getParsedChapter(index) {
  if (!extractor) return null;
  if (chapterCache.has(index)) return chapterCache.get(index);

  try {
    const chapterJson = await extractor.get_chapter_json(index);
    const parsedChapter = JSON.parse(chapterJson);
    chapterCache.set(index, parsedChapter);
    return parsedChapter;
  } catch {
    return null;
  }
}

function appendAstChapter(container, parsedChapter) {
  if (!parsedChapter?.ast) return false;

  const nodes = renderAstNode(parsedChapter.ast);
  if (!nodes.length) return false;

  for (const node of nodes) {
    container.appendChild(node);
  }

  return true;
}

function renderTextFallback(container, chapterText) {
  const paragraphs = chapterText
    .split(/\n\s*\n/g)
    .map(part => part.trim())
    .filter(Boolean);

  if (paragraphs.length === 0) {
    const empty = document.createElement('p');
    empty.textContent = chapterText.trim() || 'No text extracted for this chapter.';
    container.appendChild(empty);
  } else {
    paragraphs.forEach(text => {
      const p = document.createElement('p');
      p.textContent = text;
      container.appendChild(p);
    });
  }
}

function renderAstNode(astNode) {
  if (!astNode || typeof astNode !== 'object') return [];

  if (astNode.type === 'Text') {
    return [document.createTextNode(astNode.content || '')];
  }

  if (astNode.type !== 'Element') return [];

  const tag = String(astNode.tag || '').toLowerCase();

  if (tag === 'head' || tag === 'meta' || tag === 'link' || tag === 'script' || tag === 'style' || tag === 'title') {
    return [];
  }

  if (tag === 'html' || tag === 'body') {
    const fragment = document.createDocumentFragment();
    for (const child of astNode.children || []) {
      for (const childNode of renderAstNode(child)) {
        fragment.appendChild(childNode);
      }
    }
    return fragment.childNodes.length ? [fragment] : [];
  }

  const element = createSafeElement(tag);
  applySafeAttributes(element, astNode.attrs || {}, tag);
  applyInlineStyles(element, astNode.styles || {});

  for (const child of astNode.children || []) {
    for (const childNode of renderAstNode(child)) {
      element.appendChild(childNode);
    }
  }

  return [element];
}

function createSafeElement(tag) {
  const safeTag = tag && /^[a-z][a-z0-9-]*$/.test(tag) ? tag : 'div';
  try {
    return document.createElement(safeTag);
  } catch {
    return document.createElement('div');
  }
}

function applySafeAttributes(element, attrs, tagName = '') {
  for (const [rawName, rawValue] of Object.entries(attrs)) {
    if (!rawName || rawValue == null) continue;

    const name = rawName.toLowerCase();
    const value = String(rawValue);

    if (name.startsWith('on') || name === 'style') continue;

    if (name === 'href') {
      element.setAttribute('href', value);
      if (/^https?:\/\//i.test(value)) {
        element.setAttribute('target', '_blank');
        element.setAttribute('rel', 'noopener noreferrer');
      } else {
        element.addEventListener('click', event => {
          event.preventDefault();
          void handleInternalHref(value);
        });
      }
      continue;
    }

    if (name === 'src') {
      if (/^(data:|https?:|blob:|\/)/i.test(value)) {
        element.setAttribute('src', value);
      } else if (tagName === 'img') {
        void hydrateImageSource(element, value);
      }
      continue;
    }

    if (
      name === 'alt' ||
      name === 'title' ||
      name === 'id' ||
      name === 'class' ||
      name === 'role' ||
      name === 'lang' ||
      name === 'dir' ||
      name.startsWith('aria-')
    ) {
      element.setAttribute(name, value);
    }
  }
}

async function hydrateImageSource(imgElement, href) {
  if (!extractor || !href) return;

  try {
    const bytes = await extractor.get_chapter_resource(currentChapterIndex, href);
    const mime = inferMimeFromPath(href);
    const objectUrl = URL.createObjectURL(new Blob([bytes], { type: mime }));
    imgElement.src = objectUrl;
  } catch {
    if (!imgElement.alt) {
      imgElement.alt = `Missing image: ${href}`;
    }
  }
}

function inferMimeFromPath(path) {
  const p = path.toLowerCase();
  if (p.endsWith('.svg')) return 'image/svg+xml';
  if (p.endsWith('.png')) return 'image/png';
  if (p.endsWith('.gif')) return 'image/gif';
  if (p.endsWith('.webp')) return 'image/webp';
  if (p.endsWith('.avif')) return 'image/avif';
  return 'image/jpeg';
}

function normalizeChapterPath(path) {
  if (!path) return '';
  return String(path).split('#')[0].replace(/^\/+/, '').replace(/\\/g, '/');
}

async function handleInternalHref(href) {
  if (!href) return;

  if (href.startsWith('#')) {
    scrollToFragment(href.slice(1));
    return;
  }

  if (/^(https?:|mailto:|data:|blob:)/i.test(href)) {
    window.open(href, '_blank', 'noopener,noreferrer');
    return;
  }

  const resolved = await extractor.resolve_chapter_resource_path(currentChapterIndex, href);
  const [pathOnly, fragment = ''] = String(resolved).split('#');
  const chapterIdx = chapterPathIndex.get(normalizeChapterPath(pathOnly));

  if (chapterIdx == null) return;

  currentChapterIndex = chapterIdx;
  await renderCurrentChapter();

  if (fragment) {
    scrollToFragment(fragment);
  }
}

function scrollToFragment(fragmentId) {
  if (!fragmentId) return;
  const escaped = (window.CSS && typeof window.CSS.escape === 'function')
    ? window.CSS.escape(fragmentId)
    : fragmentId.replace(/["'\\#.:\[\]()]/g, '\\$&');

  const target = viewer.querySelector(`#${escaped}`) || viewer.querySelector(`a[name="${fragmentId}"]`);
  if (target && typeof target.scrollIntoView === 'function') {
    target.scrollIntoView({ behavior: 'smooth', block: 'start' });
  }
}

function applyInlineStyles(element, styles) {
  for (const [prop, value] of Object.entries(styles)) {
    if (!prop || value == null) continue;
    try {
      element.style.setProperty(String(prop), String(value));
    } catch {
      // Ignore unsupported style declarations.
    }
  }
}

/**
 * Build and render the table of contents from metadata spine or chapter numbers.
 */
function generateToc() {
  if (!chapters.length) return;

  tocContent.innerHTML = '';

  const items = tocEntries.length
    ? tocEntries
    : chapters.map((_, index) => ({ chapter_index: index, title: `Chapter ${index + 1}` }));

  items.forEach((entry, fallbackIndex) => {
    const index = typeof entry.chapter_index === 'number' ? entry.chapter_index : fallbackIndex;
    const tocItem = document.createElement('div');
    tocItem.className = 'toc-item';
    tocItem.textContent = entry.title || `Chapter ${index + 1}`;
    tocItem.addEventListener('click', () => {
      currentChapterIndex = index;
      void renderCurrentChapter();
      closeToc();
    });
    tocContent.appendChild(tocItem);
  });
}

function hasBookLoaded() {
  return chapters.length > 0;
}

function clampChapterIndex(index) {
  if (!chapters.length) return 0;
  return Math.max(0, Math.min(index, chapters.length - 1));
}

function goToChapter(index) {
  if (!hasBookLoaded()) return;
  currentChapterIndex = clampChapterIndex(index);
  void renderCurrentChapter();
}

function normalizedPageNumber() {
  const parsed = parseInt(currentPageInput.value, 10);
  if (Number.isNaN(parsed)) {
    return currentChapterIndex + 1;
  }
  return parsed;
}

/**
 * Navigate the viewer to the previous page.
 */
export function prevPage() {
  if (!hasBookLoaded() || currentChapterIndex <= 0) return;
  goToChapter(currentChapterIndex - 1);
}

/**
 * Advance the current rendition to the next page/location.
 */
export function nextPage() {
  if (!hasBookLoaded() || currentChapterIndex >= chapters.length - 1) return;
  goToChapter(currentChapterIndex + 1);
}

/**
 * Navigate the viewer to the page number entered in the page input field.
 */
export function goToPage() {
  if (!hasBookLoaded()) return;
  const pageNumber = normalizedPageNumber();
  goToChapter(pageNumber - 1);
}

/**
 * Handle keyboard navigation: left/right arrow keys move to the previous/next page.
 * @param {KeyboardEvent} e - Keyboard event.
 */
function handleKeyEvents(e) {
  if (!hasBookLoaded()) return;
  if (e.key === 'ArrowLeft') prevPage();
  if (e.key === 'ArrowRight') nextPage();
}

/**
 * Toggle the visibility of the table of contents overlay.
 */
export function toggleToc() {
  tocContainer.classList.toggle('open');
  overlay.classList.toggle('open');
}

/**
 * Close the table of contents overlay.
 */
export function closeToc() {
  tocContainer.classList.remove('open');
  overlay.classList.remove('open');
}
