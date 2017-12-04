use json;
use std::collections::HashSet;
use super::*;

#[test]
fn it_works() {
    let data = mock_data();
    let params = object! { "r" => "1" };
    assert_eq!(true, data.check_access(1, "action1".to_string(), &params));
    assert_eq!(false, data.check_access(1, "action2".to_string(), &params));
    assert_eq!(false, data.check_access(2, "action1".to_string(), &params));
    assert_eq!(true, data.check_access(2, "action2".to_string(), &params));
    let params2 = object! {};
    assert_eq!(false, data.check_access(2, "action2".to_string(), &params2));
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

pub fn mock_data() -> Data {
    let mut data = Data::new();
    data.map = [
        ("admin".to_string(), 0 as ItemId),
        ("user".to_string(), 1 as ItemId),
        ("action1".to_string(), 2 as ItemId),
        ("action2".to_string(), 3 as ItemId),
        ("task1".to_string(), 4 as ItemId)
    ].iter().cloned().collect();


    data.assignments = [
        (
            1,
            [0].iter().cloned().collect()
        ),
        (
            2,
            [1].iter().cloned().collect()
        )
    ].iter().cloned().collect();

    data.assignments_dict = [
        (
            0,
            Assignment {
                name: 0,
                user_id: 1,
                data: object! {},
            }
        ),
        (
            1,
            Assignment {
                name: 1,
                user_id: 2,
                data: object! {},
            }
        )
    ].iter().cloned().collect();
    data.items = [
        (0, Item {
            name: 0,
            data: json::JsonValue::new_object()
        }),
        (1, Item {
            name: 1,
            data: json::JsonValue::new_object()
        }), (2, Item {
            name: 2,
            data: json::JsonValue::new_object()
        }),
        (3, Item {
            name: 3,
            data: json::JsonValue::new_object()
        }),
        (4, Item {
            name: 4,
            data: object! {
                    "paramsKey" => "r",
                    "data" => array!["1", "2"]
                }
        })
    ].iter().cloned().collect();
    let action1_parents: HashSet<ItemId> = [0 as ItemId].iter().cloned().collect();
    let action2_parents: HashSet<ItemId> = [4 as ItemId].iter().cloned().collect();
    let task1_parents: HashSet<ItemId> = [1 as ItemId].iter().cloned().collect();
    data.parents = [
        (1, [
            (2 as ItemId, action1_parents)
        ].iter().cloned().collect()),
        (2, [
            (3 as ItemId, action2_parents),
            (4 as ItemId, task1_parents)
        ].iter().cloned().collect())
    ].iter().cloned().collect();

    return data;
}