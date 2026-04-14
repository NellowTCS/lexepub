import initLexEpub, { WasmEpubExtractor } from "./wasm/lexepub.js";
import { showLoading, showError, hideLoading } from "./main";
import { toggleLibrary } from "./library";

/***** Book Variables *****/
let extractor = null;
let metadata = null;
let chapters = [];
let currentChapterIndex = 0;
let wasmInitPromise = null;


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
 * and delegates to `loadBook` to render the book. Closes the library and shows a loading indicator while loading.
 * If an error occurs, the library is reopened and an error message is shown; the function always hides the loading indicator before returning.
 *
 * @param {Object} entry - Library entry providing an async `getFile()` method that returns a `File`/Blob.
 * @return {Promise<void>} Resolves once loading has finished or an error has been handled.
 */
export async function openBookFromEntry(entry) {
  // Close library right away
  toggleLibrary(false);
  showLoading();
  try {
    const file = (typeof entry?.getFile === 'function') ? await entry.getFile() : entry;
    const arrayBuffer = await file.arrayBuffer();
    await loadBook(arrayBuffer);
  } catch (err) {
    // If error, reopen library so user can pick another book
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

  viewer.innerHTML = '';
  extractor = new WasmEpubExtractor();
  await extractor.load_from_bytes(new Uint8Array(bookData));
  metadata = await extractor.get_metadata();
  chapters = await extractor.get_chapters_text();

  if (!Array.isArray(chapters) || chapters.length === 0) {
    throw new Error('This EPUB has no readable chapters.');
  }

  currentChapterIndex = 0;
  renderCurrentChapter();
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
 * Render the current chapter as plain text in the main viewer panel.
 */
function renderCurrentChapter() {
  if (!chapters.length) return;

  const chapterText = chapters[currentChapterIndex] || '';
  viewer.innerHTML = '';

  const chapterContainer = document.createElement('article');
  chapterContainer.className = 'chapter-view';

  const paragraphs = chapterText
    .split(/\n\s*\n/g)
    .map(part => part.trim())
    .filter(Boolean);

  if (paragraphs.length === 0) {
    const empty = document.createElement('p');
    empty.textContent = chapterText.trim() || 'No text extracted for this chapter.';
    chapterContainer.appendChild(empty);
  } else {
    paragraphs.forEach(text => {
      const p = document.createElement('p');
      p.textContent = text;
      chapterContainer.appendChild(p);
    });
  }

  viewer.appendChild(chapterContainer);
  currentPageInput.value = String(currentChapterIndex + 1);
  totalPagesSpan.textContent = String(chapters.length);
  prevButton.disabled = currentChapterIndex === 0;
  nextButton.disabled = currentChapterIndex >= chapters.length - 1;
}

/**
 * Build and render the table of contents from metadata spine or chapter numbers.
 */
function generateToc() {
  if (!chapters.length) return;

  tocContent.innerHTML = '';
  const spine = Array.isArray(metadata?.spine) ? metadata.spine : [];

  chapters.forEach((_, index) => {
    const tocItem = document.createElement('div');
    tocItem.className = 'toc-item';
    tocItem.textContent = spine[index] || `Chapter ${index + 1}`;
    tocItem.addEventListener('click', () => {
      currentChapterIndex = index;
      renderCurrentChapter();
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
  renderCurrentChapter();
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
 *
 * If a rendition is active, calls its `prev()` method; otherwise does nothing.
 */
export function prevPage() {
  if (!hasBookLoaded() || currentChapterIndex <= 0) return;
  goToChapter(currentChapterIndex - 1);
}

/**
 * Advance the current rendition to the next page/location.
 *
 * This is a no-op if no rendition is initialized.
 */
export function nextPage() {
  if (!hasBookLoaded() || currentChapterIndex >= chapters.length - 1) return;
  goToChapter(currentChapterIndex + 1);
}

/**
 * Navigate the viewer to the page number entered in the page input field.
 *
 * Reads a 1-based page number from `currentPageInput.value`, converts it to a
 * 0-based location index, validates it against the book's generated locations,
 * converts that location index to a CFI using `book.locations.cfiFromLocation`,
 * and displays it in the rendition.
 *
 * No action is taken if there is no loaded book or location data, or if the
 * entered page number is out of range or not a valid integer.
 */
export function goToPage() {
  if (!hasBookLoaded()) return;
  const pageNumber = normalizedPageNumber();
  goToChapter(pageNumber - 1);
}

/**
 * Handle keyboard navigation: left/right arrow keys move to the previous/next page.
 * @param {KeyboardEvent} e - Keyboard event; listens for 'ArrowLeft' to go to the previous page and 'ArrowRight' to go to the next page.
 */
function handleKeyEvents(e) {
  if (!hasBookLoaded()) return;
  if (e.key === 'ArrowLeft') prevPage();
  if (e.key === 'ArrowRight') nextPage();
}

/**
 * Toggle the visibility of the table of contents overlay.
 *
 * Adds or removes the 'open' class on the TOC container and the overlay element to show or hide the table of contents.
 */
export function toggleToc() {
  tocContainer.classList.toggle('open');
  overlay.classList.toggle('open');
}

/**
 * Close the table of contents overlay.
 *
 * Removes the 'open' class from the TOC container and the page overlay, hiding the table of contents.
 */
export function closeToc() {
  tocContainer.classList.remove('open');
  overlay.classList.remove('open');
}