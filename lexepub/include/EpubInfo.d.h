#ifndef EpubInfo_D_H
#define EpubInfo_D_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"





typedef struct EpubInfo {
  DiplomatStringView title;
  DiplomatStringView author;
} EpubInfo;

typedef struct EpubInfo_option {union { EpubInfo ok; }; bool is_ok; } EpubInfo_option;



#endif // EpubInfo_D_H
