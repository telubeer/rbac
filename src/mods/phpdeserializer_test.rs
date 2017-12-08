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

#[test]
fn parse_php1() {
    let test = r#"a:2:{s:9:"paramsKey";s:8:"template";s:4:"data";a:5:{i:0;i:1;i:1;i:2;i:2;i:3;i:3;i:4;i:4;i:5;}}"#;
    let mut d = Deserializer::from_str(test);
    let res = d.parse();
    let r = object! {
            "paramsKey" => "template",
            "data" => array![1,2,3,4,5]
        };
    assert_eq!(res, r);
}