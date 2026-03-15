Panic

Panic Handler

Linking

Entry points

Bare metal Runtime

## Page Table
- RAM data structure mapping virtual pages to physical frames.
- Kernel constructs it
- [virtual_page → physical_frame | flags]
- on x86 CR3 register points to the root table, MMU talks table and translates virt > physi.

