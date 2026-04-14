---
title: "C/C++ Adapter"
description: "Integration layer for native applications via Diplomat"
---

# C/C++ Adapter

The C/C++ adapter is generated using Diplomat from the Rust core and tracks the same capability set as Rust/WASM.

## Headers and Linking

The generated definition files are located in the standard `include/` directory at the project root.

To integrate this module, include the generated header:

```c
#include "EpubExtractor.h"
```

## API reference

`EpubExtractor` currently provides:

- `EpubExtractor_create(path_data, path_len)`
- `EpubExtractor_create_from_bytes(data_data, data_len)`
- `EpubExtractor_get_metadata_is_valid(extractor)`
- `EpubExtractor_get_chapter_count(extractor)`
- `EpubExtractor_get_title(extractor, writeable)`
- `EpubExtractor_get_metadata(extractor, writeable)`
- `EpubExtractor_get_metadata_json(extractor, writeable)`
- `EpubExtractor_get_chapters_text(extractor, writeable)`
- `EpubExtractor_get_chapters_text_json(extractor, writeable)`
- `EpubExtractor_get_chapter(extractor, index, writeable)`
- `EpubExtractor_get_chapter_text(extractor, index, writeable)`
- `EpubExtractor_get_chapter_json(extractor, index, writeable)`
- `EpubExtractor_get_toc(extractor, writeable)`
- `EpubExtractor_get_toc_json(extractor, writeable)`
- `EpubExtractor_resolve_chapter_resource_path(extractor, chapter_index, href_data, href_len, writeable)`
- `EpubExtractor_get_resource_json(extractor, path_data, path_len, writeable)`
- `EpubExtractor_get_chapter_resource_json(extractor, chapter_index, href_data, href_len, writeable)`
- `EpubExtractor_get_total_word_count(extractor)`
- `EpubExtractor_get_total_char_count(extractor)`
- `EpubExtractor_has_cover(extractor)`
- `EpubExtractor_get_cover_image_len(extractor)`
- `EpubExtractor_get_cover_image_format(extractor, writeable)`
- `EpubExtractor_get_cover_image_json(extractor, writeable)`

Example:

```c
#include <stdbool.h>
#include <stddef.h>
#include <string.h>
#include "EpubExtractor.h"

int main(void) {
	const char* path = "book.epub";
	EpubExtractor* ex = EpubExtractor_create(path, strlen(path));
	if (ex != NULL) {
		bool metadata_ok = EpubExtractor_get_metadata_is_valid(ex);
		size_t chapter_count = EpubExtractor_get_chapter_count(ex);

		char title_buf[512];
		DiplomatWriteable title_w = diplomat_simple_writeable(title_buf, sizeof(title_buf));
		diplomat_result_void_void title_res = EpubExtractor_get_title(ex, &title_w);

		char metadata_buf[4096];
		DiplomatWriteable metadata_w = diplomat_simple_writeable(metadata_buf, sizeof(metadata_buf));
		diplomat_result_void_void metadata_res = EpubExtractor_get_metadata(ex, &metadata_w);

		char chapter_buf[8192];
		DiplomatWriteable chapter_w = diplomat_simple_writeable(chapter_buf, sizeof(chapter_buf));
		diplomat_result_void_void chapter_res = EpubExtractor_get_chapter(ex, 0, &chapter_w);

		size_t words = EpubExtractor_get_total_word_count(ex);
		size_t chars = EpubExtractor_get_total_char_count(ex);
		bool has_cover = EpubExtractor_has_cover(ex);
		size_t cover_len = EpubExtractor_get_cover_image_len(ex);

		char mime_buf[128];
		DiplomatWriteable mime_w = diplomat_simple_writeable(mime_buf, sizeof(mime_buf));
		diplomat_result_void_void mime_res = EpubExtractor_get_cover_image_format(ex, &mime_w);

		(void)metadata_ok;
		(void)chapter_count;
		(void)title_res;
		(void)metadata_res;
		(void)chapter_res;
		(void)words;
		(void)chars;
		(void)has_cover;
		(void)cover_len;
		(void)mime_res;

		EpubExtractor_destroy(ex);
	}
	return 0;
}
```

## Returns

- Methods like `get_metadata`, `get_chapters_text`, and `get_chapter` write JSON text into `DiplomatWriteable` outputs so C can consume structured Rust data without unsafe ABI struct coupling.
- Resource and cover byte payloads are exposed as JSON arrays (`get_resource_json`, `get_chapter_resource_json`, `get_cover_image_json`).
- `get_cover_image_len` and `get_cover_image_format` provide fast metadata access without large payload transfer.

## Build + regenerate headers

```bash
cd lexepub
cargo build --release --features c-ffi
diplomat-tool c include/
```
