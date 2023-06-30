use sass_rs::{compile_string, Options};

pub fn compile(scss: &str) -> String {
    let css_opts = Options::default();
    compile_string(scss, css_opts).expect("invalid scss syntax")
}
