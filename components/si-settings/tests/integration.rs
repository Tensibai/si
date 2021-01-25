use lazy_static::lazy_static;

use std::env;

use si_settings::Settings;

lazy_static! {
    pub static ref SETTINGS: Settings = {
        env::set_var("RUN_ENV", "testing");
        env::set_var("SI_PG__PASSWORD", "testing");
        Settings::new().expect("Failed to load settings")
    };
}

#[test]
fn uses_default_values() {
    assert_eq!(SETTINGS.service.port, 8182);
}

#[test]
fn uses_run_env_values() {
    assert_eq!(SETTINGS.pg.hostname, "127.0.0.2");
}

#[test]
fn uses_local_toml_values() {
    assert_eq!(SETTINGS.pg.user, "monkey");
}

#[test]
fn uses_environment_variables() {
    assert_eq!(SETTINGS.pg.password, "testing");
}
