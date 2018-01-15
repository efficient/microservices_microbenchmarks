#include "time_utils.h"

#include <sys/time.h>
#include <errno.h>
#include <signal.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>

static volatile bool uncaught = true;

static void sigalrm(int signum) {
	(void) signum;

	uncaught = false;
}

int main(void) {
	static const suseconds_t LIMIT = 10; // us
	static const int TRIALS        = 1000000;

	struct sigaction handler = {
		.sa_handler = sigalrm,
	};
	if(sigaction(SIGALRM, &handler, NULL)) {
		perror("sigaction()");
		return errno;
	}

	long long runtime = nsnow();

#ifndef STUB_TIMER
	struct itimerval tout = {
		.it_value.tv_usec    = LIMIT,
	};
	struct itimerval notout = {0};
#endif

	long long *running = malloc(TRIALS * sizeof *running);
	for(int trial = 0; trial < TRIALS; ++trial) {
		long long ts = nsnow();
#ifndef STUB_TIMER
		if(setitimer(ITIMER_REAL, &tout, NULL)) {
			perror("setitimer()");
			return errno;
		}
		while(uncaught);
		if(setitimer(ITIMER_REAL, &notout, NULL)) {
			perror("setitimer()");
			return errno;
		}
#else
		while(nsnow() - ts < LIMIT * 1000);
#endif
		running[trial] = nsnow() - ts;
		uncaught = true;
	}

	runtime = nsnow() - runtime;
	for(int trial = 0; trial < TRIALS; ++trial)
		printf("%.03f\n", running[trial] / 1000.0);
	free(running);

	putchar('\n');
	printf("Tput: %f\n", TRIALS / (runtime / 1000000000.0));

	return 0;
}
