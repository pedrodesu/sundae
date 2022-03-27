cmake_minimum_required(VERSION 3.0.0)
project(sundae VERSION 0.1.0)

set(CMAKE_CXX_STANDARD 17)

include(FetchContent)
FetchContent_Declare(
    googletest
    # Update commit regularly
    URL https://github.com/google/googletest/archive/e2239ee6043f73722e7aa812a459f54a28552929.zip
)
# For Windows: Prevent overriding the parent project's compiler/linker settings
set(gtest_force_shared_crt ON CACHE BOOL "" FORCE)
FetchContent_MakeAvailable(googletest)

enable_testing()

include_directories(.)
list(APPEND SOURCES main lexer)

list(SUBLIST SOURCES 1 -1 TEST_SOURCES)
foreach(test ${TEST_SOURCES})
    add_executable(${test} sundae/${test}_test.cc)
    target_link_libraries(${test} gtest_main)
    add_test(${test} ${test})
endforeach()

list(TRANSFORM SOURCES PREPEND sundae/)
list(TRANSFORM SOURCES APPEND .cc)
add_executable(sundae ${SOURCES})

set(CPACK_PROJECT_NAME ${PROJECT_NAME})
set(CPACK_PROJECT_VERSION ${PROJECT_VERSION})
include(CPack)
