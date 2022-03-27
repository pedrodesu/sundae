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

std::optional<TokenType> GetType(std::string expression) noexcept {
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
           utils::EveryCharIsUnderscoreOr(expression.substr(2), predicate);
  };

  if (has_literal_bound(kStringBound) || has_literal_bound(kRuneBound) ||
      utils::IsIn(expression, kBoolValues.first, kBoolValues.second) ||
      (utils::EveryCharIsUnderscoreOr(expression,
                                      [](auto ch) { return isdigit(ch); }) ||
       (utils::EveryCharIsUnderscoreOr(expression,
                                       [](auto ch) {
                                         return isdigit(ch) ||
                                                utils::IsIn(ch, '.', 'E', '+');
                                       }) &&
        std::count(expression.begin(), expression.end(), '.') == 1) ||
       (has_special_number_bound(
            "0b", [](auto ch) { return utils::IsIn(ch, '0', '1'); }) ||
        has_special_number_bound("0o",
                                 [](auto ch) {
                                   return utils::IsIn(ch, '0', '1', '2', '3',
                                                      '4', '5', '6', '7');
                                 }) ||
        has_special_number_bound("0x", [](auto ch) { return isxdigit(ch); }))))
    return kLiteral;
  else if (utils::Includes(kKeywords, expression))
    return kKeyword;
  else if (utils::EveryCharIsUnderscoreOr(expression,
                                          [](auto ch) { return isalnum(ch); }))
    return kIdentifier;
  else if (utils::Includes(kOperators, expression))
    return kOperator;
  else if (utils::Includes(kBreakers, expression))
    return kBreaker;
  else if (expression == "\n")
    return kNewline;
  else if (utils::AnyOfComment([expression](auto pair) {
             return expression.starts_with(pair.first) &&
                    expression.ends_with(pair.second);
           }))
    return kComment;
  else
    return std::nullopt;
}

Token::Token(std::string value, TokenType type,
             std::pair<uint, uint> position) noexcept
    : value(value), type(type), position(position) {}

Lexer::Lexer(std::string buffer) noexcept
    : buffer_(std::move(buffer)), last_position_(0), current_position_(0) {}

std::vector<Token> Lexer::Tokenise() {
  auto update_pos = [this]() { last_position_ = current_position_ + 1; };
  for (; current_position_ < buffer_.length(); ++current_position_) {
    char c = buffer_[current_position_];
    if (c != '\n' && isspace(c) && last_position_ == current_position_) {
      update_pos();
      continue;
    }

    std::string current = CurrentState();

    if (auto curr_type_opt = GetType(current)) {
      TokenType &current_type = *curr_type_opt;

      auto dispatch =
          [current_type, current, update_pos, this,
           position = std::make_pair(last_position_, current_position_ + 1)]() {
            Token token(current, current_type, position);
            update_pos();
            collected_.push_back(token);
          };

      if (auto next = NextState()) {
        if (utils::AnyOfComment(
                [next](auto pair) { return pair.first == *next; })) {
          continue;
        }

        if (auto next_type_opt = GetType(*next)) {
          TokenType &next_type = *next_type_opt;

          if (current_type == next_type && current_type != kPrioritised ||
              (current_type == kIdentifier &&
               utils::Includes(kIdentifierUpgrades, next_type)))
            continue;
        }
      }

      dispatch();
    }
  }

  return collected_;
}

}  // namespace lexer

}  // namespace compiler

}  // namespace sundae
