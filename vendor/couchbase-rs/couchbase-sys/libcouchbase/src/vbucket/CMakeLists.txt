ADD_LIBRARY(vbucket-lcb OBJECT vbucket.c ketama.c ${SOURCE_ROOT}/contrib/cJSON/cJSON.c)
LCB_UTIL(vbucket-lcb)
