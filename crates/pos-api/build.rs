use std::path::Path;
use std::{fs, io};

#[allow(clippy::unnecessary_wraps)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    std::env::set_var("OUT_DIR", "src");

    /*
    tonic_build::configure()
        .build_server(true)
        .out_dir("src")
        .format(true)
        .compile(&["proto/pos_api_service/api.proto"], &["proto"])
        .unwrap_or_else(|e| panic!("error building protos {:?}", e));

    let src = Path::new("src");
    remame_protos(&src).unwrap();
    */
    Ok(())
}

fn _remame_protos(dir: &Path) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                let file_stem_renamed = &path
                    .file_stem()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .replace(".", "_");

                fs::rename(&path, dir.join(format!("{}.rs", file_stem_renamed)))?;
            }
        }
    }

    Ok(())
}
