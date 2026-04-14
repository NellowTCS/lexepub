---
title: "C/C++ Adapter"
description: "Integration layer for native applications via Diplomat"
---

# C/C++ Adapter

The C and C++ adapter allows native applications to directly leverage asynchronous LexePub functionality. The bindings are generated transparently using the `diplomat` toolchain out of the core Rust API, ensuring robust cross-language FFI integration.

## Headers and Linking

The generated definition files are located in the standard `include/` directory at the project root.

To integrate this module, you link dynamically or statically against the generated artifact `libLexePub`, including the provided headers:

```c
#include "EpubExtractor.h"
```

Because these functions correspond 1:1 with the underlying abstractions, no intermediate memory copy is required between C++ variables and internal Rust strings. You can find full invocation examples within the repository.
