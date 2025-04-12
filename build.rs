use windres::Build;

fn main() {
    Build::new().compile("icons.rc").unwrap();
}