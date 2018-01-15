override CPPFLAGS := $(CPPFLAGS)
override CFLAGS := -O2 -std=c99 -Wall -Wextra -Werror $(CFLAGS)
override LDFLAGS := $(LDFLAGS)
override LDLIBS := $(LDLIBS)

.PHONY: all
all: kill_tput minimal sigalrm_tput

kill_tput: private CPPFLAGS += -D_POSIX_C_SOURCE=199309L
kill_tput: time_utils.h

minimal: private CPPFLAGS += -D_DEFAULT_SOURCE -D_GNU_SOURCE

sigalrm_tput: private CPPFLAGS += -D_POSIX_C_SOURCE=199309L
sigalrm_tput: time_utils.h
