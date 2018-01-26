#include <sys/shm.h>
#include <stdbool.h>
#include <stddef.h>

struct smeminternal {
	int id;
	void *data;
};

bool sretrieve(struct smeminternal *region) {
	return (region->data = shmat(region->id, NULL, 0)) != (void *) -1;
}

bool salloc(struct smeminternal *region, size_t size, int flags) {
	if((region->id = shmget(0, size, IPC_CREAT | flags)) < 0)
		return false;
	return sretrieve(region);
}

void sdealloc(struct smeminternal region) {
	shmdt(region.data);
}
