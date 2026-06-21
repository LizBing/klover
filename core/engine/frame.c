#include "engine/frame.h"
#include "engine/engine_defs.h"

bool create_frame(Registers* regs, int argc, int max_locals) {
    size_t frame_slots = sizeof(Frame) / sizeof(stack_slot_t);
    
    Registers ctx = {
        .bp = regs->bp,
        .sp = (regs->sp + argc),
        .pc = regs->pc
    };

    regs->sp -= frame_slots;
    Frame* new_frame = (void*)regs->sp;
    new_frame->ctx = ctx;
    new_frame->argc = argc;
    new_frame->max_locals = max_locals;

    regs->sp -= max_locals;
    regs->bp = new_frame;
    regs->pc = 0;

    return true;
}

bool frame_unwind(Registers* regs) {
    if (regs->bp == NULL)
        return false;

    *regs = regs->bp->ctx;

    return true;
}
