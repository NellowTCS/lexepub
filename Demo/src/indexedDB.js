/**
 * Open (or create) the IndexedDB database "htmlreader-db" (version 1) and ensure the "handles" object store exists.
 *
 * Returns a promise that resolves to the opened IDBDatabase instance. During upgrade, an object store named
 * "handles" with keyPath "name" is created if missing. The promise rejects with the underlying IndexedDB error
 * if opening the database fails.
 *
 * @return {Promise<IDBDatabase>} Promise resolving to the opened IndexedDB database.
 */
function getDB() {
  return new Promise((resolve, reject) => {
    const request = indexedDB.open("htmlreader-db", 1);
    request.onupgradeneeded = e => {
      const db = e.target.result;
      db.createObjectStore("handles", { keyPath: "name" });
    };
    request.onsuccess = e => resolve(e.target.result);
    request.onerror = e => reject(e.target.error);
  });
}

/**
 * Persistently stores the provided library handle in the IndexedDB "handles" object store under the key "library".
 *
 * @param {*} handle - The library handle to store (e.g., a FileSystem handle or other serializable handle).
 * @returns {Promise<void>} Resolves when the handle has been written to the database. Rejects with the underlying error if the database write fails.
 */
export async function storeLibraryHandle(handle) {
  const db = await getDB();
  return new Promise((resolve, reject) => {
    const tx = db.transaction("handles", "readwrite");
    const store = tx.objectStore("handles");
    store.put({ name: "library", handle });
    tx.oncomplete = () => resolve();
    tx.onabort = tx.onerror = e => reject(tx.error || (e && e.target && e.target.error));
  });
}

/**
 * Retrieve the stored library handle from the "handles" object store.
 *
 * Returns a Promise that resolves to the stored handle (if present) or null
 * when no entry named "library" exists. The Promise rejects with the
 * underlying IndexedDB error if the request fails.
 *
 * @return {Promise<any|null>} The stored library handle or null.
 */
export async function getStoredLibraryHandle() {
  const db = await getDB();
  return new Promise((resolve, reject) => {
    const tx = db.transaction("handles", "readonly");
    const store = tx.objectStore("handles");
    const req = store.get("library");
    req.onsuccess = () => resolve(req.result ? req.result.handle : null);
    req.onerror = e => reject(e.target.error);
  });
}