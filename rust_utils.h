#ifndef RUST_UTILS_H_
#define RUST_UTILS_H_

#include <stdbool.h>

bool call(void (*fun)(void));
void unwind(void);

#endif
