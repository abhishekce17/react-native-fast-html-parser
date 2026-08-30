project(fasthtmlparser)
cmake_minimum_required(VERSION 3.9.0)

set(PACKAGE_NAME fasthtmlparser)
set(CMAKE_VERBOSE_MAKEFILE ON)
set(CMAKE_CXX_STANDARD 20)

# Define C++ library and compile all sources
add_library(${PACKAGE_NAME} SHARED
    src/main/cpp/cpp-adapter.cpp
    ../cpp/HybridFastHtmlParser.cpp
)

# Add Nitrogen specs :)
include(${CMAKE_SOURCE_DIR}/../nitrogen/generated/android/fasthtmlparser+autolinking.cmake)

# Set up local includes
include_directories("src/main/cpp" "../cpp")

# Tell the linker where to search for the precompiled Rust shared library for this target
target_link_directories(${PACKAGE_NAME} PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/src/main/jniLibs/${ANDROID_ABI})

find_library(LOG_LIB log)

# Link all libraries together
target_link_libraries(
        ${PACKAGE_NAME}
        ${LOG_LIB}
        android # <-- Android core
        html_2_json # <-- Link precompiled Rust library by name directly
)
