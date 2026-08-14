#[test]
fn application_layer_does_not_depend_on_appcore_or_web() {
    let manifest = include_str!("../Cargo.toml");
    assert!(!manifest.contains("appcore"));
    assert!(!manifest.contains("next"));
    assert!(!manifest.contains("react"));
}
