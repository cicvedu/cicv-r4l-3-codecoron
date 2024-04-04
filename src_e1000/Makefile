# SPDX-License-Identifier: GPL-2.0

# KDIR ?= ../linux

# default:
# 	$(MAKE) -C $(KDIR) M=$$PWD
# SPDX-License-Identifier: GPL-2.0

KDIR ?= ../linux
BUILD_DIR ?= $(PWD)/build

default:
	$(MAKE) -C $(KDIR) M=$$PWD

ra:
	$(MAKE) -C $(KDIR) LLVM=1 M=$$PWD rust-analyzer

PHONY += ra