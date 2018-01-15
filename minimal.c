#include <errno.h>
#include <signal.h>
#include <stdbool.h>
#include <stdio.h>
#include <string.h>
#include <ucontext.h>
#include <unistd.h>

static ucontext_t restore;
static volatile bool undone = true;

static void sigusr1(int signum, siginfo_t *siginfo, void *sigctxt) {
	(void) signum;
	(void) siginfo;

	mcontext_t *ctx = &((ucontext_t *) sigctxt)->uc_mcontext;
	const mcontext_t *rst = &restore.uc_mcontext;
	greg_t segs = ctx->gregs[REG_CSGSFS];

	memcpy(ctx->gregs, rst->gregs, sizeof ctx->gregs);
	ctx->gregs[REG_CSGSFS] = segs;
	memcpy(ctx->fpregs, rst->fpregs, sizeof *ctx->fpregs);
	undone = false;
}

int main(void) {
	struct sigaction handler = {
		.sa_flags = SA_SIGINFO,
		.sa_sigaction = sigusr1,
	};
	if(sigaction(SIGUSR1, &handler, NULL)) {
		perror("sigaction()");
		return errno;
	}

	if(getcontext(&restore)) {
		perror("getcontext()");
		return errno;
	}
	puts(undone ? "true" : "false");
	if(undone && kill(getpid(), SIGUSR1)) {
		perror("kill()");
		return errno;
	}

	return 0;
}
