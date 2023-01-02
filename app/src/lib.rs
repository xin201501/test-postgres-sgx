#![feature(slice_as_chunks)]
#![feature(const_fmt_arguments_new)]

use pgx::iter::TableIterator;
use pgx::*;

use itertools::Itertools;
use sgx_types::error::SgxStatus;
use sgx_types::types::*;
use sgx_urts::enclave::SgxEnclave;
pg_module_magic!();

const ENCLAVE_FILE: &str = "/home/xin201501/.pgx/data-15/enclave.signed.so";
mod example;

//# 可在entension_sql中执行sql语句
//## 创建一张表
extension_sql!(
    r#"   
    CREATE TABLE spi_example (
        id serial PRIMARY KEY,
        title text
    );
    INSERT INTO spi_example (title) VALUES ('A');
    INSERT INTO spi_example (title) VALUES ('B');
    INSERT INTO spi_example (title) VALUES ('C');
"#,
    name = "spi_example_table",
);
//声明需要使用的enclave函数
extern "C" {
    fn encrypt_and_decrypt(eid: EnclaveId, retval: *mut i32, num1: i32, num2: i32) -> SgxStatus;
    fn aes_gcm_128_encrypt(
        eid: EnclaveId,
        // retval: *mut SgxStatus,
        key: &[u8; 16],
        plaintext: *const u8,
        text_len: usize,
        iv: &[u8; 12],
        ciphertext: *mut u8,
        mac: &mut [u8; 16],
    ) -> SgxStatus;
    fn aes_gcm_128_decrypt(
        eid: EnclaveId,
        // retval: *mut SgxStatus,
        key: &[u8; 16],
        ciphertext: *const u8,
        text_len: usize,
        iv: &[u8; 12],
        mac: &[u8; 16],
        plaintext: *mut u8,
    ) -> SgxStatus;
}

#[pg_extern]
fn hello_test_pgx() -> &'static str {
    "Hello, test_pgx"
}

#[pg_extern]
fn hello_sgx_world(a: i32, b: i32) {
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

    let mut retval = -1;
    let result = unsafe { encrypt_and_decrypt(enclave.eid(), &mut retval, a, b) };
    match result {
        SgxStatus::Success => info!("[+] rust enclave returned result:{retval}"),
        _ => error!("[-] ECALL Enclave Failed {}!", result.as_str()),
    }
}

#[pg_extern]
fn test_sgx_aes_gcm_128_encrypt() -> TableIterator<
    'static,
    (
        name!(key, String),
        name!(iv, String),
        name!(ciphertext, String),
        name!(mac, String),
    ),
> {
    let key = [0; 16];
    let iv = [0; 12];
    let plaintext = [0; 16];
    let enclave = match SgxEnclave::create(ENCLAVE_FILE, true) {
        Ok(enclave) => {
            info!("[+] Init Enclave Successful {}!", enclave.eid());
            enclave
        }
        Err(err) => {
            error!("[-] Init Enclave Failed {}!", err.as_str());
        }
    };

    let mut mac = [0u8; 16];
    let mut ciphertext = [0; 16];
    let status = unsafe {
        aes_gcm_128_encrypt(
            enclave.eid(),
            &key,
            plaintext.as_ptr(),
            plaintext.len(),
            &iv,
            ciphertext.as_mut_ptr(),
            &mut mac,
        )
    };
    if status != SgxStatus::Success {
        error!("[+] Enclave Failed {}!", status);
    }
    info!(
        "[+] aes-gcm-128 expected ciphertext: {}",
        "0388dace60b6a392f328c2b971b2fe78"
    );
    let ciphertext = format!("{:02x}", ciphertext.iter().format(""));
    info!("[+] aes-gcm-128 ciphertext is: {ciphertext}");
    let mac = format!("{:02x}", mac.iter().format(""));
    info!("[+] aes-gcm-128 mac is: {mac}");
    let key = format!("{:02x}", key.iter().format(""));
    let iv = format!("{:02x}", iv.iter().format(""));
    TableIterator::once((key, iv, ciphertext, mac))
}

#[pg_extern]
fn test_sgx_aes_gcm_128_decrypt() -> Vec<u8> {
    let key = [0; 16];
    let iv = [0; 12];
    let ciphertext = [
        3, 136, 218, 206, 96, 182, 163, 146, 243, 40, 194, 185, 113, 178, 254, 120,
    ];
    let mac = [
        171, 110, 71, 212, 44, 236, 19, 189, 245, 58, 103, 178, 18, 87, 189, 223,
    ];
    let enclave = match SgxEnclave::create(ENCLAVE_FILE, true) {
        Ok(enclave) => {
            info!("[+] Init Enclave Successful {}!", enclave.eid());
            enclave
        }
        Err(err) => {
            error!("[-] Init Enclave Failed {}!", err.as_str());
        }
    };
    let mut plaintext = [0u8; 16];
    let status = unsafe {
        aes_gcm_128_decrypt(
            enclave.eid(),
            &key,
            ciphertext.as_ptr(),
            ciphertext.len(),
            &iv,
            &mac,
            plaintext.as_mut_ptr(),
        )
    };
    if status != SgxStatus::Success {
        error!("[-] decryption failed {status}!");
    }
    plaintext.to_vec()
}

#[pg_extern]
fn test_spi(title: &str) -> i64 {
    Spi::get_one_with_args(
        r#"SELECT id FROM spi_example where title=$1"#,
        vec![(PgBuiltInOids::TEXTOID.oid(), title.into_datum())],
    )
    .unwrap_or_else(|| error!("{title} not found!"))
}

#[pg_extern]
fn test_return_set_of_data(
    insert_text1: &str,
    insert_text2: &str,
) -> TableIterator<'static, (name!(id, Option<i64>), name!(title, Option<String>))> {
    // let mut results = vec![];
    Spi::connect(|client| {
        //使用update函数更改表内的数据
        client.update(
            //先删除表中所有数据，再插入两条数据
            r#"
            DELETE FROM spi_example;
            INSERT INTO spi_example (title) VALUES ($1);
            INSERT INTO spi_example (title) VALUES ($2);
            "#,
            None,
            Some(vec![
                (PgBuiltInOids::TEXTOID.oid(), insert_text1.into_datum()),
                (PgBuiltInOids::TEXTOID.oid(), insert_text2.into_datum()),
            ]),
        );
        Ok(Some(()))
    });
    info!("end modifying table spi_example");
    let result = Spi::connect(|client| {
        let results: Vec<_> = client
            .select("SELECT * FROM spi_example;", None, None)
            .map(|row| (row["id"].value(), row["title"].value()))
            .collect();
        Ok(Some(results))
    })
    .unwrap();
    TableIterator::new(result.into_iter())
}
//使用select函数只读取但不更改表内的数据

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use super::*;

    #[pg_test]
    fn test_hello_test_pgx() {
        assert_eq!("Hello, test_pgx", crate::hello_test_pgx());
    }

    #[pg_test]
    fn test_hello_sgx_world() {
        use sgx_urts::enclave::SgxEnclave;
        let enclave = SgxEnclave::create(ENCLAVE_FILE, true).unwrap();

        let mut retval = -1;
        let result = unsafe { encrypt_and_decrypt(enclave.eid(), &mut retval, 1, 2) };
        assert_eq!(result, SgxStatus::Success);
        assert_eq!(retval, 3)
    }
}

#[cfg(test)]
pub mod pg_test {
    pub fn setup(_options: Vec<&str>) {
        // perform one-off initialization when the pg_test framework starts
    }

    pub fn postgresql_conf_options() -> Vec<&'static str> {
        // return any postgresql.conf settings that are required for your tests
        vec![]
    }
}
