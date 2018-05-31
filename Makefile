override CPPFLAGS := $(CPPFLAGS) -DNDEBUG
override CFLAGS := $(if $(NOPTS),-Og,-O2) -g -std=c99 -Wall -Wextra -Werror $(CFLAGS)
override LDFLAGS := $(LDFLAGS)
override LDLIBS := $(LDLIBS)
override RUSTFLAGS := $(if $(NOPTS),,-O) -g -Dwarnings $(RUSTFLAGS)

CARGO := cargo
RUSTC := rustc

ifdef NOPTS
$(warning \vvvvvvvvvvvvvvvvvvvvvvvvvvvvv/)
$(warning = PRODUCING UNOPTIMIZED BUILD =)
$(warning /^^^^^^^^^^^^^^^^^^^^^^^^^^^^^\)
endif

host: private LDLIBS += -lstatic=ipc
host: private RUSTFLAGS += -L. --cfg 'feature="invoke_$(INVOCATION)"'
host: bytes.rs ipc.rs libipc.a job.rs pgroup.rs libpgroup.a ringbuf.rs

launcher: private LDLIBS += -ldl -lstatic=ipc
launcher: private RUSTFLAGS += -L. --cfg 'feature="$(UNLOADING)_loaded"'
launcher: ipc.rs libipc.a job.rs runtime.rs libruntime.a

shasum: private RUSTFLAGS += -Lhasher/target/release/deps
shasum: hasher/rlib

test: private RUSTFLAGS += -L. -Crpath -Funsafe-code
test: libbytes.rlib libipc.so time.rs

ipc.o: private CPPFLAGS += -D_XOPEN_SOURCE

runtime.o: private CPPFLAGS += -D_GNU_SOURCE
runtime.o: time_utils.h

libipc.so: private LDLIBS += -lstatic=ipc
libipc.so: private RUSTFLAGS += -L. --crate-type dylib -Cprefer-dynamic
libipc.so: libipc.a

libsleep.so: private RUSTFLAGS += -L. -Funsafe-code
libsleep.so: libspc.rlib time.rs

libtest.so: private RUSTFLAGS += -L. -Funsafe-code
libtest.so: libbytes.rlib libspc.rlib time.rs

.PHONY: clean
clean:
	$(RM) $(filter-out $(shell grep -H ^/ $(shell git ls-files .gitignore '*/.gitignore') | sed 's/\.gitignore:\///'),$(shell git clean -nX | cut -d" " -f3-))
	-rm -r $(shell git clean -ndX | cut -d" " -f3- | grep '/$$')

.PHONY: distclean
distclean: clean
	git clean -fX

%/rlib:
	cd $* && $(CARGO) rustc --release --no-default-features -- --crate-type rlib

%/so: libspc.rlib
	cd $* && $(CARGO) build --release

%: %.rs
	$(RUSTC) $(RUSTFLAGS) -Clink-args="$(LDFLAGS)" $< $(LDLIBS)

lib%.a: %.o
	$(AR) rs $@ $^

lib%.rlib: %.rs
	$(RUSTC) --crate-type rlib --cfg 'feature="no_mangle_main"' $(RUSTFLAGS) -Clink-args="$(LDFLAGS)" $< $(LDLIBS)

lib%.so: %.rs
	$(RUSTC) --crate-type cdylib --cfg 'feature="no_mangle_main"' $(RUSTFLAGS) -Clink-args="$(LDFLAGS)" $< $(LDLIBS)
	@objdump -t $@ | grep '\<g\>.*\<F\>' >/dev/null 2>&1 || { \
		echo "IT ONLY MAKES SENSE TO BUILD $(basename $@) AS A STATIC LIBRARY" >&2; \
		rm $@; \
		false; \
	}
