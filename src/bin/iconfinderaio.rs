use std::collections::HashMap;
use std::env;
use std::fs;
use std::fs::canonicalize;
use std::fs::read_dir;
use std::path::{Path, PathBuf};
use std::usize;
use std::vec;

use regex::Regex;
use strsim::normalized_damerau_levenshtein;

const RES: [&str; 19] = [
    "symbolic", "scalable", "2048", "1024", "512", "256", "192", "128", "96", "84", "72", "64",
    "48", "42", "36", "32", "24", "22", "16",
];

fn main() {
    let args: Vec<String> = env::args().collect();
    let home = std::env::var("HOME").unwrap();
    let paths = [
        "/run/current-system/sw/share/icons".to_string(),
        home.clone() + "/.local/share/icons",
    ];
    let mut icons: [Vec<PathBuf>; 19] = Default::default();
    for i in 0..RES.len() {
        icons[i] = vec![];
    }

    let steampaths = [
        home + "/.local/share/Steam/steamapps",
        "/mnt/G/SteamLibrary/steamapps".to_string(),
    ];

    let reg = Regex::new(r#""name"\t\t"([^"]*)""#).unwrap();
    let reg2 = Regex::new(r#""appid"\t\t"([^"]*)""#).unwrap();

    let mut steamicons: HashMap<String, String> = HashMap::new();

    const ICON_THEME: &str = "/Papirus-Dark";

    const FIRST_DIRS: [&str; 2] = [ICON_THEME, "/hicolor"];

    for path in paths.iter() {
        for fd in FIRST_DIRS {
            let p2 = path.to_owned() + fd;
            search_first(Path::new(&p2), RES.len(), &mut icons, &mut steamicons);
        }
    }

    let mut title_to_appids: HashMap<String, String> = HashMap::new();
    for s in steampaths {
        let items = read_dir(s).unwrap();
        for i in items {
            let e = i.unwrap();
            if e.metadata().unwrap().is_file()
                && e.file_name().into_string().unwrap().ends_with(".acf")
            {
                let fdata = fs::read_to_string(e.path().to_str().unwrap()).unwrap();

                let e2 = reg2.captures(&fdata).unwrap().get(1).unwrap();
                let cap = reg.captures(&fdata).unwrap().get(1).unwrap();

                match steamicons.get(e2.as_str()) {
                    Some(v) => {
                        title_to_appids
                            .insert(cap.as_str().to_string().replace("®", ""), v.to_string());
                    }
                    None => {}
                }
            }
        }
    }

    match title_to_appids.get(&args[1]) {
        Some(p) => {
            let final_icon_path = p;

            println!("{}", final_icon_path);
        }
        None => {
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
                    }
                    if time.1 >= 0.7 {
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
                    if maxes[i].1 - maxes[maxres].1 > 0.05 {
                        maxres = i;
                        maxicon = maxes[i].0;
                    }
                }
            }
            let final_icon_path = icons[maxres][maxicon].to_str().unwrap();
            println!("{}", final_icon_path);
        }
    }
}

fn search_first(
    path: &Path,
    maxres: usize,
    icons: &mut [Vec<PathBuf>],
    steamicons: &mut HashMap<String, String>,
) -> () {
    let items = fs::read_dir(path);
    match items {
        Ok(t) => {
            for entry in t {
                let e = entry.unwrap();
                let metadata = e.metadata().unwrap();
                let mut realpath = if metadata.is_symlink() {
                    canonicalize(e.path()).unwrap()
                } else {
                    e.path()
                };
                let mut index = usize::MAX;
                for i in 0..maxres {
                    if e.path().to_str().unwrap().contains(RES[i]) {
                        index = i;
                        break;
                    }
                }
                if index == usize::MAX {
                    continue;
                }
                realpath.push("apps");
                let exists = fs::exists(realpath);
                match exists {
                    Ok(t) => {
                        if t {
                            // Search the new path
                            let mut newpath = e.path().clone();
                            newpath.push("apps");
                            let mut q = search_second(&newpath, steamicons);
                            icons[index].append(&mut q);
                        }
                    }
                    Err(_e) => {
                        continue;
                    }
                }
            }
        }
        Err(_e) => {
            return;
        }
    }
    return;
}

fn search_second(path: &Path, steamicons: &mut HashMap<String, String>) -> Vec<PathBuf> {
    let items = fs::read_dir(path);
    match &items {
        Ok(_t) => {
            let v = items.unwrap().map(|x| x.unwrap().path());
            let mut ret: Vec<PathBuf> = vec![];
            for e in v {
                let name = e.file_stem().unwrap().to_str().unwrap();

                if name.contains("steam_icon_") {
                    if steamicons.contains_key(&name[11..].to_string()) {
                        let gp = e
                            .parent()
                            .unwrap()
                            .parent()
                            .unwrap()
                            .file_name()
                            .unwrap()
                            .to_str()
                            .unwrap();
                        let mut sindex = usize::MAX;
                        for i in 0..RES.len() {
                            if gp.contains(RES[i]) {
                                sindex = i;
                                break;
                            }
                        }

                        if sindex == usize::MAX {
                            println!("Something went wrong");
                            println!("Grandparent : {:?}", gp);
                            println!("E Value : {:?}", e);
                            println!("RES : {:?}", RES);
                        }
                        let oldpath =
                            PathBuf::from(steamicons.get(&name[11..].to_string()).unwrap());
                        let oldgp = oldpath
                            .parent()
                            .unwrap()
                            .parent()
                            .unwrap()
                            .file_name()
                            .unwrap()
                            .to_str()
                            .unwrap();

                        let mut oldsindex = usize::MAX;
                        for i in 0..RES.len() {
                            if oldgp.contains(RES[i]) {
                                oldsindex = i;
                                break;
                            }
                        }

                        if sindex < oldsindex {
                            steamicons
                                .insert(name[11..].to_string(), e.to_str().unwrap().to_string());
                        }
                    } else {
                        steamicons.insert(name[11..].to_string(), e.to_str().unwrap().to_string());
                    }
                } else {
                    ret.push(e);
                }
            }
            return ret;
        }
        Err(_e) => {
            return vec![];
        }
    }
}
