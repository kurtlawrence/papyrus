use kserd::*;

pub struct MyStruct {
    pub a: i32,
    pub b: String,
}

impl ToKserd<'static> for MyStruct {
    fn into_kserd(self) -> Result<Kserd<'static>, ToKserdErr> {
        (self.a, self.b).into_kserd()
    }
}
