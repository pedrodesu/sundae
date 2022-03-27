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

#ifndef SUNDAE_LEXER_H_
#define SUNDAE_LEXER_H_

#include <algorithm>
#include <array>
#include <functional>
#include <optional>
#include <string>
#include <tuple>
#include <vector>

namespace sundae {

inline namespace compiler {

namespace lexer {

// The special keywords the language contains
const std::array<std::string, 6> kKeywords = {"pub",  "const", "struct",
                                              "enum", "func",  "use"};

// The separators the language contains
// These don't have the semantic value an operator would have, and exist only
// for expression delimiting purposes
const std::array<std::string, 5> kBreakers = {"(", ")", "{", "}", ","};

// The operators the language contains
const std::array<std::string, 6> kOperators = {":=", "=", "+", "-", "*", "/"};

// The bounds for comments
const std::array<std::pair<std::string, std::string>, 2> kCommentPairs = {
    std::make_pair("//", "\n"), std::make_pair("/*", "*/")};

// The both boolean literal expressions
const std::pair<std::string, std::string> kBoolValues =
    std::make_pair("true", "false");

// The bound for string literals
const char kStringBound = '\'';

// The bound for rune literals
// In this language, a rune is not equivalent to a char, it's rather an
// extension of the latter, a Unicode grapheme
const char kRuneBound = '`';

enum TokenType {
  kKeyword,
  kBreaker,
  kOperator,
  kLiteral,
  kIdentifier,
  kNewline,
  kComment,
};

// The identifier branches (with the identifier itself)
// Technically all of the following can be replaced with the identifier type, so
// this definition allows changes to and from a desired identifier branch change
const std::array<TokenType, 2> kIdentifierBranches = {kIdentifier, kKeyword};

namespace utils {

inline bool StartsWith(std::string haystack, char needle) {
  std::string str_needle(1, needle);
  return haystack.compare(0, str_needle.length(), str_needle) == 0;
}

inline bool EndsWith(std::string haystack, char needle) {
  std::string str_needle(1, needle);
  return haystack.compare(haystack.length() - str_needle.length(),
                          str_needle.length(), str_needle) == 0;
}

inline bool StartsWith(std::string haystack, std::string needle) {
  return haystack.compare(0, needle.length(), needle) == 0;
}

inline bool EndsWith(std::string haystack, std::string needle) {
  return haystack.compare(haystack.length() - needle.length(), needle.length(),
                          needle) == 0;
}

// Returns whether the given predicate passes for any comment pair
inline bool AnyOfCommentPair(
    std::function<bool(std::pair<std::string, std::string>)> pred) {
  return std::any_of(kCommentPairs.begin(), kCommentPairs.end(),
                     [=](auto pair) { return pred(pair); });
}

// Returns whether the haystack array includes the element needle
template <typename T, std::size_t U>
inline bool Includes(std::array<T, U> haystack, T needle) {
  return std::find(haystack.begin(), haystack.end(), needle) != haystack.end();
}

inline bool EveryCharIsUnderscoreOr(std::string value,
                                    std::function<bool(char)> predicate) {
  return std::all_of(value.begin(), value.end(),
                     [=](char ch) { return ch == '_' || predicate(ch); });
}

template <typename T, typename... U>
inline bool IsIn(T first, U... t) noexcept {
  return ((first == t) || ...);
}

}  // namespace utils

std::optional<TokenType> Type(std::string expr) noexcept;

inline std::string TypeDisplay(TokenType type) noexcept {
  switch (type) {
    case kKeyword:
      return "KEYWORD";
    case kBreaker:
      return "BREAKER";
    case kOperator:
      return "OPERATOR";
    case kLiteral:
      return "LITERAL";
    case kIdentifier:
      return "IDENTIFIER";
    case kNewline:
      return "NEWLINE";
    case kComment:
      return "COMMENT";
  }
}

struct Token {
  std::string value;
  TokenType type;
  std::pair<int, int> position;
};

class Lexer {
 public:
  Lexer(std::string) noexcept;
  std::vector<Token> Tokenise();

  inline std::string CurrentState() const noexcept {
    return *Seek(current_position_);
  }

  inline std::optional<std::string> NextState() const noexcept {
    return Seek(current_position_ + 1);
  }

 private:
  std::string buffer_;
  int last_position_;
  int current_position_;
  std::vector<Token> collected_;

  // Returns a buffer slice from last_pos to provided next_position. If out of
  // bounds, returns an empty option
  inline std::optional<std::string> Seek(int next_position) const noexcept {
    return next_position < buffer_.length()
               ? std::optional(buffer_.substr(
                     last_position_, next_position - last_position_ + 1))
               : std::nullopt;
  }
};

}  // namespace lexer

}  // namespace compiler

}  // namespace sundae

#endif  // SUNDAE_LEXER_H_
