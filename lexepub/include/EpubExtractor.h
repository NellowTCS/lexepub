#ifndef EpubExtractor_H
#define EpubExtractor_H
#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#ifdef __cplusplus
namespace capi {
#endif

typedef struct EpubExtractor EpubExtractor;
#ifdef __cplusplus
} // namespace capi
#endif
#include "diplomat_result_void_void.h"
#ifdef __cplusplus
namespace capi {
extern "C" {
#endif

EpubExtractor* EpubExtractor_create(const char* path_data, size_t path_len);

EpubExtractor* EpubExtractor_create_from_bytes(const uint8_t* data_data, size_t data_len);

bool EpubExtractor_get_metadata_is_valid(EpubExtractor* self);

size_t EpubExtractor_get_chapter_count(EpubExtractor* self);

diplomat_result_void_void EpubExtractor_get_title(EpubExtractor* self, DiplomatWriteable* to);

diplomat_result_void_void EpubExtractor_get_metadata_json(EpubExtractor* self, DiplomatWriteable* to);

diplomat_result_void_void EpubExtractor_get_metadata(EpubExtractor* self, DiplomatWriteable* to);

diplomat_result_void_void EpubExtractor_get_chapters_text_json(EpubExtractor* self, DiplomatWriteable* to);

diplomat_result_void_void EpubExtractor_get_chapters_text(EpubExtractor* self, DiplomatWriteable* to);

diplomat_result_void_void EpubExtractor_get_chapter_text(EpubExtractor* self, size_t index, DiplomatWriteable* to);

diplomat_result_void_void EpubExtractor_get_chapter_json(EpubExtractor* self, size_t index, DiplomatWriteable* to);

diplomat_result_void_void EpubExtractor_get_toc_json(EpubExtractor* self, DiplomatWriteable* to);

diplomat_result_void_void EpubExtractor_get_toc(EpubExtractor* self, DiplomatWriteable* to);

diplomat_result_void_void EpubExtractor_resolve_chapter_resource_path(EpubExtractor* self, size_t chapter_index, const char* href_data, size_t href_len, DiplomatWriteable* to);

diplomat_result_void_void EpubExtractor_get_resource_json(EpubExtractor* self, const char* path_data, size_t path_len, DiplomatWriteable* to);

diplomat_result_void_void EpubExtractor_get_chapter_resource_json(EpubExtractor* self, size_t chapter_index, const char* href_data, size_t href_len, DiplomatWriteable* to);

diplomat_result_void_void EpubExtractor_get_chapter(EpubExtractor* self, size_t index, DiplomatWriteable* to);

size_t EpubExtractor_get_total_word_count(EpubExtractor* self);

size_t EpubExtractor_get_total_char_count(EpubExtractor* self);

bool EpubExtractor_has_cover(EpubExtractor* self);

size_t EpubExtractor_get_cover_image_len(EpubExtractor* self);

diplomat_result_void_void EpubExtractor_get_cover_image_format(EpubExtractor* self, DiplomatWriteable* to);

diplomat_result_void_void EpubExtractor_get_cover_image_json(EpubExtractor* self, DiplomatWriteable* to);
void EpubExtractor_destroy(EpubExtractor* self);

#ifdef __cplusplus
} // extern "C"
} // namespace capi
#endif
#endif
