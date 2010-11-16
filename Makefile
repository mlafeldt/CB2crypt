BIGINT = libbig_int

CC = gcc
WINDRES = windres

CFLAGS += -Wall -O2 -s
CFLAGS += -I$(BIGINT)/include
LIBS = -mwindows
PROG = CB2crypt.exe

OBJS += $(BIGINT)/src/basic_funcs.o
OBJS += $(BIGINT)/src/bitset_funcs.o
OBJS += $(BIGINT)/src/low_level_funcs/add.o
OBJS += $(BIGINT)/src/low_level_funcs/and.o
OBJS += $(BIGINT)/src/low_level_funcs/andnot.o
OBJS += $(BIGINT)/src/low_level_funcs/cmp.o
OBJS += $(BIGINT)/src/low_level_funcs/div.o
OBJS += $(BIGINT)/src/low_level_funcs/mul.o
OBJS += $(BIGINT)/src/low_level_funcs/or.o
OBJS += $(BIGINT)/src/low_level_funcs/sqr.o
OBJS += $(BIGINT)/src/low_level_funcs/sub.o
OBJS += $(BIGINT)/src/low_level_funcs/xor.o
OBJS += $(BIGINT)/src/memory_manager.o
OBJS += $(BIGINT)/src/modular_arithmetic.o
OBJS += $(BIGINT)/src/number_theory.o
OBJS += $(BIGINT)/src/service_funcs.o
OBJS += $(BIGINT)/src/str_funcs.o
OBJS += CB2crypt.o
OBJS += arcfour.o
OBJS += cb2_crypto.o
OBJS += mystring.o
OBJS += resource.o

all: $(PROG)

$(PROG): $(OBJS)
	$(CC) $(CFLAGS) -o $@ $? $(LIBS)

resource.o: resource.res
	$(WINDRES) -o $@ $?

clean:
	$(RM) $(PROG) $(OBJS)
