mod nobl;
use nobl::*;
//mod alf;
fn main() {
    let mut r = Hsval::parse("./data.nobl");
    let g = r.get_obj(&mut ["Objecttoooooo".to_string()].iter());
    println!("{:?}", g);
    println!("{:?}", r);
    r.stringify("./doota.nobl");
}
