use serde::{ser, Deserialize, Deserializer, Serialize, Serializer};
use soteria_rs::Protected;
use std::cell::RefCell;
use std::rc::Rc;

pub fn serialize<S: Serializer>(input: &Rc<RefCell<Protected>>, s: S) -> Result<S::Ok, S::Error> {
    let mut p = input.borrow_mut();
    let u = p
        .unprotect()
        .ok_or_else(|| ser::Error::custom("invalid secret"))?;
    u.as_ref().serialize(s)
}

pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Rc<RefCell<Protected>>, D::Error> {
    let input = Vec::<u8>::deserialize(d)?;
    Ok(Rc::new(RefCell::new(Protected::new(input.as_slice()))))
}