cmake_minimum_required(VERSION 3.20)
project(klover LANGUAGES C)

set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

add_library(klover-core STATIC
  core/gc/gc.c
  core/gc/gc_heap.c
  core/memory/virt_space.c
)
set_target_properties(klover-core PROPERTIES
  C_STANDARD 11
  C_STANDARD_REQUIRED YES
  ARCHIVE_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}"
  ARCHIVE_OUTPUT_DIRECTORY_DEBUG "${CMAKE_BINARY_DIR}"
  ARCHIVE_OUTPUT_DIRECTORY_RELEASE "${CMAKE_BINARY_DIR}"
)
target_include_directories(klover-core PUBLIC "${CMAKE_CURRENT_SOURCE_DIR}/core")

include(CTest)
if(BUILD_TESTING)
  foreach(test_name IN ITEMS align gc_heap)
    add_executable(test_${test_name} core/tests/test_${test_name}.c)
    target_link_libraries(test_${test_name} PRIVATE klover-core)
    add_test(NAME ${test_name} COMMAND test_${test_name})
  endforeach()
endif()
