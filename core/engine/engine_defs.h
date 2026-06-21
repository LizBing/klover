#ifndef ENGINE_ENGINE_DEFS_H_
#define ENGINE_ENGINE_DEFS_H_

typedef unsigned int stack_slot_t;

typedef struct Frame Frame;

typedef struct Registers Registers;
struct Registers {
    Frame* bp;
    stack_slot_t* sp;
    int pc;
};

#endif /* ENGINE_ENGINE_DEFS_H_ */
