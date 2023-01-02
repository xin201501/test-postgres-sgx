# Licensed to the Apache Software Foundation (ASF) under one
# or more contributor license agreements.  See the NOTICE file
# distributed with this work for additional information
# regarding copyright ownership.  The ASF licenses this file
# to you under the Apache License, Version 2.0 (the
# "License"); you may not use this file except in compliance
# with the License.  You may obtain a copy of the License at
#
#   http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing,
# software distributed under the License is distributed on an
# "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
# KIND, either express or implied.  See the License for the
# specific language governing permissions and limitations
# under the License.

#######  PostgresQL version #######
PG_MAJOR_VERSION_NUMBER ?= 15
PG_DETAIL_VERSION_NUMBER ?= 15.1
PG_HOME = /home/$(USER)/.pgx/$(PG_DETAIL_VERSION_NUMBER)/pgx-install
PG_DATA = /home/xin201501/.pgx/data-$(PG_MAJOR_VERSION_NUMBER)

####### debug or release ########
DEBUG ?= 1

####### C and C++ compiler #######
CC ?= gcc-10
CXX ?= g++-10

######## SGX SDK Settings ########

SGX_SDK ?= /opt/intel/sgxsdk
SGX_MODE ?= HW
SGX_ARCH ?= x64

TOP_DIR := ../..
include $(TOP_DIR)/buildenv.mk

ifeq ($(shell getconf LONG_BIT), 32)
	SGX_ARCH := x86
else ifeq ($(findstring -m32, $(CXXFLAGS)), -m32)
	SGX_ARCH := x86
endif

ifeq ($(SGX_ARCH), x86)
	SGX_COMMON_CFLAGS := -m32
	SGX_LIBRARY_PATH := $(SGX_SDK)/lib
	SGX_BIN_PATH := $(SGX_SDK)/bin/x86
else
	SGX_COMMON_CFLAGS := -m64
	SGX_LIBRARY_PATH := $(SGX_SDK)/lib64
	SGX_BIN_PATH := $(SGX_SDK)/bin/x64
endif

ifeq ($(DEBUG), 1)
SGX_COMMON_CFLAGS += -g
endif 

SGX_EDGER8R := $(SGX_BIN_PATH)/sgx_edger8r
ifneq ($(SGX_MODE), HYPER)
	SGX_ENCLAVE_SIGNER := $(SGX_BIN_PATH)/sgx_sign
else
	SGX_ENCLAVE_SIGNER := $(SGX_BIN_PATH)/sgx_sign_hyper
	SGX_EDGER8R_MODE := --sgx-mode $(SGX_MODE)
endif

######## CUSTOM Settings ########

CUSTOM_LIBRARY_PATH := ./lib
CUSTOM_BIN_PATH := ./bin
CUSTOM_SYSROOT_PATH := ./sysroot
CUSTOM_EDL_PATH := $(ROOT_DIR)/sgx_edl/edl
CUSTOM_COMMON_PATH := $(ROOT_DIR)/common

######## EDL Settings ########

Enclave_EDL_Files := enclave/enclave_t.c enclave/enclave_t.h app/enclave_u.c app/enclave_u.h

######## APP Settings ########

# App_Rust_Flags := --release
App_Src_Files := $(shell find app/ -type f -name '*.rs') $(shell find app/ -type f -name 'Cargo.toml')
App_Include_Paths := -I ./app -I$(SGX_SDK)/include -I$(CUSTOM_COMMON_PATH)/inc -I$(CUSTOM_EDL_PATH)
App_C_Flags := $(CFLAGS) -fPIC -Wno-attributes $(App_Include_Paths)

App_Enclave_u_Object := $(CUSTOM_LIBRARY_PATH)/libenclave_u.a

######## Enclave Settings ########

# BUILD_STD=no       use no_std
# BUILD_STD=cargo    use cargo-std-aware
# BUILD_STD=xargo    use xargo
BUILD_STD ?= cargo

Rust_Build_Target := x86_64-unknown-linux-sgx
Rust_Target_Path := $(ROOT_DIR)/rustlib

ifneq ($(BUILD_STD), cargo)
ifneq ($(BUILD_STD), xargo)
$(error Only supports building with build_std strategy!!)
endif
endif

ifeq ($(BUILD_STD), cargo)
	Rust_Build_Std := --release -Z build-std=core,alloc
	Rust_Std_Features := --features backtrace,stdio,thread,untrusted_fs,unsupported_process
	Rust_Target_Flags := --target $(Rust_Target_Path)/$(Rust_Build_Target).json
	Rust_Sysroot_Path := $(CURDIR)/sysroot
	Rust_Sysroot_Flags := RUSTFLAGS="--sysroot $(Rust_Sysroot_Path)"
else
	Rust_Unstable_Flags := RUSTFLAGS="-Z force-unstable-if-unmarked"
endif

ifeq ($(DEBUG),	0)
	RustEnclave_Build_Flags := --release
	PGX_Build_Flags := --release
endif

RustEnclave_Src_Files := $(shell find enclave/ -type f -name '*.rs') $(shell find enclave/ -type f -name 'Cargo.toml')
RustEnclave_Include_Paths := -I$(CUSTOM_COMMON_PATH)/inc -I$(CUSTOM_COMMON_PATH)/inc/tlibc -I$(CUSTOM_EDL_PATH)

RustEnclave_Link_Libs := -L$(SGX_LIBRARY_PATH) -L$(CUSTOM_LIBRARY_PATH)  -lenclave
RustEnclave_Compile_Flags := $(ENCLAVE_CFLAGS) $(RustEnclave_Include_Paths)
RustEnclave_Link_Flags := -Wl,--no-undefined -nostdlib -nodefaultlibs -nostartfiles \
	-Wl,--whole-archive -Wl,--no-whole-archive \
	-Wl,--start-group $(RustEnclave_Link_Libs) -Wl,--end-group \
	-Wl,--version-script=enclave/enclave.lds \
	$(ENCLAVE_LDFLAGS)

ifeq ($(DEBUG),	1)
	RustEnclave_Out_Path := ./enclave/target/$(Rust_Build_Target)/debug
else
	RustEnclave_Out_Path := ./enclave/target/$(Rust_Build_Target)/release
endif

RustEnclave_Lib_Name := $(RustEnclave_Out_Path)/libsample.a
RustEnclave_Name := $(CUSTOM_BIN_PATH)/enclave.so
RustEnclave_Signed_Name := $(CUSTOM_BIN_PATH)/enclave.signed.so

.PHONY: all
all: $(Enclave_EDL_Files) $(RustEnclave_Signed_Name) app

######## EDL Objects ########

$(Enclave_EDL_Files): $(SGX_EDGER8R) enclave/enclave.edl
	$(SGX_EDGER8R) $(SGX_EDGER8R_MODE) --trusted enclave/enclave.edl --search-path $(CUSTOM_COMMON_PATH)/inc --search-path $(CUSTOM_EDL_PATH) --trusted-dir enclave
	$(SGX_EDGER8R) $(SGX_EDGER8R_MODE) --untrusted enclave/enclave.edl --search-path $(CUSTOM_COMMON_PATH)/inc --search-path $(CUSTOM_EDL_PATH) --untrusted-dir app
	@echo "GEN => $(Enclave_EDL_Files)"

######## App Objects ########

app/enclave_u.o: $(Enclave_EDL_Files)
	@$(CC) $(App_C_Flags) -c app/enclave_u.c -o $@

$(App_Enclave_u_Object): app/enclave_u.o
	@mkdir -p $(CUSTOM_LIBRARY_PATH)
	@$(AR) rcsD $@ $^


######## Enclave Objects ########

enclave/enclave_t.o: $(Enclave_EDL_Files)
	@$(CC) $(RustEnclave_Compile_Flags) -c enclave/enclave_t.c -o $@

$(RustEnclave_Name): enclave/enclave_t.o enclave
	@mkdir -p $(CUSTOM_LIBRARY_PATH)
	@mkdir -p $(CUSTOM_BIN_PATH)
	@cp $(RustEnclave_Lib_Name) $(CUSTOM_LIBRARY_PATH)/libenclave.a
	@$(CXX) enclave/enclave_t.o -o $@ $(RustEnclave_Link_Flags)
	@echo "LINK => $@"

$(RustEnclave_Signed_Name): $(RustEnclave_Name) enclave/config.xml
	@$(SGX_ENCLAVE_SIGNER) sign -key enclave/private.pem -enclave $(RustEnclave_Name) -out $@ -config enclave/config.xml
	@echo "SIGN => $@"
	@cp $@ $(PG_DATA)

######## Build App ########

.PHONY: app
app: $(App_Enclave_u_Object)
	@cd app && SGX_SDK=$(SGX_SDK) PATH=$(PATH):$(PG_HOME)/bin cargo pgx install $(PGX_Build_Flags)

######## Build Enclave ########

.PHONY: enclave
enclave:
ifeq ($(BUILD_STD), cargo)
	@cd $(Rust_Target_Path)/std && cargo build $(Rust_Build_Std) $(Rust_Target_Flags) $(Rust_Std_Features)

	@rm -rf $(Rust_Sysroot_Path)
	@mkdir -p $(Rust_Sysroot_Path)/lib/rustlib/$(Rust_Build_Target)/lib
	@cp -r $(Rust_Target_Path)/std/target/$(Rust_Build_Target)/release/deps/* $(Rust_Sysroot_Path)/lib/rustlib/$(Rust_Build_Target)/lib

	@cd enclave && $(Rust_Sysroot_Flags) cargo build $(Rust_Target_Flags) $(RustEnclave_Build_Flags)
else
	@cd enclave && $(Rust_Unstable_Flags) RUST_TARGET_PATH=$(Rust_Target_Path) xargo build --target $(Rust_Build_Target) $(RustEnclave_Build_Flags)
endif

######## Run Enclave ########

.PHONY: run
run: $(RustEnclave_Signed_Name) $(App_Enclave_u_Object)
	@echo -e '\n===== Run Enclave =====\n'
	@cd app && SGX_SDK=$(SGX_SDK) cargo pgx run pg$(PG_MAJOR_VERSION_NUMBER) $(PGX_Build_Flags)

.PHONY: clean
clean:
	@rm -f $(App_Name) $(RustEnclave_Name) $(RustEnclave_Signed_Name) enclave/*_t.* app/*_u.*
	@cd enclave && cargo clean
	@cd app && cargo clean
	@cd $(Rust_Target_Path)/std && cargo clean
	@rm -rf $(CUSTOM_BIN_PATH) $(CUSTOM_LIBRARY_PATH) $(CUSTOM_SYSROOT_PATH)
	@rm -rf $(PG_DATA)/enclave.signed.so
