use super::*;

#[test]
fn parse_php() {
    let test = r#"a:2:{s:9:"paramsKey";s:3:"pid";s:4:"data";a:1:{i:0;s:8:"14338727";}}"#;
    let mut d = Deserializer::from_str(test);
    let res = d.parse();
    let r = object! {
            "paramsKey" => "pid",
            "data" => array!["14338727"]
        };
    assert_eq!(res, r);
}