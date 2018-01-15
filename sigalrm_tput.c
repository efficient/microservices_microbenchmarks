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
	struct itimerval clock = {
		.it_interval.tv_usec = LIMIT,
		.it_value.tv_usec    = LIMIT,
	};
	if(setitimer(ITIMER_REAL, &clock, NULL)) {
		perror("setitimer()");
		return errno;
	}

	long long *running = malloc(TRIALS * sizeof *running);
	for(int trial = 0; trial < TRIALS; ++trial) {
		long long ts = nsnow();
		while(uncaught);
		running[trial] = nsnow() - ts;
		uncaught = true;
	}

	clock.it_value.tv_usec = 0;
	if(setitimer(ITIMER_REAL, &clock, NULL)) {
		perror("setitimer()");
		return errno;
	}

	runtime = nsnow() - runtime;
	for(int trial = 0; trial < TRIALS; ++trial)
		printf("%.03f\n", running[trial] / 1000.0);
	free(running);

	putchar('\n');
	printf("Tput: %f\n", TRIALS / (runtime / 1000000000.0));

	return 0;
}
