NAME = codeharbor_sample
CC = gcc
CFLAGS = -Wall -Wextra -Werror -g
SRC = src/main.c

all: $(NAME)

$(NAME): $(SRC)
	$(CC) $(CFLAGS) -o $(NAME) $(SRC)

tests_run: $(NAME)
	./$(NAME) --self-test

clean:
	rm -f *.gcda *.gcno *.gcov *.log

fclean: clean
	rm -f $(NAME)
	rm -rf $(NAME).dSYM

re: fclean all

.PHONY: all tests_run clean fclean re
