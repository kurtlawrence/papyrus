pub struct MyStruct {
    a: i32,
    b: i32,
}

impl MyStruct {
    pub fn new(a: i32, b: i32) -> Self {
        MyStruct { a, b }
    }

    pub fn add_contents(&self) -> i32 {
        self.a + self.b
    }
}
