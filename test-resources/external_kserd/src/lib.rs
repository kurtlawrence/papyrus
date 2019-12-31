pub use kserd;

use kserd::*;
use rand::random;

pub struct MyStruct {
    pub a: i32,
    pub b: String,
}

impl MyStruct {
    pub fn rand_new() -> Self {
        Self {
            a: random(),
            b: String::new()
        }
    }
}

impl ToKserd<'static> for MyStruct {
    fn into_kserd(self) -> Result<Kserd<'static>, ToKserdErr> {
        (self.a, self.b).into_kserd()
    }
}
