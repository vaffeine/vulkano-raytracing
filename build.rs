#[macro_use]
extern crate tera;

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn to_extension_str(path: &PathBuf) -> &str {
    path.extension()
        .expect("can't get extension")
        .to_str()
        .expect("can't get string from extension")
}

fn main() {
    let root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let shaders_dir = root_dir.join("shaders");
    assert!(shaders_dir.exists(), "shaders directory doesn't exist");

    let shaders: Vec<_> = shaders_dir
        .read_dir()
        .expect("can't read shaders direcotry")
        .map(|shader_entry| shader_entry.expect("can'r read shader").path())
        .filter(|shader| {
            let shader_extension = to_extension_str(&shader);
            ["comp", "frag", "vert", "tera"].contains(&shader_extension)
        })
        .collect();

    for shader in std::iter::once(&shaders_dir).chain(shaders.iter()) {
        println!(
            "cargo:rerun-if-changed={}",
            shader.to_str().expect("can't convert path to str")
        );
    }

    let templates_glob = {
        let mut p = shaders_dir
            .to_str()
            .expect("can't get string from path")
            .to_string();
        p.push_str("/**/*.tera");
        p
    };
    let tera = compile_templates!(&templates_glob);
    let rendered_shaders_dir = root_dir.join("target").join("shaders");
    std::fs::create_dir_all(rendered_shaders_dir.clone()).expect("failed to create directory");
    for shader in shaders
        .into_iter()
        .filter(|shader| to_extension_str(&shader) == "tera")
    {
        let rendered_shader = tera.render(
            shader.strip_prefix(&shaders_dir).unwrap().to_str().unwrap(),
            &tera::Context::new(),
        ).expect("failed to render shader template");
        let output_path = rendered_shaders_dir.join(
            shader
                .strip_prefix(&shaders_dir)
                .unwrap()
                .with_extension(""),
        );
        println!("{:?}", output_path);
        let mut file = File::create(output_path).expect("can't create file for rendered shader");
        file.write_all(rendered_shader.as_bytes())
            .expect("failed to write rendered shader to file");
    }
}
