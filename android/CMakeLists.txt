project(fasthtmlparser)
cmake_minimum_required(VERSION 3.9.0)

set(PACKAGE_NAME fasthtmlparser)
set(CMAKE_VERBOSE_MAKEFILE ON)
set(CMAKE_CXX_STANDARD 20)

# ── Lexbor HTML parser (downloaded at build time via FetchContent) ────────────
include(FetchContent)
FetchContent_Declare(
  lexbor
  GIT_REPOSITORY https://github.com/lexbor/lexbor.git
  GIT_TAG        v2.3.0
)
FetchContent_MakeAvailable(lexbor)

# Define C++ library and compile all sources
add_library(${PACKAGE_NAME} SHARED
    src/main/cpp/cpp-adapter.cpp
    ../cpp/HybridFastHtmlParser.cpp
)

# Add Nitrogen specs :)
include(${CMAKE_SOURCE_DIR}/../nitrogen/generated/android/fasthtmlparser+autolinking.cmake)

# Set up local includes
include_directories("src/main/cpp" "../cpp")

# Include Lexbor headers
target_include_directories(${PACKAGE_NAME} PRIVATE ${lexbor_SOURCE_DIR}/source)

find_library(LOG_LIB log)

# Link all libraries together
target_link_libraries(
        ${PACKAGE_NAME}
        ${LOG_LIB}
        android
        lexbor_static  # <-- Lexbor C++ HTML parser
)
