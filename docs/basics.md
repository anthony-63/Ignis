# Basics of Ignis

#### This document lists all the current features of Ignis and will be updated as the langauge changes

## Hello World

```
include "std/io.ig"

main -> sub() {
    writes("hello world");
}
```

### Include rules
- Every global symbol created is exported by default

- Any symbols included by a file is not exported by the parent
- - Example: If I have ``lib.ig`` include a file called ``sdl.ig``, ``lib.ig`` will be able to access every symbol defined in ``sdl.ig``. But if I make another file called ``main.ig`` and include ``lib.ig``, ``main.ig`` can **ONLY** read symbols defined by ``lib.ig`` and will not have access to ``sdl.ig`` unless included explicitly by ``main.ig``
 