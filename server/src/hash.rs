use std::ptr::hash;

pub struct Hasher {}
impl Hasher {
    pub fn hash(n: u16) -> String {
        let digest = md5::compute(format!("{}", n));
        format!("{:x}", digest)
    }
}
