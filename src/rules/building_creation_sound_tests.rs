use crate::rules::ini_parser::IniFile;
use crate::rules::process_owner::{NativeRulesProcessOwner, NativeScenarioRulesPrefix};
use crate::rules::sound_ini::SoundRegistry;
use std::sync::Arc;

#[test]
fn building_creation_cue_retains_registered_references_and_building_scope_across_load() {
    let sounds = Arc::new(SoundRegistry::from_ini(&IniFile::from_str(
        "[SoundList]\n0=RootCue\n1=ModeCue\n",
    )));
    let mut owner = NativeRulesProcessOwner::from_cold_start_sources(
        IniFile::from_str("[BuildingTypes]\n0=VAT\n1=EMPTY\n[VehicleTypes]\n0=TANK\n[VAT]\nCreateUnitSound=RootCue\n[TANK]\nCreateUnitSound=RootCue\n"),
        Some(IniFile::from_str("[VAT]\nCreateUnitSound=Unknown\n")),
        IniFile::empty(), sounds, Default::default(),
    ).unwrap();
    let mode = IniFile::from_str("[VAT]\nCreateUnitSound=ModeCue\n");
    let (rules, _, _, _) = owner
        .load_scenario(
            NativeScenarioRulesPrefix::NonCampaign(Some(&mode)),
            &IniFile::from_str("[VAT]\nCreateUnitSound=Unknown\n[EMPTY]\nCreateUnitSound=\n"),
        )
        .unwrap()
        .into_parts();
    assert_eq!(
        rules.object("VAT").unwrap().building_create_unit_sound(),
        Some("ModeCue")
    );
    assert_eq!(
        rules.object("EMPTY").unwrap().building_create_unit_sound(),
        None
    );
    assert_eq!(
        rules.object("TANK").unwrap().building_create_unit_sound(),
        None
    );
    let bytes = crate::util::save_codec::serialize(owner.saved_scenario_rules().unwrap()).unwrap();
    let saved = crate::util::save_codec::deserialize(&bytes).unwrap();
    let restored = owner.project_saved_scenario_rules(&saved).unwrap();
    assert_eq!(
        restored.object("VAT").unwrap().building_create_unit_sound(),
        Some("ModeCue")
    );
}
