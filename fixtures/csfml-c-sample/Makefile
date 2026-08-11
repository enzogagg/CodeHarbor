NAME = codeharbor_csfml_sample
CC = gcc
CFLAGS = -Wall -Wextra -Werror -g
LDLIBS = -lcsfml-graphics -lcsfml-window -lcsfml-system
SRC = src/main.c

all: $(NAME)

$(NAME): $(SRC)
	$(CC) $(CFLAGS) -o $(NAME) $(SRC) $(LDLIBS)

tests_run: $(NAME)
	xvfb-run --auto-servernum ./$(NAME) --self-test

clean:
	rm -f *.gcda *.gcno *.gcov *.log

fclean: clean
	rm -f $(NAME)
	rm -rf $(NAME).dSYM

re: fclean all

.PHONY: all tests_run clean fclean re
