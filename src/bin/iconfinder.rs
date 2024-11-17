use bitcode::decode;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::usize;

use strsim::normalized_damerau_levenshtein;

const RES: [&str; 19] = [
    "symbolic", "scalable", "2048", "1024", "512", "256", "192", "128", "96", "84", "72", "64",
    "48", "42", "36", "32", "24", "22", "16",
];

fn main() {
    let args: Vec<String> = env::args().collect();

    let data = fs::read("./data").unwrap();

    let title_to_appids =
        decode::<HashMap<String, String>>(&fs::read("./steamdata").unwrap()).unwrap();

    match title_to_appids.get(&args[1]) {
        Some(p) => {
            let final_icon_path = p;
            println!("{}", final_icon_path);
            return;
        }
        None => {
            let icons: [Vec<PathBuf>; 19] = decode::<[Vec<String>; 19]>(&data)
                .unwrap()
                .map(|x| x.into_iter().map(|y| PathBuf::from(y)).collect());

            let mut maxes: [(usize, f64); 19] = [(0, 0.0); 19];

            for i in 0..RES.len() {
                if icons[i].len() > 0 {
                    let time = icons[i]
                        .clone()
                        .into_iter()
                        .map(|x| {
                            normalized_damerau_levenshtein(
                                &String::from(x.file_stem().unwrap().to_str().unwrap()),
                                &args[1],
                            )
                        })
                        .into_iter()
                        .enumerate()
                        .max_by(|(_idx, val), (_idx2, val2)| val.total_cmp(val2))
                        .unwrap();
                    if time.1 > 0.30 {
                        maxes[i] = time;
                        println!("Best Match at res {}", RES[i]);
                        println!("DL Value : {}", time.1);
                        println!("Index : {}", time.0);
                        println!("Icon Path : {}", icons[i][time.0].to_str().unwrap());
                    }
                    if time.1 >= 0.75 {
                        println!("Break on");
                        println!("DL Value : {}", time.1);
                        println!("Index : {}", time.0);
                        println!("Icon Path : {}", icons[i][time.0].to_str().unwrap());
                        break;
                    }
                }
            }

            let mut maxres = usize::MAX;
            let mut maxicon = usize::MAX;
            for i in 0..maxes.len() {
                if maxes[i].1 > 0.0 && maxres == usize::MAX {
                    maxres = i;
                    maxicon = maxes[i].0;
                    println!(
                        "Current Best Match : {}",
                        icons[maxres][maxicon].to_str().unwrap()
                    );
                    println!("Match % Value : {}", maxes[maxres].1);
                } else if maxes[i].1 > 0.0 {
                    // A much better match
                    if maxes[i].1 - maxes[maxres].1 > 0.08 {
                        maxres = i;
                        maxicon = maxes[i].0;
                        println!("Switching best match");
                        println!(
                            "Current Best Match : {}",
                            icons[maxres][maxicon].to_str().unwrap()
                        );
                        println!("Match % Value : {}", maxes[maxres].1);
                    }
                }
            }
            let final_icon_path = icons[maxres][maxicon].to_str().unwrap();
            println!("{}", final_icon_path);
        }
    }
}
