mod nobl;
use nobl::*;
mod paths;
use paths::*;
fn main() {
    let [_, data, template] = db();
    let template = Hsval::parse_file(template);
    let mut nobl_data =  Hsval::parse_file(data);
    nobl_data.template(&template);
    nobl_data.search(vec![Some("person".to_string()), None]);
    println!("{:?}", nobl_data);
    nobl_data.stringify("./doota.nobl");
}
