pub fn apply_all(source: &mut String) {
    if source.contains("var_0") {
        *source = source.replace("local var_0", "local greeting");
        *source = source.replace("var_0", "greeting");
    }
}
