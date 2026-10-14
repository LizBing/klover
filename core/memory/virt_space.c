#include "memory/virt_space.h"
#include "utils/align.h"

#include <stdlib.h>
#include <sys/mman.h>

struct VirtSpace {
    uintptr_t start;
    uintptr_t end;
    uintptr_t commit_top;
    bool exec;
};

uintptr_t vs_start(VirtSpace* vs) {
    return vs->start;
}

size_t vs_reserved(VirtSpace* vs) {
    return vs->end - vs->start;
}

size_t vs_committed(VirtSpace* vs) {
    return vs->commit_top - vs->start;
}

bool vs_exec(VirtSpace* vs) {
    return vs->exec;
}

/* -------------------------------------------------------------------------- */

VirtSpace* create_virt_space(uintptr_t base, size_t size, bool exec) {
    size_t page_size = get_vm_page_size();

    if (0 == size) {
        size = page_size;
    } else {
        if (size > SIZE_MAX - (page_size - 1)) {
            return NULL;
        }
        size = align_up(size, page_size);
    }

    if (base != 0 && (!is_aligned(base, page_size) || base > UINTPTR_MAX - size)) {
        return NULL;
    }
    
    /*
     * Always create the mapping READ | WRITE.
     * We do NOT combine PROT_WRITE and PROT_EXEC in a single mmap call
     * because macOS hardened runtime (Apple Silicon) enforces W^X and
     * will reject such mappings.
     *
     * The 'exec' flag is stored; a caller that needs execute permission
     * must later toggle protection via mprotect / pthread_jit_write_protect_np.
     */
    int prot = PROT_READ | PROT_WRITE;
    (void)exec;
    
    int flags = MAP_PRIVATE;
#ifdef MAP_ANONYMOUS
    flags |= MAP_ANONYMOUS;
#else
    flags |= MAP_ANON;
#endif
    
    /* Request base as a hint, so an occupied range can never be replaced.
     * Compressed pointers require this exact address: reject and release any
     * alternative mapping chosen by the OS. No separate availability check
     * is needed, so concurrent reservations cannot create a check/use race. */
    void* addr = mmap((void*)base, size, prot, flags, -1, 0);
    if (addr == MAP_FAILED) {
        return NULL;
    }
    if (base != 0 && (uintptr_t)addr != base) {
        munmap(addr, size);
        return NULL;
    }
    
    VirtSpace* vs = (VirtSpace*)malloc(sizeof(VirtSpace));
    if (vs == NULL) {
        munmap(addr, size);
        return NULL;
    }

    *vs = (VirtSpace){
        .start      = (uintptr_t)addr,
        .end        = (uintptr_t)addr + size,
        .commit_top = (uintptr_t)addr,
        .exec       = exec,
    };
    
    return vs;
}

/* -------------------------------------------------------------------------- */

void destroy_virt_space(VirtSpace* vs) {
    if (vs == NULL) return;
    
    size_t byte_size = vs->end - vs->start;
    munmap((void*)vs->start, byte_size);
    free(vs);
}

/* -------------------------------------------------------------------------- */

uintptr_t vs_expand_by(VirtSpace* vs, size_t size, bool touch) {
    size_t reserved = vs_reserved(vs);
    size_t committed = vs_committed(vs);
    if (size > reserved - committed) {
        return 0;
    }
    
    uintptr_t old_commit_top = vs->commit_top;
    vs->commit_top += size;
    
    if (touch) {
        size_t page_size = get_vm_page_size();
        uintptr_t address = old_commit_top;
        
        /* Write to the first byte of every page in the newly committed
         * range.  This forces the OS to allocate physical backing before
         * we return, avoiding on-demand page faults later. */
        while (address < vs->commit_top) {
            *(volatile char*)address = 0;
            size_t step = page_size - address % page_size;
            if (step >= vs->commit_top - address) {
                break;
            }
            address += step;
        }
    }
    
    return old_commit_top;
}

/* -------------------------------------------------------------------------- */

uintptr_t vs_shrink_by(VirtSpace* vs, size_t size) {
    size_t committed = vs_committed(vs);
    if (size > committed) {
        return 0;
    }
    
    vs->commit_top -= size;
    
    /*
     * Tell the OS the pages are no longer needed so physical memory /
     * swap can be reclaimed.
     */
    size_t release_bytes = size;
    void* release_start = (void*)vs->commit_top;
#if defined(MADV_DONTNEED)
    madvise(release_start, release_bytes, MADV_DONTNEED);
#elif defined(MADV_FREE)
    madvise(release_start, release_bytes, MADV_FREE);
#endif
    
    return vs->commit_top;
}
