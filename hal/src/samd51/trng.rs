use crate::target_device::{TRNG, MCLK};

use rand_core::{CryptoRng, RngCore};

#[cfg(feature="unproven")]
use embedded_hal::blocking::rng::Read;

pub struct Trng(TRNG);

impl Trng {
    pub fn new(mclk: &mut MCLK, trng: TRNG) -> Trng { 
        mclk.apbcmask.modify(|_, w| w.trng_().set_bit()); 
        trng.ctrla.modify(|_, w| w.enable().set_bit());
        Self(trng)
    }

    pub fn random(&self, buf: &mut [u8]) {
        for chunk in buf.chunks_exact_mut(4) { 
            chunk.copy_from_slice(&self.random_u32().to_le_bytes());
        }
        // copy_from_slice doesn't work if the slices are of different lengths
        let remainder = buf.len() % 4;
        let final_word = self.random_u32().to_le_bytes();
        for i in 0..remainder {
            buf[buf.len()-i] = final_word[i];
        }
    }


    pub fn random_u8(&self) -> u8 {
        self.random_u32() as u8
    }

    pub fn random_u16(&self) -> u16 {
        self.random_u32() as u16
    }

    pub fn random_u32(&self) -> u32 {
        while self.0.intflag.read().datardy().bit_is_clear() {}
        self.0.data.read().bits()
    }

    pub fn random_u64(&self) -> u64 {
        while self.0.intflag.read().datardy().bit_is_clear() {}
        let lower_half = self.0.data.read().bits() as u64;
        while self.0.intflag.read().datardy().bit_is_clear() {}
        let upper_half = self.0.data.read().bits() as u64;
        (upper_half << 32) | lower_half
    }
}

impl RngCore for Trng {
    fn next_u32(&mut self) -> u32 {
        self.random_u32()
    }

    fn next_u64(&mut self) -> u64 {
        self.random_u64()
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.random(dest)
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core::Error> {
        Ok(self.fill_bytes(dest))
    }
}

impl CryptoRng for Trng {}


#[cfg(feature="unproven")]
impl Read for Trng {
    type Error = ();
    fn read(&mut self, buffer: &mut [u8]) -> Result<(), Self::Error> {
        Ok(self.random(buffer))
    }
}
