extern crate pinboard;
extern crate crossbeam;

#[derive(Clone)]
struct Test(Box<u32>);

impl Drop for Test {
    fn drop(&mut self) {
        println!("Dropping");
    }
}

fn main() {
    let p = pinboard::Pinboard::new(Test(Box::new(0u32)));

    crossbeam::scope(|s| {
        s.spawn(|| {
            for i in 0..1000 {
                println!("Modifying");
                p.set(Test(Box::new(i)));
            }
        });
    });

    println!("Exiting");
}
