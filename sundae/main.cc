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

#include <fstream>
#include <iostream>
#include <regex>
#include <string>

#include "sundae/lexer.h"

#define ENTRY_POINT "../examples/use_cases.su"

int main() {
  std::ifstream f(ENTRY_POINT);
  if (!f) {
    std::cerr << "error getting file\n";
    exit(EXIT_FAILURE);
  }
  std::string buffer((std::istreambuf_iterator<char>(f)),
                     (std::istreambuf_iterator<char>()));
  sundae::lexer::Lexer lexer(buffer);
  std::vector<sundae::lexer::Token> tokens = lexer.Tokenise();

  // TODO: impl unit testing
  // TODO: implement error handling
  // TODO: start parsing
  for (auto i = tokens.begin(); i != tokens.end(); ++i) {
    auto [value, type, position] = *i;
    auto [init, end] = position;
    std::cout << "TYPE: " << sundae::lexer::TypeDisplay(type)
              << "\r\t\t\tPOS: [" << init << "..." << end << "]"
              << "\r\t\t\t\t\t\tVALUE: \""
              << std::regex_replace(value, std::regex("\n"), "\\n") << "\"\n";
  }
}
