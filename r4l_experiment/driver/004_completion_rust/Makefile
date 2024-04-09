ifneq ($(KERNELRELEASE),)

# In kbuild context
module-objs := completion_rust.o	
obj-m := completion_rust.o

CFLAGS_hello_world.o := -DDEBUG
else
KDIR := ../../../linux
PWD := $(shell pwd)

all:
	$(MAKE) LLVM=1 -C $(KDIR)  M=$(PWD) modules

.PHONY: clean

ra:
	$(MAKE) -C $(KDIR) LLVM=1 M=$$PWD rust-analyzer

PHONY += ra

clean:
	rm -f *.ko *.o .*.cmd .*.o.d *.mod *.mod.o *.mod.c *.symvers *.markers *.unsigned *.order *~
endif