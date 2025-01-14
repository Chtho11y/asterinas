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
#[derive(Debug)]
pub enum CryptoOperation {
    Encrypt = 1,
    Decrypt = 2,
}

pub trait AnyCryptoDevice: Send + Sync + Any + Debug {
    //Test device function 
    fn test_device(&self);

    //Create Hash session, return session id.
    fn create_hash_session(&self, algo: CryptoHashAlgorithm, result_len: u32)->Result<i64, CryptoError>;

    fn create_cipher_session(&self, algo: CryptoCipherAlgorithm, op: CryptoOperation, key: &[u8])->Result<i64, CryptoError>;
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
