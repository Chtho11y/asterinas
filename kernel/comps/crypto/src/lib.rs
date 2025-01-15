// SPDX-License-Identifier: MPL-2.0

//! The console device of Asterinas.
#![no_std]
#![deny(unsafe_code)]
#![feature(fn_traits)]

extern crate alloc;

use alloc::{collections::BTreeMap, fmt::Debug, string::String, sync::Arc, vec::Vec};
use core::{any::Any, error::Error, fmt::Display};

use component::{init_component, ComponentInitError};
use ostd::sync::SpinLock;
use spin::Once;

// pub type CryptoCallback = dyn Fn(VmReader<Infallible>) + Send + Sync;

#[derive(Debug)]
pub enum CryptoError{
    UnknownError,
    BadMessage,
    NotSupport,
    InvalidSession,
    NoFreeSession,
    KeyReject,
}

impl Display for CryptoError{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            CryptoError::UnknownError => write!(f, "Unknown error occurred"),
            CryptoError::BadMessage => write!(f, "Authentication failed for AEAD"),
            CryptoError::NotSupport => write!(f, "Operation not supported"),
            CryptoError::InvalidSession => write!(f, "Invalid session ID"),
            CryptoError::NoFreeSession => write!(f, "No free session available"),
            CryptoError::KeyReject => write!(f, "Signature verification failed"),
        }
    }
}

impl Error for CryptoError {}

#[derive(Debug, Clone, Copy)]
#[repr(i32)]
pub enum CryptoHashAlgorithm {
    NoHash = 0,
    Md5 = 1,
    Sha1 = 2,
    Sha224 = 3,
    Sha256 = 4,
    Sha384 = 5,
    Sha512 = 6,
    Sha3_224 = 7,
    Sha3_256 = 8,
    Sha3_384 = 9,
    Sha3_512 = 10,
    Sha3Shake128 = 11,
    Sha3Shake256 = 12,
}

#[derive(Debug, Clone, Copy)]
#[repr(i32)]
pub enum CryptoMacAlgorithm {
    NoMac = 0,
    HmacMd5 = 1,
    HmacSha1 = 2,
    HmacSha224 = 3,
    HmacSha256 = 4,
    HmacSha384 = 5,
    HmacSha512 = 6,
    Cmac3Des = 25,
    CmacAes = 26,
    KasumiF9 = 27,
    Snow3gUia2 = 28,
    GmacAes = 41,
    GmacTwofish = 42,
    CbcMacAes = 49,
    CbcMacKasumiF9 = 50,
    XcbcAes = 53,
    ZucEia3 = 54,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum CryptoCipherAlgorithm {
    NoCipher = 0,
    Arc4 = 1,
    AesEcb = 2,
    AesCbc = 3,
    AesCtr = 4,
    DesEcb = 5,
    DesCbc = 6,
    ThreeDesEcb = 7,
    ThreeDesCbc = 8,
    ThreeDesCtr = 9,
    KasumiF8 = 10,
    Snow3gUea2 = 11,
    AesF8 = 12,
    AesXts = 13,
    ZucEea3 = 14,
}


#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum CryptoHashAlgo {
    NoHash = 0,
    MD2 = 1,
    MD3 = 2,
    MD4 = 3,
    MD5 = 4,
    SHA1 = 5,
    SHA256 = 6,
    SHA384 = 7,
    SHA512 = 8,
    SHA224 = 9,
}


#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum CryptoAkCipherAlgorithm {
    NoAkCipher = 0,
    AkCipherRSA = 1,
    AkCipherECDSA = 2,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum CryptoPaddingAlgo {
    RAW = 0,
    PKCS1 = 1,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum CryptoCurve {
    Unknown = 0,
    NistP192 = 1,
    NistP224 = 2,
    NistP256 = 3,
    NistP384 = 4,
    NistP521 = 5,
}


#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum CryptoAkCipherKeyType {
    Public = 1,
    Private = 2,
}

#[repr(u32)]
#[derive(Debug)]
pub enum CryptoOperation {
    Encrypt = 1,
    Decrypt = 2,
}

#[repr(u32)]
#[derive(Debug)]
pub enum CryptoSymOp{
    None = 0,
    Cipher = 1,
    AlgorithmChaining = 2,
}

#[repr(u32)]
#[derive(Debug)]
pub enum CryptoSymAlgChainOrder {
    HashThenCipher = 1,
    CipherThenHash = 2
}

#[repr(u32)]
#[derive(Debug)]
pub enum CryptoSymHashMode {
    Plain = 1, 
    Auth = 2,
    Nested = 3
}


pub enum CryptoService{
    Cipher = 0,
    Hash = 1,
    Mac = 2,
    Aead = 3,
    AkCipher = 4,
}

pub const fn crypto_services_opcode(service: CryptoService, op: i32)-> i32{
    ((service as i32) << 8) | op
}

#[derive(Debug, Clone, Copy)]
#[repr(i32)]
pub enum CryptoSessionOperation{
    CipherCreate = crypto_services_opcode(CryptoService::Cipher, 0x02),
    CipherDestroy = crypto_services_opcode(CryptoService::Cipher, 0x03),
    HashCreate = crypto_services_opcode(CryptoService::Hash, 0x02),
    HashDestroy = crypto_services_opcode(CryptoService::Hash, 0x03),
    MacCreate = crypto_services_opcode(CryptoService::Mac, 0x02),
    MacDestroy = crypto_services_opcode(CryptoService::Mac, 0x03),
    AeadCreate = crypto_services_opcode(CryptoService::Aead, 0x02),
    AeadDestroy = crypto_services_opcode(CryptoService::Aead, 0x03),
    AkCipherCreate = crypto_services_opcode(CryptoService::AkCipher, 0x04),
    AkCipherDestroy = crypto_services_opcode(CryptoService::AkCipher, 0x05),
}

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

pub trait AnyCryptoDevice: Send + Sync + Any + Debug {
    //Test device function 
    fn test_device(&self);

    //Create Hash session, return session id.
    fn create_hash_session(&self, algo: CryptoHashAlgorithm, result_len: u32)->Result<i64, CryptoError>;
    fn handle_hash_service_req(&self, op : CryptoServiceOperation, algo: CryptoHashAlgorithm, session_id : i64, src_data: &[u8], hash_result_len: i32) -> Result<Vec<u8>, CryptoError>;
    fn destroy_hash_session(&self, session_id : i64) -> Result<u8, CryptoError>;

    fn create_mac_session(&self, algo: CryptoMacAlgorithm, result_len: u32, auth_key: &[u8])->Result<i64, CryptoError>;
    fn handle_mac_service_req(&self, op : CryptoServiceOperation, algo: CryptoMacAlgorithm, session_id : i64, src_data: &[u8], hash_result_len: i32) -> Result<Vec<u8>, CryptoError>;
    fn destroy_mac_session(&self, session_id : i64) -> Result<u8, CryptoError>;
    
    fn create_cipher_session(&self, algo: CryptoCipherAlgorithm, op: CryptoOperation, key: &[u8])->Result<i64, CryptoError>;
    fn create_alg_chain_auth_session(&self, algo: CryptoCipherAlgorithm, op: CryptoOperation, alg_chain_order: CryptoSymAlgChainOrder, mac_algo: CryptoMacAlgorithm, result_len: u32, aad_len: i32, cipher_key: &[u8], auth_key: &[u8])->Result<i64, CryptoError>;
    fn create_alg_chain_plain_session(&self, algo: CryptoCipherAlgorithm, op: CryptoOperation, alg_chain_order: CryptoSymAlgChainOrder, hash_algo: CryptoHashAlgorithm, result_len: u32, aad_len: i32, cipher_key: &[u8])->Result<i64, CryptoError>;
    fn handle_cipher_service_req(&self, op : CryptoServiceOperation, algo: CryptoCipherAlgorithm, session_id : i64, iv : &[u8], src_data : &[u8], dst_data_len : i32) -> Result<Vec<u8>, CryptoError>;
    fn handle_alg_chain_service_req(&self, op : CryptoServiceOperation, algo: CryptoCipherAlgorithm, session_id: i64, iv : &[u8], src_data : &[u8], dst_data_len: i32, cipher_start_src_offset: i32, len_to_cipher: i32, hash_start_src_offset: i32, len_to_hash: i32, aad_len: i32, hash_result_len: i32) -> Result<(Vec<u8>, Vec<u8>), CryptoError>;
    fn destroy_cipher_session(&self, session_id: i64) -> Result<u8, CryptoError>;
    

    fn create_akcipher_ecdsa_session(&self, algo: CryptoAkCipherAlgorithm,
        op: CryptoOperation,
        curve_id: CryptoCurve,
        key_type: CryptoAkCipherKeyType,
        key: &[u8],
    ) -> Result<i64, CryptoError>;
    fn create_akcipher_rsa_session(&self, algo: CryptoAkCipherAlgorithm,
        op: CryptoOperation,
        padding_algo: CryptoPaddingAlgo,
        hash_algo: CryptoHashAlgo,
        key_type: CryptoAkCipherKeyType,
        key: &[u8],
    ) -> Result<i64, CryptoError>;
    fn handle_akcipher_serivce_req(&self, op : CryptoServiceOperation, algo: CryptoAkCipherAlgorithm, session_id: i64, src_data : &[u8], dst_data_len : i32) -> Result<Vec<u8>, CryptoError>;
    fn destroy_akcipher_session(&self, session_id: i64) -> Result<u8, CryptoError>;
}

pub fn register_device(name: String, device: Arc<dyn AnyCryptoDevice>) {
    COMPONENT
        .get()
        .unwrap()
        .crypto_device_table
        .disable_irq()
        .lock()
        .insert(name, device);
}

pub fn all_devices() -> Vec<(String, Arc<dyn AnyCryptoDevice>)> {
    let crypto_devs = COMPONENT
        .get()
        .unwrap()
        .crypto_device_table
        .disable_irq()
        .lock();
    crypto_devs
        .iter()
        .map(|(name, device)| (name.clone(), device.clone()))
        .collect()
}

static COMPONENT: Once<Component> = Once::new();

#[init_component]
fn component_init() -> Result<(), ComponentInitError> {
    let a = Component::init()?;
    COMPONENT.call_once(|| a);
    Ok(())
}

#[derive(Debug)]
struct Component {
    crypto_device_table: SpinLock<BTreeMap<String, Arc<dyn AnyCryptoDevice>>>,
}

impl Component {
    pub fn init() -> Result<Self, ComponentInitError> {
        Ok(Self {
            crypto_device_table: SpinLock::new(BTreeMap::new()),
        })
    }
}
