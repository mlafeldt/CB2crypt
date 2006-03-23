##
# Makefile for CB2crypt
##

CC	= mingw32-gcc $(CINCS)
CINCS	= -I$(BIG_INT_DIR)/include
COPTS	= -O2 -Wall -s
CLIBS	= -mwindows
BIN	= CB2crypt.exe

# BIG_INT library
BIG_INT_DIR = libbig_int
BIG_INT_OBJS = \
	$(BIG_INT_DIR)/src/basic_funcs.o \
	$(BIG_INT_DIR)/src/bitset_funcs.o \
	$(BIG_INT_DIR)/src/memory_manager.o \
	$(BIG_INT_DIR)/src/modular_arithmetic.o \
	$(BIG_INT_DIR)/src/number_theory.o \
	$(BIG_INT_DIR)/src/service_funcs.o \
	$(BIG_INT_DIR)/src/str_funcs.o \
	$(BIG_INT_DIR)/src/low_level_funcs/add.o \
	$(BIG_INT_DIR)/src/low_level_funcs/and.o \
	$(BIG_INT_DIR)/src/low_level_funcs/andnot.o \
	$(BIG_INT_DIR)/src/low_level_funcs/cmp.o \
	$(BIG_INT_DIR)/src/low_level_funcs/div.o \
	$(BIG_INT_DIR)/src/low_level_funcs/mul.o \
	$(BIG_INT_DIR)/src/low_level_funcs/or.o \
	$(BIG_INT_DIR)/src/low_level_funcs/sqr.o \
	$(BIG_INT_DIR)/src/low_level_funcs/sub.o \
	$(BIG_INT_DIR)/src/low_level_funcs/xor.o

OBJS	= $(BIG_INT_OBJS) arcfour.o cb2_crypto.o CB2crypt.o mystring.o resource.o

all:		${OBJS}
		${CC} $(COPTS) -o ${BIN} ${OBJS} $(CLIBS)
clean:
		rm -f ${BIN} ${OBJS}

rebuild:	clean all

pack:
		upx -9 ${BIN}

resource.o:	resource.res
		windres -o resource.o resource.res
