// Copyright (C) 2022 Pedro Ferreira
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

#include "sundae/lexer.h"

#include "gtest/gtest.h"

namespace sundae {

inline namespace compiler {

namespace lexer {

namespace utils {

TEST(String, StartsWith) {
  EXPECT_TRUE(StartsWith("Hello, world!", "Hello, "));
  EXPECT_FALSE(StartsWith("Hello, world!", "world"));

  EXPECT_TRUE(StartsWith("Hello, world!", 'H'));
  EXPECT_FALSE(StartsWith("Hello, world!", 'w'));
}

TEST(String, EndsWith) {
  EXPECT_TRUE(EndsWith("Hello, world!", "ld!"));
  EXPECT_FALSE(EndsWith("Hello, world!", "Hel"));

  EXPECT_TRUE(EndsWith("Hello, world!", '!'));
  EXPECT_FALSE(EndsWith("Hello, world!", ','));
}

TEST(TypeHelpers, AnyOfCommentPair) {
  EXPECT_TRUE(AnyOfCommentPair([](auto pair) { return pair.first == "//"; }));
  EXPECT_TRUE(AnyOfCommentPair([](auto pair) { return pair.second == "*/"; }));

  EXPECT_FALSE(AnyOfCommentPair([](auto pair) { return pair.first == "!"; }));
}

TEST(TypeHelpers, IncludesWithNumbers) {
  std::array<int, 3> test_case_1{2, 4, 8};
  std::array<int, 3> test_case_2{8, 43, 4565};
  std::array<float, 6> test_case_3{2.0f, 4.5f, 8.9f, 1.6f, 45.76f, 43.7f};
  std::array<float, 6> test_case_4{58.4f,  445.57f, 1245.7f,
                                   125.0f, 14.87f,  1185.6f};

  EXPECT_TRUE(Includes(test_case_1, 4));
  EXPECT_FALSE(Includes(test_case_2, 9));

  EXPECT_TRUE(Includes(test_case_3, 43.7f));
  EXPECT_FALSE(Includes(test_case_4, 9.0f));
}

TEST(TypeHelpers, IncludesWithString) {
  std::array<std::string, 3> test_case_1{"One", "Two", "Three"};
  std::array<std::string, 2> test_case_2{"Four", "Five"};

  EXPECT_TRUE(Includes(test_case_1, std::string("Two")));
  EXPECT_FALSE(Includes(test_case_2, std::string("Two")));
}

TEST(TypeHelpers, EveryCharIsUnderscoreOr) {
  EXPECT_TRUE(EveryCharIsUnderscoreOr("1_000_000",
                                      [](char ch) { return isdigit(ch); }));

  EXPECT_FALSE(EveryCharIsUnderscoreOr("hELLO_WORLD",
                                       [](char ch) { return isupper(ch); }));
}

TEST(TypeHelpers, IsIn) {
  EXPECT_TRUE(IsIn('c', 'a', 'b', 'c', 'd'));
  EXPECT_FALSE(IsIn(24.0f, 12.45f, 858.0f));
}

}  // namespace utils

TEST(Type, Strings) {}

}  // namespace lexer

}  // namespace compiler

}  // namespace sundae
