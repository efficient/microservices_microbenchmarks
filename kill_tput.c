#include "time_utils.h"

#include <sys/wait.h>
#include <assert.h>
#include <errno.h>
#include <signal.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

static volatile bool uncaught = true;
static volatile long long timestamp;

static void sigchld(int signum, siginfo_t *siginfo, void *sigctxt) {
	(void) signum;
	(void) sigctxt;

	int errnom = errno;
	timestamp = nsnow();
	waitpid(siginfo->si_pid, NULL, 0);
	errno = errnom;
	uncaught = false;
}

int main(void) {
	static const int TRIALS = 10000;

	struct sigaction handler = {
		.sa_flags = SA_SIGINFO,
		.sa_sigaction = sigchld,
	};
	if(sigaction(SIGCHLD, &handler, NULL)) {
		perror("sigaction()");
		return errno;
	}

	long long *running = malloc(TRIALS * sizeof *running);
	for(int trial = 0; trial < TRIALS; ++trial) {
		int pid = fork();
		if(pid < 0) {
			perror("fork()");
			return errno;
		} else if(pid == 0) {
			while(true);
			assert(false);
		}

		long long ts = nsnow();
		if(kill(pid, SIGKILL)) {
			perror("kill()");
			return errno;
		}
		while(uncaught);
		uncaught = true;
		running[trial] = timestamp - ts;
	}

	for(int trial = 0; trial < TRIALS; ++trial)
		printf("%.03f\n", running[trial] / 1000.0);
	free(running);

	return 0;
}
