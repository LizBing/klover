cmake_minimum_required(VERSION 3.15)

project(myjvm C)

set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

set(CMAKE_C_STANDARD 11)

add_library(jvm SHARED
  core/gc/gc.c
)

target_include_directories(jvm
  PUBLIC
  ${CMAKE_SOURCE_DIR}/core
)

