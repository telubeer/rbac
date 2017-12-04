#![feature(test)]
extern crate test;
extern crate rbac;
#[macro_use] extern crate json;

#[cfg(test)]
mod bench {
    use test::Bencher;
    use rbac::mods::*;

    /* #[bench]
     fn bench_rua(b: &mut Bencher) {
         let dsn = env::var("DSN").ok()
             .expect("You should set mysql connection settings mysql://user:pass@ip:port in DSN env var");
         let pool = Pool::new_manual(1, 1, &dsn).unwrap();
         let d = load(&pool);
         let params = object! {
            "region" => "54",
            "project" => "1",
         };
         b.iter(|| {
             d.check_access(
                 "14338667".to_string(),
                 "ncc.records.update.access".to_string(),
                 &params
             )
         });
     }

     #[bench]
     fn bench_rua2(b: &mut Bencher) {
         let dsn = env::var("DSN").ok()
             .expect("You should set mysql connection settings mysql://user:pass@ip:port in DSN env var");
         let pool = Pool::new_manual(1, 1, &dsn).unwrap();
         let d = load(&pool);
         let params = object! {
            "region" => "55",
            "project" => "1",
         };
         b.iter(|| {
             d.check_access(
                 "14338667".to_string(),
                 "ncc.records.update.access".to_string(),
                 &params
             )
         });
     }

     #[bench]
     fn bench_regions(b: &mut Bencher) {
         let dsn = env::var("DSN").ok()
             .expect("You should set mysql connection settings mysql://user:pass@ip:port in DSN env var");
         let pool = Pool::new_manual(1, 1, &dsn).unwrap();
         let d = load(&pool);
         let regions = [
             "54", "24", "55", "22", "42", "70", "38", "123", "43403", "1077", "181490", "52", "45", "59", "76",
             "72", "74", "29", "27", "33", "73", "31", "23", "93", "66", "63", "2", "34", "61", "47", "44", "21",
             "69", "58", "56", "57", "71", "67", "46", "48", "62", "32", "36", "39", "30", "114160", "14", "16",
             "142982", "182028", "18", "26", "35", "43", "51", "53", "60", "64", "75", "86", "89", "68", "124",
             "155", "142", "138", "170", "166", "154"
         ];

         b.iter(|| {
             for region in regions.iter() {
                 let params = object! {"region" => *region};
                 d.check_access("11414968".to_string(), "ncc.region.access".to_string(), &params);
             }
         })
     } */

    #[bench]
    fn bench_rule(b: &mut Bencher) {
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
}