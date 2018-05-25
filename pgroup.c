#include <sys/wait.h>
#include <stddef.h>

int *wexitstatus(int *wstatus) {
	if(!wstatus || !WIFEXITED(*wstatus))
		return NULL;

	*wstatus = WEXITSTATUS(*wstatus);
	return wstatus;
}
