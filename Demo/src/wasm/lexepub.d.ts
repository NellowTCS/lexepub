/* tslint:disable */
/* eslint-disable */

export class WasmEpubExtractor {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Get chapter by index
     */
    get_chapter(index: number): Promise<any>;
    /**
     * Get chapter count from metadata
     */
    get_chapter_count(): Promise<number>;
    /**
     * Get chapter by index serialized as JSON string
     */
    get_chapter_json(index: number): Promise<string>;
    /**
     * Read a chapter-relative resource as bytes (for images, linked assets)
     */
    get_chapter_resource(chapter_index: number, href: string): Promise<Uint8Array>;
    /**
     * Get chapter text by index
     */
    get_chapter_text(index: number): Promise<string>;
    /**
     * Get all chapters as text array
     */
    get_chapters_text(): Promise<any>;
    /**
     * Get all chapter text serialized as JSON string
     */
    get_chapters_text_json(): Promise<string>;
    /**
     * Get cover image as byte array
     */
    get_cover_image(): Promise<Uint8Array>;
    /**
     * Get cover image MIME format from metadata
     */
    get_cover_image_format(): Promise<string>;
    /**
     * Get cover image byte length
     */
    get_cover_image_len(): Promise<number>;
    /**
     * Get EPUB metadata as JSON
     */
    get_metadata(): Promise<any>;
    /**
     * Validate EPUB metadata against required constraints
     */
    get_metadata_is_valid(): Promise<boolean>;
    /**
     * Get metadata serialized as JSON string
     */
    get_metadata_json(): Promise<string>;
    /**
     * Read an internal EPUB resource as bytes by normalized path
     */
    get_resource(path: string): Promise<Uint8Array>;
    /**
     * Get title string from metadata
     */
    get_title(): Promise<string>;
    /**
     * Get table of contents entries
     */
    get_toc(): Promise<any>;
    /**
     * Get table of contents entries serialized as JSON
     */
    get_toc_json(): Promise<string>;
    /**
     * Get total character count
     */
    get_total_char_count(): Promise<number>;
    /**
     * Get total word count
     */
    get_total_word_count(): Promise<number>;
    /**
     * Check if EPUB has a cover image
     */
    has_cover(): Promise<boolean>;
    /**
     * Load EPUB from byte array
     */
    load_from_bytes(data: Uint8Array): Promise<void>;
    constructor();
    /**
     * Resolve a chapter-relative href into a normalized internal EPUB path
     */
    resolve_chapter_resource_path(chapter_index: number, href: string): Promise<string>;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_wasmepubextractor_free: (a: number, b: number) => void;
    readonly wasmepubextractor_get_chapter: (a: number, b: number) => any;
    readonly wasmepubextractor_get_chapter_count: (a: number) => any;
    readonly wasmepubextractor_get_chapter_json: (a: number, b: number) => any;
    readonly wasmepubextractor_get_chapter_resource: (a: number, b: number, c: number, d: number) => any;
    readonly wasmepubextractor_get_chapter_text: (a: number, b: number) => any;
    readonly wasmepubextractor_get_chapters_text: (a: number) => any;
    readonly wasmepubextractor_get_chapters_text_json: (a: number) => any;
    readonly wasmepubextractor_get_cover_image: (a: number) => any;
    readonly wasmepubextractor_get_cover_image_format: (a: number) => any;
    readonly wasmepubextractor_get_cover_image_len: (a: number) => any;
    readonly wasmepubextractor_get_metadata: (a: number) => any;
    readonly wasmepubextractor_get_metadata_is_valid: (a: number) => any;
    readonly wasmepubextractor_get_metadata_json: (a: number) => any;
    readonly wasmepubextractor_get_resource: (a: number, b: number, c: number) => any;
    readonly wasmepubextractor_get_title: (a: number) => any;
    readonly wasmepubextractor_get_toc: (a: number) => any;
    readonly wasmepubextractor_get_toc_json: (a: number) => any;
    readonly wasmepubextractor_get_total_char_count: (a: number) => any;
    readonly wasmepubextractor_get_total_word_count: (a: number) => any;
    readonly wasmepubextractor_has_cover: (a: number) => any;
    readonly wasmepubextractor_load_from_bytes: (a: number, b: any) => any;
    readonly wasmepubextractor_new: () => number;
    readonly wasmepubextractor_resolve_chapter_resource_path: (a: number, b: number, c: number, d: number) => any;
    readonly wasm_bindgen__convert__closures_____invoke__hab5d802ec56f1a23: (a: number, b: number, c: any) => [number, number];
    readonly wasm_bindgen__convert__closures_____invoke__h00187f8f3fee2983: (a: number, b: number, c: any, d: any) => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_destroy_closure: (a: number, b: number) => void;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
