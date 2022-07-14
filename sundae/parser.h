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

#ifndef SUNDAE_PARSER_H
#define SUNDAE_PARSER_H

#include "sundae/lexer.h"

#include <vector>

namespace sundae {

namespace parser {

class Parser {
 public:
  Parser(std::vector<sundae::lexer::Token>) noexcept;
  void Parse();

 private:
  std::vector<sundae::lexer::Token> tokens_;
};

} // namespace parser

} // namespace sundae

#endif // SUNDAE_PARSER_H