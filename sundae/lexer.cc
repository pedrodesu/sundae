/*
Copyright (C) 2022 Pedro Ferreira

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <http://www.gnu.org/licenses/>.
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

    // Whether the expression is a valid "special" number (supports numbers in
    // binary, octal and hexadecimal form)
    auto has_special_number_bound =
        [expression](std::string initial_pattern,
                     std::function<bool(char)> predicate) -> bool {
        return expression.starts_with(initial_pattern) &&
               utils::every_char_is_underscore_or(expression.substr(2),
                                                  predicate);
    };

    if (has_literal_bound(kStringBound) || has_literal_bound(kRuneBound) ||
        utils::is_in(expression, kBoolValues.first, kBoolValues.second) ||
        (utils::every_char_is_underscore_or(
             expression, [](auto ch) { return isdigit(ch); }) ||
         (utils::every_char_is_underscore_or(expression,
                                             [](auto ch) {
                                                 return isdigit(ch) ||
                                                        utils::is_in(ch, '.',
                                                                     'E', '+');
                                             }) &&
          std::count(expression.begin(), expression.end(), '.') == 1) ||
         (has_special_number_bound(
              "0b", [](auto ch) { return utils::is_in(ch, '0', '1'); }) ||
          has_special_number_bound("0o",
                                   [](auto ch) {
                                       return utils::is_in(ch, '0', '1', '2',
                                                           '3', '4', '5', '6',
                                                           '7');
                                   }) ||
          has_special_number_bound("0x",
                                   [](auto ch) { return isxdigit(ch); }))))
        return kLiteral;
    else if (utils::includes(kKeywords, expression))
        return kKeyword;
    else if (utils::every_char_is_underscore_or(
                 expression, [](auto ch) { return isalnum(ch); }))
        return kIdentifier;
    else if (utils::includes(kOperators, expression))
        return kOperator;
    else if (utils::includes(kBreakers, expression))
        return kBreaker;
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

            auto dispatch = [curr_type, curr, update_pos, this,
                             pos = std::make_pair(last_pos, curr_pos + 1)]() {
                Token token(curr, curr_type, pos);
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
