#[cfg(test)]
mod tests {
    use test::Bencher;
    use super::super::*;
    use std::collections::{HashSet};

    #[bench]
    fn bench_rua(b: &mut Bencher) {
        let d = load();
        let params = object! {
           "region" => "54",
           "project" => "1",
        };
        b.iter(||  {
            d.check_access(
                "14338667".to_string(),
                "ncc.records.update.access".to_string(),
                &params
            )
        });
    }
    #[bench]
    fn bench_rua2(b: &mut Bencher) {
        let d = load();
        let params = object! {
           "region" => "55",
           "project" => "1",
        };
        b.iter(||  {
            d.check_access(
                "14338667".to_string(),
                "ncc.records.update.access".to_string(),
                &params
            )
        });
    }
    #[bench]
    fn bench_regions(b: &mut Bencher) {
        let d = load();
        let regions = [
            "54","24","55","22","42","70","38","123","43403","1077","181490","52","45","59","76",
            "72","74","29","27","33","73","31","23","93","66","63","2","34","61","47","44","21",
            "69","58","56","57","71","67","46","48","62","32","36","39","30","114160","14","16",
            "142982","182028","18","26","35","43","51","53","60","64","75","86","89","68","124",
            "155","142","138","170","166","154"
        ];

        b.iter(|| {
            for region in regions.iter() {
                let  params = object! {"region" => *region};
                d.check_access("11414968".to_string(), "ncc.region.access".to_string(), &params);
            }
        })
    }

    #[bench]
    fn bech_rule(b: &mut Bencher) {
        let item = object! {
            "paramsKey" => "pid",
            "data" => array!["23", "312", "545", "66", "14338727"]
            };
        let data = Data::new();
        let params = object! { "pid" => "14338727"};
        b.iter(|| {
            data.rule(&item, &params);
        });
    }

    #[test]
    fn it_works() {
        let data = mock_data();
        let params = object! { "r" => "1" };
        assert_eq!(true, data.check_access("1".to_string(), "action1".to_string(), &params));
        assert_eq!(false, data.check_access("1".to_string(), "action2".to_string(), &params));
        assert_eq!(false, data.check_access("2".to_string(), "action1".to_string(), &params));
        assert_eq!(true, data.check_access("2".to_string(), "action2".to_string(), &params));
        let params2 = object!{};
        assert_eq!(false, data.check_access("2".to_string(), "action2".to_string(), &params2));
    }

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
    fn rule() {
        let item = object! {
            "paramsKey" => "pid",
            "data" => array!["14338727"]
            };
        let data = Data::new();
        let params = object! { "pid" => "14338727"};
        assert!(data.rule(&item, &params));
    }

    fn mock_data() -> Data {
        let mut data = Data::new();
        let mut user1 = HashSet::new();
        user1.insert("admin".to_string());
        let mut user2 = HashSet::new();
        user2.insert("user".to_string());
        data.assignments.insert("1".to_string(), user1);
        data.assignments.insert("2".to_string(), user2);
        data.assignments_dict.insert("user".to_string(), Assignment {
            name: "user".to_string(),
            user_id: "2".to_string(),
            data: object!{},
            rule: Some("".to_string())
        });
        data.assignments_dict.insert("admin".to_string(), Assignment {
            name: "admin".to_string(),
            user_id: "1".to_string(),
            data: object!{},
            rule: Some("".to_string())
        });
        data.items.insert("admin".to_string(), Item::new("admin".to_string(), 1));
        data.items.insert("user".to_string(), Item::new("user".to_string(), 1));
        data.items.insert("action1".to_string(), Item::new("action1".to_string(), 3));
        data.items.insert("action2".to_string(), Item::new("action2".to_string(), 3));
        let task = Item {
            name: "task1".to_string(),
            rule: Some("".to_string()),
            data: object!{
                "paramsKey" => "r",
                "data" => array!["1", "2"]
            },
            item_type: 2
        };
        data.items.insert("task1".to_string(), task);

        let mut action1 = Vec::new();
        action1.push("admin".to_string());
        data.parents.insert("action1".to_string(), action1);

        let mut action2 = Vec::new();
        action2.push("task1".to_string());
        data.parents.insert("action2".to_string(), action2);

        let mut task1 = Vec::new();
        task1.push("user".to_string());
        data.parents.insert("task1".to_string(), task1);


        return data;
    }
}
