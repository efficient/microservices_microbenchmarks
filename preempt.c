#include "time_utils.h"

#include <sys/time.h>
#include <assert.h>
#include <errno.h>
#include <signal.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <ucontext.h>

static void uservice(int duration) {
	long long ts = nsnow();
	while(nsnow() - ts < duration);
}

static volatile long deadline;
static volatile bool expected;
static int kills;
static ucontext_t restore;
static volatile int trial;

static void sigalrm(int signum, siginfo_t *siginfo, void *sigctxt) {
	(void) signum;
	(void) siginfo;

	int errbak = errno;
	long time = uscpu();
	if(time < deadline) {
		struct itimerval defer = {
			.it_value.tv_usec = deadline - time,
		};
		if(setitimer(ITIMER_REAL, &defer, NULL)) {
			perror("setitimer(defer)");
			exit(errno);
		}
		errno = errbak;
		return;
	}
	errno = errbak;

	assert(expected);
	++kills;

	mcontext_t *ctx = &((ucontext_t *) sigctxt)->uc_mcontext;
	const mcontext_t *rst = &restore.uc_mcontext;
	greg_t segs = ctx->gregs[REG_CSGSFS];

	memcpy(ctx->gregs, rst->gregs, sizeof ctx->gregs);
	ctx->gregs[REG_CSGSFS] = segs;
	memcpy(ctx->fpregs, rst->fpregs, sizeof *ctx->fpregs);

	++trial;
}

int main(void) {
	static const int DELAY_MIN = 0; // us
	static const int DELAY_MAX = 6; // us
	static const suseconds_t LIMIT = 5; // us
	static const int TRIALS = 1000000;

	int *jobs = malloc(TRIALS * sizeof *jobs);
	srand(nsnow());
	for(int job = 0; job < TRIALS; ++job)
		jobs[job] = rand() % (DELAY_MAX - DELAY_MIN + 1) + DELAY_MIN;

	struct sigaction handler = {
		.sa_flags = SA_SIGINFO,
		.sa_sigaction = sigalrm,
	};
	if(sigaction(SIGALRM, &handler, NULL)) {
		perror("sigaction()");
		return errno;
	}

	struct itimerval tout = {
		.it_value.tv_usec = LIMIT,
	};
	struct itimerval notout = {0};

	int bounties = 0;
	if(getcontext(&restore)) {
		perror("getcontext()");
		return errno;
	}
	for(; trial < TRIALS; ++trial) {
		int delay = jobs[trial];
		if((expected = delay >= LIMIT))
			++bounties;

		if(setitimer(ITIMER_REAL, &tout, NULL)) {
			perror("setitimer(tout)");
			return errno;
		}
		deadline = uscpu() + LIMIT;
		uservice(expected * 1000);
		if(setitimer(ITIMER_REAL, &notout, NULL)) {
			perror("setitimer(notout)");
			return errno;
		}
	}

	printf("Caught %d of %d CPU hogs\n", kills, bounties);

	free(jobs);
	return 0;
}
