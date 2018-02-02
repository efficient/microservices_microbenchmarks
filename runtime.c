#ifndef RUNTIME_C_
#define RUNTIME_C_

#include "time_utils.h"

#include <sys/time.h>
#include <dlfcn.h>
#include <errno.h>
#include <pthread.h>
#include <signal.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <ucontext.h>

struct libfunny {
	void *lib;
	void (*fun)(void);
	int *sbox;
};

#define BYTES_STACK 4096

static const char *ENTRY_POINT = "main";
static const char *STORAGE_LOC = "SBOX";

uint8_t stack[BYTES_STACK];
static struct {
	volatile const bool *enforcing;
	void (*response)(void);
	volatile const long long *checkpoint;
	volatile long long limit;
} preempt_conf;

static struct {
	long long last;
	long long durations;
	unsigned long count;
} preempt_stat;

static double preempt_mean_ns(void) {
	return (double) preempt_stat.durations / preempt_stat.count;
}

static void sigalrm(int signum, siginfo_t *siginfo, void *sigctxt) {
	(void) signum;
	(void) siginfo;

	int errnot = errno;
	long long ts = nsnow();
	errno = errnot;

	++preempt_stat.count;
	preempt_stat.durations += ts - preempt_stat.last;
	preempt_stat.last = ts;

	if(!*preempt_conf.enforcing)
		return;

	if(ts - *preempt_conf.checkpoint >= preempt_conf.limit) {
		mcontext_t *ctx = &((ucontext_t *) sigctxt)->uc_mcontext;
		long long *rsp = (long long *) ctx->gregs[REG_RSP];
		--*rsp;
		*rsp = ctx->gregs[REG_RIP];
		ctx->gregs[REG_RIP] = (long long) preempt_conf.response;
	}
}

static void sigterm(int signum) {
	(void) signum;

	sigset_t alrm;
	sigemptyset(&alrm);
	sigaddset(&alrm, SIGALRM);
	sigprocmask(SIG_BLOCK, &alrm, NULL);

	printf("\nQuantum: %f\n", preempt_mean_ns() / 1000.0);
	exit(0);
}

bool preempt_setup(long quantum, long long limit, volatile const bool *enforcing, volatile const long long *checkpoint, void (*response)(void)) {
	preempt_conf.enforcing = enforcing;
	preempt_conf.response = response;
	preempt_conf.checkpoint = checkpoint;
	preempt_conf.limit = limit;

	stack_t storage = {
		.ss_sp = stack,
		.ss_size = BYTES_STACK,
	};
	if(sigaltstack(&storage, NULL))
		return false;

	struct sigaction handler = {
		.sa_flags = SA_SIGINFO | SA_ONSTACK | SA_RESTART,
		.sa_sigaction = sigalrm,
	};
	if(sigaction(SIGALRM, &handler, NULL))
		return false;

	preempt_stat.last = nsnow();

	struct itimerval interval = {
		.it_value.tv_usec = quantum,
		.it_interval.tv_usec = quantum,
	};
	if(setitimer(ITIMER_REAL, &interval, NULL))
		return false;

	handler.sa_flags = 0;
	handler.sa_handler = sigterm;
	sigaction(SIGTERM, &handler, NULL);

	return true;
}

const char *dl_load(struct libfunny *exec, const char *sofile, bool preserve) {
	exec->lib = dlopen(sofile, RTLD_LAZY | RTLD_LOCAL | (-preserve & RTLD_NODELETE));
	if(!exec->lib)
		return dlerror();

	*(void **) &exec->fun = dlsym(exec->lib, ENTRY_POINT);
	if(!exec->fun)
		return dlerror();

	exec->sbox = dlsym(exec->lib, STORAGE_LOC);
	if(!exec->sbox)
		return dlerror();

	return NULL;
}

void dl_unload(struct libfunny exec) {
	dlclose(exec.lib);
}

#endif
