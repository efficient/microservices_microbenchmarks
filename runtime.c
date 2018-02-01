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
static volatile const bool *enforcing;
static void (*response)(void);
static volatile const long long *checkpoint;
static volatile long long limit;

static void sigalrm(int signum, siginfo_t *siginfo, void *sigctxt) {
	(void) signum;
	(void) siginfo;

	if(!*enforcing)
		return;

	int errnot = errno;
	if(nsnow() - *checkpoint >= limit) {
		mcontext_t *ctx = &((ucontext_t *) sigctxt)->uc_mcontext;
		long long *rsp = (long long *) ctx->gregs[REG_RSP];
		--*rsp;
		*rsp = ctx->gregs[REG_RIP];
		ctx->gregs[REG_RIP] = (long long) response;
	}
	errno = errnot;
}

bool preempt_setup(long quantum, long long limit_val, volatile const bool *enforcing_loc, volatile const long long *checkpoint_loc, void (*response_code)(void)) {
	enforcing = enforcing_loc;
	response = response_code;
	checkpoint = checkpoint_loc;
	limit = limit_val;

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

	struct itimerval interval = {
		.it_value.tv_usec = quantum,
		.it_interval.tv_usec = quantum,
	};
	if(setitimer(ITIMER_REAL, &interval, NULL))
		return false;

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
