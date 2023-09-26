#include <iostream>

template <typename T>
void swap(T &a, T &b)
{
    T c = a;
    a = b;
    b = c;
}

unsigned int op(int a, int b)
{
    return a * b;
}

int main()
{
    unsigned int a = 10;
    unsigned int b = 32;

    op(a, b);

    swap(a, b);

    std::cout << a << std::endl;
}
