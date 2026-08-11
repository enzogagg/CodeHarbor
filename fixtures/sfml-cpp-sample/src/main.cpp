#include <SFML/Graphics.hpp>
#include <iostream>
#include <string>

static int run_self_test()
{
    sf::RenderWindow window(sf::VideoMode(64, 64), "CodeHarbor SFML sample", sf::Style::Close);
    if (!window.isOpen()) {
        std::cerr << "SFML self-test failed: window did not open\n";
        return 84;
    }

    window.clear(sf::Color::Black);
    window.display();
    window.close();
    std::cout << "SFML self-test passed\n";
    return 0;
}

int main(int argc, char **argv)
{
    if (argc == 2 && std::string(argv[1]) == "--self-test") {
        return run_self_test();
    }

    std::cout << "CodeHarbor SFML sample ready\n";
    return 0;
}
