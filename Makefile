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

host: private RUSTFLAGS += -L. --cfg 'feature="invoke_$(INVOCATION)"'
host: ipc.rs libipc.a job.rs

kill_tput: private CPPFLAGS += -D_POSIX_C_SOURCE=199309L
kill_tput: time_utils.h

launcher: private LDLIBS += -ldl
launcher: private RUSTFLAGS += -L.
launcher: runtime.rs libruntime.a

minimal: private CPPFLAGS += -D_GNU_SOURCE

preempt: private CPPFLAGS += -D_GNU_SOURCE
preempt: time_utils.h

sigalrm_tput: private CPPFLAGS += -D_POSIX_C_SOURCE=199309L
sigalrm_tput: time_utils.h

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

lib%.so: %.rs
	$(RUSTC) --crate-type cdylib $(RUSTFLAGS) -Clink-args="$(LDFLAGS)" $< $(LDLIBS)
