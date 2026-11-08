use std::sync::Arc;

use super::{NativeRulesProcessOwner, NativeScenarioRulesPrefix};
use crate::rules::ini_parser::IniFile;
use crate::rules::sound_ini::SoundRegistry;

fn owner(root: &str) -> NativeRulesProcessOwner {
    NativeRulesProcessOwner::from_cold_start_sources(
        IniFile::from_str(root),
        Some(IniFile::from_str("[General]\nNewPlayableCountries=50\n")),
        IniFile::empty(),
        Arc::new(SoundRegistry::from_ini(&IniFile::empty())),
        Default::default(),
    )
    .unwrap()
}

#[test]
fn country_random_bound_uses_the_startup_root_not_later_layers() {
    let mut process = owner("[General]\nNewPlayableCountries=-7\n");
    let expected = if crate::rules::NPATCH_ENABLED { 2 } else { 9 };
    assert_eq!(process.ordinary_random_country_upper_bound(), expected);
    let mode = IniFile::from_str("[General]\nNewPlayableCountries=3\n");
    let map = IniFile::from_str("[General]\nNewPlayableCountries=8\n");
    process
        .load_scenario(NativeScenarioRulesPrefix::NonCampaign(Some(&mode)), &map)
        .unwrap();
    assert_eq!(process.ordinary_random_country_upper_bound(), expected);
}

#[test]
fn absent_country_extension_keeps_the_stock_constructor_bound() {
    let process = owner("");
    assert_eq!(process.ordinary_random_country_upper_bound(), 9);
}
