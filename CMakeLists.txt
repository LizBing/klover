cmake_minimum_required(VERSION 3.15)

project(myjvm C)

set(CMAKE_C_COMPILER clang)

set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

set(CMAKE_C_STANDARD 11)

# --- jvm shared library ---

add_library(klover-core SHARED
  core/gc/gc.c
  core/gc/gc_heap.c
  core/gc/oop_storage.c
  core/memory/virt_space.c
  core/metaspace/metaspace.c
)

target_include_directories(klover-core
  PUBLIC
  ${CMAKE_SOURCE_DIR}/core
)

# --- tests ---

enable_testing()

add_executable(test_virt_space
  core/tests/test_virt_space.c
)

target_link_libraries(test_virt_space PRIVATE klover-core)
target_include_directories(test_virt_space PRIVATE ${CMAKE_SOURCE_DIR})

add_test(NAME virt_space COMMAND test_virt_space)

add_executable(test_metaspace
  core/tests/test_metaspace.c
)

target_link_libraries(test_metaspace PRIVATE klover-core)
target_include_directories(test_metaspace PRIVATE ${CMAKE_SOURCE_DIR})

add_test(NAME metaspace COMMAND test_metaspace)

add_executable(test_oop_storage
  core/tests/test_oop_storage.c
)

target_link_libraries(test_oop_storage PRIVATE klover-core)
target_include_directories(test_oop_storage PRIVATE ${CMAKE_SOURCE_DIR})

add_test(NAME oop_storage COMMAND test_oop_storage)

add_executable(test_gc_heap
  core/tests/test_gc_heap.c
)

target_link_libraries(test_gc_heap PRIVATE klover-core)
target_include_directories(test_gc_heap PRIVATE ${CMAKE_SOURCE_DIR})

add_test(NAME gc_heap COMMAND test_gc_heap)
