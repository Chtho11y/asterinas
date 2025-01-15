use core::hash;

use alloc::vec::Vec;
use aster_crypto::{CryptoCipherAlgorithm, CryptoError, CryptoHashAlgorithm, CryptoOperation};
use ostd::Pod;
use crate::device::crypto::session::*;

#[derive(Debug, Clone, Copy)]
#[repr(i32)]
pub enum CryptoServiceOperation{
    CipherEncrypt = crypto_services_opcode(CryptoService::Cipher, 0x00),
    CipherDecrypt = crypto_services_opcode(CryptoService::Cipher, 0x01),
    Hash = crypto_services_opcode(CryptoService::Hash, 0x00),
    Mac = crypto_services_opcode(CryptoService::Mac, 0x00),
    AeadEncrypt = crypto_services_opcode(CryptoService::Aead, 0x00),
    AeadDecrypt = crypto_services_opcode(CryptoService::Aead, 0x01),
    AkCipherEncrypt = crypto_services_opcode(CryptoService::AkCipher, 0x00),
    AkCipherDecrypt = crypto_services_opcode(CryptoService::AkCipher, 0x01),
    AkCipherSign = crypto_services_opcode(CryptoService::AkCipher, 0x02),
    AkCipherVerify = crypto_services_opcode(CryptoService::AkCipher, 0x03),
}
#[derive(Debug, Pod, Clone, Copy)]
#[repr(C)]
pub struct CryptoServiceHeader {
    pub opcode : i32,
    pub algo : i32,
    pub session_id : i64,
    pub flag : i32,
    pub padding : i32
}

#[derive(Debug, Pod, Clone, Copy)]
#[repr(C)]
pub struct VirtioCryptoInhdr {
    pub status : u8
}

impl VirtioCryptoInhdr {
    pub fn get_result(&self) -> Result<u8, CryptoError> {
        match VirtioCryptoStatus::try_from(self.status as i32){
            Ok(code) => code.get_or_error(self.status),
            Err(err) => Err(err)
        }
    }
}

pub trait CryptoServiceRequest: Pod {
    fn to_bytes(&self, padding: bool)->Vec<u8>;
}


#[derive(Pod, Clone, Copy)]
#[repr(C)]
pub struct CryptoCipherServiceReq {
    pub header : CryptoServiceHeader,
    pub op_flf : VirtioCryptoSymDataFlf,
}

impl CryptoServiceRequest for CryptoCipherServiceReq {
    fn to_bytes(&self, padding: bool)->Vec<u8> {
        Vec::from(<Self as Pod>::as_bytes(&self))
    }
}

#[derive(Pod, Clone, Copy)]
#[repr(C)]
pub struct VirtioCryptoSymDataFlf {
    pub op_type_flf : VirtioCryptoSymDataFlfWrapper,
    pub op_type : i32,
    pub padding : i32
}

#[derive(Pod, Clone, Copy)]
#[repr(C)]
pub union VirtioCryptoSymDataFlfWrapper {
    pub CipherFlf : VirtioCryptoCipherDataFlf,
    pub AlgChainFlf : VirtioCryptoAlgChainDataFlf
} 

#[derive(Debug, Pod, Clone, Copy)]
#[repr(C)]
pub struct VirtioCryptoCipherDataFlf {
    pub iv_len : i32,
    pub src_data_len : i32,
    pub dst_data_len : i32,
    pub padding : i32
}

impl VirtioCryptoCipherDataFlf {
    pub fn new(iv_len : i32, src_data_len : i32, dst_data_len : i32) -> Self {
        Self {
            iv_len,
            src_data_len,
            dst_data_len,
            padding : 0
        }
    }
}

#[derive(Debug, Pod, Clone, Copy)]
#[repr(C)]
pub struct VirtioCryptoAlgChainDataFlf {
    pub iv_len : i32,
    pub src_data_len : i32,
    pub dst_data_len : i32,
    pub cipher_start_src_offset : i32,
    pub len_to_cipher : i32,
    pub hash_start_src_offset : i32,
    pub len_to_hash : i32,
    pub aad_len : i32,
    pub hash_result_len : i32,
    pub reserved : i32
}

impl VirtioCryptoAlgChainDataFlf {
    pub fn new(iv_len : i32, src_data_len : i32, dst_data_len : i32, cipher_start_src_offset : i32, len_to_cipher : i32, hash_start_src_offset : i32, len_to_hash : i32, aad_len : i32, hash_result_len : i32) -> Self {
        Self {
            iv_len,
            src_data_len,
            dst_data_len,
            cipher_start_src_offset,
            len_to_cipher,
            hash_start_src_offset,
            len_to_hash,
            aad_len,
            hash_result_len,
            reserved: 0
        }
    }
}




