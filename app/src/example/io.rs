use pgx::*;
use sgx_types::{error::SgxStatus, types::EnclaveId};

const ENCLAVE_FILE: &str = "enclave.signed.so";

extern "C" {
    fn write_file(
        eid: EnclaveId,
        filename: *const u8,
        filename_len: usize,
        text: *const u8,
        text_len: usize,
    ) -> SgxStatus;
}
#[pg_extern]
fn sgx_write_file(filename: &str, text: &str) {
    use sgx_urts::enclave::SgxEnclave;
    let enclave = match SgxEnclave::create(ENCLAVE_FILE, true) {
        Ok(enclave) => {
            info!("[+] Init Enclave Successful {}!", enclave.eid());
            enclave
        }
        Err(err) => {
            error!("[-] Init Enclave Failed {}!", err.as_str());
        }
    };
    unsafe {
        match write_file(
            enclave.eid(),
            filename.as_ptr(),
            filename.len(),
            text.as_ptr(),
            text.len(),
        ) {
            SgxStatus::Success => {
                info!("[+] Successfully write {text} to {filename}!")
            }
            e => {
                error!("[-] Failed to write {text} to {filename} with error:\n{e:?}")
            }
        }
    }
}
