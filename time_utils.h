#ifndef TIME_UTILS_H_
#define TIME_UTILS_H_

#include <assert.h>
#include <time.h>

static inline long long ns(const struct timespec *split) {
	return split->tv_sec * 1000000000 + split->tv_nsec;
}

static inline long long nsnow(void) {
	struct timespec stamp;
	int failure = clock_gettime(CLOCK_REALTIME, &stamp);
	assert(!failure);
	(void) failure;
	return ns(&stamp);
}

static inline long long nscpu(void) {
	struct timespec stamp;
	int failure = clock_gettime(CLOCK_PROCESS_CPUTIME_ID, &stamp);
	assert(!failure);
	(void) failure;
	return ns(&stamp);
}

#endif
