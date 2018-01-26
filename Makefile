override CPPFLAGS := $(CPPFLAGS)
override CFLAGS := -O2 -std=c99 -Wall -Wextra -Werror $(CFLAGS)
override LDFLAGS := $(LDFLAGS)
override LDLIBS := $(LDLIBS)
override RUSTFLAGS := -O -Dwarnings $(RUSTFLAGS)

RUSTC := rustc

.PHONY: all
all: kill_tput minimal preempt rust_test sigalrm_tput

kill_tput: private CPPFLAGS += -D_POSIX_C_SOURCE=199309L
kill_tput: time_utils.h

minimal: private CPPFLAGS += -D_GNU_SOURCE

preempt: private CPPFLAGS += -D_GNU_SOURCE
preempt: time_utils.h

rust_test: private LDFLAGS += -Wl,-u,unwind
rust_test: private RUSTFLAGS += --extern rust_utils=librust_utils.rlib
rust_test: librust_utils.rlib

sigalrm_tput: private CPPFLAGS += -D_POSIX_C_SOURCE=199309L
sigalrm_tput: time_utils.h

%: %.rs
	$(RUSTC) $(RUSTFLAGS) -Clink-args="$(LDFLAGS)" $< $(LDLIBS)

lib%.rlib: %.rs
	$(RUSTC) --crate-type lib $(RUSTFLAGS) -Clink-args="$(LDFLAGS)" $< $(LDLIBS)
