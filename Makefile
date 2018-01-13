override CFLAGS := -O2 -std=c99 -Wall -Wextra -Werror $(CFLAGS)

.PHONY: all
all: kill_tput sigalrm_tput

kill_tput: private CPPFLAGS += -D_POSIX_C_SOURCE=199309L
kill_tput: time_utils.h

sigalrm_tput: private CPPFLAGS += -D_POSIX_C_SOURCE=199309L
sigalrm_tput: time_utils.h
