#ifndef MEMORY_HEAP_WORD_H_
#define MEMORY_HEAP_WORD_H_

/* The object heap still allocates in words; virtual spaces use byte counts. */
typedef struct WordImpl WordImpl;
typedef WordImpl* Word;
typedef Word HeapWord;

#endif /* MEMORY_HEAP_WORD_H_ */
