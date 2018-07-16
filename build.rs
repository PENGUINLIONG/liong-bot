use std::fs::create_dir_all;
use std::path::Path;
use std::process::Command;

const OUT_FILE: &str = "./lib/CQP.lib";
const IN_FILE: &str = "./cqp.def";

pub fn main() {
    let path = Path::new(OUT_FILE);
    if !path.exists() {
        if !path.parent().unwrap().exists() {
            create_dir_all(path.parent().unwrap())
                .expect("Unable to create symbol library output directory.");
        }
        Command::new("lib")
            .arg(format!("/def:{} /OUT:{} /MACHINE:X86", IN_FILE, OUT_FILE))
            .spawn()
            .expect("Unable to create symbol library, which is necessary for \
                    MSVC linker to work.")
            .wait()
            .expect("Unable to wait for `lib` to quit.");
    }

    println!("cargo:rustc-link-search=./lib");

}
