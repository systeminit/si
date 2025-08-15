/* Stub libdevmapper.h for remote execution environments */
#ifndef _LIB_DEVICE_MAPPER_H
#define _LIB_DEVICE_MAPPER_H

#include <stdint.h>
#include <sys/types.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Minimal type definitions needed for devicemapper-sys bindings */
struct dm_task;
struct dm_info;

/* Function declarations that devicemapper-sys might need for bindings */
struct dm_task *dm_task_create(int type);
void dm_task_destroy(struct dm_task *dmt);
int dm_task_run(struct dm_task *dmt);

#ifdef __cplusplus
}
#endif

#endif /* _LIB_DEVICE_MAPPER_H */