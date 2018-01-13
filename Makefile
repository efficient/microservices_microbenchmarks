override CFLAGS := -O2 -std=c99 -Wall -Wextra -Werror $(CFLAGS)

sigalrm_tput: private CPPFLAGS += -D_POSIX_C_SOURCE=199309L
