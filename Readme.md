# Test-Postgres-SGX

In this demo I write UDFs for postgres using
[cargo-pgx](https://github.com/tcdi/pgx),some of them call SGX enclave to finish their jobs.

This demo uses [cargo-pgx](https://github.com/tcdi/pgx) and [teaclave sgx sdk](https://github.com/apache/incubator-teaclave-sgx-sdk).

## Project Template

**The template is from samplecode in teaclave sgx sdk v2.0.0 branch.
See <https://github.com/apache/incubator-teaclave-sgx-sdk/tree/v2.0.0-preview/samplecode/template> for the latest version.**

There are some settings for you to tweak:

>
>Default settings:
>
>- `std-aware-cargo` used by default. Tweak `BUILD_STD ?= cargo` in Makefile to switch between `no_std`, `xargo`, and >`std-aware-cargo` mode.
>- `sgx_tstd` enables its default feature gate which only contains `stdio`. Tweak >`Rust_Std_Features` in Makefile to enable more features. Options are `backtrace`, >`stdio`, `env`, `net`, `pipe`, `thread`, >`untrusted_fs`, `untrusted_time` >`unsupported_process`.
>- StackMaxSize: 0x40000, HeapMaxSize: 0x100000, TCSNum: 1