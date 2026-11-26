/* Synthetic .modinfo fixture. Compile as an ELF relocatable object. */
__attribute__((used, section(".modinfo"))) static const char release[] =
    "vermagic=6.1.0 SMP";
__attribute__((used, section(".modinfo"))) static const char license[] =
    "license=MIT";
extern void fixture_init(void);
void fixture_entry(void) { fixture_init(); }
