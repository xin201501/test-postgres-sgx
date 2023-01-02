use pgx::*;
use sgx_types::{error::SgxStatus, types::EnclaveId};


const ENCLAVE_FILE: &str = "enclave.signed.so";

// C enclave functions
extern "C" {
    fn test_c_enclave_function(id: EnclaveId, ret: *mut i32, a: i32, b: i32) -> SgxStatus;
}

#[pg_extern]
pub fn test_cfunction_melborne() {
    use cfunction::run;
    unsafe { run() };
}

#[pg_extern]
pub fn test_c_enclave(a: i32, b: i32) -> i32 {
    use sgx_urts::enclave::SgxEnclave;
    use SgxStatus::Success;
    let enclave = match SgxEnclave::create(ENCLAVE_FILE, true) {
        Ok(enclave) => {
            info!("[+] Init Enclave Successful {}!", enclave.eid());
            enclave
        }
        Err(err) => {
            error!("[-] Init Enclave Failed {}!", err.as_str());
        }
    };
    let mut result = 0;
    if Success != unsafe { test_c_enclave_function(enclave.eid(), &mut result as *mut i32, a, b) } {
        error!("enclave run failed!");
    }
    info!("C enclave returned result:{result}");
    result
}
