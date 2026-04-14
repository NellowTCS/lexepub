import { storeLibraryHandle, getStoredLibraryHandle } from "./indexedDB";
import { openBookFromEntry } from "./book"
import initLexEpub, { WasmEpubExtractor } from "./wasm/lexepub.js";
import { showError } from "./main";

let wasmInitPromise = null;

async function ensureWasmReady() {
  if (!wasmInitPromise) {
    wasmInitPromise = initLexEpub();
  }
  await wasmInitPromise;
}

/***** DOM Elements *****/
const libraryContainer = document.getElementById('library-container');
const libraryContent = document.getElementById('library-content');
const overlay = document.getElementById('overlay');

/**
 * Open the user's EPUB library: get or prompt for a directory, scan for .epub files, display them, and open the library UI.
 *
 * Attempts to use a previously stored directory handle; if none is available, prompts the user to pick a directory and stores the handle.
 * Scans the directory for files whose names end with ".epub", passes those entries to the library grid renderer, and opens the library overlay.
 * On failure, reports a user-facing error via showError; the function catches errors and does not rethrow.
 *
 * @returns {Promise<void>} Resolves after the library grid is displayed or after an error has been reported.
 */
export async function openLibrary() {
  try {
    // Try to retrieve stored library directory handle
    let dirHandle = await getStoredLibraryHandle();
    if (!dirHandle) {
      // If no stored handle, prompt user
      if (!('showDirectoryPicker' in window)) {
        // Fallback: trigger multiple file input flow
        document.getElementById('library-input')?.click();
        return;
      }
      dirHandle = await window.showDirectoryPicker();
      await storeLibraryHandle(dirHandle);
    }
    // Permissions for persisted handles can be lost between sessions
    if (dirHandle.queryPermission && dirHandle.requestPermission) {
      const perm = await dirHandle.queryPermission({ mode: 'read' });
      if (perm !== 'granted') {
        const res = await dirHandle.requestPermission({ mode: 'read' });
        if (res !== 'granted') throw new Error('Read permission was denied for the library directory.');
      }
    }
    const files = [];
    for await (const entry of dirHandle.values()) {
      if (entry.kind === 'file' && entry.name.endsWith('.epub')) {
        files.push(entry);
      }
    }
    displayLibraryGrid(files);
    toggleLibrary(true);
  } catch (err) {
    showError('Failed to open library: ' + err.message);
  }
}

/**
 * Handle a file-input change by displaying selected EPUB files in the library and opening the library UI.
 * @param {Event} e - Change event from a file input (`<input type="file" multiple>`); selected File objects are read and shown in the library grid.
 */
export function handleLibraryFiles(e) {
  const files = Array.from(e.target.files);
  displayLibraryGrid(files);
  toggleLibrary(true);
}

/**
 * Render a grid of EPUB items into the library UI.
 *
 * Clears the library content area and, for each entry in `fileEntries`, creates
 * a library item (cover + title) and appends it to the grid. If `fileEntries`
 * is empty, shows a "No EPUB files found." message instead.
 *
 * @param {Array<import('./types').FileEntry|File>} fileEntries - Array of file entries to display. Each entry may be a File, FileSystemFileHandle, or similar object accepted by createLibraryItem.
 * @return {Promise<void>}
 */
async function displayLibraryGrid(fileEntries) {
  libraryContent.innerHTML = '';
  if (fileEntries.length === 0) {
    const msg = document.createElement('div');
    msg.textContent = 'No EPUB files found.';
    libraryContent.appendChild(msg);
    return;
  }
  for (const entry of fileEntries) {
    const item = await createLibraryItem(entry);
    libraryContent.appendChild(item);
  }
}

/**
 * Create a DOM element representing an EPUB library item (cover + title) for the given file entry.
 *
 * The function accepts either a File object or a FileSystemFileHandle (from the File System Access API),
 * reads the EPUB to extract a cover image and metadata title when available, and falls back to the
 * file name and a generic placeholder cover if not. It attaches a click handler that opens the book
 * via openBookFromEntry(fileEntry). Errors while loading cover/metadata are caught and logged; they
 * do not prevent the returned element from being used.
 *
 * @param {File|FileSystemFileHandle} fileEntry - The EPUB file or a handle for the EPUB file.
 * @return {HTMLElement} A '.library-item' element containing an image ('.library-cover') and title ('.library-title').
 */
async function createLibraryItem(fileEntry) {
  const item = document.createElement('div');
  item.className = 'library-item';
  const img = document.createElement('img');
  img.className = 'library-cover';
  img.src = '';
  const titleDiv = document.createElement('div');
  titleDiv.className = 'library-title';
  titleDiv.textContent = fileEntry.name;
  item.appendChild(img);
  item.appendChild(titleDiv);

  try {
    // If using the File System Access API:
    const file = (typeof fileEntry.getFile === 'function')
                  ? await fileEntry.getFile()
                  : fileEntry;

    const arrayBuffer = await file.arrayBuffer();

    await ensureWasmReady();
    const extractor = new WasmEpubExtractor();
    await extractor.load_from_bytes(new Uint8Array(arrayBuffer));

    const metadata = await extractor.get_metadata();
    if (metadata?.title) {
      titleDiv.textContent = metadata.title;
    }

    if (await extractor.has_cover()) {
      const coverBytes = await extractor.get_cover_image();
      const mime = (await extractor.get_cover_image_format()) || 'image/jpeg';
      const coverBlob = new Blob([coverBytes], { type: mime });
      img.src = URL.createObjectURL(coverBlob);
    } else {
      // Use a generic placeholder if no cover
      img.src = 'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAMgAAADICAMAAACahl6sAAAAM1BMVEX///+hoaGcnJzPz8/Nzc3FxcXn5+fQ0NDy8vL29vbw8PDv7+/d3d2+vr6UlJSakGz1AAACNklEQVR4nO3d2ZKDIBAFUa8El//+uvLFT6qkSpknG/JpLve86o3QF8AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAD8S/w66a8vEcn8eHS6HQ58n8eHS6HQ58n8eHS6HQ58n8eHS6HQ58n8eHS6HQ58n8eHS6HQ58n8eHS6HQ58n8eHS6HQ58n8eHS6HQ58n8eHS6HQ58n8eHS6HQ58n8eHS6HQ58n8eHS6HQ58n8eHS6HQ58n8eHS6HQ58n8eHS6HQ58n8eHS6HQ58n8eHS6HQ58n8eHS6HQ58n8eHS6HQ58n8eHS6HQ58n8eHS6HQ58n8eHS6HQ58n8eHS6HQ58n8eHS6HQ58n8eHS6HQ58n8eHS6HQ58n8eHS6HQ58n8eHS6HQ58n8eHS6HQ58n8eHS6HQ58n8eHS6HQ58n8eHS6HQ58n8eHS6HQ58n8eHS6HQ58n8eHS6HQ58n8eHS6HQ58n8eHS6HQ58n8eHS6HQ5+n/wP2S/3mmugUsAAAAASUVORK5CYII=';
    }
  } catch (err) {
    console.error('Error loading cover for', fileEntry.name, err);
  }

  // No { once: true } so user can try again if there's an error
  item.addEventListener('click', () => {
    openBookFromEntry(fileEntry);
  });

  return item;
}

/**
 * Open, close, or toggle the library UI.
 *
 * @param {boolean|undefined} forceOpen - If true, ensures the library is opened; if false, ensures it is closed; if omitted, toggles the current state.
 */
export function toggleLibrary(forceOpen) {
  if (forceOpen === true) {
    libraryContainer.classList.add('open');
    overlay.classList.add('open');
  } else if (forceOpen === false) {
    libraryContainer.classList.remove('open');
    overlay.classList.remove('open');
  } else {
    libraryContainer.classList.toggle('open');
    overlay.classList.toggle('open');
  }
}