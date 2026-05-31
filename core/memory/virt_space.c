#include "core/memory/virt_space.h"

#include <stdlib.h>
#include <sys/mman.h>
#include <unistd.h>

/* -------------------------------------------------------------------------- */

VirtSpace* create_virt_space(uintptr_t base, size_t word_size, bool exec) {
    size_t byte_size = word_size * sizeof(HeapWord);
    
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
    
    if (base != 0) {
        flags |= MAP_FIXED;
    }
    
    void* addr = mmap((void*)base, byte_size, prot, flags, -1, 0);
    if (addr == MAP_FAILED) {
        return NULL;
    }
    
    VirtSpace* vs = (VirtSpace*)malloc(sizeof(VirtSpace));
    if (vs == NULL) {
        munmap(addr, byte_size);
        return NULL;
    }

    vs->start      = (HeapWord*)addr;
    vs->end        = vs->start + word_size;
    vs->commit_top = vs->start;
    vs->exec       = exec;
    
    return vs;
}

/* -------------------------------------------------------------------------- */

void destroy_virt_space(VirtSpace* vs) {
    if (vs == NULL) return;
    
    size_t byte_size = (vs->end - vs->start) * sizeof(HeapWord);
    munmap((void*)vs->start, byte_size);
    free(vs);
}

/* -------------------------------------------------------------------------- */

bool vs_expand(VirtSpace* vs, size_t word_size, bool touch) {
    size_t reserved = vs_reserved(vs);
    size_t committed = vs_committed(vs);
    if (word_size > reserved - committed) {
        return false;
    }
    
    HeapWord* old_commit_top = vs->commit_top;
    vs->commit_top += word_size;
    
    if (touch) {
        size_t page_size = (size_t)getpagesize();
        size_t byte_len = word_size * sizeof(HeapWord);
        volatile char* p = (volatile char*)old_commit_top;
        volatile char* end = p + byte_len;
        
        /* Write to the first byte of every page in the newly committed
         * range.  This forces the OS to allocate physical backing before
         * we return, avoiding on-demand page faults later. */
        for (; p < end; p += page_size) {
            *p = 0;
        }
    }
    
    return true;
}

/* -------------------------------------------------------------------------- */

bool vs_shrink(VirtSpace* vs, size_t word_size) {
    size_t committed = vs_committed(vs);
    if (word_size > committed) {
        return false;
    }
    
    vs->commit_top -= word_size;
    
    /*
     * Tell the OS the pages are no longer needed so physical memory /
     * swap can be reclaimed.
     */
    size_t release_bytes = word_size * sizeof(HeapWord);
    void* release_start = (void*)vs->commit_top;
#if defined(MADV_DONTNEED)
    madvise(release_start, release_bytes, MADV_DONTNEED);
#elif defined(MADV_FREE)
    madvise(release_start, release_bytes, MADV_FREE);
#endif
    
    return true;
}
