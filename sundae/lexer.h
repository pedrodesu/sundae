/*
Copyright (c) 2022 Pedro Ferreira

Permission is hereby granted, free of charge, to any person obtaining a copy of
this software and associated documentation files (the "Software"), to deal in
the Software without restriction, including without limitation the rights to
use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies
of the Software, and to permit persons to whom the Software is furnished to do
so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/

#pragma once

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

// The direct allowed upgrades to identifiers
// This has to do with the lexer's internal logic. Technically all of the
// following can be described as generic identifiers, so we define this as so
// the lexer does not panic on a desired type change
const std::array<TokenType, 1> kIdentifierUpgrades = {kKeyword};

// Type which overrides getting the other types and whose predicate will be
// checked even for the peeked value, so as to check if its type should be
// switched accordingly
const TokenType kPrioritised = kComment;

namespace utils {

inline bool any_of_comment(
    std::function<bool(std::pair<std::string, std::string>)> pred) {
    return std::any_of(kCommentPairs.begin(), kCommentPairs.end(),
                       [pred](auto pair) { return pred(pair); });
}

template <typename T, std::size_t U>
inline bool includes(std::array<T, U> haystack, T needle) {
    return std::find(haystack.begin(), haystack.end(), needle) !=
           haystack.end();
}

}  // namespace utils

std::optional<TokenType> get_type(std::string expr);

inline std::string type_display(TokenType type) {
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
    std::pair<uint, uint> pos;
    Token(std::string, TokenType, std::pair<uint, uint>);
};

class Lexer {
   public:
    Lexer(std::string);
    std::vector<Token> tokenise();

    inline std::string curr_state() const { return seek(curr_pos).value(); }

    inline std::optional<std::string> next_state() const {
        return seek(curr_pos + 1);
    }

   private:
    std::string buf;
    uint last_pos;
    uint curr_pos;
    std::vector<Token> collected;

    // Returns a buffer slice from last_pos to provided next_pos. If out of
    // bounds, returns an empty option
    inline std::optional<std::string> seek(uint next_pos) const {
        return next_pos < buf.length() ? std::optional(buf.substr(
                                             last_pos, next_pos - last_pos + 1))
                                       : std::nullopt;
    }
};

}  // namespace lexer

}  // namespace compiler

}  // namespace sundae
