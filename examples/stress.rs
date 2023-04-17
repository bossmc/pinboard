#[derive(Clone)]
struct Test(u32);

impl Drop for Test {
    fn drop(&mut self) {
        println!("Dropping ({})", self.0);
    }
}

fn main() {
    let p = pinboard::Pinboard::new(Test(0u32));

    crossbeam::scope(|s| {
        for _ in 0..100 {
            s.spawn(|_| {
                for i in 0..1000 {
                    println!("Modifying");
                    p.set(Test(i));
                }
            });
        }
    }).unwrap();

    println!("Exiting");
}
