cmake_minimum_required(VERSION 3.0.0)
project(sundae VERSION 0.1.0)

include(CTest)
enable_testing()

include_directories(.)
add_executable(sundae sundae/lexer.cc sundae/main.cc)

set_property(TARGET sundae PROPERTY CXX_STANDARD 20)
set(CPACK_PROJECT_NAME ${PROJECT_NAME})
set(CPACK_PROJECT_VERSION ${PROJECT_VERSION})
include(CPack)
