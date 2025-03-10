#[test]
fn test_string() {
    let s = String::from("<");
    println!("Size in bytes od \"{s}\" = {}", s.bytes().len());
}
