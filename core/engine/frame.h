#ifndef ENGINE_FRAME_H_
#define ENGINE_FRAME_H_

#include "engine/engine_defs.h"
#include "utils/global_defs.h"

struct Frame {
    Registers ctx;
    
    int argc;
    int max_locals;
};

bool create_frame(Registers*, int argc, int max_locals);
bool frame_unwind(Registers*);

#endif /* FRAME_FRAME_H_ */
