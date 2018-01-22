#include "rust_utils.h"

#include <dlfcn.h>
#include <pthread.h>
#include <stddef.h>
#include <stdio.h>

int main(void) {
	void *lib = dlopen("./libtest.so", RTLD_NOW);
	if(!lib) {
		fprintf(stderr, "%s\n", dlerror());
		return 2;
	}
	void (*fun)(void) = NULL;
	*(void **) &fun = dlsym(lib, "main");
	if(!fun) {
		fprintf(stderr, "%s\n", dlerror());
		return 3;
	}

	call(fun);

	dlclose(lib);
	// Eliminate valgrind false positive that obfuscates testing
	pthread_exit(NULL);
	return 0;
}
