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

#include "sundae/lexer.h"

#include <ctype.h>

#include <algorithm>
#include <iostream>
#include <optional>
#include <string>
#include <vector>

namespace sundae {

inline namespace compiler {

namespace lexer {

std::optional<TokenType> get_type(std::string expression) {
    auto has_literal_bound = [expression](char delim) -> bool {
        return expression.starts_with(delim) && expression.ends_with(delim) &&
               !expression.ends_with('\\' + delim) && expression.length() > 1;
    };

    auto all_is_underscore_or =
        [expression](std::function<bool(char)> predicate) -> bool {
        return all_of(
            expression.begin(), expression.end(),
            [&predicate](auto ch) { return ch == '_' || predicate(ch); });
    };

    if (utils::includes(kKeywords, expression))
        return kKeyword;
    else if (utils::includes(kBreakers, expression))
        return kBreaker;
    else if (utils::includes(kOperators, expression))
        return kOperator;
    // TODO: finish complex number literal form impl (0x..., 0b..., 3.5)
    else if (has_literal_bound(kStringBound) || has_literal_bound(kRuneBound) ||
             expression == kBoolValues.first ||
             expression == kBoolValues.second ||
             all_is_underscore_or([](auto ch) { return isdigit(ch); }))
        return kLiteral;
    else if (all_is_underscore_or([](auto ch) { return isalnum(ch); }))
        return kIdentifier;
    else if (expression == "\n")
        return kNewline;
    else if (utils::any_of_comment([expression](auto pair) {
                 return expression.starts_with(pair.first) &&
                        expression.ends_with(pair.second);
             }))
        return kComment;
    else
        return std::nullopt;
}

Token::Token(std::string value, TokenType type, std::pair<uint, uint> pos)
    : value(value), type(type), pos(pos) {}

Lexer::Lexer(std::string buf) : buf(buf), last_pos(0), curr_pos(0) {}

std::vector<Token> Lexer::tokenise() {
    auto update_pos = [this]() { last_pos = curr_pos + 1; };
    for (; curr_pos < buf.length(); ++curr_pos) {
        char c = buf[curr_pos];
        if (c != '\n' && isspace(c) && last_pos == curr_pos) {
            update_pos();
            continue;
        }

        std::string curr = curr_state();

        if (auto curr_type_opt = get_type(curr)) {
            TokenType &curr_type = *curr_type_opt;

            auto dispatch = [this, update_pos, &curr_type, &curr]() {
                Token token(curr, curr_type,
                            std::make_pair(last_pos, curr_pos + 1));
                update_pos();
                collected.push_back(token);
            };

            if (auto next = next_state()) {
                if (utils::any_of_comment(
                        [next](auto pair) { return pair.first == *next; })) {
                    continue;
                }

                if (auto next_type_opt = get_type(*next)) {
                    TokenType &next_type = *next_type_opt;
                    if (curr_type == next_type && curr_type != kPrioritised ||
                        (curr_type == kIdentifier &&
                         utils::includes(kIdentifierUpgrades, next_type)))
                        continue;
                }
            }

            dispatch();
        }
    }

    return collected;
}

}  // namespace lexer

}  // namespace compiler

}  // namespace sundae
