#ifndef TIME_UTILS_H_
#define TIME_UTILS_H_

#include <sys/resource.h>
#include <time.h>

static inline long long ns(const struct timespec *split) {
	return split->tv_sec * 1000000000 + split->tv_nsec;
}

static inline long long nsnow(void) {
	struct timespec stamp;
	clock_gettime(CLOCK_REALTIME, &stamp);
	return ns(&stamp);
}

static inline long us(const struct timeval *split) {
	return split->tv_sec * 1000000 + split->tv_usec;
}

static inline long uscpu(void) {
	struct rusage use;
	getrusage(RUSAGE_SELF, &use);
	return us(&use.ru_utime) + us(&use.ru_stime);
}

#endif
