#ifndef THREAD_JAVA_THREAD_H_
#define THREAD_JAVA_THREAD_H_

#include "utils/global_defs.h"

typedef struct JavaThread JavaThread;
struct JavaThread {
    size_t stack_size;
    char* stack;
};

#endif /* THREAD_JAVA_THREAD_H_ */
