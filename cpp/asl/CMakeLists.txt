cmake_minimum_required(VERSION 3.30)
project(ASL LANGUAGES CXX)

file(GLOB ASL_SRC "src/*.cpp")
add_library(asl ${ASL_SRC})
target_include_directories(asl PUBLIC "include")
target_compile_features(asl PUBLIC cxx_std_20)
