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

launcher: private LDFLAGS += -Wl,-u,unwind
launcher: private LDLIBS += -ldl
launcher: rust_utils.o libstd.so

libstd.so:
ifeq ($(wildcard $(shell $(RUSTC) --print sysroot)/lib/libstd-*.so),)
	ln -s $(wildcard $(shell $(RUSTC) --print sysroot)/lib/$(shell $(CC) -v 2>&1 | sed -n 's/Target: //p')/libstd-*.so) $@
else
	ln -sf $(wildcard $(shell $(RUSTC) --print sysroot)/lib/libstd-*.so) $@
endif

minimal: private CPPFLAGS += -D_GNU_SOURCE

preempt: private CPPFLAGS += -D_GNU_SOURCE
preempt: time_utils.h

rust_test: private LDFLAGS += -Wl,-u,unwind
rust_test: private RUSTFLAGS += --extern rust_utils=librust_utils.rlib
rust_test: librust_utils.rlib

rust_utils.o: rust_utils.h

sigalrm_tput: private CPPFLAGS += -D_POSIX_C_SOURCE=199309L
sigalrm_tput: time_utils.h

%: %.rs
	$(RUSTC) $(RUSTFLAGS) -Clink-args="$(LDFLAGS)" $< $(LDLIBS)

%.o: %.rs
	$(RUSTC) --emit obj --crate-type lib $(RUSTFLAGS) $<

lib%.rlib: %.rs
	$(RUSTC) --crate-type lib $(RUSTFLAGS) -Clink-args="$(LDFLAGS)" $< $(LDLIBS)

lib%.so: %.rs
	$(RUSTC) --crate-type cdylib $(RUSTFLAGS) -Clink-args="$(LDFLAGS)" $< $(LDLIBS)
