#ifndef EpubExtractor_H
#define EpubExtractor_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"


#include "EpubExtractor.d.h"






EpubExtractor* EpubExtractor_create(void);

size_t EpubExtractor_get_total_word_count(const EpubExtractor* self);

size_t EpubExtractor_get_total_char_count(const EpubExtractor* self);

void EpubExtractor_destroy(EpubExtractor* self);





#endif // EpubExtractor_H
