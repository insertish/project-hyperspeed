use rand::distributions::Uniform;
use rand::{thread_rng, Rng};
use hex::encode;

pub fn generate_hmac() -> String {
    let dist = Uniform::new(0x00, 0xFF);
    let mut hmac_payload: Vec<u8> = Vec::new();
    let mut rng = thread_rng();
    
    for _ in 0..128 {
        hmac_payload.push(rng.sample(dist));
    }

    encode(hmac_payload.as_slice())
}
