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
    std::string buf((std::istreambuf_iterator<char>(f)),
                    (std::istreambuf_iterator<char>()));
    sundae::lexer::Lexer lexer(buf);
    std::vector<sundae::lexer::Token> tokens = lexer.tokenise();

    // TODO: implement error handling, revise and add/improve comments, start
    // parsing (comment this for after)
    // TODO: choose fit LICENSE to unallow sublicensing and/or selling
    for (auto i = tokens.begin(); i != tokens.end(); ++i) {
        auto [value, type, pos] = *i;
        auto [init, end] = pos;
        std::cout << "TYPE: " << sundae::lexer::type_display(type)
                  << "\r\t\t\tPOS: [" << init << "..." << end << "]"
                  << "\r\t\t\t\t\t\tVALUE: \""
                  << std::regex_replace(value, std::regex("\n"), "\\n")
                  << "\"\n";
    }
}
