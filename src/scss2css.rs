use sass_rs::{compile_file, Options};

pub fn getter(path: &str) -> impl Fn() -> String + Clone {
    let css_opts = Options::default();
    let css = compile_file(path, css_opts).expect("scss file not found");

    move || css.clone()
}
