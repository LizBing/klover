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

set(KLOVER_SANITIZERS "" CACHE STRING "C sanitizers: address,undefined or undefined")
if(KLOVER_SANITIZERS)
  if(NOT CMAKE_C_COMPILER_ID MATCHES "Clang|GNU")
    message(FATAL_ERROR "KLOVER_SANITIZERS requires Clang or GCC")
  endif()
  target_compile_options(klover-core PUBLIC "-fsanitize=${KLOVER_SANITIZERS}" -fno-omit-frame-pointer)
  target_link_options(klover-core PUBLIC "-fsanitize=${KLOVER_SANITIZERS}")
endif()

include(CTest)
if(BUILD_TESTING)
  find_package(Threads REQUIRED)
  foreach(test_name IN ITEMS harness align obj_model virt_space gc_heap gc_exhaustion gc_concurrent)
    add_executable(test_${test_name} core/tests/test_${test_name}.c)
    target_link_libraries(test_${test_name} PRIVATE klover-core)
    if(test_name STREQUAL "harness")
      add_test(NAME harness COMMAND "${CMAKE_COMMAND}"
        "-DHARNESS=$<TARGET_FILE:test_harness>"
        -P "${CMAKE_CURRENT_SOURCE_DIR}/tests/check-harness.cmake")
    else()
      add_test(NAME ${test_name} COMMAND test_${test_name})
    endif()
    set_tests_properties(${test_name} PROPERTIES TIMEOUT 30)
    if(test_name MATCHES "^(harness|align|obj_model|virt_space)$")
      set_tests_properties(${test_name} PROPERTIES LABELS "address-compatible")
    endif()
  endforeach()
  target_link_libraries(test_gc_concurrent PRIVATE Threads::Threads)
endif()
