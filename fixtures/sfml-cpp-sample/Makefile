NAME = codeharbor_sfml_sample
CXX = g++
CXXFLAGS = -Wall -Wextra -Werror -g
LDLIBS = -lsfml-graphics -lsfml-window -lsfml-system
SRC = src/main.cpp

all: $(NAME)

$(NAME): $(SRC)
	$(CXX) $(CXXFLAGS) -o $(NAME) $(SRC) $(LDLIBS)

tests_run: $(NAME)
	xvfb-run --auto-servernum ./$(NAME) --self-test

clean:
	rm -f *.gcda *.gcno *.gcov *.log

fclean: clean
	rm -f $(NAME)
	rm -rf $(NAME).dSYM

re: fclean all

.PHONY: all tests_run clean fclean re
