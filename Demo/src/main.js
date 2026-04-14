import { openBook, openBuiltinBook, prevPage, nextPage, goToPage, toggleToc, closeToc } from "./book";
import { openLibrary, handleLibraryFiles, toggleLibrary } from "./library";

/***** DOM Elements *****/
const openButton = document.getElementById('open-button');
const openDemoButton = document.getElementById('open-demo-button');
const demoBookSelect = document.getElementById('demo-book-select');
const fileInput = document.getElementById('file-input');
const libraryInput = document.getElementById('library-input');
const libraryButton = document.getElementById('library-button');
const closeLibraryButton = document.getElementById('close-library');
const tocButton = document.getElementById('toc-button');
const closeTocButton = document.getElementById('close-toc');
const prevButton = document.getElementById('prev-button');
const nextButton = document.getElementById('next-button');
const currentPageInput = document.getElementById('current-page');
const overlay = document.getElementById('overlay');
const loadingMessage = document.getElementById('loading-message');
const errorMessage = document.getElementById('error-message');
const errorText = document.getElementById('error-text');
const closeErrorButton = document.getElementById('close-error');



/***** Event Listeners *****/
openButton.addEventListener('click', () => fileInput.click());
openDemoButton.addEventListener('click', () => {
  const selected = demoBookSelect.value;
  if (!selected) return;
  openBuiltinBook(selected);
});
fileInput.addEventListener('change', openBook);
prevButton.addEventListener('click', prevPage);
nextButton.addEventListener('click', nextPage);
currentPageInput.addEventListener('change', goToPage);
tocButton.addEventListener('click', toggleToc);
closeTocButton.addEventListener('click', toggleToc);
libraryButton.addEventListener('click', openLibrary);
closeLibraryButton.addEventListener('click', () => toggleLibrary(false));
overlay.addEventListener('click', () => {
  closeToc();
  toggleLibrary(false);
  hideError();
});
closeErrorButton.addEventListener('click', hideError);
// Fallback: multiple file input for library import
libraryInput.addEventListener('change', handleLibraryFiles);

/**
 * Show the global loading message/overlay.
 *
 * Makes the loadingMessage element visible by adding the CSS `show` class.
 */
export function showLoading() {
  loadingMessage.classList.add('show');
}

/**
 * Hide the global loading indicator.
 *
 * Removes the 'show' CSS class from the loading message element to hide the loading UI.
 */
export function hideLoading() {
  loadingMessage.classList.remove('show');
}

/**
 * Display an error message in the UI.
 *
 * Sets the visible error panel's text to `message` and makes the panel visible by adding the `show` class.
 *
 * @param {string} message - The error text to display to the user.
 */
export function showError(message) {
  errorText.textContent = message;
  errorMessage.classList.add('show');
}

/**
 * Hide the visible error message UI.
 *
 * Removes the 'show' class from the error message element so the error overlay is hidden.
 */
export function hideError() {
  errorMessage.classList.remove('show');
}
