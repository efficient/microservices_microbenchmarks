#ifndef RUNTIME_C_
#define RUNTIME_C_

#include <dlfcn.h>
#include <pthread.h>

static const char *ENTRY_POINT = "main";

struct libfun {
	void *lib;
	void (*fun)(void);
};

const char *dl_load(struct libfun *exec, const char *sofile) {
	exec->lib = dlopen(sofile, RTLD_NOW);
	if(!exec->lib)
		return dlerror();

	*(void **) &exec->fun = dlsym(exec->lib, ENTRY_POINT);
	if(!exec->fun)
		return dlerror();

	return NULL;
}

void dl_unload(struct libfun *exec) {
	dlclose(exec->lib);
}

#endif
