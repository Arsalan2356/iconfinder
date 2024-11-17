use bitcode::encode;
use std::collections::HashMap;
use std::fs::canonicalize;
use std::fs::{self, read_dir};
use std::path::{Path, PathBuf};
use std::usize;
use std::vec;

use regex::Regex;

const RES: [&str; 19] = [
    "symbolic", "scalable", "2048", "1024", "512", "256", "192", "128", "96", "84", "72", "64",
    "48", "42", "36", "32", "24", "22", "16",
];

fn main() {
    let home = std::env::var("HOME").unwrap();
    let paths = [
        "/run/current-system/sw/share/icons".to_string(),
        home.clone() + "/.local/share/icons",
    ];
    println!("PATHS : {:?}", paths);

    let mut icons: [Vec<PathBuf>; 19] = Default::default();
    for i in 0..RES.len() {
        icons[i] = vec![];
    }

    const ICON_THEME: &str = "/Papirus-Dark";

    const FIRST_DIRS: [&str; 2] = [ICON_THEME, "/hicolor"];

    let mut steamicons: HashMap<String, String> = HashMap::new();

    for path in paths.iter() {
        for fd in FIRST_DIRS {
            let p2 = path.to_owned() + fd;
            println!("Searching... : {}", p2);
            search_first(Path::new(&p2), RES.len(), &mut icons, &mut steamicons);
        }
    }

    let saved_icons: [Vec<String>; 19] = icons.map(|x| {
        x.into_iter()
            .map(|y| y.into_os_string().into_string().unwrap())
            .collect()
    });

    let steampaths = [
        home + "/.local/share/Steam/steamapps",
        "/mnt/G/SteamLibrary/steamapps".to_string(),
    ];

    let reg = Regex::new(r#""name"\t\t"([^"]*)""#).unwrap();
    let reg2 = Regex::new(r#""appid"\t\t"([^"]*)""#).unwrap();

    let mut title_to_appids: HashMap<String, String> = HashMap::new();
    for s in steampaths {
        println!("Steam Searching... : {}", s);
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
    let edata = encode(&saved_icons);
    let sdata = encode(&title_to_appids);

    let _ = fs::write("./data", edata);
    let _ = fs::write("./steamdata", sdata);
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
