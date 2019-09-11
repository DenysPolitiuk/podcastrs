use fxhash;

pub fn compute_hash(target: &str) -> u32 {
    fxhash::hash32(target)
}
