
#[derive(Clone, Copy)]
pub struct Value(pub f64);

impl Value {
    pub fn print(&self) {
        print!("{}", self.0);
    }
}