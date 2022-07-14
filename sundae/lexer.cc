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

#include <cctype>
#include <iostream>

namespace sundae {

namespace lexer {

std::optional<TokenType> GetType(std::string expression) noexcept {
  auto has_literal_bound = [=](char delim) -> bool {
    return expression.length() > 1 && utils::StartsWith(expression, delim) &&
           utils::EndsWith(expression, delim) &&
           !utils::EndsWith(expression, '\\' + delim);
  };

  // Whether the expression is a valid "special" number (supports numbers in
  // binary, octal and hexadecimal form)
  auto has_special_number_bound =
      [=](std::string initial_pattern,
          std::function<bool(char)> predicate) -> bool {
    return utils::StartsWith(expression, initial_pattern) &&
           utils::EveryCharIsUnderscoreOr(expression.substr(2), predicate);
  };

  if (
      // Strings
      has_literal_bound(kStringBound) ||
      // Runes
      has_literal_bound(kRuneBound) ||
      // Bools
      utils::IsIn(expression, kBoolValues.first, kBoolValues.second) ||
      (
          // Integers
          utils::EveryCharIsUnderscoreOr(expression,
                                         [](char ch) { return isdigit(ch); }) ||
          // Floats
          (utils::EveryCharIsUnderscoreOr(expression,
                                          [](char ch) {
                                            return isdigit(ch) ||
                                                   utils::IsIn(ch, '.', 'E',
                                                               '+');
                                          }) &&
           std::count(expression.begin(), expression.end(), '.') == 1) ||
          (
              // Binary
              has_special_number_bound(
                  "0b", [](char ch) { return utils::IsIn(ch, '0', '1'); }) ||
              // Octal
              has_special_number_bound("0o",
                                       [](char ch) {
                                         return utils::IsIn(ch, '0', '1', '2',
                                                            '3', '4', '5', '6',
                                                            '7');
                                       }) ||
              // Hexadecimal
              has_special_number_bound("0x",
                                       [](char ch) { return isxdigit(ch); }))))
    return TokenType::kLiteral;
  else if (utils::Includes(kKeywords, expression))
    return TokenType::kKeyword;
  else if (utils::EveryCharIsUnderscoreOr(expression,
                                          [](char ch) { return isalnum(ch); }))
    return TokenType::kIdentifier;
  else if (utils::Includes(kOperators, expression))
    return TokenType::kOperator;
  else if (utils::Includes(kBreakers, expression))
    return TokenType::kBreaker;
  else if (expression == "\n")
    return TokenType::kNewline;
  else if (utils::AnyOfCommentPair([=](auto pair) {
             return utils::StartsWith(expression, pair.first) &&
                    utils::EndsWith(expression, pair.second);
           }))
    return TokenType::kComment;
  else
    return std::nullopt;
}

Lexer::Lexer(std::string buffer) noexcept
    : buffer_(std::move(buffer)), last_position_(0), current_position_(0) {}

std::vector<Token> Lexer::Tokenise() noexcept {
  auto update_pos = [&] { last_position_ = current_position_ + 1; };
  for (; current_position_ < buffer_.length(); ++current_position_) {
    char c = buffer_[current_position_];

    // Skip whitespace if the whitespace is irrelevant (isn't a newline) and we
    // haven't started collecting a new token (the whitespace isn't associated
    // with any token, such as a string literal)
    if (c != '\n' && isspace(c) && last_position_ == current_position_) {
      update_pos();
      continue;
    }

    std::string current = CurrentState();

    // Current must have a type for a dispatch to happen
    if (auto curr_type_opt = GetType(current)) {
      TokenType current_type = *std::move(curr_type_opt);

      // Should dispatch immediately if there's no next (EOF)
      if (auto next_opt = NextState()) {
        std::string &next = *next_opt;
        // We want to keep going if next is a comment's left bound (comments
        // have 2 chars bounds, so we have to check for them ahead, in next, a
        // 1-char-difference current check won't work)
        if (utils::AnyOfCommentPair(
                [=](auto pair) { return pair.first == next; })) {
          continue;
        }

        // Should dispatch immediately if next has no type (current is the last
        // form of a valid token continuously at this point)
        if (auto next_type_opt = GetType(next)) {
          TokenType next_type = *std::move(next_type_opt);

          // We shouldn't dispatch when we will get the same type on next or if
          // both are identifier branches
          if (current_type == next_type ||
              (utils::Includes(kIdentifierBranches, current_type) &&
               utils::Includes(kIdentifierBranches, next_type)))
            continue;
        }
      };

      // Dispatch new token
      Token token = {
          .value = current,
          .type = current_type,
          .position = std::make_pair(last_position_, current_position_)};
      update_pos();
      collected_.push_back(token);
    } else {
      if (!NextState().has_value()) {
        std::cout << "undefined token: '" << current << "'\n";
        exit(EXIT_FAILURE);
      }
    }
  }

  return collected_;
}

}  // namespace lexer

}  // namespace sundae
