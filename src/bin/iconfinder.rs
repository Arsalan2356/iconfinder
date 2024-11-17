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
    let mut arg = match env::args().nth(1) {
        Some(v) => v,
        None => "".to_string(),
    };

    let data = fs::read("./data").unwrap();

    let _ = arg.retain(|c| !c.is_ascii_whitespace());

    let title_to_appids =
        decode::<HashMap<String, String>>(&fs::read("./steamdata").unwrap()).unwrap();

    for e in title_to_appids.keys() {
        if normalized_damerau_levenshtein(e, &arg) > 0.75 {
            let final_icon_path = title_to_appids.get(e);
            match final_icon_path {
                Some(p) => {
                    println!("{}", p);
                    return;
                }
                None => {}
            }
        }
    }

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
                        &arg,
                    )
                })
                .into_iter()
                .enumerate()
                .max_by(|(_idx, val), (_idx2, val2)| val.total_cmp(val2))
                .unwrap();
            if time.1 > 0.30 {
                maxes[i] = time;
            }
            if time.1 >= 0.75 {
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
        } else if maxes[i].1 > 0.0 {
            // A much better match
            if maxes[i].1 - maxes[maxres].1 > 0.08 {
                maxres = i;
                maxicon = maxes[i].0;
            }
        }
    }
    if maxres != usize::MAX {
        let final_icon_path = icons[maxres][maxicon].to_str().unwrap();
        println!("{}", final_icon_path);
    }
}
