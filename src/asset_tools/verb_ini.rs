//! Read-only INI inspection through production source selection and readers.
//!
//! Accessor reports describe the retained compatibility INI projection. Type
//! reports read the final scenario RuleSet returned by that same processing
//! owner; this tool never recreates constructors, readers or post-passes.

use std::path::Path;

use serde_json::{Value, json};

use crate::asset_tools::report::ErrorReport;
use crate::assets::asset_manager::AssetManager;
use crate::rules::ini_parser::{IniFile, IniSection};
use crate::rules::retail_sources::{RetailRulesSources, select_ini};
use crate::rules::ruleset::RuleSet;
use crate::util::fixed_math::SimFixed;

#[derive(Clone, Copy, Debug)]
pub enum IniQuery<'a> {
    Accessor { section: &'a str, key: &'a str },
    Type { type_id: &'a str },
}

impl IniQuery<'_> {
    fn verb(self) -> &'static str {
        match self {
            Self::Accessor { .. } => "ini-get",
            Self::Type { .. } => "ini-type",
        }
    }
}

#[derive(Debug, Default)]
pub struct IniOptions {
    pub domain: Option<String>,
    pub reader: Option<String>,
    pub default: Option<String>,
    pub capacity: Option<usize>,
    pub map: Option<String>,
    pub mode_id: Option<i32>,
}

impl IniOptions {
    pub fn validate(&self, query: IniQuery<'_>) -> Result<(), String> {
        let verb = query.verb();
        match self.domain.as_deref() {
            Some("rules") => {
                if self.map.as_deref().is_none_or(str::is_empty) || self.mode_id.is_none() {
                    return Err(format!(
                        "{verb} --domain rules requires explicit --map and --mode-id"
                    ));
                }
            }
            Some("art") => {
                if self.map.is_some() || self.mode_id.is_some() {
                    return Err(
                        "ARTMD is fixed, not scenario-layered; omit --map and --mode-id".into(),
                    );
                }
            }
            _ => return Err(format!("{verb} requires --domain rules|art")),
        }
        if let IniQuery::Type { type_id } = query {
            if self.domain.as_deref() != Some("rules") {
                return Err("ini-type requires --domain rules".into());
            }
            if type_id.is_empty() {
                return Err("ini-type requires a nonempty type identity".into());
            }
            if self.reader.is_some() || self.default.is_some() || self.capacity.is_some() {
                return Err("ini-type rejects --reader, --default and --capacity; it reads final stored type fields".into());
            }
            return Ok(());
        }
        let reader = self.reader.as_deref().ok_or("ini-get requires --reader")?;
        if !matches!(
            reader,
            "raw" | "int" | "bool" | "double" | "string" | "range" | "speed" | "coord"
        ) {
            return Err("--reader must be raw|int|bool|double|string|range|speed|coord".into());
        }
        if reader == "raw" && self.default.is_some() {
            return Err("--default is not valid for the raw reader".into());
        }
        if reader != "raw" && self.default.is_none() {
            return Err(
                "typed readers require an explicit --default from the caller contract".into(),
            );
        }
        if (reader == "string") != self.capacity.is_some() {
            return Err("--capacity is required only for the string reader".into());
        }
        // Validate CLI values before mounting retail assets. These are supplied
        // typed defaults, not INI tokens, so strict CLI parsing is intentional.
        accessor(&IniFile::empty(), "", "", self)?;
        Ok(())
    }
}

fn cli_i32(text: &str) -> Result<i32, String> {
    text.parse()
        .map_err(|_| "--default requires a signed decimal i32 for this reader".into())
}

fn float_result(value: f64) -> Value {
    json!({
        "value": if value.is_finite() { json!(value) } else { Value::Null },
        "representation": value.to_string(),
        "binary64_bits": format!("0x{:016x}", value.to_bits()),
    })
}

fn accessor(
    ini: &IniFile,
    section: &str,
    key: &str,
    options: &IniOptions,
) -> Result<Value, String> {
    let empty = IniSection::new(section.to_owned());
    let section = ini.section(section).unwrap_or(&empty);
    let default = options.default.as_deref().unwrap_or("");
    Ok(match options.reader.as_deref() {
        Some("raw") => json!(raw_value(section, key)),
        Some("int") => json!(section.read_int(key, cli_i32(default)?)),
        Some("bool") => {
            let default = default
                .parse::<bool>()
                .map_err(|_| "--default for bool must be true or false")?;
            json!(section.read_bool(key, default))
        }
        Some("double") => {
            let default = default
                .parse::<f64>()
                .map_err(|_| "--default for double must be a binary64 number")?;
            float_result(section.read_double(key, default))
        }
        Some("string") => json!(
            section.read_string(
                key,
                default,
                options
                    .capacity
                    .ok_or("string reader requires --capacity")?
            )
        ),
        Some("range") => json!(section.read_range(key, cli_i32(default)?)),
        Some("speed") => json!(section.read_speed(key, cli_i32(default)?)),
        Some("coord") => {
            let values = default
                .split(',')
                .map(cli_i32)
                .collect::<Result<Vec<_>, _>>()?;
            let values: [i32; 3] = values
                .try_into()
                .map_err(|_| "--default for coord must be three signed decimal integers: x,y,z")?;
            json!(section.read_coord3(key, values))
        }
        _ => return Err("ini-get requires a supported --reader".into()),
    })
}

/// The stored text, shown as it is: the tool's raw view, not a reader.
fn raw_value<'a>(section: &'a IniSection, key: &str) -> Option<&'a str> {
    section
        .raw_entries()
        .find_map(|(entry, value)| (entry == key).then_some(value))
}

fn presence(ini: &IniFile, section: &str, key: &str) -> Value {
    let selected = ini.section(section);
    json!({
        "section_present": selected.is_some(),
        "key_present": selected.is_some_and(|s| s.is_present(key)),
        "raw_value": selected.and_then(|s| raw_value(s, key)),
    })
}

fn layer(kind: &str, source: Value, ini: Option<&IniFile>, query: IniQuery<'_>) -> Value {
    let mut report = json!({
        "layer": kind,
        "source": source,
        "source_present": ini.is_some(),
    });
    if let IniQuery::Accessor { section, key } = query {
        report["authored"] = json!(ini.map(|ini| presence(ini, section, key)));
    }
    report
}

fn float32_result(value: f32) -> Value {
    json!({
        "value": if value.is_finite() { json!(value) } else { Value::Null },
        "representation": value.to_string(),
        "binary32_bits": format!("0x{:08x}", value.to_bits()),
    })
}

fn fixed_result(value: SimFixed) -> Value {
    json!({
        "value": value.to_num::<f64>(),
        "representation": value.to_string(),
        "storage": "SimFixed/I16F16",
        "fractional_bits": SimFixed::FRAC_NBITS,
        "raw_bits": value.to_bits(),
        "raw_bits_hex": format!("0x{:08x}", value.to_bits() as u32),
    })
}

fn final_type_fields(object: &crate::rules::object_type::ObjectType, processed: &IniFile) -> Value {
    let params = &object.jumpjet_params;
    json!({
        "id": object.id,
        "name": object.name,
        "ui_name": object.ui_name,
        "image": object.image,
        "category": object.category,
        "no_shadow": object.no_shadow,
        "considered_aircraft": object.considered_aircraft,
        "voxel_shadow_index": object.voxel_shadow_index(),
        "locomotor": object.locomotor,
        "locomotor_clsid": object.locomotor.clsid(),
        "speed_type": object.speed_type,
        "movement_zone": object.movement_zone,
        "balloon_hover": object.balloon_hover,
        "is_simple_deployer": object.is_simple_deployer,
        "deploy_to_land": object.deploy_to_land,
        "hover_attack": object.hover_attack,
        "jumpjet": object.jumpjet,
        "crashable": object.crashable,
        "tilt_crash_jumpjet": object.tilt_crash_jumpjet,
        "natural_particle_system": object.natural_particle_system,
        "natural_particle_location": object.natural_particle_location.to_array(),
        "retained_ini": processed.section(&object.id).map(|section| {
            section.raw_entries().collect::<std::collections::BTreeMap<_, _>>()
        }),
        "jumpjet_params": {
            "turn_rate": params.turn_rate,
            "speed": fixed_result(params.speed),
            "climb": float32_result(params.climb),
            "crash": float32_result(params.crash),
            "height": params.height,
            "accel": float32_result(params.accel),
            "wobbles": float32_result(params.wobbles),
            "no_wobbles": params.no_wobbles,
            "deviation": params.deviation,
        },
    })
}

fn assemble_type(
    rules: &RuleSet,
    processed: &IniFile,
    type_id: &str,
    layers: Vec<Value>,
) -> Result<Value, String> {
    if type_id == "*" {
        let mut types = rules
            .all_objects()
            .map(|object| final_type_fields(object, processed))
            .collect::<Vec<_>>();
        types.sort_by(|left, right| left["id"].as_str().cmp(&right["id"].as_str()));
        return Ok(json!({
            "schema_version": 1,
            "domain": "rules",
            "requested_type": "*",
            "result_kind": "final_object_type_inventory",
            "source_layers": layers,
            "final_types": types,
            "units": {"natural_particle_location": "relative leptons", "jumpjet_params.height": "leptons", "jumpjet_params.deviation": "leptons"},
            "limits": [
                "Registered ObjectTypes from the final production RuleSet, sorted by ID for reporting only; this is not a live or buildable object inventory.",
                "retained_ini is exact-case processed text, not proof of final field values or active consumers. Use ini-get with the native reader/default when interpreting an unimplemented key.",
                "The supported noncampaign skirmish path only; campaign, TMCJ4F and generated random-map passes are not included. No new native equivalence is established."
            ]
        }));
    }
    let object = rules
        .object(type_id)
        .ok_or_else(|| format!("type {type_id:?} is absent from the final scenario RuleSet"))?;
    Ok(json!({
        "schema_version": 1,
        "domain": "rules",
        "requested_type": type_id,
        "lookup": "case_insensitive_type_identity",
        "result_kind": "final_object_type",
        "source_layers": layers,
        "final_type": final_type_fields(object, processed),
        "units": {"natural_particle_location": "relative leptons", "jumpjet_params.height": "leptons", "jumpjet_params.deviation": "leptons"},
        "limits": [
            "Fields come from the final Rust scenario RuleSet, including its constructors/readers/post-passes; this report alone establishes no new native equivalence.",
            "Jumpjet parameters are stored type values on every ObjectType, independent of JumpJet. Locomotor Link conversions/clamps and live flight state are not applied.",
            "retained_ini contains exact-case text from the processed INI, not final fields or proof that a reader/consumer uses a key.",
            "Rules inspection executes the supported noncampaign skirmish path with the special TMCJ4F flag absent; campaign passes, LANGRULE Digest handling and that flagged extra pass are not modeled.",
            "Existing production parser/reader residuals remain; no replacement parser or type loader is used."
        ],
    }))
}

fn assemble(
    ini: &IniFile,
    section: &str,
    key: &str,
    options: &IniOptions,
    layers: Vec<Value>,
) -> Result<Value, String> {
    Ok(json!({
        "schema_version": 1,
        "domain": options.domain,
        "section": section,
        "key": key,
        "lookup": "exact_case",
        "reader": options.reader,
        "supplied_default": options.default,
        "string_capacity_bytes": options.capacity,
        "authored_layers": layers,
        "processed": presence(ini, section, key),
        "accessor_result": accessor(ini, section, key, options)?,
        "result_kind": "production_ini_accessor",
        "limits": [
            "An accessor result is not a final gameplay field: constructors, type allocation timing, field-specific defaults/clamps, art indirection and post-read passes may change it.",
            "Rules inspection executes the supported noncampaign skirmish path with the special TMCJ4F flag absent; campaign passes, LANGRULE Digest handling and that flagged extra pass are not modeled.",
            "Authored values reflect the production byte parser: empty values are omitted and duplicate-key resolution retains its documented stock-inert residual.",
            "Reader residuals remain: malformed ReadDouble/coord values use deterministic Rust recovery; ReadRange overflow differs from native. No new native equivalence is claimed."
        ],
    }))
}

fn resource_inputs_inventory(
    rules: &RuleSet,
    basic: &crate::map::basic::BasicSection,
    flags: &crate::map::basic::SpecialFlagsSection,
    terrain: &[crate::map::overlay::TerrainObject],
    actions: &crate::map::actions::ActionMap,
) -> Value {
    let mut spawners = std::collections::BTreeMap::new();
    for object in terrain {
        if let Some(kind) = rules
            .terrain_object_type_case_insensitive(&object.name)
            .filter(|kind| kind.spawns_tiberium)
        {
            let count = spawners
                .entry(kind.name.clone())
                .or_insert((0usize, kind.is_animated));
            count.0 += 1;
        }
    }
    let toggles = actions
        .iter()
        .filter_map(|(id, record)| {
            let entries = record
                .entries()
                .iter()
                .filter(|entry| entry.kind == 86)
                .collect::<Vec<_>>();
            (!entries.is_empty()).then_some((id, entries))
        })
        .collect::<std::collections::BTreeMap<_, _>>();
    json!({
        "basic_tiberium_growth_enabled": basic.tiberium_growth_enabled,
        "special_flags": {"tiberium_grows": flags.tiberium_grows, "tiberium_spreads": flags.tiberium_spreads},
        "tiberium_types": rules.tiberium_types.types().iter().map(|kind| json!({
            "id": kind.id.0, "section": kind.section, "image": kind.image,
            "growth": kind.growth, "growth_percentage": float_result(f64::from_bits(kind.growth_percentage_bits)),
            "spread": kind.spread, "spread_percentage": float_result(f64::from_bits(kind.spread_percentage_bits)),
        })).collect::<Vec<_>>(),
        "authored_terrain_spawners": spawners.into_iter().map(|(name,(count,animated))| json!({
            "type_id":name,"authored_instance_count":count,"is_animated":animated,
        })).collect::<Vec<_>>(),
        "declared_growth_actions": toggles,
        "limits": [
            "Map Basic/SpecialFlags use MapFile's production readers, not the layered Rules Basic section. Null means unset, not false.",
            "OreGrowthConfig::resolve owns defaults and live gates: fresh Basic growth defaults true; noncampaign session flags override map SpecialFlags. This report does not create a session or resolve live flags.",
            "Tiberium fields come from the final layered RuleSet; spawners count authored map instances of registered SpawnsTiberium types before placement/asset binding. Actions86 are declarations, not trigger reachability or executed effects.",
            "No resources are consumed or generated. Initial usable ore, dynamic spawners, growth queues, admission, timers and permanent exhaustion require the actual runtime owners."
        ],
    })
}

fn team_ai_inventory(
    registry: &crate::rules::team_ai_ini::TeamAiIniRegistry,
    rules: &RuleSet,
) -> Result<Value, String> {
    let mut sim = crate::sim::world::Simulation::with_seed(0);
    let diagnostics = sim
        .install_team_ai_registry(registry, rules)
        .map_err(|diagnostics| format!("Team AI definition install refused: {diagnostics:?}"))?;
    let types = sim
        .team_script_vm()
        .registered_team_types()
        .map(|summary| {
            let definition = summary.definition;
            json!({
                "id": sim.interner().resolve(definition.id),
                "script": sim.interner().resolve(definition.script_id),
                "task_force": sim.interner().resolve(definition.task_force_id),
                "is_base_defense": definition.is_base_defense,
                "reinforce": summary.reinforce,
            })
        })
        .collect::<Vec<_>>();
    Ok(json!({
        "result_kind": "installed_team_type_inventory",
        "team_types": types,
        "trigger_inputs": registry.ai_triggers.iter().map(|trigger| json!({
            "id": trigger.id,
            "primary_team_type": trigger.primary_team_type,
            "secondary_team_type": trigger.secondary_team_type,
            "enabled": trigger.enabled,
            "multiplayer": trigger.multiplayer,
            "difficulty_enabled": trigger.difficulty_enabled,
            "source": trigger.source,
        })).collect::<Vec<_>>(),
        "diagnostics": diagnostics.iter().map(|diagnostic| format!("{diagnostic:?}")).collect::<Vec<_>>(),
        "limits": [
            "TeamTypes use the production definition installer and recruitment owner's reinforcement decision. No live teams are created or ticks advanced.",
            "Trigger inputs retain pre-resolution references, not eligibility, selection or a proof of live consumers. Rules layers do not overlay AIMD: fixed AI root plus the scenario own these definitions."
        ],
    }))
}

pub fn run(
    assets: &mut AssetManager,
    retail_dir: &Path,
    query: IniQuery<'_>,
    options: &IniOptions,
) -> Result<Value, ErrorReport> {
    run_inner(assets, retail_dir, query, options).map_err(|error| ErrorReport {
        error,
        hint: Some(match query {
            IniQuery::Accessor { .. } => "Use asset ini-get --help; typed results require the native caller's default and reader.",
            IniQuery::Type { .. } => "Use asset ini-type --help; select a rules type, map and skirmish mode explicitly.",
        }.into()),
    })
}

fn run_inner(
    assets: &mut AssetManager,
    retail_dir: &Path,
    query: IniQuery<'_>,
    options: &IniOptions,
) -> Result<Value, String> {
    options.validate(query)?;
    if options.domain.as_deref() == Some("art") {
        let IniQuery::Accessor { section, key } = query else {
            unreachable!("validated art accessor")
        };
        let art = select_ini(assets, assets.ini_roots().art())?;
        let layers = vec![layer("fixed_art", json!(art.source), Some(&art.ini), query)];
        let mut report = assemble(&art.ini, section, key, options, layers)?;
        report["ini_roots"] = json!(assets.ini_roots());
        return Ok(report);
    }
    let audio_definitions = crate::rules::audio_sources::AudioDefinitions::select(assets);
    let sources = RetailRulesSources::select(assets)?;
    // The app retains these registrations between cold Rules selection and
    // roster/map lookup. A shell-discovered YRO may supply either later input.
    // Preserve the app's tolerant failure policy and partially registered VFS.
    let neutral_archives = assets.register_neutral_archives();
    if let Err(error) = &neutral_archives {
        log::warn!("Could not register retail neutral shell archives: {error:#}");
    }
    let scenario_records = crate::map::scenario_sources::list_skirmish_scenario_records_with_assets(
        retail_dir, assets, None,
    );
    if let Err(error) = &scenario_records {
        log::warn!("Could not enumerate retail skirmish scenarios: {error:#}");
    }
    let roster = select_ini(assets, "MPModesMD.ini")?;
    let modes = crate::skirmish_modes::skirmish_modes_from_selected_ini(
        assets,
        &roster.ini,
        &roster.source.source_archive,
    )
    .map_err(|e| e.to_string())?;
    let id = options.mode_id.expect("validated mode id");
    let mode = crate::skirmish_modes::mode_by_id(&modes, id)
        .ok_or_else(|| format!("mode id {id} is absent from MPModesMD.ini"))?;
    if !mode.class.lists_in_skirmish() {
        return Err(format!(
            "mode id {id} is not in the supported noncampaign skirmish list"
        ));
    }
    let map = crate::map::source::load_map_by_name_or_path_with_assets(
        retail_dir,
        options.map.as_deref().expect("validated map"),
        assets,
    )
    .map_err(|e| e.to_string())?;
    // App match loading activates theater archives after selecting startup
    // sources and before resolving the mode override. Follow that route;
    // headless's older pre-theater mode lookup is not the claimed path.
    crate::map::theater::load_theater(assets, &map.map.header.theater).ok_or_else(|| {
        format!(
            "load theater {} for app rule-source selection",
            map.map.header.theater
        )
    })?;
    let selected_mode = if mode.override_file.trim().is_empty() {
        None
    } else {
        Some(select_ini(assets, mode.override_file.trim())?)
    };
    let layers = vec![
        layer(
            "rulesmd",
            json!(sources.rulesmd.source),
            Some(&sources.rulesmd.ini),
            query,
        ),
        layer(
            "langrule",
            sources
                .langrule
                .as_ref()
                .map_or(Value::Null, |ini| json!(ini.source)),
            sources.langrule.as_ref().map(|ini| &ini.ini),
            query,
        ),
        layer(
            "game_mode",
            selected_mode
                .as_ref()
                .map_or(Value::Null, |ini| json!(ini.source)),
            selected_mode.as_ref().map(|ini| &ini.ini),
            query,
        ),
        layer("scenario", json!(map.source), Some(&map.map.ini), query),
    ];
    // Retain provenance before moving the selected snapshots into their owner.
    let fixed_art_source = json!(sources.artmd.source);
    let fixed_sound_source = audio_definitions.sound_source().map(|source| json!(source));
    let (_, _, mut owner) = sources
        .into_startup(
            std::sync::Arc::clone(audio_definitions.sounds()),
            std::sync::Arc::clone(audio_definitions.eva()),
        )?
        .into_parts();
    let (rules, processed, _, _) = owner
        .load_scenario(
            crate::rules::process_owner::NativeScenarioRulesPrefix::NonCampaign(
                selected_mode.as_ref().map(|ini| &ini.ini),
            ),
            &map.map.ini,
        )
        .map_err(|e| e.to_string())?
        .into_parts();
    let mut report = match query {
        IniQuery::Accessor { section, key } => assemble(&processed, section, key, options, layers)?,
        IniQuery::Type { type_id } => assemble_type(&rules, &processed, type_id, layers)?,
    };
    report["mode_roster_source"] = json!(roster.source);
    report["ini_roots"] = json!(assets.ini_roots());
    report["mode"] = json!({"id": mode.id, "class": format!("{:?}", mode.class), "override_file": mode.override_file});
    report["source_selection_route"] =
        json!("app_noncampaign_startup_then_shell_registration_then_map_then_theater_then_mode");
    report["fixed_art_source"] = fixed_art_source;
    report["fixed_sound_source"] = json!(fixed_sound_source);
    if matches!(query, IniQuery::Type { .. }) {
        report["scenario"] = json!({
            "requested_map": options.map,
            "name": map.map.basic.name,
            "theater": map.map.header.theater,
        });
    }
    if matches!(query, IniQuery::Type { type_id: "*" }) {
        report["map_resources"] = resource_inputs_inventory(
            &rules,
            &map.map.basic,
            &map.map.special_flags,
            &map.map.terrain_objects,
            &map.map.actions,
        );
        report["team_ai"] =
            crate::rules::team_ai_ini::TeamAiIniRegistry::load_retail(assets, &map.map.ini, true)
                .and_then(|registry| team_ai_inventory(&registry, &rules))
                .unwrap_or_else(|error| json!({"error": error, "team_types": null}));
        report["archive_registration_error"] = neutral_archives
            .as_ref()
            .err()
            .map(|error| error.to_string())
            .into();
        report["scenario_discovery_error"] = scenario_records
            .as_ref()
            .err()
            .map(|error| error.to_string())
            .into();
        report["mode_catalog"] = modes
            .iter()
            .map(|mode| {
                let records = scenario_records.as_ref().ok().map(|records| {
                    let valid = records.iter().filter(|record| {
                        record.kind
                            == crate::map::skirmish_scenarios::SkirmishScenarioKind::ConcreteMap
                            && crate::map::skirmish_scenarios::is_valid_map(record, mode, false)
                    });
                    valid
                        .map(|record| {
                            json!({
                                "file_name": record.file_name,
                                "source_ordinal": record.source_ordinal,
                                "source": format!("{:?}", record.source),
                            })
                        })
                        .collect::<Vec<_>>()
                });
                json!({
                    "id": mode.id,
                    "class": format!("{:?}", mode.class),
                    "lists_in_skirmish": mode.class.lists_in_skirmish(),
                    "override_file": mode.override_file,
                    "map_filter": mode.map_filter,
                    "concrete_scenarios": records,
                })
            })
            .collect::<Vec<_>>()
            .into();
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::process_owner::NativeRulesProcessOwner;

    #[test]
    fn resource_inventory_keeps_map_flags_separate_and_counts_only_authored_spawners() {
        let mut owner = NativeRulesProcessOwner::from_cold_start_sources(
            IniFile::from_str(
                "[TerrainTypes]\n0=USED\n1=UNUSED\n2=TREE\n\
                [USED]\nSpawnsTiberium=yes\nIsAnimated=yes\n\
                [UNUSED]\nSpawnsTiberium=yes\n[TREE]\nSpawnsTiberium=no\n\
                [Tiberiums]\n0=Ore\n[Ore]\nGrowth=3\nGrowthPercentage=0.5\n\
                [Basic]\nTiberiumGrowthEnabled=yes\n",
            ),
            None,
            IniFile::empty(),
            std::sync::Arc::default(),
            Default::default(),
        )
        .unwrap();
        let map = IniFile::from_str(
            "[Basic]\nTiberiumGrowthEnabled=no\n\
            [SpecialFlags]\nTiberiumGrows=no\n\
            [Tiberiums]\n0=Ore\n\
            [Ore]\nGrowthPercentage=0\n\
            [Terrain]\n1001=used\n1002=USED\n1003=TREE\n1004=UNREGISTERED\n\
            [Actions]\nA=1,86,0,0,0,0,0,0,A\nB=1,7,0,0,0,0,0,0,A\n",
        );
        let (rules, _, _, _) = owner
            .load_scenario(
                crate::rules::process_owner::NativeScenarioRulesPrefix::NonCampaign(None),
                &map,
            )
            .unwrap()
            .into_parts();
        let report = resource_inputs_inventory(
            &rules,
            &crate::map::basic::parse_basic_section(&map),
            &crate::map::basic::parse_special_flags_section(&map),
            &crate::map::overlay::parse_terrain_objects(&map, 0),
            &crate::map::actions::parse_actions(&map),
        );
        assert_eq!(report["basic_tiberium_growth_enabled"], false);
        assert_eq!(report["special_flags"]["tiberium_grows"], false);
        assert_eq!(
            report["tiberium_types"][0]["growth_percentage"]["value"],
            0.0
        );
        assert_eq!(
            report["authored_terrain_spawners"],
            json!([
                {"type_id":"USED","authored_instance_count":2,"is_animated":true}
            ])
        );
        assert!(report["declared_growth_actions"]["A"].is_array());
        assert!(report["declared_growth_actions"].get("B").is_none());
        let unset = resource_inputs_inventory(
            &rules,
            &Default::default(),
            &Default::default(),
            &[],
            &Default::default(),
        );
        assert!(unset["basic_tiberium_growth_enabled"].is_null());
        assert_eq!(unset["authored_terrain_spawners"], json!([]));
    }

    #[test]
    fn team_inventory_uses_installed_reinforce_defaults_and_map_rereads() {
        use crate::rules::team_ai_ini::TeamAiIniRegistry;
        let fixed = IniFile::from_str(
            "[TeamTypes]\n0=R\n1=DEFAULT\n[R]\nScript=S\nTaskForce=F\nReinforce=yes\n\
             [DEFAULT]\nScript=S\nTaskForce=F\n[ScriptTypes]\n0=S\n[S]\n0=2,0\n\
             [TaskForces]\n0=F\n[F]\nGroup=-1\n",
        );
        let rules = RuleSet::from_ini(&IniFile::empty()).unwrap();
        let registry = TeamAiIniRegistry::from_sources(&fixed, &IniFile::empty(), true);
        let report = team_ai_inventory(&registry, &rules).unwrap();
        assert_eq!(report["team_types"][0]["id"], "R");
        assert_eq!(report["team_types"][0]["reinforce"], true);
        assert_eq!(report["team_types"][1]["reinforce"], false);
        let registry = TeamAiIniRegistry::from_sources(
            &fixed,
            &IniFile::from_str("[TeamTypes]\n0=R\n[R]\nReinforce=no\n"),
            true,
        );
        let report = team_ai_inventory(&registry, &rules).unwrap();
        assert_eq!(report["team_types"].as_array().unwrap().len(), 2);
        assert_eq!(report["team_types"][0]["reinforce"], false);
        assert_eq!(report["team_types"][0]["script"], "S");
        assert_eq!(report["team_types"][0]["task_force"], "F");
    }

    fn options(reader: &str, default: &str) -> IniOptions {
        IniOptions {
            domain: Some("art".into()),
            reader: Some(reader.into()),
            default: (reader != "raw").then(|| default.into()),
            ..Default::default()
        }
    }

    #[test]
    fn ini_roots_art_probe_reads_selected_mo_source_and_reports_choice() {
        use crate::util::ini_roots::{IniProfile, IniRootConfig};
        let dir = crate::map::source::test_support::TestDirectory::new("ini-roots-art-tool");
        dir.write("rulesmo.ini", b"[General]\nBuildSpeed=.7\n");
        dir.write("artmo.ini", b"[TEST]\nVoxel=yes\n");
        dir.write("aimo.ini", b"[Marker]\nValue=MO\n");
        dir.write("artmd.ini", b"[TEST]\nVoxel=no\n");
        let mut assets = AssetManager::from_loose_root_for_test(dir.path())
            .with_ini_roots(&IniRootConfig {
                profile: IniProfile::Mo,
                ..Default::default()
            })
            .unwrap();
        let report = run(
            &mut assets,
            dir.path(),
            IniQuery::Accessor {
                section: "TEST",
                key: "Voxel",
            },
            &options("bool", "false"),
        )
        .unwrap();
        assert_eq!(report["ini_roots"]["art"], "artmo.ini");
        assert_eq!(
            report["authored_layers"][0]["source"]["logical_name"],
            "artmo.ini"
        );
        assert_eq!(report["accessor_result"], serde_json::json!(true));
    }

    #[test]
    fn absent_sections_still_use_native_string_default_capacity_and_trim() {
        let mut query = options("string", "  default  ");
        query.capacity = Some(6);
        assert_eq!(
            accessor(&IniFile::empty(), "missing", "key", &query).unwrap(),
            "def"
        );
    }

    #[test]
    fn exact_case_and_omitted_empty_values_remain_visible() {
        let ini = IniFile::from_str("[Case]\nKey=42\nEmpty=\n");
        assert_eq!(presence(&ini, "Case", "key")["key_present"], false);
        assert_eq!(presence(&ini, "Case", "Key")["raw_value"], "42");
        assert_eq!(presence(&ini, "Case", "Empty")["key_present"], false);
        assert_eq!(
            accessor(&ini, "Case", "Empty", &options("int", "73")).unwrap(),
            73
        );
    }

    #[test]
    fn nonfinite_double_keeps_bits_in_valid_json() {
        let result = accessor(
            &IniFile::from_str("[S]\nK=1e39%\n"),
            "S",
            "K",
            &options("double", "0"),
        )
        .unwrap();
        assert_eq!(result["value"], Value::Null);
        assert_eq!(result["binary64_bits"], "0x7ff0000000000000");
        serde_json::from_str::<Value>(&serde_json::to_string(&result).unwrap()).unwrap();
    }

    #[test]
    fn accessor_keeps_layered_default_retention() {
        let mut owner = NativeRulesProcessOwner::from_cold_start_sources(
            IniFile::from_str("[General]\nTreeStrength=42\n"),
            None,
            IniFile::empty(),
            std::sync::Arc::default(),
            Default::default(),
        )
        .unwrap();
        let (_, processed, _, _) = owner
            .load_scenario(
                crate::rules::process_owner::NativeScenarioRulesPrefix::NonCampaign(None),
                &IniFile::from_str("[General]\nTreeStrength=$xyz\n"),
            )
            .unwrap()
            .into_parts();
        assert_eq!(
            accessor(&processed, "General", "TreeStrength", &options("int", "7")).unwrap(),
            42
        );
        assert_eq!(
            presence(&processed, "General", "TreeStrength")["raw_value"],
            "$xyz"
        );
    }

    #[test]
    fn raw_weapon_speed_is_not_claimed_as_final_postpass_field() {
        // Same production reader/postpass as rules::weapon_speed_tests; no
        // replacement implementation of the projectile-dependent calculation.
        let root = IniFile::from_str(
            "[VehicleTypes]\n0=UNIT\n[UNIT]\nPrimary=GUN\n[GUN]\nSpeed=40\nRange=5\nProjectile=SHOT\n[SHOT]\nROT=0\n",
        );
        let authored = presence(&root, "GUN", "Speed");
        let mut owner = NativeRulesProcessOwner::from_cold_start_sources(
            root,
            None,
            IniFile::empty(),
            std::sync::Arc::default(),
            Default::default(),
        )
        .unwrap();
        let (rules, processed, _, _) = owner
            .load_scenario(
                crate::rules::process_owner::NativeScenarioRulesPrefix::NonCampaign(None),
                &IniFile::empty(),
            )
            .unwrap()
            .into_parts();
        let report = assemble(&processed, "GUN", "Speed", &options("raw", ""), vec![]).unwrap();
        assert_eq!(authored["raw_value"], "40");
        assert_eq!(report["accessor_result"], "40");
        assert_ne!(rules.weapon("GUN").unwrap().speed, 40);
        assert_eq!(report["result_kind"], "production_ini_accessor");
        assert!(
            report["limits"][0]
                .as_str()
                .unwrap()
                .contains("not a final gameplay field")
        );
    }

    #[test]
    fn final_type_report_uses_layered_object_fields_and_full_jumpjet_block() {
        let root = IniFile::from_str(
            "[VehicleTypes]\n0=UNIT\n[UNIT]\nName=Fixture unit\n\
             Locomotor={92612C46-F71F-11D1-AC9F-006008055BB5}\n\
             BalloonHover=yes\nIsSimpleDeployer=yes\nDeployToLand=no\nHoverAttack=yes\n\
             NoShadow=yes\nConsideredAircraft=yes\nShadowIndex=7\n\
             JumpjetTurnRate=7\nJumpjetSpeed=27\nJumpjetClimb=3.125\nJumpjetCrash=9.5\n\
             JumpjetHeight=600\nJumpjetAccel=1.25\nJumpjetWobbles=0.25\n\
             JumpjetNoWobbles=yes\nJumpjetDeviation=42\n",
        );
        let mode = IniFile::from_str(
            "[UNIT]\nBalloonHover=no\nDeployToLand=yes\nJumpjetHeight=650\nNoShadow=no\n",
        );
        let map = IniFile::from_str(
            "[UNIT]\nIsSimpleDeployer=no\nJumpjetClimb=6.5\nJumpjetSpeed=29.75\n\
             JumpjetWobbles=-0.0\nJumpJetTurnRate=99\nJumpJetAccel=99\n",
        );
        let mut owner = NativeRulesProcessOwner::from_cold_start_sources(
            root,
            None,
            IniFile::empty(),
            std::sync::Arc::default(),
            Default::default(),
        )
        .unwrap();
        let (rules, processed, _, _) = owner
            .load_scenario(
                crate::rules::process_owner::NativeScenarioRulesPrefix::NonCampaign(Some(&mode)),
                &map,
            )
            .unwrap()
            .into_parts();
        let source = json!({"logical_name": "fixture.map", "source_sha256": "fixture identity"});
        let report = assemble_type(
            &rules,
            &processed,
            "unit",
            vec![layer(
                "scenario",
                source.clone(),
                Some(&map),
                IniQuery::Type { type_id: "unit" },
            )],
        )
        .unwrap();
        assert_eq!(report["result_kind"], "final_object_type");
        assert_eq!(report["requested_type"], "unit");
        assert_eq!(report["source_layers"][0]["source"], source);
        assert!(report["source_layers"][0].get("authored").is_none());
        let object = &report["final_type"];
        assert_eq!(object["id"], "UNIT");
        assert_eq!(object["category"], "Vehicle");
        assert_eq!(object["locomotor"], "Jumpjet");
        assert_eq!(object["no_shadow"], false);
        assert_eq!(object["considered_aircraft"], true);
        // ShadowIndex belongs to fixed ART, so the rules token cannot replace
        // the native constructor default when ART supplies no value.
        assert_eq!(object["voxel_shadow_index"], 0);
        assert_eq!(object["balloon_hover"], false);
        assert_eq!(object["is_simple_deployer"], false);
        assert_eq!(object["deploy_to_land"], true);
        assert_eq!(object["hover_attack"], true);
        assert_eq!(object["jumpjet"], false);
        let params = &object["jumpjet_params"];
        assert_eq!(params.as_object().unwrap().len(), 9);
        assert_eq!(params["turn_rate"], 7);
        assert_eq!(params["speed"]["value"], 29.0);
        assert_eq!(params["speed"]["raw_bits"], 29 * 65536);
        assert_eq!(params["speed"]["raw_bits_hex"], "0x001d0000");
        assert_eq!(params["speed"]["fractional_bits"], 16);
        assert_eq!(params["climb"]["value"], 6.5);
        assert_eq!(params["climb"]["binary32_bits"], "0x40d00000");
        assert_eq!(params["crash"]["value"], 9.5);
        assert_eq!(params["height"], 650);
        assert_eq!(params["accel"]["value"], 1.25);
        assert_eq!(params["wobbles"]["binary32_bits"], "0x80000000");
        assert_eq!(params["no_wobbles"], true);
        assert_eq!(params["deviation"], 42);
        // The wrong-case authored tokens remain visible to ini-get, but are
        // not substituted for the values returned by the type reader.
        assert_eq!(
            presence(&processed, "UNIT", "JumpJetAccel")["raw_value"],
            "99"
        );
        serde_json::from_str::<Value>(&serde_json::to_string(&report).unwrap()).unwrap();
    }

    #[test]
    fn final_type_report_rejects_an_ini_section_without_a_registered_type() {
        let root = IniFile::from_str(
            "[VehicleTypes]\n0=UNIT\n[UNIT]\nStrength=100\n[UNREGISTERED]\nBalloonHover=yes\n",
        );
        let mut owner = NativeRulesProcessOwner::from_cold_start_sources(
            root,
            None,
            IniFile::empty(),
            std::sync::Arc::default(),
            Default::default(),
        )
        .unwrap();
        let (rules, processed, _, _) = owner
            .load_scenario(
                crate::rules::process_owner::NativeScenarioRulesPrefix::NonCampaign(None),
                &IniFile::empty(),
            )
            .unwrap()
            .into_parts();
        assert_eq!(
            presence(&processed, "UNREGISTERED", "BalloonHover")["raw_value"],
            "yes"
        );
        assert!(
            assemble_type(&rules, &processed, "UNREGISTERED", vec![])
                .unwrap_err()
                .contains("absent from the final scenario RuleSet")
        );
        let report = assemble_type(&rules, &processed, "UNIT", vec![]).unwrap();
        assert_eq!(report["final_type"]["balloon_hover"], false);
        assert_eq!(report["final_type"]["jumpjet_params"]["turn_rate"], 4);
        assert_eq!(
            report["final_type"]["jumpjet_params"]["speed"]["value"],
            14.0
        );
        assert_eq!(
            report["final_type"]["jumpjet_params"]["accel"]["value"],
            2.0
        );
    }

    #[test]
    fn final_type_float_report_keeps_nonfinite_payload_bits_in_valid_json() {
        let result = float32_result(f32::from_bits(0x7fc01234));
        assert_eq!(result["value"], Value::Null);
        assert_eq!(result["binary32_bits"], "0x7fc01234");
        serde_json::from_str::<Value>(&serde_json::to_string(&result).unwrap()).unwrap();
    }

    #[test]
    fn final_type_inventory_uses_registered_types_and_retained_layered_inputs() {
        let mut owner = NativeRulesProcessOwner::from_cold_start_sources(
            IniFile::from_str(
                "[BuildingTypes]\n0=PLANT\n[VehicleTypes]\n0=UNIT\n1=*\n[*]\nBalloonHover=yes\n\
                 [PLANT]\nNaturalParticleLocation=1,2,3\n\
                 [UNIT]\nTrueConversion=no\n[UNREGISTERED]\nTrueConversion=yes\n",
            ),
            None,
            IniFile::empty(),
            std::sync::Arc::default(),
            Default::default(),
        )
        .unwrap();
        let (rules, processed, _, _) = owner
            .load_scenario(
                crate::rules::process_owner::NativeScenarioRulesPrefix::NonCampaign(None),
                &IniFile::from_str("[UNIT]\nTrueConversion=yes\ntrueconversion=no\n"),
            )
            .unwrap()
            .into_parts();
        let report = assemble_type(&rules, &processed, "*", vec![]).unwrap();
        let types = report["final_types"].as_array().unwrap();
        assert_eq!(types.len(), rules.all_objects().count());
        assert!(types.iter().all(|object| object["id"] != "UNREGISTERED"));
        assert!(
            types
                .iter()
                .any(|object| object["id"] == "*" && object["balloon_hover"] == true)
        );
        let unit = types.iter().find(|object| object["id"] == "UNIT").unwrap();
        assert_eq!(unit["retained_ini"]["TrueConversion"], "yes");
        assert_eq!(unit["retained_ini"]["trueconversion"], "no");
        let plant = types.iter().find(|object| object["id"] == "PLANT").unwrap();
        assert_eq!(plant["natural_particle_location"], json!([1, 2, 3]));
        assert_eq!(plant["natural_particle_system"], Value::Null);
    }

    #[test]
    #[ignore = "requires retail YR installation containing RiverRam.yro; run explicitly"]
    fn retail_yro_scenario_inspection_uses_retained_discovery() {
        let retail = std::path::PathBuf::from(std::env::var("RA2_DIR").expect("set RA2_DIR"));
        let mut assets =
            crate::asset_tools::root::open_manager(&retail, false).expect("retail manager");
        let query = IniOptions {
            domain: Some("rules".into()),
            reader: Some("speed".into()),
            default: Some("0".into()),
            map: Some("RiverRam.MAP".into()),
            mode_id: Some(1),
            ..Default::default()
        };
        let report = run_inner(
            &mut assets,
            &retail,
            IniQuery::Accessor {
                section: "HoverMissile",
                key: "Speed",
            },
            &query,
        )
        .expect("inspect map registered by production scenario discovery");
        assert_eq!(report["accessor_result"], 102);
        let source = &report["authored_layers"][3]["source"];
        assert_eq!(source["kind"], "mix");
        assert!(
            source["source_archive"]
                .as_str()
                .unwrap()
                .eq_ignore_ascii_case("RiverRam.yro")
        );
        let selected = assets
            .resolve_ref("RiverRam.MAP")
            .expect("retained scenario archive");
        assert_eq!(
            source["source_sha256"],
            crate::util::sha256::sha256_hex(selected.bytes)
        );
    }
}
