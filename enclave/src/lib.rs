// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License..

/// C/C++ enclave functions
pub use cfunction::*;
use pgx::prelude::TableIterator;
use pgx::*;
use sgx_crypto::aes::gcm::{Aad, AesGcm, Nonce};
use sgx_types::types::{AESGCM_IV_SIZE, KEY_128BIT_SIZE, MAC_128BIT_SIZE};
use std::fs::OpenOptions;
use std::io::Write;
use std::{ptr, slice};

#[no_mangle]
pub extern "C" fn encrypt_and_decrypt(num1: i32, num2: i32) -> i32 {
    use sgx_crypto::aes::ctr::{AesCtr, Counter};
    let key = [1; 16];
    let mut aes_ctr = AesCtr::new(&key, Counter::zeroed());
    let mut enc_result = [0; 4];
    aes_ctr
        .encrypt(&(num1 + num2).to_le_bytes(), &mut enc_result)
        .unwrap();
    let mut aes_ctr = AesCtr::new(&key, Counter::zeroed());
    aes_ctr.decrypt_in_place(&mut enc_result).unwrap();
    i32::from_le_bytes(enc_result)
}

/// An AES-GCM-128 encrypt function sample.
///
/// # Parameters
///
/// **key**
///
/// Key used in AES encryption, typed as &[u8;16].
///
/// **plaintext**
///
/// Plain text to be encrypted.
///
/// **text_len**
///
/// Length of plain text, unsigned int.
///
/// **iv**
///
/// Initialization vector of AES encryption, typed as &[u8;12].
///
/// **ciphertext**
///
/// A pointer to destination ciphertext buffer.
///
/// **mac**
///
/// A pointer to destination mac buffer, typed as &mut [u8;16].
///
///
/// # Errors
///
/// **SGX_ERROR_INVALID_PARAMETER** Indicates the parameter is invalid.
///
/// **SGX_ERROR_UNEXPECTED** Indicates that encryption failed.
///
/// # Requirements
///
/// The caller should allocate the ciphertext buffer. This buffer should be
/// at least same length as plaintext buffer. The caller should allocate the
/// mac buffer, at least 16 bytes.
///
/// # Safety
#[no_mangle]
pub unsafe extern "C" fn aes_gcm_128_encrypt(
    key: &[u8; KEY_128BIT_SIZE],
    plaintext: *const u8,
    text_len: usize,
    iv: &[u8; AESGCM_IV_SIZE],
    ciphertext: *mut u8,
    mac: &mut [u8; MAC_128BIT_SIZE],
) {
    println!("aes_gcm_128_encrypt invoked!");

    // First, we need slices for input
    let plaintext_slice = slice::from_raw_parts(plaintext, text_len);

    // Here we need to initiate the ciphertext buffer, though nothing in it.
    // Thus show the length of ciphertext buffer is equal to plaintext buffer.
    // If not, the length of ciphertext_vec will be 0, which leads to argument
    // illegal.
    let mut ciphertext_vec = vec![0_u8; text_len];
    let ciphertext_slice = &mut ciphertext_vec[..];
    println!(
        "aes_gcm_128_encrypt parameter prepared! {}, {}",
        plaintext_slice.len(),
        ciphertext_slice.len()
    );

    let aad = Aad::empty();
    let iv = Nonce::from(iv);
    let mut aes_gcm = match AesGcm::new(key, iv, aad) {
        Ok(aes_gcm) => aes_gcm,
        Err(_) => {
            panic!()
        }
    };

    match aes_gcm.encrypt(plaintext_slice, ciphertext_slice) {
        Ok(mac_array) => {
            ptr::copy_nonoverlapping(ciphertext_slice.as_ptr(), ciphertext, text_len);
            *mac = mac_array;
        }
        Err(_) => {
            panic!()
        }
    };
}

/// An AES-GCM-128 decrypt function sample.
///
/// # Parameters
///
/// **key**
///
/// Key used in AES encryption, typed as &[u8;16].
///
/// **ciphertext**
///
/// Cipher text to be decrypted.
///
/// **text_len**
///
/// Length of cipher text.
///
/// **iv**
///
/// Initialization vector of AES encryption, typed as &[u8;12].
///
/// **mac**
///
/// A pointer to source mac buffer, typed as &[u8;16].
///
/// **plaintext**
///
/// A pointer to destination plaintext buffer.
///
///
/// # Errors
///
/// **SGX_ERROR_INVALID_PARAMETER** Indicates the parameter is invalid.
///
/// **SGX_ERROR_UNEXPECTED** means that decryption failed.
///
/// # Requirements
//
/// The caller should allocate the plaintext buffer. This buffer should be
/// at least same length as ciphertext buffer.
///
/// # Safety
#[no_mangle]
pub unsafe extern "C" fn aes_gcm_128_decrypt(
    key: &[u8; KEY_128BIT_SIZE],
    ciphertext: *const u8,
    text_len: usize,
    iv: &[u8; AESGCM_IV_SIZE],
    mac: &[u8; MAC_128BIT_SIZE],
    plaintext: *mut u8,
) {
    println!("aes_gcm_128_decrypt invoked!");

    // First, we need slices for input
    let ciphertext_slice = slice::from_raw_parts(ciphertext, text_len);

    // Second, for data with unknown length, we use vector as builder.
    let mut plaintext_vec: Vec<u8> = vec![0; text_len];
    let plaintext_slice = &mut plaintext_vec[..];

    println!(
        "aes_gcm_128_decrypt parameter prepared! {}, {}",
        ciphertext_slice.len(),
        plaintext_slice.len()
    );

    let aad = Aad::empty();
    let iv = Nonce::from(iv);
    let mut aes_gcm = match AesGcm::new(key, iv, aad) {
        Ok(aes_gcm) => aes_gcm,
        Err(_) => {
            panic!()
        }
    };

    match aes_gcm.decrypt(ciphertext_slice, plaintext_slice, mac) {
        Ok(_) => ptr::copy_nonoverlapping(plaintext_slice.as_ptr(), plaintext, text_len),
        Err(_) => {
            panic!()
        }
    };
}

/// # Safety
/// filename and text must not be null
#[no_mangle]
pub unsafe extern "C" fn write_file(
    filename: *const u8,
    filename_len: usize,
    text: *const u8,
    text_len: usize,
) {
    let filename = slice::from_raw_parts(filename, filename_len);
    let filename = String::from_utf8(filename.to_vec()).unwrap();
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(filename)
        .unwrap();
    let text = slice::from_raw_parts(text, text_len);
    file.write_all(text).unwrap();
}

#[no_mangle]
pub extern "C" fn test_return_set_of_data(
    insert_text1: *const u8,
    insert_text1_len: usize,
    insert_text2: *const u8,
    insert_text2_len: usize,
) {
    // let mut results = vec![];
    // let insert_text1 = unsafe {
    //     let slice = slice::from_raw_parts(insert_text1, insert_text1_len);
    //     core::str::from_utf8(slice).unwrap()
    // };
    // let insert_text2 = unsafe {
    //     let slice = slice::from_raw_parts(insert_text2, insert_text2_len);
    //     core::str::from_utf8(slice).unwrap()
    // };
    // Spi::connect(|client| {
    //     //使用update函数更改表内的数据
    //     client.update(
    //         //先删除表中所有数据，再插入两条数据
    //         r#"
    //         DELETE FROM spi_example;
    //         INSERT INTO spi_example (title) VALUES ($1);
    //         INSERT INTO spi_example (title) VALUES ($2);
    //         "#,
    //         None,
    //         Some(vec![
    //             (PgBuiltInOids::TEXTOID.oid(), insert_text1.into_datum()),
    //             (PgBuiltInOids::TEXTOID.oid(), insert_text2.into_datum()),
    //         ]),
    //     );
    //     Ok(Some(()))
    // });
    // info!("end modifying table spi_example");
    // let result = Spi::connect(|client| {
    //     let results: Vec<(Option<i64>, Option<String>)> = client
    //         .select("SELECT * FROM spi_example;", None, None)
    //         .map(|row| (row["id"].value(), row["title"].value()))
    //         .collect();
    //     Ok(Some(results))
    // })
    // .unwrap();
    // let _ = TableIterator::new(result.into_iter());
    // info!("[test return set of data]:function ended successfully!")
}
