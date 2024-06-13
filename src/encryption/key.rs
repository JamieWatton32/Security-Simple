use std::{
    ffi::CString,
    ptr::{null, null_mut},
    slice,
};

use crate::encryption::encryption::generate_key;
use winapi::{
    ctypes::c_void,
    shared::{
        minwindef::{DWORD, HKEY},
        winerror::ERROR_SUCCESS,
    },
    um::{
        dpapi::{CryptProtectData, CryptUnprotectData},
        winbase::LocalFree,
        wincrypt::CRYPTOAPI_BLOB,
        winnt::{KEY_READ, KEY_WRITE},
        winreg::{
            RegCloseKey, RegCreateKeyExA, RegGetValueA, RegOpenKeyExA, RegSetValueExA,
            HKEY_CURRENT_USER, RRF_RT_REG_BINARY,
        },
    },
};

const REG_PATH: &str = "Software\\security_simple";
const REG_VALUE: &str = "EncryptionKey";
pub struct StoreKey;

impl StoreKey {
    pub fn make_key() -> Vec<u8> {
        let encryption_key = generate_key();
        encryption_key
    }
    pub fn store_key(key: &[u8]) {
        unsafe {
            let mut hkey: HKEY = null_mut();
            let sub_key = CString::new(REG_PATH).expect("CString::new failed");
            if RegCreateKeyExA(
                HKEY_CURRENT_USER,
                sub_key.as_ptr(),
                0,
                null_mut(),
                0,
                KEY_WRITE,
                null_mut(),
                &mut hkey,
                null_mut(),
            ) != ERROR_SUCCESS.try_into().unwrap()
            {
                panic!("Failed to create registry key");
            }

            let value_name = CString::new(REG_VALUE).expect("CString::new failed");
            if RegSetValueExA(
                hkey,
                value_name.as_ptr(),
                0,
                winapi::um::winnt::REG_BINARY,
                key.as_ptr(),
                key.len() as DWORD,
            ) != ERROR_SUCCESS.try_into().unwrap()
            {
                panic!("Failed to set registry value");
            }

            RegCloseKey(hkey);
        }
    }

    pub fn retrieve_key() -> Option<Vec<u8>> {
        unsafe {
            let mut hkey: HKEY = null_mut();
            let sub_key = CString::new(REG_PATH).expect("CString::new failed");
            if RegOpenKeyExA(HKEY_CURRENT_USER, sub_key.as_ptr(), 0, KEY_READ, &mut hkey)
                != ERROR_SUCCESS.try_into().unwrap()
            {
                return None;
            }

            let value_name = CString::new(REG_VALUE).expect("CString::new failed");
            let mut data_type: DWORD = 0;
            let mut data_size: DWORD = 0;
            if RegGetValueA(
                hkey,
                null(),
                value_name.as_ptr(),
                RRF_RT_REG_BINARY,
                &mut data_type,
                null_mut(),
                &mut data_size,
            ) != ERROR_SUCCESS.try_into().unwrap()
            {
                RegCloseKey(hkey);
                return None;
            }

            let mut data = vec![0u8; data_size as usize];
            if RegGetValueA(
                hkey,
                null(),
                value_name.as_ptr(),
                RRF_RT_REG_BINARY,
                &mut data_type,
                data.as_mut_ptr() as *mut c_void,
                &mut data_size,
            ) != ERROR_SUCCESS.try_into().unwrap()
            {
                RegCloseKey(hkey);
                return None;
            }

            RegCloseKey(hkey);
            Some(data)
        }
    }
    pub fn decrypt_data(data: &[u8]) -> Vec<u8> {
        let mut data_in = CRYPTOAPI_BLOB {
            cbData: data.len() as u32,
            pbData: data.as_ptr() as *mut u8,
        };
        let mut data_out = CRYPTOAPI_BLOB {
            cbData: 0,
            pbData: null_mut(),
        };

        unsafe {
            if CryptUnprotectData(
                &mut data_in,
                null_mut(),
                null_mut(),
                null_mut(),
                null_mut(),
                0,
                &mut data_out,
            ) == 0
            {
                panic!("Failed to decrypt data");
            }

            let decrypted_data =
                slice::from_raw_parts(data_out.pbData, data_out.cbData as usize).to_vec();
            LocalFree(data_out.pbData as *mut c_void);

            decrypted_data
        }
    }
    pub fn encrypt_data(data: &[u8]) -> Vec<u8> {
        let mut data_in = CRYPTOAPI_BLOB {
            cbData: data.len() as u32,
            pbData: data.as_ptr() as *mut u8,
        };
        let mut data_out = CRYPTOAPI_BLOB {
            cbData: 0,
            pbData: null_mut(),
        };

        unsafe {
            if CryptProtectData(
                &mut data_in,
                null_mut(),
                null_mut(),
                null_mut(),
                null_mut(),
                0,
                &mut data_out,
            ) == 0
            {
                panic!("Failed to encrypt data");
            }

            let encrypted_data =
                slice::from_raw_parts(data_out.pbData, data_out.cbData as usize).to_vec();
            LocalFree(data_out.pbData as *mut c_void);

            encrypted_data
        }
    }
}
