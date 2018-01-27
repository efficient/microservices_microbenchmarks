override CPPFLAGS := $(CPPFLAGS)
override CFLAGS := $(if $(NOPTS),-Og,-O2) -g -std=c99 -Wall -Wextra -Werror $(CFLAGS)
override LDFLAGS := $(LDFLAGS)
override LDLIBS := $(LDLIBS)
override RUSTFLAGS := $(if $(NOPTS),,-O) -g -Dwarnings $(RUSTFLAGS)

RUSTC := rustc

ifdef NOPTS
$(warning \vvvvvvvvvvvvvvvvvvvvvvvvvvvvv/)
$(warning = PRODUCING UNOPTIMIZED BUILD =)
$(warning /^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\)
endif

.PHONY: all
all: kill_tput minimal preempt rust_test sigalrm_tput

host: private LDLIBS += -lstatic=ipc
host: private RUSTFLAGS += -L. --cfg 'feature="invoke_$(INVOCATION)"'
host: ipc.rs libipc.a job.rs

kill_tput: private CPPFLAGS += -D_POSIX_C_SOURCE=199309L
kill_tput: time_utils.h

launcher: private LDLIBS += -ldl
launcher: private RUSTFLAGS += -L.
launcher: job.rs runtime.rs libruntime.a

minimal: private CPPFLAGS += -D_GNU_SOURCE

preempt: private CPPFLAGS += -D_GNU_SOURCE
preempt: time_utils.h

sigalrm_tput: private CPPFLAGS += -D_POSIX_C_SOURCE=199309L
sigalrm_tput: time_utils.h

test: private RUSTFLAGS += -L. -Crpath -Funsafe-code
test: libipc.so time.rs

libipc.rlib: libipc.a

libipc.so: private LDLIBS += -lstatic=ipc
libipc.so: private RUSTFLAGS += -L. --crate-type dylib -Cprefer-dynamic
libipc.so: libipc.a

libtest.so: private RUSTFLAGS += -L. -Funsafe-code
libtest.so: libspc.rlib time.rs

.PHONY: clean
clean:
	$(RM) $(filter-out $(shell grep -H ^/ $(shell git ls-files .gitignore '*/.gitignore') | sed 's/\.gitignore:\///'),$(shell git clean -nX | cut -d" " -f3-))

.PHONY: distclean
distclean:
	git clean -fX

%: %.rs
	$(RUSTC) $(RUSTFLAGS) -Clink-args="$(LDFLAGS)" $< $(LDLIBS)

lib%.a: %.o
	$(AR) rs $@ $^

lib%.rlib: %.rs
	$(RUSTC) --crate-type rlib --cfg 'feature="no_mangle_main"' $(RUSTFLAGS) -Clink-args="$(LDFLAGS)" $< $(LDLIBS)

lib%.so: %.rs
	$(RUSTC) --crate-type cdylib --cfg 'feature="no_mangle_main"' $(RUSTFLAGS) -Clink-args="$(LDFLAGS)" $< $(LDLIBS)
