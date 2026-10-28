//! Headless retail-scenario loading for cross-engine parity runs.
//!
//! Loads a real retail map with production rules, art, theater and terrain, then
//! constructs a `Simulation` from an explicit seed — no GPU, no window, no atlases.
//! Depends on `assets`/`rules`/`map`/`sim` only; nothing here may reach into `render`,
//! `ui`, `sidebar`, `audio` or `net`.
//!
//! **Scope.** Construction goes through the same GPU-free funnel the app uses
//! (`sim::runtime::finalize_and_populate_staged_authored_scenario`, on a `Simulation`
//! staged before terrain Fill): map-roster houses are created before objects,
//! terrain objects before map entities, and map-placed units/structures spawn with
//! terrain-attached animations. A Battle launch session ([`battle_session`]) creates
//! the player houses, the local one and any computer opponents, and places them on
//! their starts. What a headless scenario still lacks versus an app launch is
//! atlas-derived voxel animation frame counts (a GPU concern).
//!
//! The seed contract mirrors the original engine: one 32-bit word seeds the scenario and
//! main streams identically, fixed before any setup-phase draw.

#[cfg(any(test, feature = "test-support"))]
pub mod radiation_test_evidence;

#[cfg(any(test, feature = "test-support"))]
pub mod battle_test_evidence;

use std::path::Path;

use crate::assets::asset_manager::{AssetManager, MediaArchiveMode};
use crate::map::map_file::MapFile;
use crate::map::resolved_terrain::ResolvedTerrainGrid;
use crate::map::theater;
use crate::map::tile_variant_selector::TileVariantSelectorCache;
use crate::map::waypoints;
use crate::sim::intern::InternedId;
use crate::sim::scenario_bootstrap::ScenarioBootstrapRng;
use crate::sim::scenario_session::ScenarioDescriptor;
use crate::sim::world::Simulation;
use crate::skirmish_launch::{AiDifficulty, LaunchCountry, SkirmishLaunchSession};

/// A Battle session on `selected_map_file`: the local player against one
/// computer house for each of `opponents` (its country and difficulty), every
/// house on a start the launch draws. The lobby allows seven opponents.
pub fn battle_session(
    selected_map_file: &str,
    opponents: &[(LaunchCountry, AiDifficulty)],
) -> SkirmishLaunchSession {
    use crate::skirmish_launch::{
        LaunchStartPosition, LaunchTeam, PreFillHouseRoster, SkirmishAiSlot, SkirmishLaunchMode,
        SkirmishLaunchOptions, SkirmishLocalSlot,
    };

    SkirmishLaunchSession {
        mode: SkirmishLaunchMode {
            class: crate::skirmish_modes::MpModeClass::Battle,
            id: 1,
            ui_name_key: "GUI:Battle".to_string(),
            tooltip_key: "STT:ModeBattle".to_string(),
            override_file: "MPBattleMD.ini".to_string(),
            map_filter: "standard".to_string(),
            random_maps_allowed: true,
            allies_allowed: true,
            must_ally: false,
        },
        selected_map_file: Some(selected_map_file.to_string()),
        humans: vec![crate::skirmish_launch::SkirmishHumanSlot {
            raw_slot_index: Some(0),
            name: "Player".to_string(),
            slot: SkirmishLocalSlot {
                country: LaunchCountry::America,
                country_random: false,
                color_index: 0,
                color_random: false,
                start_position: LaunchStartPosition::Auto,
                team: LaunchTeam::None,
            },
            role: crate::skirmish_launch::LaunchHumanRole::from_observer(false),
        }],
        local_player_index: 0,
        ai_player_count: opponents.len() as i32,
        opponents: opponents
            .iter()
            .enumerate()
            .map(|(slot, &(country, difficulty))| SkirmishAiSlot {
                raw_slot_index: (slot + 1) as u8,
                country,
                country_random: false,
                color_index: slot as u8 + 1,
                color_random: false,
                start_position: LaunchStartPosition::Auto,
                team: LaunchTeam::None,
                difficulty,
            })
            .collect(),
        pre_fill_house_roster: PreFillHouseRoster::from_compact_skirmish(opponents.len()),
        options: SkirmishLaunchOptions::default(),
    }
}

/// A loaded scenario plus the per-tick inputs `advance_tick` needs.
pub struct HeadlessScenario {
    /// The runtime owner (F09): simulation plus its bound immutable
    /// resources - no independently swappable execution inputs remain.
    pub runtime: crate::sim::runtime::SimRuntime,
    pub map: MapFile,
}

impl HeadlessScenario {
    /// Read access for digest/tooling callers.
    pub fn sim(&self) -> &Simulation {
        &self.runtime.simulation
    }

    /// Add a human player's house of `country` after the load, built by the
    /// launch's own house constructor with the session's credits and tech
    /// level. The launch gave it no start, starting force or alliance, so it
    /// is at war with every house.
    pub fn add_human_house(&mut self, owner_name: &str, country: LaunchCountry) -> InternedId {
        let slot = crate::sim::scenario_bootstrap::NormalizedSkirmishSlot {
            owner_name: owner_name.to_string(),
            country,
            color_index: 0,
            start_position: crate::skirmish_launch::LaunchStartPosition::Auto,
            team: crate::skirmish_launch::LaunchTeam::None,
            is_human: true,
            observer: false,
            source_human_index: None,
            handle: None,
            difficulty: crate::sim::house_state::HouseDifficulty::Normal,
        };
        let runtime = &mut self.runtime;
        crate::sim::scenario_bootstrap::populate_launch_houses(
            &mut runtime.simulation,
            std::slice::from_ref(&slot),
            &runtime.resources.rules,
        );
        runtime
            .simulation
            .interner
            .get(owner_name)
            .expect("the house constructor interns its name")
    }
}

/// Native logic-frame cadence (66 ms). NOTE: this does NOT match the
/// client, which steps the sim at `app::types::SIM_TICK_MS` = 22 ms — a
/// recorded 3x tooling divergence; headless digests are not tick-comparable
/// to client runs until it is resolved.
pub const SIM_TICK_MS: u32 = 1000 / crate::util::fixed_math::RA2_LOGIC_FRAMES_PER_SECOND;

/// Load `map_file_name` from the retail install at `retail_dir` with a pinned seed.
///
/// `map_file_name` uses the game's shared loose/MIX source resolver (e.g.
/// `"Dustbowl.mmx"` or `"XMP03T4.MAP"`).
pub fn load(retail_dir: &Path, map_file_name: &str, seed: u32) -> Result<HeadlessScenario, String> {
    load_session(
        retail_dir,
        map_file_name,
        seed,
        battle_session(map_file_name, &[]),
    )
}

/// Load `map_file_name` as `session` launches it (see [`battle_session`]).
pub fn load_session(
    retail_dir: &Path,
    map_file_name: &str,
    seed: u32,
    session: SkirmishLaunchSession,
) -> Result<HeadlessScenario, String> {
    let launch = crate::sim::scenario_bootstrap::MatchLaunchDescriptor::from_resolved(session)
        .map_err(|error| format!("resolve Battle launch: {error}"))?;
    load_with_launch(retail_dir, map_file_name, seed, launch)
}

/// Run the same construction and bound-resource handoff with an explicit
/// resolved launch. The quickplay regression uses its real app descriptor here
/// so house creation and starting forces cannot drift from the tested session.
pub fn load_with_launch(
    retail_dir: &Path,
    map_file_name: &str,
    seed: u32,
    launch: crate::sim::scenario_bootstrap::MatchLaunchDescriptor,
) -> Result<HeadlessScenario, String> {
    load_with_launch_and_ini(retail_dir, map_file_name, seed, launch, &Default::default())
}

/// Explicit host policy for non-YR callers; no hidden cwd/config reread.
/// Rules, fixed Art and AI all consume the one resolved provider choice.
pub fn load_with_launch_and_ini(
    retail_dir: &Path,
    map_file_name: &str,
    seed: u32,
    launch: crate::sim::scenario_bootstrap::MatchLaunchDescriptor,
    ini: &crate::util::ini_roots::IniRootConfig,
) -> Result<HeadlessScenario, String> {
    let mut assets = AssetManager::new(retail_dir, MediaArchiveMode::STOCK_DIGITAL)
        .and_then(|assets| assets.with_ini_roots(ini))
        .map_err(|error| {
            format!(
                "open retail MIX archives in {}: {error}",
                retail_dir.display()
            )
        })?;
    let mut map = crate::map::source::load_map_by_name_or_path_with_assets(
        retail_dir,
        map_file_name,
        &assets,
    )
    .map_err(|error| format!("load map {map_file_name}: {error}"))?
    .map;
    // Preserve the process-owned cold Rules registry before theater archive
    // priority changes. The same owner then performs the active noncampaign
    // reset/rebuild and transfers its move-only native-ID receipt.
    let audio_definitions = crate::rules::audio_sources::AudioDefinitions::select(&assets);
    let (_, _, mut native_rules_owner) = crate::rules::retail_sources::load_startup_rules(
        &assets,
        std::sync::Arc::clone(audio_definitions.sounds()),
        std::sync::Arc::clone(audio_definitions.eva()),
    )
    .ok_or_else(|| "load native startup rules".to_string())?
    .into_parts();
    let scenario_prefix_plan =
        crate::sim::scenario_bootstrap::prepare_stock_offline_scenario_prefix_plan(
            &launch,
            &map,
            &map.waypoints,
            seed,
        )
        .map_err(|error| format!("prepare stock-offline scenario prefix: {error}"))?;
    let mode_override = crate::rules::retail_sources::select_mode_ini(
        &assets,
        Some(&launch.session().mode.override_file),
    )?;
    let (mut rules, _, art_ini, native_rules_receipt) = native_rules_owner
        .load_scenario(
            crate::rules::process_owner::NativeScenarioRulesPrefix::NonCampaign(
                mode_override.as_ref(),
            ),
            &map.ini,
        )
        .map_err(|error| format!("load native noncampaign rules: {error}"))?
        .into_parts();
    let bound_scenario_prefix = scenario_prefix_plan
        .bind_native_rules_receipt(native_rules_receipt, &launch, &rules)
        .map_err(|error| format!("admit rebuilt launch Countries: {error}"))?;
    let team_ai_registry =
        crate::rules::team_ai_ini::TeamAiIniRegistry::load_retail(&assets, &map.ini, true)?;
    let theater = theater::load_theater(&mut assets, &map.header.theater)
        .ok_or_else(|| format!("load theater {}", map.header.theater))?;
    rules.install_art_data(crate::rules::art_data::ArtRegistry::from_ini(&art_ini));
    let infantry_sequences =
        crate::rules::infantry_sequence::parse_infantry_sequence_registry(&art_ini);
    let overlay_registry = rules.overlay_registry().clone();
    let house_roster = crate::map::houses::parse_house_roster(
        &map.ini,
        rules.color_schemes.as_slice(),
        Some(&rules),
    );
    let lighting_profiles = crate::map::lighting::parse_lighting_profiles(&map.ini);
    let native_start_bounds =
        crate::sim::scenario_bootstrap::NativeStartBounds::from_map_header(&map.header)
            .ok_or_else(|| "map Size does not produce a valid fresh cell array".to_string())?;
    let scenario_cell_extent = native_start_bounds
        .min_rx
        .checked_add(native_start_bounds.width)
        .ok_or_else(|| "fresh cell-array extent overflow".to_string())?;
    let descriptor = ScenarioDescriptor {
        seed,
        map_name: map_file_name.to_string(),
        theater: map.header.theater.clone(),
        game_mode_nonzero: true,
        no_damage: false,
        // Non-campaign: the session staging word's Harvester Truce byte
        // (`0x005E7495..0x005E74D4`) replaces the map's bit at
        // `0x00687C16..0x00687C29`.
        harvester_immune: launch.session().options.harvester_truce,
        free_radar: map.basic.free_radar.unwrap_or(false),
        ignore_global_ai_triggers: map.basic.ignore_global_ai_triggers.unwrap_or(false),
        timer_inherit: map.basic.timer_inherit.unwrap_or(false),
        carry_over_money: map.basic.carry_over_money,
        carry_over_cap: map.basic.carry_over_cap,
        train_crate: map.basic.train_crate.unwrap_or(false),
        truck_crate: map.basic.truck_crate.unwrap_or(false),
        ice_growth_enabled: map.basic.ice_growth_enabled,
        vein_growth_enabled: map.basic.vein_growth_enabled,
        // Skirmish start forces `TiberiumGrows|TiberiumSpreads` (`OR 0xC0`
        // at `0x005E74CD`), copied into the scenario at `0x00687C23`.
        tiberium_grows_flag: true,
        tiberium_spreads_flag: true,
        map_width: scenario_cell_extent,
        map_height: scenario_cell_extent,
        mp_start_waypoints: waypoints::multiplayer_start_waypoints(
            bound_scenario_prefix
                .projection()
                .active_scenario_waypoints(),
        )
        .into_iter()
        .map(|wp| (wp.index, (wp.rx, wp.ry)))
        .collect(),
        pixel_conversion_bounds: Default::default(),
        lighting: crate::sim::scenario_session::ScenarioLightingState::from_map(&lighting_profiles),
    };
    let bootstrap_rng = ScenarioBootstrapRng::new(seed);
    let (mut sim, scenario_prefix_projection) = bootstrap_rng
        .into_stock_offline_staged_simulation(&descriptor, bound_scenario_prefix)
        .map_err(|error| format!("stage stock-offline Simulation: {error}"))?;
    let shared_cell_dummy = crate::map::resolved_terrain::SharedCellDummy::fresh();
    shared_cell_dummy.reconstruct_for_map_resize();
    sim.bind_shared_cell_dummy(shared_cell_dummy.clone());
    sim.session.install_waypoints(&map.waypoints);
    sim.initialize_trigger_definitions(
        &map,
        &team_ai_registry,
        Some(audio_definitions.eva()),
        Some(audio_definitions.sounds().as_ref()),
    );
    let (mut scenario_fill_rng, variant_main_rng) = sim.terrain_load_draws();
    let mut scenario_fill_ranged =
        |low, high| scenario_fill_rng.next_range_u32_inclusive(low, high);
    let mut variant_draw = move || variant_main_rng.next_u32();
    let mut variant_selector_cache = TileVariantSelectorCache::default();
    let mut variant_selector = variant_selector_cache.begin_load(&mut variant_draw);
    let terrain_fill =
        ResolvedTerrainGrid::build_pending_authored_with_variant_selector_and_shared_dummy(
            &map,
            Some(&theater),
            Some(&assets),
            Some(&rules.terrain_rules),
            Some(&overlay_registry),
            true,
            rules.general.cliff_back_impassability,
            &mut scenario_fill_ranged,
            &mut variant_selector,
            shared_cell_dummy,
        );
    drop(variant_draw);

    // Pending authored Fill has no eager Tile##Anim list. Bind damage-fire
    // roots now and let the authored host bind each actually reached map Anim
    // synchronously before its native constructor spends an ID.
    let mut scheduler_roots =
        crate::map::resolved_terrain::scheduler_anim_roots(&rules, &overlay_registry, &[]);
    scheduler_roots.extend(crate::map::actions::trigger_anim_class_roots(
        &rules,
        &map.actions,
    ));
    rules
        .bind_scheduler_anim_assets(
            &scheduler_roots,
            &assets,
            theater.extension,
            &map.header.theater,
        )
        .map_err(|error| format!("bind authoritative animation assets: {error}"))?;
    // Every other AnimClass producer and the building animations: tolerant pass,
    // after the strict one, which rewrites the scheduler-owned set wholesale.
    let tolerant_roots = crate::rules::effect_asset_catalog::tolerant_anim_class_roots(&rules);
    let unbound_explosion_roots = rules.bind_anim_class_assets(
        &tolerant_roots,
        &assets,
        theater.extension,
        &map.header.theater,
    );
    crate::rules::effect_asset_catalog::log_unbound_combat_explosion_roots(unbound_explosion_roots);
    let (populated_smudge_dims, fallback_smudge_dims) =
        rules.populate_anim_frame_dims(&assets, theater.extension, &map.header.theater);
    log::info!(
        "Anim frame dims: {} populated, {} fallback (defaults to 30x30)",
        populated_smudge_dims,
        fallback_smudge_dims,
    );
    rules.bind_effect_assets(&assets, theater.extension, &map.header.theater);
    rules.bind_building_buildup_assets(&assets, &map.header.theater);
    rules.bind_building_rubble_assets(&assets, theater.extension, &map.header.theater);
    rules.bind_terrain_assets(
        &assets,
        theater.extension,
        crate::rules::terrain_asset_catalog::TerrainAssetLoadPhase::Fresh,
    );
    let _alpha_image_bindings =
        crate::rules::alpha_image_assets::bind(&rules, &overlay_registry, &assets);
    rules.bind_animation_sequences(&infantry_sequences);
    let overlay_shp_ids = crate::rules::effect_asset_catalog::resolved_overlay_shp_ids(
        &overlay_registry,
        &assets,
        theater.extension,
        &map.header.theater,
    );
    sim.construct_native_map_tubes(&map.ini)
        .map_err(|error| format!("construct native [Tubes]: {error}"))?;
    let bridge_mode = crate::map::basic::BridgeDestroyabilityMode::SkirmishOrMultiplayer {
        bridge_destruction: true,
    };
    let signed_new_ini_format = map.basic.new_ini_format.unwrap_or(0);
    let cliff_back_impassability = rules.general.cliff_back_impassability;
    let output = crate::sim::runtime::finalize_and_populate_staged_authored_scenario(
        &mut sim,
        &mut map,
        terrain_fill,
        &theater,
        &assets,
        &mut rules,
        &mut native_rules_owner,
        &overlay_registry,
        &overlay_shp_ids,
        signed_new_ini_format,
        true,
        cliff_back_impassability,
        theater.extension,
        &descriptor.theater,
        bridge_mode,
        &descriptor,
        |sim, rules| {
            crate::sim::scenario_bootstrap::initialize_skirmish_launch_houses(
                sim,
                &house_roster,
                rules,
                &launch,
            );
        },
    )
    .map_err(|error| format!("finalize authored headless load: {error}"))?;
    let resolved_terrain = output.resolved_terrain;
    let overlay_grid = output.overlay_grid;
    // Match the ordinary app's pre-launch binding: production options need
    // identities for unspawned types too, and snapshots carry this interner.
    sim.intern_rule_type_ids(&rules);
    sim.resolve_type_handles(&rules);
    sim.install_team_ai_registry(&team_ai_registry, &rules)
        .map_err(|refused| format!("active YR aimd.ini failed RuleSet resolution: {refused:?}"))?;
    let _launch_result = crate::sim::scenario_bootstrap::
        apply_pre_fill_scenario_prefix_launch_session_with_overlay_registry(
            &mut sim,
            &map,
            &house_roster,
            &rules,
            &resolved_terrain,
            &launch,
            &overlay_registry,
            &scenario_prefix_projection,
        );
    let post_map = crate::sim::runtime::finalize_constructed_scenario(
        &mut sim,
        &map,
        &rules,
        &overlay_registry,
        overlay_grid,
        &house_roster,
        Some(&launch),
    );
    if !post_map.navigation_published {
        return Err("publish headless post-map navigation".to_string());
    }
    if sim.path_grid().is_none() {
        return Err("headless post-map navigation is unavailable".to_string());
    }
    sim.session.publish_started();
    Ok(HeadlessScenario {
        runtime: crate::sim::runtime::SimRuntime {
            simulation: sim,
            resources: crate::sim::runtime::SimResources {
                overlay_registry,
                terrain_template: None,
                rules,
            },
        },
        map,
    })
}

impl HeadlessScenario {
    /// Advance one committed simulation frame with no player commands,
    /// through the same bound-resource runtime transaction the app uses.
    pub fn tick(&mut self) {
        let _ = self
            .runtime
            .advance_frame(
                &[],
                SIM_TICK_MS,
                crate::sim::world::TickLane::Ordinary,
                crate::sim::world::FrameEffects::default(),
            )
            .expect("simulation frame failed; prior world mutations remain");
    }
}

#[cfg(test)]
mod retail_construction_tests {
    #[cfg(feature = "npatch")]
    use super::battle_test_evidence::{
        aymod_grappled_ship_at_shore, visit_aymod_dolphin_rescue_after_load,
    };
    use super::battle_test_evidence::{
        cells_nearest, house_of, spawn_on_free_cells, tick_with_due_commands,
    };
    use super::*;
    use crate::sim::rng::SimRng;

    /// A synthetic map through the load in production order, with production
    /// functions only: stage the one `Simulation`, let terrain Fill draw from
    /// it, then populate it. No theater, assets or rules are needed, which the
    /// retail funnel (`load`) cannot do without.
    ///
    /// Active YR `MapClass::Clear @ 0x00565B00` clears the fixed cell table, then
    /// `MapClass::Resize @ 0x00565C10` allocates the complete Size diamond before
    /// IsoMapPack records overwrite it (allocation loop `0x0056639E..0x00566451`).
    fn stage_fill_populate<F>(
        map: &MapFile,
        theater_name: &str,
        rules: Option<&crate::rules::ruleset::RuleSet>,
        bridge_destroyability_mode: crate::map::basic::BridgeDestroyabilityMode,
        descriptor: ScenarioDescriptor,
        initialize_houses_before_objects: F,
    ) -> (Simulation, ResolvedTerrainGrid)
    where
        F: FnOnce(&mut Simulation),
    {
        // Native Resize constructs a square cell-array extent of SizeW+SizeH;
        // the descriptor carries it before Fill exists, as the app load does.
        let bounds =
            crate::sim::scenario_bootstrap::NativeStartBounds::from_map_header(&map.header)
                .expect("map Size produces a fresh cell array");
        let extent = bounds.min_rx + bounds.width;
        let descriptor = ScenarioDescriptor {
            map_width: extent,
            map_height: extent,
            ..descriptor
        };
        let mut sim = ScenarioBootstrapRng::new(descriptor.seed).into_simulation(&descriptor);
        // This synthetic fixture omits the production native Rules prefix.
        sim.native_unique_ids =
            Some(crate::sim::native_identity::NativeUniqueIdCursor::for_synthetic_simulation());
        let resolved = {
            let (mut scenario_fill_rng, variant_main_rng) = sim.terrain_load_draws();
            let mut scenario_fill_ranged =
                |low, high| scenario_fill_rng.next_range_u32_inclusive(low, high);
            let mut variant_draw = move || variant_main_rng.next_u32();
            let mut variant_selector_cache = TileVariantSelectorCache::default();
            let mut variant_selector = variant_selector_cache.begin_load(&mut variant_draw);
            let shared_cell_dummy = crate::map::resolved_terrain::SharedCellDummy::fresh();
            shared_cell_dummy.reconstruct_for_map_resize();
            ResolvedTerrainGrid::build_with_variant_selector_and_shared_dummy(
                map,
                None,
                None,
                None,
                None,
                None,
                false,
                2,
                &mut scenario_fill_ranged,
                &mut variant_selector,
                shared_cell_dummy,
                crate::map::resolved_terrain::OverlayLoadSource::Authored,
                descriptor.pixel_conversion_bounds,
            )
        };
        assert_eq!(
            (resolved.width(), resolved.height()),
            (extent, extent),
            "Fill allocates the extent the descriptor announced"
        );
        crate::sim::runtime::populate_staged_scenario_with_generated_inits(
            &mut sim,
            map,
            &resolved,
            theater_name,
            rules.unwrap_or(crate::rules::ruleset::RuleSet::empty_for_test()),
            None,
            None,
            bridge_destroyability_mode,
            &descriptor,
            None,
            initialize_houses_before_objects,
        )
        .expect("fixed-map Techno constructor projection cannot fail");
        (sim, resolved)
    }

    /// One valid LZO chunk whose decompressed bytes are the `(0, 0)`
    /// IsoMapPack terminator, so the parsed map has no explicit cell records.
    const EMPTY_ISO_MAP_PACK: &str = "CAAEABUAAAAAEQAA";

    #[test]
    fn free_radar_map_reload_and_recording_callback_rebuild_scenario_authority() {
        use crate::sim::replay::{NativeReplayHeader, NativeReplayStream};
        let rules =
            crate::rules::ruleset::RuleSet::from_ini(&crate::rules::ini_parser::IniFile::from_str(
                "[InfantryTypes]\n[VehicleTypes]\n[AircraftTypes]\n[BuildingTypes]\n",
            ))
            .unwrap();
        // A native recording supplies identity/seed to a normal-load callback.
        // This exercises that API with the actual GPU-free construction funnel;
        // the app's native recording UI/loader is not yet connected to it.
        for (key, expected) in [
            ("FreeRadar=yes\n", true),
            ("", false),
            ("FreeRadar=no\n", false),
            ("FreeRadar=invalid\n", false),
        ] {
            let map_bytes = format!(
                "[Basic]\n{key}[Map]\nTheater=TEMPERATE\nSize=0,0,2,2\nLocalSize=0,0,2,2\nFill=Water\n[IsoMapPack5]\n1={EMPTY_ISO_MAP_PACK}\n"
            );
            let map = MapFile::from_bytes(map_bytes.as_bytes()).unwrap();
            let header = NativeReplayHeader::new(0x12345678, "radar.map");
            let recording = header.encode();
            let playback = NativeReplayStream::initialize(
                &recording,
                NativeReplayHeader::default(),
                |loaded| {
                    let descriptor = ScenarioDescriptor {
                        free_radar: map.basic.free_radar.unwrap_or(false),
                        ..ScenarioDescriptor::from_native_replay_header(loaded)
                    };
                    let (sim, _terrain) = stage_fill_populate(
                        &map,
                        "TEMPERATE",
                        Some(&rules),
                        crate::map::basic::BridgeDestroyabilityMode::CampaignOrEditor,
                        descriptor,
                        |sim| {
                            let americans = sim.interner.intern("Americans");
                            sim.houses
                                .register(crate::sim::house_state::HouseState::new(
                                    americans, 0, None, true, 0, 10,
                                ));
                        },
                    );
                    Ok::<_, ()>(sim)
                },
            )
            .unwrap();
            assert_eq!(playback.state.session.free_radar(), expected);
            // House 0x00508DF0 reads Scenario FreeRadar when the House's
            // first update rechecks radar.
            let mut sim = playback.state;
            sim.advance_tick(&[], Some(&rules), None, None, 0);
            assert_eq!(
                crate::sim::radar::has_radar_for_owner(&sim, &rules, "Americans"),
                expected
            );
            assert_eq!(sim.session.map_name(), "radar.map");
        }
    }

    #[test]
    fn headless_battle_launch_is_the_exact_single_local_stock_session() {
        let launch = crate::sim::scenario_bootstrap::MatchLaunchDescriptor::from_resolved(
            battle_session("TEST.MAP", &[]),
        )
        .expect("one-player Battle launch");
        let session = launch.session();

        assert_eq!(session.mode.id, 1);
        assert_eq!(session.mode.override_file, "MPBattleMD.ini");
        assert_eq!(session.selected_map_file.as_deref(), Some("TEST.MAP"));
        assert!(session.opponents.is_empty());
        assert_eq!(session.local_slot().color_index, 0);
    }

    #[test]
    fn gsi_04_01_headless_sparse_water_load_materializes_and_draws_on_the_staged_owner() {
        let seed = 0x0401_5EED;
        let map_bytes = format!(
            "[Map]\n\
             Theater=TEMPERATE\n\
             Size=0,0,2,1\n\
             LocalSize=0,0,2,1\n\
             Fill=Water\n\
             [IsoMapPack5]\n\
             1={EMPTY_ISO_MAP_PACK}\n"
        );
        let map = MapFile::from_bytes(map_bytes.as_bytes()).expect("parse sparse INI map");
        assert!(
            map.cells.is_empty(),
            "fixture has no explicit terrain cells"
        );

        let descriptor = ScenarioDescriptor {
            seed,
            ..ScenarioDescriptor::default()
        };
        let (sim, resolved) = stage_fill_populate(
            &map,
            &map.header.theater,
            None,
            crate::map::basic::BridgeDestroyabilityMode::SkirmishOrMultiplayer {
                bridge_destruction: true,
            },
            descriptor,
            |_| {},
        );
        assert_eq!(
            (resolved.width(), resolved.height()),
            (3, 3),
            "the canonical cell array spans the Size diamond's highest coordinate"
        );
        let mut allocated: Vec<_> = resolved.iter().map(|cell| (cell.rx, cell.ry)).collect();
        allocated.sort_unstable();
        assert_eq!(allocated, vec![(1, 2), (2, 1), (2, 2)]);

        let mut expected_scenario = SimRng::new(u64::from(seed));
        for _ in 0..3 {
            let _ = expected_scenario.next_range_u32_inclusive(0, 3);
        }
        let state = sim.rng_state();
        assert_eq!(state.scenario, expected_scenario.logical_state());
        assert_eq!(
            state.main,
            SimRng::new(u64::from(seed)).logical_state(),
            "no theater TMP selection means the Main cursor stays at its seed"
        );
    }

    #[test]
    fn retail_signed_terrain_key_constructs_a_live_display_nonmember() {
        use crate::map::source::test_support::TestDirectory;
        use crate::util::ini_writer::{replace_ini_section, set_ini_values};

        let Some((ra2, assets)) = crate::rules::retail_ini_fixture::retail_assets() else {
            return;
        };
        let bytes = assets.resolve_ref("XXmas.MAP").unwrap().bytes;
        let map = MapFile::from_bytes(bytes).unwrap();
        let control_cell = map.terrain_objects.first().unwrap();
        let key = (u32::from(control_cell.ry) * 1000 + u32::from(control_cell.rx)).to_string();
        let bytes = set_ini_values(bytes, "Basic", &[("NewINIFormat", "4")]);
        let directory = TestDirectory::new("signed-terrain-key");
        directory.write(
            "control.map",
            &replace_ini_section(&bytes, "Terrain", &[(&key, "TREE01")]),
        );
        directory.write(
            "signed.map",
            &replace_ini_section(&bytes, "Terrain", &[("-1", "TREE01"), (&key, "TREE01")]),
        );
        let control = load(
            &ra2,
            directory.path().join("control.map").to_str().unwrap(),
            31,
        )
        .unwrap();
        let signed = load(
            &ra2,
            directory.path().join("signed.map").to_str().unwrap(),
            31,
        )
        .unwrap();
        // ReadMap71CAEF..71CB19 keeps the signed remainder word; ctor71BC34
        // tests those input words before Get_Cell_Packed41BEA0 derives0,0.
        assert_eq!(signed.map.terrain_objects.len(), 2);
        assert_eq!(
            (
                signed.map.terrain_objects[0].rx,
                signed.map.terrain_objects[0].ry
            ),
            (u16::MAX, 0)
        );
        assert_eq!(signed.sim().terrain_objects().objects().len(), 2);
        let tree = signed
            .sim()
            .terrain_objects()
            .objects()
            .values()
            .next()
            .unwrap();
        assert_eq!(
            tree.world_coord(),
            crate::sim::projectile::ProjectileCoord::new(-128, 128, 0)
        );
        assert_eq!(tree.cell(), (0, 0));
        assert!(tree.object_alive() && tree.is_live() && !tree.in_logic_vector());
        assert!(
            signed
                .sim()
                .display_layers()
                .layer_of(tree.stable_id())
                .is_some()
        );
        assert!(signed.sim().terrain_objects().id_at((0, 0)).is_none());
        assert!(
            signed
                .sim()
                .terrain_objects()
                .id_at((u16::MAX, 0))
                .is_none()
        );
        let ordinary = signed
            .sim()
            .terrain_objects()
            .id_at((control_cell.rx, control_cell.ry))
            .unwrap();
        let baseline = control
            .sim()
            .terrain_objects()
            .objects()
            .values()
            .next()
            .unwrap();
        assert_eq!(tree.native_unique_id(), baseline.native_unique_id());
        assert_eq!(
            signed
                .sim()
                .terrain_objects()
                .get(ordinary)
                .unwrap()
                .native_unique_id(),
            baseline.native_unique_id().map(|id| id.wrapping_add(1))
        );
        assert_eq!(signed.sim().rng_state(), control.sim().rng_state());
        assert_eq!(
            signed.sim().substrate.raw_cell_occupation.ground_bits(0, 0),
            0
        );
        assert_eq!(
            signed
                .sim()
                .substrate
                .raw_cell_occupation
                .ground_bits(u16::MAX, 0),
            0
        );
    }

    #[test]
    fn retail_signed_terrain_alias_uses_the_real_cell_chain_through_restore_and_pickup() {
        use crate::map::source::test_support::TestDirectory;
        use crate::sim::snapshot::GameSnapshot;
        use crate::sim::world::UninitContext;
        use crate::util::ini_writer::{replace_ini_section, set_ini_values};

        let Some((ra2, assets)) = crate::rules::retail_ini_fixture::retail_assets() else {
            return;
        };
        let bytes = assets.resolve_ref("XXmas.MAP").unwrap().bytes;
        let bytes = set_ini_values(bytes, "Basic", &[("NewINIFormat", "4")]);
        let directory = TestDirectory::new("signed-terrain-alias");
        let bytes = replace_ini_section(
            &bytes,
            "Structures",
            &[(
                "0",
                "Neutral,GAPOWR,256,26,60,0,None,1,0,1,0,0,None,None,None,0,0",
            )],
        );
        directory.write(
            "alias.map",
            &replace_ini_section(
                &bytes,
                "Terrain",
                &[
                    ("-65474999", "TREE01"),
                    ("60026", "TREE01"),
                    ("-65433999", "TREE01"),
                ],
            ),
        );
        let scene = load(
            &ra2,
            directory.path().join("alias.map").to_str().unwrap(),
            31,
        )
        .unwrap();
        let sim = scene.sim();
        let alias = sim.terrain_objects().objects().values().next().unwrap();
        let alias_id = alias.stable_id();
        // ReadMap71CB10 narrows quotient-65474 to word62; signed center
        // (-999*256+128,62*256+128) derives(-998,62). Map5657A0's fixed
        // index62*512-998 is the ordinary allocated slot(26,60).
        let coord = alias.world_coord();
        assert_eq!((coord.x, coord.y), (-255616, 16000));
        assert_eq!(alias.cell(), ((-998i16) as u16, 62));
        let ordinary_id = sim.terrain_objects().id_at((26, 60)).unwrap();
        let hole_id = sim
            .terrain_objects()
            .objects()
            .values()
            .last()
            .unwrap()
            .stable_id();
        assert_ne!(ordinary_id, alias_id);
        assert_eq!(
            coord.z,
            sim.terrain_objects()
                .get(ordinary_id)
                .unwrap()
                .world_coord()
                .z,
            "both locations query the same native ground receiver"
        );
        assert_eq!(
            sim.terrain_objects()
                .members_at((26, 60))
                .collect::<Vec<_>>(),
            [ordinary_id, alias_id]
        );
        assert_eq!(sim.terrain_objects().cell_index().len(), 1);
        use crate::sim::occupancy::CellObjectMember;
        let building = sim
            .substrate
            .occupancy
            .first_building_on_layer(
                26,
                60,
                crate::sim::movement::locomotor::MovementLayer::Ground,
            )
            .unwrap();
        assert_eq!(
            sim.next_cell_object(CellObjectMember::Terrain(ordinary_id)),
            Some(CellObjectMember::Terrain(alias_id))
        );
        assert_eq!(
            sim.next_cell_object(CellObjectMember::Terrain(alias_id)),
            Some(CellObjectMember::Entity(building)),
            "canonical Cell+E4 list continues into its appended Building"
        );
        let rules = &scene.runtime.resources.rules;
        let bytes = GameSnapshot::save_validated(
            sim,
            0,
            rules.simulation_config_hash(),
            "signed Terrain alias",
            0,
        );
        let mut loaded = GameSnapshot::load(&bytes).unwrap().sim;
        loaded.restore_after_snapshot_load().unwrap();
        loaded.rebuild_caches_after_load(
            sim.resolved_terrain().unwrap().clone(),
            Default::default(),
            rules,
        );
        loaded
            .restore_map_authority_after_snapshot_load(rules, rules.overlay_registry(), None)
            .unwrap();
        loaded.resolve_type_handles(rules);
        assert_eq!(
            loaded
                .terrain_objects()
                .members_at((26, 60))
                .collect::<Vec<_>>(),
            [ordinary_id, alias_id]
        );
        let warhead_ref = loaded.interner.intern("HE");
        loaded.commit_direct_terrain_damage_receiver(
            rules,
            Some(rules.overlay_registry()),
            crate::sim::combat::TerrainDamageEvent {
                damage_writeback: None,
                stable_id: alias_id,
                rx: 26,
                ry: 60,
                damage: 1,
                distance_leptons: 0,
                warhead_ref,
                near_center_ic_isolation_eligible: false,
            },
            true,
            Default::default(),
        );
        assert_eq!(
            loaded.terrain_objects().get(alias_id).unwrap().health(),
            alias.health() - 1
        );
        loaded.dirty_cells.clear_radar_for_test();
        loaded.uninit_with_context(alias_id, UninitContext::with_rules(rules));
        assert_eq!(
            loaded
                .terrain_objects()
                .members_at((26, 60))
                .collect::<Vec<_>>(),
            [ordinary_id]
        );
        assert_eq!(loaded.dirty_cells.radar(), [alias.cell()]);
        loaded.process_pending_delete_with(
            rules,
            Some(rules.overlay_registry()),
            Default::default(),
            1,
            true,
        );
        assert!(loaded.terrain_objects().get(alias_id).is_none());
        assert!(loaded.terrain_objects().get(ordinary_id).unwrap().is_live());
        assert_eq!(
            loaded.next_cell_object(CellObjectMember::Terrain(ordinary_id)),
            Some(CellObjectMember::Entity(building))
        );

        // Native71CA42/51 is independent of Map allocation. This second
        // source derives(-998,103): Map fixed512 selects the hole(26,101),
        // but the retained157-stride zone record selects allocated(101,96).
        let terrain = loaded.resolved_terrain().unwrap();
        assert!(terrain.native_fixed_cell_index(26, 101).is_none());
        assert!(terrain.native_fixed_cell_index(101, 96).is_some());
        let zones = loaded.zone_grid.as_mut().unwrap();
        let width = usize::from(zones.width());
        let target = 96 * width + 101;
        let base = zones.base_topology_mut_for_test();
        let cluster = base.zone_ids().iter().copied().find(|&id| id > 1).unwrap();
        base.movement_classes_mut_for_test()[target] =
            crate::map::resolved_terrain::zone_class::GROUND;
        base.zone_ids_mut_for_test()[target] = 0;
        for (dx, dy) in crate::util::direction::DIRECTION_DELTAS {
            let index = ((96 + dy) as usize) * width + (101 + dx) as usize;
            base.movement_classes_mut_for_test()[index] =
                crate::map::resolved_terrain::zone_class::GROUND;
            base.zone_ids_mut_for_test()[index] = cluster;
        }
        loaded.uninit_with_context(hole_id, UninitContext::with_rules(rules));
        assert_eq!(
            loaded
                .zone_grid
                .as_mut()
                .unwrap()
                .base_topology_mut_for_test()
                .zone_ids()[target],
            cluster,
            "AssignOrphaned must repair its retained zone receiver after Dummy Recalc"
        );
    }

    #[test]
    #[ignore = "requires RA2_DIR with installed retail RA2/YR assets"]
    fn retail_authored_unknown_terrain_allocates_before_instance_and_clears_ore_without_membership()
    {
        use crate::map::source::test_support::TestDirectory;
        use crate::util::ini_writer::{replace_ini_section, set_ini_values};

        let (ra2, assets) = crate::rules::retail_ini_fixture::retail_assets()
            .expect("set RA2_DIR to the retail RA2/YR install directory");
        let battle =
            crate::rules::retail_ini_fixture::retail_battle_rules().expect("retail Battle rules");
        assert!(
            assets.get_ref("SMOKEY.SHP").is_some(),
            "same-name physical image exists"
        );
        assert!(
            battle.fixed_art.section("SMOKEY").is_some(),
            "stock Anim ART exists"
        );
        let ore_id = battle
            .rules
            .overlay_registry()
            .id_for_name("TIB01")
            .unwrap();
        let bytes = assets
            .resolve_ref("XMP03T4.MAP")
            .expect("retail Anytown")
            .bytes;
        let map = MapFile::from_bytes(bytes).expect("retail map parser");
        let known = map.terrain_objects.first().expect("retail Terrain control");
        let source = map
            .overlays
            .iter()
            .find(|entry| {
                entry.overlay_id == ore_id && (entry.rx, entry.ry) != (known.rx, known.ry)
            })
            .expect("retail ore cell");
        let source_key = (u32::from(source.ry) * 1000 + u32::from(source.rx)).to_string();
        let known_key = (u32::from(known.ry) * 1000 + u32::from(known.rx)).to_string();
        // Full_Init68774F processes this orphan body before Read_Map_Section
        // 71CA70 reaches FindOrAllocate71E2A0. Late allocation must not read it.
        let bytes = set_ini_values(bytes, "Basic", &[("NewINIFormat", "4")]);
        let bytes = set_ini_values(
            &bytes,
            "SMOKEY",
            &[
                ("Strength", "9000"),
                ("Foundation", "2x2"),
                ("SpawnsTiberium", "yes"),
                ("Image", "TREE01"),
            ],
        );
        let directory = TestDirectory::new("late-terrain-factory");
        directory.write(
            "control.map",
            &replace_ini_section(&bytes, "Terrain", &[(&known_key, "TREE01")]),
        );
        directory.write(
            "late.map",
            &replace_ini_section(
                &bytes,
                "Terrain",
                &[(&source_key, "SMOKEY"), (&known_key, "TREE01")],
            ),
        );
        let control_path = directory.path().join("control.map");
        let late_path = directory.path().join("late.map");
        let control =
            load(&ra2, control_path.to_str().unwrap(), 31).expect("control authored load");
        let late = load(&ra2, late_path.to_str().unwrap(), 31).expect("late factory authored load");
        assert!(
            control
                .runtime
                .resources
                .rules
                .terrain_object_type_case_insensitive("SMOKEY")
                .is_none()
        );
        assert_eq!(
            late.sim().terrain_objects().objects().len(),
            2,
            "TerrainReadMap71CAD2/71CB6E constructs the missing type and then its instance"
        );
        let control_id = control
            .sim()
            .terrain_objects()
            .objects()
            .values()
            .next()
            .unwrap()
            .native_unique_id()
            .unwrap();
        let unknown = late
            .sim()
            .terrain_objects()
            .objects()
            .values()
            .find(|object| late.sim().interner.resolve(object.type_ref()) == "SMOKEY")
            .expect("live ctor-only Terrain instance");
        // TerrainTypeCtor71DA80 at71DB11 spends an ID before instance71BC19;
        // the following known TREE01 constructor must observe both allocations.
        assert_eq!(unknown.native_unique_id(), Some(control_id.wrapping_add(1)));
        let known_id = late
            .sim()
            .terrain_objects()
            .id_at((known.rx, known.ry))
            .unwrap();
        assert_eq!(
            late.sim()
                .terrain_objects()
                .get(known_id)
                .unwrap()
                .native_unique_id(),
            Some(control_id.wrapping_add(2))
        );
        assert_eq!((unknown.health(), unknown.max_health()), (-1, -1));
        assert!(unknown.is_live() && !unknown.in_logic_vector());
        assert!(
            late.sim()
                .display_layers()
                .layer_of(unknown.stable_id())
                .is_some()
        );
        assert_eq!(
            late.runtime
                .resources
                .rules
                .terrain_shape_filename("SMOKEY"),
            None
        );
        // Ctor-null Occupy_List71DAF1/71DE40 visits no Cell in MapPlaceDown,
        // but TerrainUnlimbo71D0E2..71D12B still clears the source ore.
        assert_eq!(
            late.sim().terrain_objects().id_at((source.rx, source.ry)),
            None
        );
        assert_eq!(
            late.sim()
                .substrate
                .raw_cell_occupation
                .ground_bits(source.rx, source.ry)
                & 0x1C,
            0
        );
        assert_eq!(
            late.sim()
                .resolved_terrain
                .as_ref()
                .unwrap()
                .cell(source.rx, source.ry)
                .unwrap()
                .terrain_object_occupation,
            None
        );
        let control_cell = control
            .sim()
            .overlay_grid
            .as_ref()
            .unwrap()
            .cell(source.rx, source.ry);
        assert_eq!(control_cell.overlay_id, Some(ore_id));
        assert_eq!(control_cell.overlay_data, source.frame);
        let cleared = late
            .sim()
            .overlay_grid
            .as_ref()
            .unwrap()
            .cell(source.rx, source.ry);
        assert_eq!((cleared.overlay_id, cleared.overlay_data), (None, 0));
    }

    /// F09 certification: the shared GPU-free funnel produces a deterministic,
    /// fully populated headless scenario on a retail map. Two loads of the same
    /// map and seed must yield identical parity digests at construction and on
    /// every tick, and the construction gains the funnel promises — stock launch
    /// houses, overlay/smudge/bridge authority, published navigation — must all
    /// be present. App-vs-headless assembly drift is prevented structurally:
    /// both call `finalize_and_populate_staged_authored_scenario` followed by
    /// `finalize_constructed_scenario`.
    #[test]
    #[ignore = "requires RA2_DIR with installed retail RA2/YR assets"]
    fn retail_headless_funnel_construction_is_deterministic_and_populated() {
        let ra2 = std::path::PathBuf::from(
            std::env::var("RA2_DIR").expect("set RA2_DIR to the retail RA2/YR install directory"),
        );
        let seed = 0x00C0_FFEE;
        let mut a = load(&ra2, "Dustbowl.mmx", seed).expect("first headless load");
        let mut b = load(&ra2, "Dustbowl.mmx", seed).expect("second headless load");

        let rules = &a.runtime.resources.rules;
        for name in rules
            .infantry_ids
            .iter()
            .chain(&rules.vehicle_ids)
            .chain(&rules.aircraft_ids)
            .chain(&rules.building_ids)
        {
            let id = a
                .sim()
                .interner
                .get(name)
                .expect("bound unspawned type identity");
            assert!(a.sim().interner.resolve(id).eq_ignore_ascii_case(name));
            assert_eq!(
                a.sim().object_type(id, rules).map(|object| &object.id),
                rules.object(name).map(|object| &object.id)
            );
        }
        assert!(
            a.sim().resolved_rule_handles().is_some(),
            "headless combat rule bindings"
        );
        assert_eq!(rules.general.metallic_debris.len(), 15);
        assert_eq!(
            rules.general.metallic_debris.last().map(String::as_str),
            Some("D")
        );
        let d = rules
            .art()
            .anim_runtime_config("D")
            .expect("native unread type receipt");
        assert!(!d.art_body_read);
        assert!(!d.bouncer);
        assert_eq!(d.raw_shp_frame_count, None);
        assert!(rules.art().scheduler_anim_types().contains("D"));
        assert_eq!(
            rules.simulation_config_hash(),
            b.runtime.resources.rules.simulation_config_hash(),
            "each fresh load publishes the same final bound ART owner"
        );

        assert_eq!(
            a.sim().parity_digest(),
            b.sim().parity_digest(),
            "same map+seed must produce an identical construction fingerprint"
        );
        assert!(
            !a.sim().houses.is_empty(),
            "map-roster houses must be constructed before objects"
        );
        assert!(
            a.sim().overlay_grid.is_some(),
            "overlay authority must be installed by the shared finalization"
        );
        assert!(
            a.sim().smudge_grid.is_some(),
            "smudge authority must be installed by the shared finalization"
        );
        assert!(
            a.sim().bridge_state.is_some(),
            "bridge runtime state must be constructed by the funnel"
        );
        assert!(
            a.sim().authored_tiberium_value_total().is_some(),
            "authored Full_Init stores the value-only InitCellAttributes(0) total"
        );

        for tick in 0..30u32 {
            a.tick();
            b.tick();
            assert_eq!(
                a.sim().parity_digest(),
                b.sim().parity_digest(),
                "runtime-backed headless execution diverged at tick {tick}"
            );
        }

        // The same pair as a LAN session, whose GameMode takes the multiplayer
        // Queue_AI route: both instances store an identical retail sync
        // checksum every frame.
        for scenario in [&mut a, &mut b] {
            let session = &mut scenario.runtime.simulation.session;
            session.set_game_mode(crate::sim::scenario_session::ScenarioGameMode::Lan);
            session.install_network_timing(10);
        }
        for tick in 0..60u32 {
            let [checksum_a, checksum_b] = [&mut a, &mut b].map(|scenario| {
                tick_with_due_commands(scenario)
                    .multiplayer_checksum
                    .expect("multiplayer route checksums every frame")
            });
            assert!(
                checksum_a.checksum.is_ok(),
                "retail registries cover every display/Logic entry: {checksum_a:?}"
            );
            assert_eq!(
                checksum_a, checksum_b,
                "multiplayer sync checksum diverged at tick {tick}"
            );
        }
    }

    #[test]
    #[ignore = "requires RA2_DIR with installed retail RA2/YR assets"]
    fn retail_authored_terrain_anims_consume_the_final_bound_art_owner() {
        let ra2 = std::path::PathBuf::from(
            std::env::var("RA2_DIR").expect("set RA2_DIR to the retail RA2/YR install directory"),
        );
        // Stock Anytown supplies waterfall tiles. Exercise the full authored
        // finalizer, whose pending Fill has no eagerly constructed tile Anims.
        let scenario = load(&ra2, "XMP03T4.MAP", 0x00C0_FFEE).expect("Anytown headless load");
        let rules = &scenario.runtime.resources.rules;
        let mut terrain_anims = 0;
        for (id, anim) in scenario
            .sim()
            .anims()
            .filter(|(_, anim)| anim.terrain_attached())
        {
            terrain_anims += 1;
            let name = scenario.sim().interner.resolve(anim.type_id());
            let config = rules
                .art()
                .anim_runtime_config(name)
                .expect("bound terrain type");
            assert!(rules.art().scheduler_anim_types().contains(name));
            assert!(config.art_body_read && config.raw_shp_frame_count.is_some());
            assert_eq!(anim.effective_end(), config.end, "{name} final bound End");
            assert_eq!(
                anim.effective_loop_end(),
                config.loop_end,
                "{name} final bound LoopEnd"
            );
            assert!(anim.use_cell_drawer());
            assert!(scenario.sim().anim_display_layer(*id, rules).is_some());
        }
        assert!(
            terrain_anims > 0,
            "retail finalizer coverage must not be vacuous"
        );
        eprintln!("Anytown final bound ART owner: {terrain_anims} live terrain Anims");
    }

    /// XMP03T4 with the local house idle and three computer houses: Russia
    /// (Hard), Yuri (Normal) and France (Easy). Returns the computer houses.
    fn retail_skirmish() -> (HeadlessScenario, Vec<crate::sim::intern::InternedId>) {
        use crate::skirmish_launch::{AiDifficulty, LaunchCountry};

        retail_battle(
            |_| {},
            &[
                (LaunchCountry::Russia, AiDifficulty::Hard),
                (LaunchCountry::Yuri, AiDifficulty::Normal),
                (LaunchCountry::France, AiDifficulty::Easy),
            ],
        )
    }

    /// XMP03T4 with the local house idle, as `configure` adjusts the battle
    /// session, against the computer `opponents`. Returns the computer houses.
    fn retail_battle(
        configure: impl FnOnce(&mut crate::skirmish_launch::SkirmishLaunchSession),
        opponents: &[(
            crate::skirmish_launch::LaunchCountry,
            crate::skirmish_launch::AiDifficulty,
        )],
    ) -> (HeadlessScenario, Vec<crate::sim::intern::InternedId>) {
        let ra2 = std::path::PathBuf::from(
            std::env::var("RA2_DIR").expect("set RA2_DIR to the retail RA2/YR install directory"),
        );
        retail_battle_in(&ra2, configure, opponents)
    }

    fn retail_battle_in(
        ra2: &Path,
        configure: impl FnOnce(&mut crate::skirmish_launch::SkirmishLaunchSession),
        opponents: &[(LaunchCountry, AiDifficulty)],
    ) -> (HeadlessScenario, Vec<InternedId>) {
        // A four-start map: fewer start waypoints than houses would seed the
        // extra starts at random, which the lobby never allows.
        let map = "XMP03T4.MAP";
        let mut session = battle_session(map, opponents);
        configure(&mut session);
        let scenario = load_session(ra2, map, 0x00C0_FFEE, session).unwrap();
        let computers: Vec<InternedId> = scenario
            .sim()
            .houses
            .iter()
            .filter(|(_, house)| !house.is_controlled_by_human(true) && !house.multiplay_passive())
            .map(|(&id, _)| id)
            .collect();
        assert_eq!(computers.len(), opponents.len());
        (scenario, computers)
    }

    #[cfg(feature = "npatch")]
    fn aymod_battle(local_country: u32) -> HeadlessScenario {
        let root = std::path::PathBuf::from(
            std::env::var_os("VERA20K_AYMOD_DIR")
                .expect("set VERA20K_AYMOD_DIR to the unmodified AYMod install"),
        );
        let opponent = if local_country == 1 { 0 } else { 1 };
        retail_battle_in(
            &root,
            |session| {
                // AYMod's registry is Americans=0, Russians=1, YuriCountry=2.
                session.local_slot_mut().country = LaunchCountry::from_country_index(local_country);
                session.options.unit_count = 0;
            },
            &[(
                LaunchCountry::from_country_index(opponent),
                AiDifficulty::Easy,
            )],
        )
        .0
    }

    #[cfg(feature = "npatch")]
    fn aymod_anchor(sim: &Simulation, owner: InternedId) -> (i32, i32) {
        sim.entities()
            .values()
            .find(|entity| entity.owner() == owner)
            .map(|entity| {
                (
                    i32::from(entity.position.rx()),
                    i32::from(entity.position.ry()),
                )
            })
            .expect("the loaded house has its starting force")
    }

    #[cfg(feature = "npatch")]
    fn aymod_emp_duel(local_country: u32, victim_type: &str) -> (HeadlessScenario, u64, u64) {
        use crate::sim::command::{Command, CommandEnvelope};
        use crate::sim::mission::MissionType;
        use crate::sim::movement::locomotor::MovementLayer;
        use crate::sim::production::ProductionCategory;

        let mut scenario = aymod_battle(local_country);
        let victim = {
            let runtime = &mut scenario.runtime;
            let (sim, rules) = (&mut runtime.simulation, &runtime.resources.rules);
            let america = house_of(sim, "Americans");
            let anchor = aymod_anchor(sim, america);
            let sites: Vec<_> =
                cells_nearest(anchor)
                    .into_iter()
                    .filter(|&(x, y)| {
                        let level = u16::try_from(x)
                            .ok()
                            .zip(u16::try_from(y).ok())
                            .and_then(|(x, y)| sim.terrain_cell_level(x, y));
                        level.is_some()
                            && (-2..=2).all(|dx| {
                                (-2..=2).all(|dy| {
                                    let Some((rx, ry)) =
                                        u16::try_from(x + dx).ok().zip(u16::try_from(y + dy).ok())
                                    else {
                                        return false;
                                    };
                                    sim.terrain_cell_level(rx, ry) == level
                                        && sim.path_grid().is_some_and(|grid| {
                                            grid.is_walkable_for_infantry(rx, ry)
                                        })
                                        && sim.substrate.occupancy.get(rx, ry).is_none_or(|cell| {
                                            cell.is_empty_on(MovementLayer::Ground)
                                        })
                                })
                            })
                    })
                    .collect();
            let victim = spawn_on_free_cells(sim, rules, america, sites, &[victim_type])[0];
            if victim_type == "GAWEAP" {
                let at = sim.entities().get(victim).unwrap().position;
                let cells = cells_nearest((i32::from(at.rx()), i32::from(at.ry())))
                    .into_iter()
                    .filter(|&(x, y)| {
                        (x - i32::from(at.rx()))
                            .abs()
                            .max((y - i32::from(at.ry())).abs())
                            >= 4
                    });
                spawn_on_free_cells(sim, rules, america, cells, &["GAPOWR"]);
            }
            victim
        };
        for _ in 0..240 {
            tick_with_due_commands(&mut scenario);
            let entity = scenario
                .sim()
                .entities()
                .get(victim)
                .expect("the victim lives");
            if entity.mission.effective().known() != Some(MissionType::Construction)
                && (victim_type != "GAWEAP"
                    || scenario
                        .sim()
                        .building_operational_state(victim, &scenario.runtime.resources.rules)
                        == Some(true))
            {
                break;
            }
        }
        if victim_type == "GAWEAP" {
            assert_eq!(
                scenario
                    .sim()
                    .building_operational_state(victim, &scenario.runtime.resources.rules),
                Some(true)
            );
            let sim = &mut scenario.runtime.simulation;
            let owner = sim.entities().get(victim).unwrap().owner();
            let type_id = sim.interner.get("MTNK").expect("real AY Grizzly type");
            sim.queue_command(CommandEnvelope::new(
                owner,
                sim.session.tick() + 1,
                Command::QueueProduction { type_id },
            ));
            for _ in 0..30 {
                tick_with_due_commands(&mut scenario);
                if scenario
                    .sim()
                    .production
                    .factories()
                    .view(owner, ProductionCategory::Vehicle)
                    .is_some_and(|head| head.object.is_some() && !head.suspended)
                {
                    break;
                }
            }
            assert!(
                scenario
                    .sim()
                    .production
                    .factories()
                    .view(owner, ProductionCategory::Vehicle)
                    .is_some_and(|head| head.object.is_some() && !head.suspended),
                "real GAWEAP must start its paid MTNK head before EMP"
            );
        }
        let shooter = {
            let runtime = &mut scenario.runtime;
            let (sim, rules) = (&mut runtime.simulation, &runtime.resources.rules);
            let russia = house_of(sim, "Russians");
            let at = sim.entities().get(victim).unwrap().position;
            let cells: Vec<_> = cells_nearest((i32::from(at.rx()), i32::from(at.ry())))
                .into_iter()
                .filter(|&(x, y)| {
                    (x - i32::from(at.rx()))
                        .abs()
                        .max((y - i32::from(at.ry())).abs())
                        <= 1
                        && u16::try_from(x)
                            .ok()
                            .zip(u16::try_from(y).ok())
                            .is_some_and(|(x, y)| sim.terrain_cell_level(x, y) == Some(at.z()))
                })
                .collect();
            assert_eq!(rules.object("TTNK").unwrap().primary(), Some("TankBolt"));
            assert!(
                rules.object("TTNK").unwrap().secondary().is_none(),
                "do not pre-register the impact child through a fake Secondary"
            );
            spawn_on_free_cells(sim, rules, russia, cells, &["TTNK"])[0]
        };
        (scenario, shooter, victim)
    }

    #[cfg(feature = "npatch")]
    fn aymod_attack_until_emp(scenario: &mut HeadlessScenario, shooter: u64, target: u64) {
        use crate::sim::command::{Command, CommandEnvelope};
        let sim = &mut scenario.runtime.simulation;
        let owner = sim.entities().get(shooter).unwrap().owner();
        sim.queue_command(CommandEnvelope::new(
            owner,
            sim.session.tick() + 1,
            Command::Attack {
                attacker_id: shooter,
                target_id: target,
            },
        ));
        for _ in 0..240 {
            tick_with_due_commands(scenario);
            if scenario
                .sim()
                .entities()
                .get(target)
                .is_some_and(|entity| entity.emp_remaining() > 0)
            {
                return;
            }
        }
        panic!("actual AY TTNK Attack never delivered EMP to live target {target}");
    }

    #[cfg(feature = "npatch")]
    fn aymod_remove_shooter(scenario: &mut HeadlessScenario, shooter: u64) {
        let runtime = &mut scenario.runtime;
        runtime.simulation.uninit_with_context(
            shooter,
            crate::sim::world::UninitContext::new(
                &runtime.resources.rules,
                Some(&runtime.resources.overlay_registry),
            ),
        );
        runtime.simulation.flush_pending_delete();
    }

    #[cfg(feature = "npatch")]
    #[test]
    #[ignore = "requires VERA20K_AYMOD_DIR with the original AYMod rules/assets"]
    fn aymod_tesla_attack_delivers_emp_and_actual_frames_recover() {
        let (mut scenario, shooter, target) = aymod_emp_duel(1, "APOC");
        aymod_attack_until_emp(&mut scenario, shooter, target);
        assert_eq!(
            scenario
                .sim()
                .entities()
                .get(target)
                .unwrap()
                .deactivated
                .is_offline(),
            cfg!(feature = "ares"),
            "pure NP ground Drive EMP does not acquire the provider latch"
        );
        aymod_remove_shooter(&mut scenario, shooter);
        for _ in 0..1000 {
            tick_with_due_commands(&mut scenario);
            let victim = scenario
                .sim()
                .entities()
                .get(target)
                .expect("EMP must not kill the mechanical victim");
            if victim.emp_remaining() == 0 {
                assert!(
                    !victim.deactivated.is_offline(),
                    "real Techno AI must restore control"
                );
                return;
            }
        }
        panic!("actual AY EMP never expired after removing the shooter");
    }

    #[cfg(feature = "npatch")]
    #[test]
    #[ignore = "requires VERA20K_AYMOD_DIR with the original AYMod rules/assets"]
    fn aymod_enemy_emp_and_manual_power_hold_and_resume_paid_factory() {
        use crate::sim::command::{Command, CommandEnvelope};
        use crate::sim::production::ProductionCategory;
        let (mut scenario, shooter, factory) = aymod_emp_duel(0, "GAWEAP");
        aymod_attack_until_emp(&mut scenario, shooter, factory);
        aymod_remove_shooter(&mut scenario, shooter);
        let owner = scenario.sim().entities().get(factory).unwrap().owner();
        for _ in 0..3 {
            tick_with_due_commands(&mut scenario);
        }
        assert_eq!(
            scenario
                .sim()
                .building_operational_state(factory, &scenario.runtime.resources.rules),
            Some(false)
        );
        assert!(
            scenario
                .sim()
                .production
                .factories()
                .view(owner, ProductionCategory::Vehicle)
                .is_some_and(|head| head.object.is_some() && head.suspended),
            "EMP must hold the real paid production head: {:?}",
            scenario
                .sim()
                .production
                .factories()
                .view(owner, ProductionCategory::Vehicle)
                .map(|head| (head.object, head.suspended, head.progress))
        );
        let sim = &mut scenario.runtime.simulation;
        sim.queue_command(CommandEnvelope::new(
            owner,
            sim.session.tick() + 1,
            Command::SetBuildingOnLine {
                entity_id: factory,
                on: false,
            },
        ));
        for _ in 0..3 {
            tick_with_due_commands(&mut scenario);
        }
        assert!(
            !scenario
                .sim()
                .entities()
                .get(factory)
                .unwrap()
                .building_online()
        );
        let held = scenario
            .sim()
            .production
            .factories()
            .view(owner, ProductionCategory::Vehicle)
            .unwrap()
            .progress;
        for _ in 0..1000 {
            tick_with_due_commands(&mut scenario);
            if scenario
                .sim()
                .entities()
                .get(factory)
                .unwrap()
                .emp_remaining()
                == 0
            {
                break;
            }
        }
        assert_eq!(
            scenario
                .sim()
                .entities()
                .get(factory)
                .unwrap()
                .emp_remaining(),
            0
        );
        assert!(
            !scenario
                .sim()
                .entities()
                .get(factory)
                .unwrap()
                .building_online(),
            "EMP expiry must not override the player's Off choice"
        );
        let head = scenario
            .sim()
            .production
            .factories()
            .view(owner, ProductionCategory::Vehicle)
            .unwrap();
        assert!(head.suspended);
        assert_eq!(head.progress, held);
        let sim = &mut scenario.runtime.simulation;
        sim.queue_command(CommandEnvelope::new(
            owner,
            sim.session.tick() + 1,
            Command::SetBuildingOnLine {
                entity_id: factory,
                on: true,
            },
        ));
        for _ in 0..120 {
            tick_with_due_commands(&mut scenario);
            if scenario
                .sim()
                .production
                .factories()
                .view(owner, ProductionCategory::Vehicle)
                .is_some_and(|head| !head.suspended && head.progress > held)
            {
                assert_eq!(
                    scenario
                        .sim()
                        .building_operational_state(factory, &scenario.runtime.resources.rules),
                    Some(true)
                );
                return;
            }
        }
        panic!("real paid MTNK production did not resume after manual On");
    }

    #[cfg(feature = "npatch")]
    #[test]
    #[ignore = "requires VERA20K_AYMOD_DIR; real Prism support after power orders and save/resume"]
    fn aymod_prism_support_respects_saved_offline_state_and_recovers() {
        use crate::sim::command::{Command, CommandEnvelope};
        use crate::sim::world::{
            DelayedFire,
            bridge_test_evidence::{restored_retail, save_scene},
        };
        // A friendly heavy target keeps the supporting tower on Guard;
        // the master receives an explicit player Attack, not an AI override.
        let (mut scene, master, target) = aymod_nearby_pair("ATESLA", "APOC", true);
        let supporter = {
            let runtime = &mut scene.runtime;
            let (sim, rules) = (&mut runtime.simulation, &runtime.resources.rules);
            assert!(rules.is_prism_type(rules.object("ATESLA").unwrap()));
            let owner = sim.entities().get(master).unwrap().owner();
            let at = sim.entities().get(master).unwrap().position;
            let center = (i32::from(at.rx()), i32::from(at.ry()));
            spawn_on_free_cells(sim, rules, owner, cells_nearest(center), &["GAPOWR"]);
            spawn_on_free_cells(sim, rules, owner, cells_nearest(center), &["ATESLA"])[0]
        };
        let queue = |scene: &mut HeadlessScenario, command| {
            let sim = &mut scene.runtime.simulation;
            let owner = sim.entities().get(master).unwrap().owner();
            sim.queue_command(CommandEnvelope::new(owner, sim.session.tick() + 1, command));
        };
        for _ in 0..3 {
            tick_with_due_commands(&mut scene);
        }
        for id in [master, supporter] {
            assert!(
                scene
                    .sim()
                    .entities()
                    .get(id)
                    .unwrap()
                    .building_last_operational
                    .is_operational()
            );
        }
        queue(
            &mut scene,
            Command::SetBuildingOnLine {
                entity_id: supporter,
                on: false,
            },
        );
        for _ in 0..3 {
            tick_with_due_commands(&mut scene);
        }
        assert!(
            !scene
                .sim()
                .entities()
                .get(supporter)
                .unwrap()
                .building_last_operational
                .is_operational()
        );
        let pristine = scene.sim().resolved_terrain().unwrap().clone();
        let bytes = save_scene(&scene, "AY offline Prism supporter");
        scene.runtime.simulation = restored_retail(&scene, &pristine, &bytes);
        assert!(
            !scene
                .sim()
                .entities()
                .get(supporter)
                .unwrap()
                .building_last_operational
                .is_operational()
        );
        queue(
            &mut scene,
            Command::Attack {
                attacker_id: master,
                target_id: target,
            },
        );
        let mut fired = false;
        for _ in 0..240 {
            let output = tick_with_due_commands(&mut scene);
            assert_eq!(
                scene
                    .sim()
                    .entities()
                    .get(master)
                    .unwrap()
                    .prism_support_count
                    .count(),
                0
            );
            assert!(
                !scene
                    .sim()
                    .entities()
                    .get(supporter)
                    .unwrap()
                    .pending_building_fire
                    .pending()
                    .is_some_and(|pending| matches!(
                        pending.fire(),
                        DelayedFire::SupportBeam { .. }
                    ))
            );
            if output
                .fire_events
                .iter()
                .any(|event| event.attacker_id == master)
            {
                fired = true;
                break;
            }
        }
        assert!(
            fired,
            "the online master must still fire without offline support"
        );
        queue(
            &mut scene,
            Command::SetBuildingOnLine {
                entity_id: supporter,
                on: true,
            },
        );
        let mut recruited = false;
        for _ in 0..240 {
            let output = tick_with_due_commands(&mut scene);
            let sim = scene.sim();
            let supporter = sim.entities().get(supporter).unwrap();
            if supporter
                .pending_building_fire
                .pending()
                .is_some_and(|pending| matches!(pending.fire(), DelayedFire::SupportBeam { .. }))
            {
                assert!(supporter.building_last_operational.is_operational());
                assert!(
                    sim.entities()
                        .get(master)
                        .unwrap()
                        .prism_support_count
                        .count()
                        > 0
                );
                recruited = true;
            }
            if recruited
                && output
                    .fire_events
                    .iter()
                    .any(|event| event.attacker_id == master)
            {
                return;
            }
        }
        panic!("the restored supporter never rejoined Prism support after power On");
    }

    #[cfg(feature = "npatch")]
    #[test]
    #[ignore = "requires VERA20K_AYMOD_DIR; real Guard acquisition after restored Iron Curtain expiry"]
    fn aymod_guard_waits_for_restored_iron_curtain_before_acquiring_a_target() {
        use crate::sim::superweapon::invulnerability::{InvulnKind, apply_invulnerability};
        use crate::sim::world::bridge_test_evidence::{restored_retail, save_scene};
        let (mut scene, tank, target) = aymod_nearby_pair("MTNK", "MTNK", false);
        {
            let runtime = &mut scene.runtime;
            let rules = &runtime.resources.rules;
            // A sleeping victim cannot trigger the scanner's distinct
            // retaliation path before protection expires.
            runtime
                .simulation
                .mission_queue_exact(
                    target,
                    crate::sim::mission::MissionId::from_known(
                        crate::sim::mission::MissionType::Sleep,
                    ),
                    1,
                    runtime.simulation.session.binary_frame(),
                    &crate::sim::mission::authority::LiveReadyInputProvider { rules },
                )
                .unwrap();
            assert_eq!(
                runtime.simulation.greatest_threat_represented(
                    rules,
                    None,
                    tank,
                    crate::sim::combat::threat_range::ScanMission::Guard,
                    None,
                ),
                Some(target),
                "the unprotected target must be eligible before this test protects it"
            );
        }
        let pristine = scene.sim().resolved_terrain().unwrap().clone();
        let expires = scene.sim().session.binary_frame() + 45;
        let now = scene.sim().session.binary_frame();
        apply_invulnerability(
            scene
                .runtime
                .simulation
                .substrate
                .entities
                .get_mut(target)
                .unwrap(),
            now,
            45,
            InvulnKind::IronCurtain,
        );
        for _ in 0..10 {
            let output = tick_with_due_commands(&mut scene);
            assert!(
                !output
                    .fire_events
                    .iter()
                    .any(|event| event.attacker_id == tank)
            );
        }
        let bytes = save_scene(&scene, "AY protected target");
        scene.runtime.simulation = restored_retail(&scene, &pristine, &bytes);
        for _ in 0..180 {
            let frame = scene.sim().session.binary_frame();
            let output = tick_with_due_commands(&mut scene);
            let fired = output.fire_events.iter().any(|event| {
                event.attacker_id == tank
                    && event.target == crate::sim::combat::TargetKind::Entity(target)
            });
            if frame < expires {
                assert!(
                    !fired,
                    "Guard must not acquire the protected target after load"
                );
            } else if fired {
                return;
            }
        }
        panic!("Guard never reacquired the target after the saved protection expired");
    }

    #[cfg(feature = "npatch")]
    #[test]
    #[ignore = "requires VERA20K_AYMOD_DIR; actual aircraft Alt move over a service depot"]
    fn aymod_aircraft_alt_move_does_not_enter_or_repair_at_a_depot() {
        use crate::sim::command::{Command, CommandEnvelope};
        use crate::sim::mission::MissionType;
        use crate::sim::world::{EntryBaseAction, MoveAction};
        for building_type in ["GADEPT", "GAYARD"] {
            let mut scene = aymod_battle(0);
            let (plane, depot) = {
                let runtime = &mut scene.runtime;
                let (sim, rules) = (&mut runtime.simulation, &runtime.resources.rules);
                let owner = house_of(sim, "Americans");
                let anchor = aymod_anchor(sim, owner);
                let mut sites = if rules.object(building_type).unwrap().naval {
                    let terrain = sim.resolved_terrain().unwrap();
                    let (width, height) =
                        rules.object(building_type).unwrap().foundation.dimensions();
                    terrain
                        .cells()
                        .iter()
                        .filter(|cell| {
                            (0..height).all(|dy| {
                                (0..width).all(|dx| {
                                    terrain
                                        .cell(cell.rx + dx, cell.ry + dy)
                                        .is_some_and(|part| {
                                            !part.outside_playfield
                                                && part.level == cell.level
                                                && part.land_type
                                                    == crate::rules::terrain_rules::LandType::Water
                                                        .as_index()
                                                && sim.occupancy().get(part.rx, part.ry).is_none()
                                        })
                                })
                            })
                        })
                        .map(|cell| (i32::from(cell.rx), i32::from(cell.ry)))
                        .collect::<Vec<_>>()
                } else {
                    cells_nearest(anchor)
                };
                sites.sort_by_key(|&(x, y)| (x - anchor.0).pow(2) + (y - anchor.1).pow(2));
                let owner_name = sim.interner.resolve(owner).to_owned();
                let depot = sites
                    .into_iter()
                    .find_map(|(x, y)| {
                        sim.spawn_object(building_type, &owner_name, x as u16, y as u16, 0, rules)
                    })
                    .expect("an actual terrain-admitted depot/yard foundation");
                let at = sim.entities().get(depot).unwrap().position;
                let plane = spawn_on_free_cells(
                    sim,
                    rules,
                    owner,
                    cells_nearest((i32::from(at.rx()), i32::from(at.ry()))),
                    &["ORCA"],
                )[0];
                (plane, depot)
            };
            {
                let runtime = &mut scene.runtime;
                let (sim, resources) = (&mut runtime.simulation, &runtime.resources);
                let owner = sim.entities().get(plane).unwrap().owner();
                let at = sim.entities().get(plane).unwrap().position;
                let sites = cells_nearest((i32::from(at.rx()), i32::from(at.ry())))
                    .into_iter()
                    .filter(|&(x, y)| {
                        (x - i32::from(at.rx()))
                            .abs()
                            .max((y - i32::from(at.ry())).abs())
                            >= 5
                    });
                spawn_on_free_cells(sim, &resources.rules, owner, sites, &["GAAIRC"]);
                for base in [EntryBaseAction::Select, EntryBaseAction::Move] {
                    assert_eq!(
                        sim.entry_click_action(
                            plane,
                            depot,
                            base,
                            &resources.rules,
                            Default::default(),
                        ),
                        None
                    );
                }
                let destination = sim
                    .object_move_click(
                        owner,
                        plane,
                        depot,
                        MoveAction::Move,
                        true,
                        &resources.rules,
                    )
                    .unwrap()
                    .expect("Alt click resolves a normal movement destination");
                let command = match destination {
                    crate::sim::combat::TargetKind::Entity(target_id) => Command::MoveToObject {
                        entity_id: plane,
                        target_id,
                    },
                    crate::sim::combat::TargetKind::Cell(target_rx, target_ry) => Command::Move {
                        entity_id: plane,
                        target_rx,
                        target_ry,
                    },
                };
                sim.substrate
                    .entities
                    .get_mut(plane)
                    .unwrap()
                    .health
                    .set_for_test(100);
                sim.queue_command(CommandEnvelope::new(owner, sim.session.tick() + 1, command));
            }
            let start = scene.sim().entities().get(plane).unwrap().position;
            let mut moved = false;
            for _ in 0..180 {
                tick_with_due_commands(&mut scene);
                let aircraft = scene.sim().entities().get(plane).unwrap();
                moved |= crate::sim::movement::ground_pose::position_world_xy(&aircraft.position)
                    != crate::sim::movement::ground_pose::position_world_xy(&start);
                if aircraft.mission.effective().known() == Some(MissionType::Enter) {
                    assert_ne!(
                        aircraft.navigation.nav_com(),
                        Some(crate::sim::components::NavTargetRef::object(depot))
                    );
                }
                assert_ne!(aircraft.pending_entry(), Some(depot));
                assert!(!aircraft.radio_contacts.contains(depot));
                assert!(
                    aircraft.health.current() > 0,
                    "the aircraft must not merely crash without an airfield"
                );
            }
            assert!(
                moved,
                "the test must execute movement, not merely refuse the click"
            );
        }
    }

    #[cfg(feature = "npatch")]
    #[test]
    #[ignore = "requires VERA20K_AYMOD_DIR; actual Cloak crate pickup and validated save/resume"]
    fn aymod_cloak_crate_grants_mobile_instances_and_survives_real_load() {
        use crate::sim::command::{Command, CommandEnvelope};
        use crate::sim::world::bridge_test_evidence::{restored_retail, save_scene};
        let (mut scene, tank, infantry) = aymod_nearby_pair("MTNK", "E1", true);
        let pristine = scene.sim().resolved_terrain().unwrap().clone();
        {
            let runtime = &mut scene.runtime;
            let (sim, resources) = (&mut runtime.simulation, &runtime.resources);
            let rules = &resources.rules;
            assert_eq!(rules.powerups.weights[3], 10);
            assert_eq!(rules.crate_rules.radius, 768);
            assert!(!rules.object("MTNK").unwrap().cloakable);
            assert!(!rules.object("E1").unwrap().cloakable);
            let at = sim.entities().get(tank).unwrap().position;
            let friend = sim.entities().get(infantry).unwrap().position;
            let cell = cells_nearest((i32::from(at.rx()), i32::from(at.ry())))
                .into_iter()
                .find(|&(x, y)| {
                    let near = |p: crate::sim::components::Position| {
                        (x - i32::from(p.rx())).abs().max((y - i32::from(p.ry())).abs()) <= 1
                    };
                    near(at)
                        && near(friend)
                        && u16::try_from(x)
                            .ok()
                            .zip(u16::try_from(y).ok())
                            .is_some_and(|(x, y)| {
                                sim.terrain_cell_level(x, y) == Some(at.z())
                                    && sim.path_grid().unwrap().is_walkable_for_infantry(x, y)
                                    && sim.substrate.occupancy.get(x, y).is_none_or(|cell| {
                                        cell.is_empty_on(
                                            crate::sim::movement::locomotor::MovementLayer::Ground,
                                        )
                                    })
                                    && sim.overlay_grid.as_ref().unwrap().cell(x, y).overlay_id.is_none()
                            })
                })
                .expect("an empty adjacent crate cell inside both units' radius");
            assert!(crate::sim::crates::place_crate(
                sim,
                rules,
                &resources.overlay_registry,
                (cell.0 as i16, cell.1 as i16),
                Some(3),
            ));
            let owner = sim.entities().get(tank).unwrap().owner();
            sim.queue_command(CommandEnvelope::new(
                owner,
                sim.session.tick() + 1,
                Command::Move {
                    entity_id: tank,
                    target_rx: cell.0 as u16,
                    target_ry: cell.1 as u16,
                },
            ));
        }
        let granted = |scene: &HeadlessScenario, id| {
            scene.sim().entities().get(id).unwrap().cloak.as_ref()
                .is_some_and(|cloak| cloak.crate_granted())
        };
        for _ in 0..600 {
            tick_with_due_commands(&mut scene);
            if granted(&scene, tank) {
                break;
            }
        }
        assert!(granted(&scene, tank), "ordinary Drive must collect the Cloak crate");
        assert!(granted(&scene, infantry), "nearby infantry receives the instance ability");
        let bytes = save_scene(&scene, "AY Cloak crate");
        scene.runtime.simulation = restored_retail(&scene, &pristine, &bytes);
        assert!(granted(&scene, tank) && granted(&scene, infantry));
        for _ in 0..300 {
            tick_with_due_commands(&mut scene);
            if [tank, infantry].into_iter().all(|id| {
                scene.sim().entities().get(id).unwrap().cloak.as_ref().unwrap().is_fully_cloaked()
            }) {
                return;
            }
        }
        panic!("actual mobile cloak AI did not finish after restored crate pickup");
    }

    #[cfg(feature = "npatch")]
    #[test]
    #[ignore = "requires VERA20K_AYMOD_DIR; cloak crate building and landed aircraft ability and state"]
    fn aymod_cloak_crate_cloaks_a_building_and_landed_aircraft_across_load() {
        use crate::sim::command::{Command, CommandEnvelope};
        use crate::sim::mission::{MissionId, MissionType};
        use crate::sim::world::bridge_test_evidence::{restored_retail, save_scene};
        let (mut scene, building, plane) = aymod_nearby_pair("GAPOWR", "ORCA", true);
        let pristine = scene.sim().resolved_terrain().unwrap().clone();
        let tank = {
            let runtime = &mut scene.runtime;
            let (sim, resources) = (&mut runtime.simulation, &runtime.resources);
            let rules = &resources.rules;
            let owner = sim.entities().get(building).unwrap().owner();
            let at = sim.entities().get(building).unwrap().position;
            let center = (i32::from(at.rx()), i32::from(at.ry()));
            let tank = spawn_on_free_cells(sim, rules, owner, cells_nearest(center), &["MTNK"])[0];
            for (id, kind) in [(building, "GAPOWR"), (plane, "ORCA")] {
                assert!(!rules.object(kind).unwrap().cloakable);
                sim.mission_queue_exact(
                    id,
                    MissionId::from_known(MissionType::Sleep),
                    1,
                    sim.session.binary_frame(),
                    &crate::sim::mission::authority::LiveReadyInputProvider { rules },
                )
                .unwrap();
            }
            assert_eq!(
                crate::sim::movement::air_movement::current_fly_height(
                    sim.entities().get(plane).unwrap(),
                    sim.resolved_terrain()
                ),
                0
            );
            let recipients = [building, plane];
            let cell = cells_nearest(center)
                .into_iter()
                .find_map(|(x, y)| {
                    let (x, y) = (u16::try_from(x).ok()?, u16::try_from(y).ok()?);
                    let near = recipients.iter().all(|id| {
                        let recipient = crate::sim::movement::ground_pose::position_world_coord(
                            &sim.entities().get(*id).unwrap().position,
                        );
                        let ground = crate::sim::movement::ground_pose::position_world_coord(&at);
                        crate::util::native_x87::distance_3d_leptons(
                            [i32::from(x) * 256 + 128, i32::from(y) * 256 + 128, ground.z],
                            [recipient.x, recipient.y, recipient.z],
                        ) < rules.crate_rules.radius
                    });
                    (near
                        && sim.terrain_cell_level(x, y) == Some(at.z())
                        && sim.path_grid().unwrap().is_walkable_for_infantry(x, y)
                        && sim.occupancy().get(x, y).is_none_or(|cell| {
                            cell.is_empty_on(crate::sim::movement::locomotor::MovementLayer::Ground)
                        })
                        && sim
                            .overlay_grid
                            .as_ref()
                            .unwrap()
                            .cell(x, y)
                            .overlay_id
                            .is_none())
                    .then_some((x, y))
                })
                .expect("an empty crate cell near the building, landed plane and picker");
            assert!(crate::sim::crates::place_crate(
                sim,
                rules,
                &resources.overlay_registry,
                (cell.0 as i16, cell.1 as i16),
                Some(3)
            ));
            sim.queue_command(CommandEnvelope::new(
                owner,
                sim.session.tick() + 1,
                Command::Move {
                    entity_id: tank,
                    target_rx: cell.0,
                    target_ry: cell.1,
                },
            ));
            tank
        };
        let granted = |scene: &HeadlessScenario, id| {
            scene
                .sim()
                .entities()
                .get(id)
                .unwrap()
                .cloak
                .as_ref()
                .is_some_and(|cloak| cloak.crate_granted())
        };
        assert!(
            (0..600).any(|_| {
                tick_with_due_commands(&mut scene);
                granted(&scene, tank)
            }),
            "the tank must collect the real Cloak crate through ordinary movement"
        );
        assert!(
            granted(&scene, building),
            "the marked ground building must receive the instance ability"
        );
        assert!(
            granted(&scene, plane),
            "the landed aircraft must receive the instance ability"
        );
        let bytes = save_scene(&scene, "AY crate ability on building and landed aircraft");
        scene.runtime.simulation = restored_retail(&scene, &pristine, &bytes);
        assert!(granted(&scene, building) && granted(&scene, plane));
        assert!(
            (0..360).any(|_| {
                tick_with_due_commands(&mut scene);
                [building, plane].iter().all(|id| {
                    scene
                        .sim()
                        .entities()
                        .get(*id)
                        .unwrap()
                        .cloak
                        .as_ref()
                        .unwrap()
                        .is_fully_cloaked()
                })
            }),
            "native Building454DB0 and Aircraft Techno AI must finish their own cloak transitions: {:?}",
            [building, plane].map(|id| scene.sim().entities().get(id).map(|actor| &actor.cloak))
        );
        let bytes = save_scene(&scene, "AY fully cloaked building and landed aircraft");
        scene.runtime.simulation = restored_retail(&scene, &pristine, &bytes);
        for id in [building, plane] {
            let actor = scene.sim().entities().get(id).unwrap();
            let cloak = actor.cloak.as_ref().unwrap();
            assert!(cloak.crate_granted() && cloak.is_fully_cloaked());
            assert_eq!(
                actor.visual_character(
                    scene.runtime.resources.rules.general.cloaking_stages,
                    false,
                    crate::sim::cloak_disguise::VisualCharacterQuery::sensor(true, false, false)
                ),
                5,
                "the class visual query must hide an unsensed recipient, including Building4544A0"
            );
        }

        // Building CanAuto457770/Should4578C0 ignore own Sensors, but a
        // change of House makes the same physical neighbour foreign.
        let (dog, owner, destination) = {
            let runtime = &mut scene.runtime;
            let (sim, resources) = (&mut runtime.simulation, &runtime.resources);
            let rules = &resources.rules;
            let actor = sim.entities().get(building).unwrap();
            let owner = actor.owner();
            let at = actor.position;
            let (w, h) = rules.object("GAPOWR").unwrap().foundation.dimensions();
            assert!(rules.object("DOG").unwrap().sensors);
            let edge: Vec<_> = cells_nearest((i32::from(at.rx()), i32::from(at.ry())))
                .into_iter()
                .filter(|&(x, y)| {
                    x == i32::from(at.rx()) + i32::from(w)
                        && (i32::from(at.ry())..i32::from(at.ry()) + i32::from(h)).contains(&y)
                })
                .collect();
            let dog = spawn_on_free_cells(sim, rules, owner, edge, &["DOG"])[0];
            for id in [dog, tank] {
                sim.mission_queue_exact(
                    id,
                    MissionId::from_known(MissionType::Sleep),
                    1,
                    sim.session.binary_frame(),
                    &crate::sim::mission::authority::LiveReadyInputProvider { rules },
                )
                .unwrap();
            }
            let destination = cells_nearest((i32::from(at.rx()), i32::from(at.ry())))
                .into_iter()
                .find_map(|(x, y)| {
                    let (x, y) = (u16::try_from(x).ok()?, u16::try_from(y).ok()?);
                    let distance = (i32::from(x) - i32::from(at.rx()))
                        .abs()
                        .max((i32::from(y) - i32::from(at.ry())).abs());
                    ((6..=8).contains(&distance)
                        && sim.terrain_cell_level(x, y) == Some(at.z())
                        && sim.path_grid().unwrap().is_walkable_for_infantry(x, y)
                        && sim.occupancy().get(x, y).is_none_or(|cell| {
                            cell.is_empty_on(crate::sim::movement::locomotor::MovementLayer::Ground)
                        }))
                    .then_some((x, y))
                })
                .unwrap();
            (dog, owner, destination)
        };
        let state = |scene: &HeadlessScenario| {
            scene
                .sim()
                .entities()
                .get(building)
                .unwrap()
                .cloak
                .as_ref()
                .unwrap()
                .state()
        };
        for _ in 0..30 {
            tick_with_due_commands(&mut scene);
            assert_eq!(
                state(&scene),
                2,
                "own contact sensor must not reveal the building"
            );
        }
        // Use a human recipient: an AI legitimately sells a foreign-initial
        // building (Building4492D7), which is a separate lifecycle test.
        let country = scene
            .runtime
            .resources
            .rules
            .country_index("Russians")
            .unwrap();
        let enemy = scene.add_human_house(
            "CloakRecipient",
            LaunchCountry::from_country_index(u32::from(country.0)),
        );
        {
            let runtime = &mut scene.runtime;
            runtime.simulation.change_owner_with_rules(
                building,
                enemy,
                &runtime.resources.rules,
                Some(&runtime.resources.overlay_registry),
                Default::default(),
            );
        }
        assert!(
            granted(&scene, building),
            "ChangeOwner must retain the instance ability"
        );
        assert!(
            (0..90).any(|_| {
                tick_with_due_commands(&mut scene);
                state(&scene) == 3
            }),
            "the now-foreign contact sensor must start Building's reveal ramp"
        );
        let bytes = save_scene(
            &scene,
            "AY captured crate-cloaked building revealing near old owner's DOG",
        );
        scene.runtime.simulation = restored_retail(&scene, &pristine, &bytes);
        assert_eq!(scene.sim().entities().get(building).unwrap().owner(), enemy);
        assert!(granted(&scene, building));
        assert!(
            (0..90).any(|_| {
                tick_with_due_commands(&mut scene);
                state(&scene) == 0
            }),
            "saved reveal must reach stage zero with the neighbour still present"
        );
        for _ in 0..30 {
            tick_with_due_commands(&mut scene);
            assert_eq!(state(&scene), 0);
        }
        scene.runtime.simulation.queue_command(CommandEnvelope::new(
            owner,
            scene.sim().session.tick() + 1,
            Command::Move {
                entity_id: dog,
                target_rx: destination.0,
                target_ry: destination.1,
            },
        ));
        assert!(
            (0..600).any(|_| {
                tick_with_due_commands(&mut scene);
                state(&scene) == 2
            }),
            "ordinary detector movement away must allow the new owner's building to recloak"
        );
        let bytes = save_scene(
            &scene,
            "AY captured building recloaked after old owner's detector left",
        );
        scene.runtime.simulation = restored_retail(&scene, &pristine, &bytes);
        assert!(granted(&scene, building));
        assert_eq!(state(&scene), 2);
    }

    #[cfg(feature = "npatch")]
    #[test]
    #[ignore = "requires VERA20K_AYMOD_DIR; real AY fixed Explosion crate through ordinary Drive movement"]
    fn aymod_explosion_crate_uses_actual_rules_and_finishes_after_picker_damage() {
        use crate::sim::{
            combat::CombatLightRequest,
            command::{Command, CommandEnvelope},
        };
        let mut scenario = aymod_battle(0);
        let (actor, cell, before, c4) = {
            let runtime = &mut scenario.runtime;
            let (sim, resources) = (&mut runtime.simulation, &runtime.resources);
            let rules = &resources.rules;
            assert_eq!(rules.powerups.weights[4], 15);
            assert_eq!(f64::from_bits(rules.powerups.magnitudes[4].bits()), 500.0);
            let owner = house_of(sim, "Americans");
            let sites = cells_nearest(aymod_anchor(sim, owner));
            let actor = spawn_on_free_cells(sim, rules, owner, sites, &["MTNK"])[0];
            let at = sim.entities().get(actor).unwrap().position;
            let sites: Vec<_> = cells_nearest((i32::from(at.rx()), i32::from(at.ry())))
                .into_iter()
                .filter(|&(x, y)| {
                    (x - i32::from(at.rx()))
                        .abs()
                        .max((y - i32::from(at.ry())).abs())
                        == 2
                        && u16::try_from(x)
                            .ok()
                            .zip(u16::try_from(y).ok())
                            .is_some_and(|(x, y)| {
                                sim.terrain_cell_level(x, y) == Some(at.z())
                                    && sim
                                        .overlay_grid
                                        .as_ref()
                                        .is_some_and(|grid| grid.cell(x, y).overlay_id.is_none())
                                    && sim
                                        .path_grid()
                                        .is_some_and(|grid| grid.is_walkable_for_infantry(x, y))
                            })
                })
                .collect();
            let slot = sim
                .crate_authority
                .slots()
                .iter()
                .position(|slot| slot.is_empty())
                .unwrap();
            let (x, y) = sites[0];
            assert!(crate::sim::crates::place_crate(
                sim,
                rules,
                &resources.overlay_registry,
                (x as i16, y as i16),
                Some(4)
            ));
            let placed = sim.crate_authority.slots()[slot];
            let cell = (placed.cell_x as u16, placed.cell_y as u16);
            let overlay = sim
                .overlay_grid
                .as_ref()
                .unwrap()
                .cell(cell.0, cell.1)
                .overlay_id
                .unwrap();
            assert!(
                resources
                    .overlay_registry
                    .flags(overlay)
                    .unwrap()
                    .crate_type
            );
            let before = sim.entities().get(actor).unwrap().health.current();
            let c4 = sim.interner.intern(&rules.bridge_warheads.c4_name);
            sim.queue_command(CommandEnvelope::new(
                owner,
                sim.session.tick() + 1,
                Command::Move {
                    entity_id: actor,
                    target_rx: cell.0,
                    target_ry: cell.1,
                },
            ));
            (actor, cell, before, c4)
        };
        for _ in 0..600 {
            let output = tick_with_due_commands(&mut scenario);
            let sim = scenario.sim();
            if sim
                .overlay_grid
                .as_ref()
                .unwrap()
                .cell(cell.0, cell.1)
                .overlay_id
                .is_some()
            {
                continue;
            }
            assert!(
                sim.entities()
                    .get(actor)
                    .is_none_or(|unit| unit.health.current() < before)
            );
            let bursts=output.combat_lights.iter().filter(|request|matches!(request,
                CombatLightRequest::Impact{target_id:None,damage:500,warhead_ref,force_create:false,flags:0,..} if *warhead_ref==c4)).count();
            assert_eq!(
                bursts, 5,
                "actual pickup finishes all five sourceless bursts"
            );
            return;
        }
        panic!("actual AY Drive never reached its fixed Explosion crate");
    }

    #[cfg(feature = "npatch")]
    #[test]
    #[ignore = "requires VERA20K_AYMOD_DIR; natural building particle slot applicability in actual scenarios"]
    fn aymod_building_natural_particle_slot_is_dormant_in_battle_scenarios() {
        let root = std::path::PathBuf::from(std::env::var_os("VERA20K_AYMOD_DIR").unwrap());
        for map in [None, Some("重返迪拜.yrm")] {
            let scene = if let Some(map) = map {
                let mut launch = battle_session(
                    map,
                    &[(LaunchCountry::from_country_index(1), AiDifficulty::Hard)],
                );
                launch.local_slot_mut().country = LaunchCountry::from_country_index(0);
                launch.options.unit_count = 0;
                load_session(&root, map, 0x00C0_FFEE, launch).unwrap()
            } else {
                aymod_battle(0)
            };
            // BuildingUnlimbo440E99 requires NaturalParticleSystem; the
            // uncloak recreation454F3D first rejects the empty location.
            // No manual null slot or synthetic offset may hide a consumer.
            let active = scene
                .runtime
                .resources
                .rules
                .all_objects()
                .filter(|object| {
                    object.category == crate::rules::object_type::ObjectCategory::Building
                        && (object.natural_particle_system.is_some()
                            || object.natural_particle_location != glam::IVec3::ZERO)
                })
                .map(|object| {
                    (
                        &object.id,
                        &object.natural_particle_system,
                        object.natural_particle_location,
                    )
                })
                .collect::<Vec<_>>();
            assert!(
                active.is_empty(),
                "{}: natural particle profiles now require a live +30C owner: {active:?}",
                scene.sim().session.map_name()
            );
        }
    }

    #[cfg(feature = "npatch")]
    #[test]
    #[ignore = "requires VERA20K_AYMOD_DIR; real scenario weapon allocation closure for friendly-only warheads"]
    fn aymod_friendly_only_warhead_has_no_allocated_weapon_consumer_in_battle_maps() {
        let root = std::path::PathBuf::from(std::env::var_os("VERA20K_AYMOD_DIR").unwrap());
        for map in [None, Some("重返迪拜.yrm")] {
            let scene = if let Some(map) = map {
                let mut launch = battle_session(
                    map,
                    &[(LaunchCountry::from_country_index(1), AiDifficulty::Hard)],
                );
                launch.local_slot_mut().country = LaunchCountry::from_country_index(0);
                launch.options.unit_count = 0;
                load_session(&root, map, 0x00C0_FFEE, launch).unwrap()
            } else {
                aymod_battle(0)
            };
            let rules = &scene.runtime.resources.rules;
            let flagged = rules
                .warheads_iter()
                .filter(|wh| wh.doesnt_affect_enemies)
                .map(|wh| wh.id.as_str())
                .collect::<std::collections::BTreeSet<_>>();
            assert_eq!(flagged, std::collections::BTreeSet::from(["TankSnapshot"]));
            // RuleSet's native-ordered weapon family retains every allocated
            // reference, even orphaned prior-layer allocations (Action42 can
            // index them). This is stronger than scanning current Primary.
            let consumers = rules
                .weapons_iter()
                .filter(|weapon| {
                    weapon
                        .warhead
                        .as_deref()
                        .and_then(|id| rules.warhead(id))
                        .is_some_and(|wh| wh.doesnt_affect_enemies)
                })
                .map(|weapon| weapon.id.as_str())
                .collect::<Vec<_>>();
            assert!(
                consumers.is_empty(),
                "{}: actual allocated friendly-only consumers {consumers:?}",
                scene.sim().session.map_name()
            );
            assert!(rules.weapon("TankMakeupKit").is_none());
            assert!(rules.weapon("CRMakeupKit").is_none());
        }
    }

    #[cfg(feature = "npatch")]
    #[test]
    #[ignore = "requires VERA20K_AYMOD_DIR; actual AY Kirov warhead and barrel delayed death"]
    fn aymod_flat_explosion_blimp_warhead_arms_and_expires_barrel_fuse() {
        use crate::sim::combat::{EntityDamageEvent, ReceiverCallFlags};
        let (mut scenario, actor, barrel) = aymod_nearby_pair("ZEP", "CAMISC02", false);
        {
            let runtime = &mut scenario.runtime;
            let (sim, resources) = (&mut runtime.simulation, &runtime.resources);
            let rules = &resources.rules;
            let primary = rules.object("ZEP").unwrap().primary().unwrap();
            let warhead_name = rules.weapon(primary).unwrap().warhead.as_deref().unwrap();
            assert_eq!(warhead_name, "BlimpHE");
            assert!(rules.warhead(warhead_name).unwrap().causes_delay_kill);
            assert!(rules.object("CAMISC02").unwrap().eligible_for_delay_kill);
            let wh = sim.interner.intern(warhead_name);
            let source_owner = sim.entities().get(actor).unwrap().owner();
            let hp = sim.entities().get(barrel).unwrap().health.current();
            let hit = EntityDamageEvent::direct_receiver(
                barrel,
                hp,
                0,
                actor,
                Some(source_owner),
                wh,
                ReceiverCallFlags {
                    ignore_defenses: true,
                    arg6: true,
                },
            );
            sim.commit_direct_damage_receiver(
                rules,
                Some(&resources.overlay_registry),
                hit,
                Default::default(),
            );
            assert_eq!(
                sim.entities().get(barrel).unwrap().health.current(),
                1,
                "actual alias-authored bomb leaves the eligible barrel alive on its fuse"
            );
        }
        for _ in 0..12 {
            scenario.tick();
        }
        assert!(
            scenario
                .sim()
                .entities()
                .get(barrel)
                .is_none_or(|e| e.health.current() <= 0 || e.death.is_dying()),
            "the ordinary object AI must expire and retire the delayed barrel"
        );
    }

    #[cfg(feature = "npatch")]
    #[test]
    #[ignore = "requires VERA20K_AYMOD_DIR; actual Kirov flight, saved falling bomb and PostMortem expiry"]
    fn aymod_kirov_bomb_flight_and_delayed_barrel_death_survive_load() {
        use crate::sim::combat::TargetKind;
        use crate::sim::command::{Command, CommandEnvelope};
        use crate::sim::world::bridge_test_evidence::{restored_retail, save_scene};
        let mut scene = aymod_battle(0);
        let (actor, barrel) = {
            let runtime = &mut scene.runtime;
            let (sim, rules) = (&mut runtime.simulation, &runtime.resources.rules);
            let owner = house_of(sim, "Americans");
            let enemy = house_of(sim, "Russians");
            let actor = spawn_on_free_cells(
                sim,
                rules,
                owner,
                cells_nearest(aymod_anchor(sim, owner)),
                &["ZEP"],
            )[0];
            let at = sim.entities().get(actor).unwrap().position;
            let center = (i32::from(at.rx()), i32::from(at.ry()));
            let sites = cells_nearest(center)
                .into_iter()
                .filter(|&(x, y)| (x - center.0).abs().max((y - center.1).abs()) == 8)
                .collect::<Vec<_>>();
            let barrel = spawn_on_free_cells(sim, rules, enemy, sites, &["CAMISC02"])[0];
            (actor, barrel)
        };
        let pristine = scene.sim().resolved_terrain().unwrap().clone();
        let (owner, expected, initial_health) = {
            let runtime = &mut scene.runtime;
            let (sim, rules) = (&mut runtime.simulation, &runtime.resources.rules);
            let primary = rules
                .weapon(rules.object("ZEP").unwrap().primary().unwrap())
                .unwrap();
            assert!(
                !rules
                    .projectile(primary.projectile.as_deref().unwrap())
                    .unwrap()
                    .inviso
            );
            assert!(
                rules
                    .warhead(primary.warhead.as_deref().unwrap())
                    .unwrap()
                    .causes_delay_kill
            );
            assert!(rules.object("CAMISC02").unwrap().eligible_for_delay_kill);
            let owner = sim.entities().get(actor).unwrap().owner();
            let health = sim.entities().get(barrel).unwrap().health.current();
            sim.queue_command(CommandEnvelope::new(
                owner,
                sim.session.tick() + 1,
                Command::Attack {
                    attacker_id: actor,
                    target_id: barrel,
                },
            ));
            (owner, primary.id.clone(), health)
        };
        assert!(
            (0..900).any(|_| {
                let output = tick_with_due_commands(&mut scene);
                output.fire_events.iter().any(|event| {
                    event.attacker_id == actor
                        && scene.sim().interner.resolve(event.weapon_id) == expected
                        && event.target == TargetKind::Entity(barrel)
                })
            }),
            "normal Kirov Attack must approach and release its authored bomb"
        );
        assert!(crate::sim::movement::air_movement::is_high_flying(
            scene.sim().entities().get(actor).unwrap(),
            scene.sim().resolved_terrain(),
            Some((&scene.runtime.resources.rules, &scene.sim().interner)),
        ));
        assert_eq!(
            scene.sim().entities().get(barrel).unwrap().health.current(),
            initial_health,
            "the bomb must still be falling, not a direct receiver or instant hit"
        );
        let tick = scene.sim().session.tick() + 1;
        // Stop future releases without deleting or completing the existing
        // projectile. Its normal fall and impact remain authoritative.
        scene.runtime.simulation.queue_command(CommandEnvelope::new(
            owner,
            tick,
            Command::Stop { entity_id: actor },
        ));
        let bytes = save_scene(&scene, "AY Kirov bomb before impact");
        scene.runtime.simulation = restored_retail(&scene, &pristine, &bytes);
        assert!(
            (0..360).any(|_| {
                let output = tick_with_due_commands(&mut scene);
                assert!(
                    !output
                        .fire_events
                        .iter()
                        .any(|event| event.attacker_id == actor),
                    "the saved falling bomb, not a replacement release, must cause the hit"
                );
                scene.sim().entities().get(barrel).is_some_and(|victim| {
                    victim.health.current() == 1 && victim.pending_c4_detonation.pending().is_some()
                })
            }),
            "the actual saved bomb must hit and arm the barrel's PostMortem fuse"
        );
        let victim = scene.sim().entities().get(barrel).unwrap();
        assert!(victim.is_object_alive());
        assert_eq!(
            victim
                .pending_c4_detonation
                .pending()
                .unwrap()
                .source_entity_id(),
            None,
            "PostMortem does not invent a C4 planter from the bomb firer"
        );
        let bytes = save_scene(&scene, "AY barrel restored at health one on its fuse");
        scene.runtime.simulation = restored_retail(&scene, &pristine, &bytes);
        assert_eq!(
            scene.sim().entities().get(barrel).unwrap().health.current(),
            1
        );
        assert!(
            scene
                .sim()
                .entities()
                .get(barrel)
                .unwrap()
                .pending_c4_detonation
                .pending()
                .is_some()
        );
        assert!(
            (0..30).any(|_| {
                tick_with_due_commands(&mut scene);
                scene
                    .sim()
                    .entities()
                    .get(barrel)
                    .is_none_or(|victim| !victim.is_object_alive() || victim.death.is_dying())
            }),
            "normal building AI must expire the restored fuse and destroy the barrel"
        );
    }

    #[cfg(feature = "npatch")]
    #[test]
    #[ignore = "requires VERA20K_AYMOD_DIR; real AY bounty reader/kill-credit integration"]
    fn aymod_tanya_kill_receiver_pays_real_house_bounty() {
        use crate::sim::combat::{EntityDamageEvent, ReceiverCallFlags};
        let mut scenario = aymod_battle(0);
        let runtime = &mut scenario.runtime;
        let (sim, resources) = (&mut runtime.simulation, &runtime.resources);
        let america = house_of(sim, "Americans");
        let russia = house_of(sim, "Russians");
        let american_cells = cells_nearest(aymod_anchor(sim, america));
        let russian_cells = cells_nearest(aymod_anchor(sim, russia));
        let tanya =
            spawn_on_free_cells(sim, &resources.rules, america, american_cells, &["TANY"])[0];
        let target =
            spawn_on_free_cells(sim, &resources.rules, russia, russian_cells, &["APOC"])[0];
        let weapon = resources.rules.object("TANY").unwrap().primary().unwrap();
        let warhead = resources
            .rules
            .weapon(weapon)
            .unwrap()
            .warhead
            .as_deref()
            .unwrap();
        let warhead = sim.interner.intern(warhead);
        let health = sim.entities().get(target).unwrap().health.current();
        let before = crate::sim::credit_income::available_money(&sim.houses, america);
        let hit = EntityDamageEvent::direct_receiver(
            target,
            health,
            0,
            tanya,
            Some(america),
            warhead,
            ReceiverCallFlags {
                ignore_defenses: true,
                arg6: true,
            },
        );
        sim.commit_direct_damage_receiver(
            &resources.rules,
            Some(&resources.overlay_registry),
            hit,
            Default::default(),
        );
        assert!(
            sim.entities()
                .get(target)
                .is_none_or(|entity| entity.health.current() <= 0 || entity.death.is_dying())
        );
        assert!(
            crate::sim::credit_income::available_money(&sim.houses, america) > before,
            "the real AY Tanya kill must pay spendable credits, not only score or experience"
        );
    }

    #[cfg(feature = "npatch")]
    #[test]
    #[ignore = "requires VERA20K_AYMOD_DIR; real AY Tanya cloak and firing"]
    fn aymod_tanya_cloaks_and_fires_without_decloaking() {
        use crate::sim::command::{Command, CommandEnvelope};
        let mut scenario = aymod_battle(0);
        let actor = {
            let runtime = &mut scenario.runtime;
            let sim = &mut runtime.simulation;
            let owner = house_of(sim, "Americans");
            let sites = cells_nearest(aymod_anchor(sim, owner));
            assert!(runtime.resources.rules.object("TANY").unwrap().cloakable);
            spawn_on_free_cells(sim, &runtime.resources.rules, owner, sites, &["TANY"])[0]
        };
        for _ in 0..180 {
            tick_with_due_commands(&mut scenario);
            if scenario
                .sim()
                .entities()
                .get(actor)
                .unwrap()
                .cloak
                .as_ref()
                .unwrap()
                .state()
                == 2
            {
                break;
            }
        }
        assert_eq!(
            scenario
                .sim()
                .entities()
                .get(actor)
                .unwrap()
                .cloak
                .as_ref()
                .unwrap()
                .state(),
            2
        );
        let (target, health) = {
            let runtime = &mut scenario.runtime;
            let sim = &mut runtime.simulation;
            let owner = sim.entities().get(actor).unwrap().owner();
            let enemy = house_of(sim, "Russians");
            let at = sim.entities().get(actor).unwrap().position;
            let sites = cells_nearest((i32::from(at.rx()), i32::from(at.ry())));
            let target =
                spawn_on_free_cells(sim, &runtime.resources.rules, enemy, sites, &["E1"])[0];
            let health = sim.entities().get(target).unwrap().health.current();
            sim.queue_command(CommandEnvelope::new(
                owner,
                sim.session.tick() + 1,
                Command::Attack {
                    attacker_id: actor,
                    target_id: target,
                },
            ));
            (target, health)
        };
        for _ in 0..180 {
            tick_with_due_commands(&mut scenario);
            assert_eq!(
                scenario
                    .sim()
                    .entities()
                    .get(actor)
                    .unwrap()
                    .cloak
                    .as_ref()
                    .unwrap()
                    .state(),
                2,
                "AY Tanya's actual DecloakToFire=no weapon must not surface her"
            );
            if scenario
                .sim()
                .entities()
                .get(target)
                .is_none_or(|entity| entity.health.current() < health)
            {
                return;
            }
        }
        panic!("the cloaked Tanya never damaged her target through the actual Attack mission");
    }

    #[cfg(feature = "npatch")]
    #[test]
    #[ignore = "requires VERA20K_AYMOD_DIR; detector cargo limbo and unload restore sensor coverage"]
    fn aymod_transported_ifv_removes_and_restores_sensor_coverage_after_load() {
        use crate::sim::command::Command;
        use crate::sim::mission::{MissionId, MissionType};
        use crate::sim::world::bridge_test_evidence::{restored_retail, save_scene};
        let (mut scene, cloaker, observer) = aymod_nearby_pair("TANY", "GAPILL", false);
        let pristine = scene.sim().resolved_terrain().unwrap().clone();
        {
            let runtime = &mut scene.runtime;
            let (sim, rules) = (&mut runtime.simulation, &runtime.resources.rules);
            for id in [cloaker, observer] {
                sim.mission_queue_exact(
                    id,
                    MissionId::from_known(MissionType::Sleep),
                    1,
                    sim.session.binary_frame(),
                    &crate::sim::mission::authority::LiveReadyInputProvider { rules },
                )
                .unwrap();
            }
        }
        assert!((0..360).any(|_| {
            tick_with_due_commands(&mut scene);
            scene
                .sim()
                .entities()
                .get(cloaker)
                .unwrap()
                .cloak
                .as_ref()
                .unwrap()
                .is_fully_cloaked()
        }));
        let (sensor, carrier, enemy, cell) = {
            let runtime = &mut scene.runtime;
            let (sim, rules) = (&mut runtime.simulation, &runtime.resources.rules);
            let at = sim.entities().get(cloaker).unwrap().position;
            let enemy = sim.entities().get(observer).unwrap().owner();
            let center = (i32::from(at.rx()), i32::from(at.ry()));
            let sites = cells_nearest(center)
                .into_iter()
                .filter(|&(x, y)| (x - center.0).abs().max((y - center.1).abs()) == 3)
                .collect::<Vec<_>>();
            let members = spawn_on_free_cells(sim, rules, enemy, sites, &["FV", "LCRF"]);
            let (sensor, carrier) = (members[0], members[1]);
            assert!(rules.object("FV").unwrap().sensors_sight > 0);
            assert_eq!(rules.object("LCRF").unwrap().sensors_sight, 0);
            assert!(!rules.object("GAPILL").unwrap().sensors);
            assert!(rules.object("FV").unwrap().size <= rules.object("LCRF").unwrap().size_limit);
            for id in members {
                sim.mission_queue_exact(
                    id,
                    MissionId::from_known(MissionType::Sleep),
                    1,
                    sim.session.binary_frame(),
                    &crate::sim::mission::authority::LiveReadyInputProvider { rules },
                )
                .unwrap();
            }
            (sensor, carrier, enemy, (at.rx(), at.ry()))
        };
        let count = |scene: &HeadlessScenario| {
            let fog = &scene.sim().fog;
            fog.sensors_by_house()
                .get(&enemy)
                .and_then(|values| {
                    values.get(usize::from(cell.1) * usize::from(fog.width()) + usize::from(cell.0))
                })
                .copied()
                .unwrap_or(0)
        };
        let scan = |scene: &mut HeadlessScenario| {
            let runtime = &mut scene.runtime;
            runtime.simulation.greatest_threat_represented(
                &runtime.resources.rules,
                Some(&runtime.resources.overlay_registry),
                observer,
                crate::sim::combat::threat_range::ScanMission::Guard,
                None,
            )
        };
        let issue = |scene: &mut HeadlessScenario, command| {
            let runtime = &mut scene.runtime;
            let owner = runtime.simulation.interner.resolve(enemy).to_owned();
            assert!(runtime.simulation.apply_command(
                &owner,
                &command,
                Some(&runtime.resources.rules)
            ));
        };
        assert!((0..30).any(|_| {
            tick_with_due_commands(&mut scene);
            count(&scene) == 1
        }));
        assert_eq!(scan(&mut scene), Some(cloaker));
        // Unit PerCell73A5EA..73A7CF attaches only after ordinary
        // Limbo; FootLimbo4DB376 removes its current sensor circle first.
        issue(
            &mut scene,
            Command::clicked_mission(
                sensor,
                MissionType::Enter,
                crate::sim::combat::TargetKind::Entity(carrier),
            ),
        );
        assert!(
            (0..900).any(|_| {
                tick_with_due_commands(&mut scene);
                scene
                    .sim()
                    .entities()
                    .get(sensor)
                    .unwrap()
                    .passenger_role
                    .inside_transport_id()
                    == Some(carrier)
            }),
            "the real IFV must drive and board LCRF through ordinary Enter"
        );
        assert_eq!(count(&scene), 0);
        assert!(
            scene
                .sim()
                .entities()
                .get(sensor)
                .unwrap()
                .lifecycle
                .in_limbo()
        );
        assert!(
            scene
                .sim()
                .entities()
                .get(sensor)
                .unwrap()
                .sensor_deposit
                .is_none()
        );
        assert_ne!(scan(&mut scene), Some(cloaker));
        let bytes = save_scene(&scene, "AY sensor vehicle inside a transport");
        scene.runtime.simulation = restored_retail(&scene, &pristine, &bytes);
        assert_eq!(count(&scene), 0);
        assert_eq!(
            scene
                .sim()
                .entities()
                .get(carrier)
                .unwrap()
                .passenger_role
                .cargo()
                .unwrap()
                .passengers(),
            &[sensor]
        );
        assert!(
            scene
                .sim()
                .entities()
                .get(sensor)
                .unwrap()
                .sensor_deposit
                .is_none()
        );
        issue(
            &mut scene,
            Command::UnloadPassengers {
                transport_id: carrier,
            },
        );
        assert!(
            (0..600).any(|_| {
                tick_with_due_commands(&mut scene);
                scene
                    .sim()
                    .entities()
                    .get(sensor)
                    .unwrap()
                    .passenger_role
                    .inside_transport_id()
                    .is_none()
            }),
            "ordinary Unload must reveal the sensor vehicle"
        );
        assert_eq!(
            count(&scene),
            1,
            "FootUnlimbo4D7318 must add exactly one fresh footprint: sensor={:?}, carrier={:?}, sampled={cell:?}",
            scene.sim().entities().get(sensor).map(|actor| (
                &actor.position,
                &actor.sensor_deposit,
                &actor.passenger_role,
                &actor.mission
            )),
            scene.sim().entities().get(carrier).map(|actor| (
                &actor.position,
                &actor.navigation,
                &actor.mission
            ))
        );
        assert_eq!(
            scene
                .sim()
                .entities()
                .get(sensor)
                .unwrap()
                .sensor_deposit
                .unwrap()
                .owner(),
            enemy
        );
        assert!(
            scene
                .sim()
                .entities()
                .get(sensor)
                .unwrap()
                .is_object_alive()
        );
        assert_eq!(scan(&mut scene), Some(cloaker));
        let bytes = save_scene(&scene, "AY detector after leaving transport");
        scene.runtime.simulation = restored_retail(&scene, &pristine, &bytes);
        assert_eq!(count(&scene), 1);
        assert_eq!(
            scene
                .sim()
                .entities()
                .get(carrier)
                .unwrap()
                .passenger_role
                .passenger_count(),
            0
        );
        assert_eq!(scan(&mut scene), Some(cloaker));
    }

    #[cfg(feature = "npatch")]
    #[test]
    #[ignore = "requires VERA20K_AYMOD_DIR; actual mind control transfers sensor coverage and death removes it"]
    fn aymod_sensor_capture_return_and_death_update_ai_cloak_admission_after_load() {
        use crate::sim::combat::{EntityDamageEvent, RAD_NO_ATTACKER, ReceiverCallFlags};
        use crate::sim::command::{Command, CommandEnvelope};
        use crate::sim::mission::{MissionId, MissionType};
        use crate::sim::world::bridge_test_evidence::{restored_retail, save_scene};
        let (mut scene, cloaker, observer) = aymod_nearby_pair("TANY", "GAPILL", false);
        let pristine = scene.sim().resolved_terrain().unwrap().clone();
        {
            let runtime = &mut scene.runtime;
            let (sim, rules) = (&mut runtime.simulation, &runtime.resources.rules);
            for id in [cloaker, observer] {
                sim.mission_queue_exact(
                    id,
                    MissionId::from_known(MissionType::Sleep),
                    1,
                    sim.session.binary_frame(),
                    &crate::sim::mission::authority::LiveReadyInputProvider { rules },
                )
                .unwrap();
            }
        }
        assert!((0..360).any(|_| {
            tick_with_due_commands(&mut scene);
            scene
                .sim()
                .entities()
                .get(cloaker)
                .unwrap()
                .cloak
                .as_ref()
                .unwrap()
                .is_fully_cloaked()
        }));
        let (sensor, controller, owner, enemy, cell) = {
            let runtime = &mut scene.runtime;
            let (sim, rules) = (&mut runtime.simulation, &runtime.resources.rules);
            let owner = sim.entities().get(cloaker).unwrap().owner();
            let enemy = sim.entities().get(observer).unwrap().owner();
            let at = sim.entities().get(cloaker).unwrap().position;
            assert!(rules.object("FV").unwrap().sensors);
            assert!(!rules.object("GAPILL").unwrap().sensors);
            let center = (i32::from(at.rx()), i32::from(at.ry()));
            let sites = cells_nearest(center)
                .into_iter()
                .filter(|&(x, y)| (x - center.0).abs().max((y - center.1).abs()) == 3)
                .collect::<Vec<_>>();
            let sensor = spawn_on_free_cells(sim, rules, enemy, sites, &["FV"])[0];
            let at_sensor = sim.entities().get(sensor).unwrap().position;
            let center = (i32::from(at_sensor.rx()), i32::from(at_sensor.ry()));
            let sites = cells_nearest(center)
                .into_iter()
                .filter(|&(x, y)| (2..=3).contains(&(x - center.0).abs().max((y - center.1).abs())))
                .collect::<Vec<_>>();
            let controller = spawn_on_free_cells(sim, rules, owner, sites, &["YURI"])[0];
            for id in [sensor, controller] {
                sim.mission_queue_exact(
                    id,
                    MissionId::from_known(MissionType::Sleep),
                    1,
                    sim.session.binary_frame(),
                    &crate::sim::mission::authority::LiveReadyInputProvider { rules },
                )
                .unwrap();
            }
            (sensor, controller, owner, enemy, (at.rx(), at.ry()))
        };
        let count = |scene: &HeadlessScenario, house| {
            let fog = &scene.sim().fog;
            fog.sensors_by_house()
                .get(&house)
                .and_then(|values| {
                    values.get(usize::from(cell.1) * usize::from(fog.width()) + usize::from(cell.0))
                })
                .copied()
                .unwrap_or(0)
        };
        let scan = |scene: &mut HeadlessScenario| {
            let runtime = &mut scene.runtime;
            runtime.simulation.greatest_threat_represented(
                &runtime.resources.rules,
                Some(&runtime.resources.overlay_registry),
                observer,
                crate::sim::combat::threat_range::ScanMission::Guard,
                None,
            )
        };
        assert!((0..30).any(|_| {
            tick_with_due_commands(&mut scene);
            count(&scene, enemy) == 1
        }));
        assert_eq!(scan(&mut scene), Some(cloaker));
        let original_own_count = count(&scene, owner);
        let tick = scene.sim().session.tick() + 1;
        scene.runtime.simulation.queue_command(CommandEnvelope::new(
            owner,
            tick,
            Command::Attack {
                attacker_id: controller,
                target_id: sensor,
            },
        ));
        assert!(
            (0..240).any(|_| {
                tick_with_due_commands(&mut scene);
                scene.sim().entities().get(sensor).unwrap().owner() == owner
            }),
            "actual YURI firing must capture the IFV, not a raw owner store"
        );
        assert_eq!(
            scene
                .sim()
                .entities()
                .get(sensor)
                .unwrap()
                .mind_control
                .controller(),
            Some(controller)
        );
        assert_eq!(
            scene
                .sim()
                .entities()
                .get(sensor)
                .unwrap()
                .sensor_deposit
                .unwrap()
                .owner(),
            owner
        );
        assert_eq!(count(&scene, enemy), 0);
        assert_eq!(count(&scene, owner), original_own_count + 1);
        assert_ne!(scan(&mut scene), Some(cloaker));
        let bytes = save_scene(&scene, "AY captured IFV sensor coverage");
        scene.runtime.simulation = restored_retail(&scene, &pristine, &bytes);
        assert_eq!(count(&scene, enemy), 0);
        assert_eq!(count(&scene, owner), original_own_count + 1);
        let kill = |scene: &mut HeadlessScenario, id| {
            let runtime = &mut scene.runtime;
            let (sim, rules) = (&mut runtime.simulation, &runtime.resources.rules);
            let damage = sim.entities().get(id).unwrap().health.current() + 1;
            let warhead = sim.interner.intern("Super");
            sim.commit_direct_damage_receiver(
                rules,
                Some(&runtime.resources.overlay_registry),
                EntityDamageEvent::direct_receiver(
                    id,
                    damage,
                    0,
                    RAD_NO_ATTACKER,
                    None,
                    warhead,
                    ReceiverCallFlags {
                        ignore_defenses: true,
                        arg6: false,
                    },
                ),
                Default::default(),
            );
        };
        // The real Yuri can contribute its own sensor circle too. Removing
        // that source must not mask a leaked IFV contribution on return.
        let controller_circle = scene
            .sim()
            .entities()
            .get(controller)
            .unwrap()
            .sensor_deposit;
        let controller_contribution = i16::from(controller_circle.is_some_and(|deposit| {
            scene
                .sim()
                .fog
                .sensor_circle_cells(deposit.center(), deposit.add_radius())
                .contains(&cell)
        }));
        let remaining_own_count = original_own_count - controller_contribution;
        // Techno's normal death owner FreeAllCaptures releases the real
        // manager link and invokes the ordinary Foot ChangeOwner receiver.
        kill(&mut scene, controller);
        assert_eq!(scene.sim().entities().get(sensor).unwrap().owner(), enemy);
        assert_eq!(
            scene
                .sim()
                .entities()
                .get(sensor)
                .unwrap()
                .mind_control
                .controller(),
            None
        );
        assert_eq!(
            scene
                .sim()
                .entities()
                .get(sensor)
                .unwrap()
                .sensor_deposit
                .unwrap()
                .owner(),
            enemy
        );
        assert_eq!(count(&scene, enemy), 1);
        assert_eq!(count(&scene, owner), remaining_own_count);
        assert_eq!(scan(&mut scene), Some(cloaker));
        kill(&mut scene, sensor);
        assert_eq!(
            count(&scene, enemy),
            0,
            "UnInit must remove the last hostile sensor footprint"
        );
        assert_ne!(scan(&mut scene), Some(cloaker));
        tick_with_due_commands(&mut scene);
        let bytes = save_scene(&scene, "AY IFV detector removed after controller death");
        scene.runtime.simulation = restored_retail(&scene, &pristine, &bytes);
        assert_eq!(count(&scene, enemy), 0);
        assert_eq!(count(&scene, owner), remaining_own_count);
        assert!(scene.sim().entities().get(sensor).is_none());
        assert_ne!(scan(&mut scene), Some(cloaker));
    }

    #[cfg(feature = "npatch")]
    #[test]
    #[ignore = "requires VERA20K_AYMOD_DIR; moving AI sensors, cloak acquisition and real load"]
    fn aymod_mobile_sensors_update_cloaked_infantry_ai_acquisition_after_load() {
        use crate::sim::command::Command;
        use crate::sim::mission::{MissionId, MissionType};
        use crate::sim::world::bridge_test_evidence::{restored_retail, save_scene};
        let (mut scene, cloaker, observer) = aymod_nearby_pair("TANY", "GAPILL", false);
        let pristine = scene.sim().resolved_terrain().unwrap().clone();
        let (sensor, home, near, enemy, target_cell) = {
            let runtime = &mut scene.runtime;
            let (sim, rules) = (&mut runtime.simulation, &runtime.resources.rules);
            for id in [cloaker, observer] {
                // Keep combat out of the sensor-edge probes; wake the actual
                // computer observer on Guard only after rediscovery below.
                sim.mission_queue_exact(
                    id,
                    MissionId::from_known(MissionType::Sleep),
                    1,
                    sim.session.binary_frame(),
                    &crate::sim::mission::authority::LiveReadyInputProvider { rules },
                )
                .unwrap();
            }
            let at = sim.entities().get(cloaker).unwrap().position;
            let enemy = sim.entities().get(observer).unwrap().owner();
            assert!(!sim.houses.get(&enemy).unwrap().event_dispatch_eligible());
            let radius = i32::from(rules.object("DOG").unwrap().sensors_sight);
            assert!(radius > 0 && rules.object("DOG").unwrap().sensors);
            assert!(!rules.object("GAPILL").unwrap().sensors);
            let center = (i32::from(at.rx()), i32::from(at.ry()));
            let far = cells_nearest(center)
                .into_iter()
                .filter(|&(x, y)| {
                    let distance = (x - center.0).abs().max((y - center.1).abs());
                    (radius + 3..=radius + 5).contains(&distance)
                })
                .collect::<Vec<_>>();
            let sensor = spawn_on_free_cells(sim, rules, enemy, far, &["DOG"])[0];
            let home_at = sim.entities().get(sensor).unwrap().position;
            let home = (home_at.rx(), home_at.ry());
            let near = cells_nearest(center)
                .into_iter()
                .find_map(|(x, y)| {
                    if (x - center.0).abs().max((y - center.1).abs()) != 3 {
                        return None;
                    }
                    let (x, y) = (u16::try_from(x).ok()?, u16::try_from(y).ok()?);
                    if sim.occupancy().get(x, y).is_some_and(|cell| {
                        !cell.is_empty_on(crate::sim::movement::locomotor::MovementLayer::Ground)
                    }) {
                        return None;
                    }
                    sim.foot_path_zone_precheck(
                        sensor,
                        crate::sim::components::DriveCoord::cell(x, y, 0),
                        rules,
                    )
                    .unwrap()
                    .then_some((x, y))
                })
                .expect("a reachable sensor destination near the cloaked infantry");
            (sensor, home, near, enemy, (at.rx(), at.ry()))
        };
        let sensed = |scene: &HeadlessScenario| {
            scene
                .sim()
                .fog
                .has_sensor_for_house(enemy, target_cell.0, target_cell.1)
        };
        let scan = |scene: &mut HeadlessScenario| {
            let runtime = &mut scene.runtime;
            runtime.simulation.greatest_threat_represented(
                &runtime.resources.rules,
                Some(&runtime.resources.overlay_registry),
                observer,
                crate::sim::combat::threat_range::ScanMission::Guard,
                None,
            )
        };
        assert_eq!(
            scan(&mut scene),
            Some(cloaker),
            "uncloaked fixture must be eligible before testing sensors"
        );
        for _ in 0..180 {
            tick_with_due_commands(&mut scene);
            if scene
                .sim()
                .entities()
                .get(cloaker)
                .unwrap()
                .cloak
                .as_ref()
                .unwrap()
                .is_fully_cloaked()
            {
                break;
            }
        }
        assert!(
            scene
                .sim()
                .entities()
                .get(cloaker)
                .unwrap()
                .cloak
                .as_ref()
                .unwrap()
                .is_fully_cloaked()
        );
        assert!(!sensed(&scene));
        assert_ne!(scan(&mut scene), Some(cloaker));
        let move_sensor = |scene: &mut HeadlessScenario, goal: (u16, u16)| {
            let runtime = &mut scene.runtime;
            let owner = runtime.simulation.interner.resolve(enemy).to_owned();
            // Supply the opponent's movement intent at the real command
            // payload boundary; this does not claim AI planning coverage.
            assert!(runtime.simulation.apply_command(
                &owner,
                &Command::Move {
                    entity_id: sensor,
                    target_rx: goal.0,
                    target_ry: goal.1,
                },
                Some(&runtime.resources.rules)
            ));
        };
        move_sensor(&mut scene, near);
        assert!((0..480).any(|_| {
            tick_with_due_commands(&mut scene);
            sensed(&scene)
        }));
        assert!(
            scene
                .sim()
                .entities()
                .get(cloaker)
                .unwrap()
                .cloak
                .as_ref()
                .unwrap()
                .is_fully_cloaked(),
            "sensor deposits reveal to a house, not globally uncloak the infantry"
        );
        assert_eq!(scan(&mut scene), Some(cloaker));
        let bytes = save_scene(&scene, "AY moving sensor over hidden infantry");
        scene.runtime.simulation = restored_retail(&scene, &pristine, &bytes);
        assert!(sensed(&scene));
        assert_eq!(scan(&mut scene), Some(cloaker));
        move_sensor(&mut scene, home);
        assert!((0..480).any(|_| {
            tick_with_due_commands(&mut scene);
            !sensed(&scene)
        }));
        assert_ne!(scan(&mut scene), Some(cloaker));
        move_sensor(&mut scene, near);
        assert!((0..480).any(|_| {
            tick_with_due_commands(&mut scene);
            sensed(&scene)
        }));
        assert_eq!(scan(&mut scene), Some(cloaker));
        let runtime = &mut scene.runtime;
        runtime
            .simulation
            .mission_queue_exact(
                observer,
                MissionId::from_known(MissionType::Guard),
                1,
                runtime.simulation.session.binary_frame(),
                &crate::sim::mission::authority::LiveReadyInputProvider {
                    rules: &runtime.resources.rules,
                },
            )
            .unwrap();
        assert!(
            (0..180).any(|_| {
                let output = tick_with_due_commands(&mut scene);
                output.fire_events.iter().any(|event| {
                    event.attacker_id == observer
                        && event.target == crate::sim::combat::TargetKind::Entity(cloaker)
                })
            }),
            "the actual computer pillbox must acquire and fire after sensor rediscovery"
        );
        // ReceiveDamage70281D surfaces a damaged cloaker. Pause the observer
        // through the mission owner so normal recloaking can happen; merely
        // removing sensors does not make a damaged, visible target invisible.
        let runtime = &mut scene.runtime;
        let owner = runtime.simulation.interner.resolve(enemy).to_owned();
        assert!(runtime.simulation.apply_command(
            &owner,
            &Command::Stop {
                entity_id: observer
            },
            Some(&runtime.resources.rules),
        ));
        runtime
            .simulation
            .mission_queue_exact(
                observer,
                MissionId::from_known(MissionType::Sleep),
                1,
                runtime.simulation.session.binary_frame(),
                &crate::sim::mission::authority::LiveReadyInputProvider {
                    rules: &runtime.resources.rules,
                },
            )
            .unwrap();
        move_sensor(&mut scene, home);
        assert!((0..480).any(|_| {
            tick_with_due_commands(&mut scene);
            !sensed(&scene)
        }));
        // A normal AI dog resumes hunting visible infantry after reaching
        // its destination. Park it outside the sensor radius while Tanya
        // finishes her authored, slow recloak rather than letting it bite.
        let runtime = &mut scene.runtime;
        let owner = runtime.simulation.interner.resolve(enemy).to_owned();
        assert!(runtime.simulation.apply_command(
            &owner,
            &Command::Stop { entity_id: sensor },
            Some(&runtime.resources.rules),
        ));
        runtime
            .simulation
            .mission_queue_exact(
                sensor,
                MissionId::from_known(MissionType::Sleep),
                1,
                runtime.simulation.session.binary_frame(),
                &crate::sim::mission::authority::LiveReadyInputProvider {
                    rules: &runtime.resources.rules,
                },
            )
            .unwrap();
        let bytes = save_scene(&scene, "AY sensor gone after passive acquisition");
        scene.runtime.simulation = restored_retail(&scene, &pristine, &bytes);
        assert!(
            !sensed(&scene),
            "load must not resurrect the old sensor deposit"
        );
        assert!(
            (0..360).any(|_| {
                let output = tick_with_due_commands(&mut scene);
                assert!(
                    !output
                        .fire_events
                        .iter()
                        .any(|e| e.attacker_id == observer || e.attacker_id == sensor),
                    "parked observer/detector must not interrupt natural recloaking"
                );
                scene
                    .sim()
                    .entities()
                    .get(cloaker)
                    .unwrap()
                    .cloak
                    .as_ref()
                    .unwrap()
                    .is_fully_cloaked()
            }),
            "the damaged infantry must naturally recloak without continuing fire"
        );
        assert_ne!(
            scene
                .sim()
                .entities()
                .get(observer)
                .unwrap()
                .attack_target
                .target(),
            Some(crate::sim::combat::TargetKind::Entity(cloaker)),
            "the stopped observer must not acquire the unsensed infantry"
        );
        let runtime = &mut scene.runtime;
        runtime
            .simulation
            .mission_queue_exact(
                observer,
                MissionId::from_known(MissionType::Guard),
                1,
                runtime.simulation.session.binary_frame(),
                &crate::sim::mission::authority::LiveReadyInputProvider {
                    rules: &runtime.resources.rules,
                },
            )
            .unwrap();
        for _ in 0..60 {
            let output = tick_with_due_commands(&mut scene);
            assert!(
                !output
                    .fire_events
                    .iter()
                    .any(|event| event.attacker_id == observer
                        && event.target == crate::sim::combat::TargetKind::Entity(cloaker)),
                "an unsensed full cloak must stop subsequent actual fire"
            );
        }
        assert_ne!(scan(&mut scene), Some(cloaker));
        move_sensor(&mut scene, near);
        assert!(
            (0..480).any(|_| {
                tick_with_due_commands(&mut scene);
                sensed(&scene)
            }),
            "parked sensor failed to return: {:?}",
            scene
                .sim()
                .entities()
                .get(sensor)
                .map(|e| (&e.mission, &e.position, &e.attack_target))
        );
        assert_eq!(scan(&mut scene), Some(cloaker));
        // Guard may have legitimately acquired the nearby starting MCV
        // while Tanya was hidden. Clear that admitted target with Stop;
        // rediscovery is not a rule forcing it to abandon an existing target.
        let runtime = &mut scene.runtime;
        let owner = runtime.simulation.interner.resolve(enemy).to_owned();
        assert!(runtime.simulation.apply_command(
            &owner,
            &Command::Stop {
                entity_id: observer
            },
            Some(&runtime.resources.rules),
        ));
        assert!(
            (0..180).any(|_| {
                let output = tick_with_due_commands(&mut scene);
                output.fire_events.iter().any(|event| {
                    event.attacker_id == observer
                        && event.target == crate::sim::combat::TargetKind::Entity(cloaker)
                })
            }),
            "ordinary Stop/Guard failed after the mobile sensor returned: {:?}",
            scene
                .sim()
                .entities()
                .get(observer)
                .map(|e| (&e.mission, &e.attack_target))
        );
    }

    // NPAE174 Assign_Target6FCF5B and GetFireErrorC2C649 preserve the
    // burst across player orders and gate it on the saved short interval.
    #[cfg(feature = "npatch")]
    #[test]
    #[ignore = "requires VERA20K_AYMOD_DIR; real AY TTNK burst orders and validated save/resume"]
    fn aymod_tesla_burst_survives_stop_guard_and_save_resume() {
        use crate::sim::command::{Command, CommandEnvelope};
        use crate::sim::world::bridge_test_evidence::{restored_retail, save_scene};
        for guard in [false, true] {
            let (mut scene, actor, target) = aymod_nearby_pair("TTNK", "APOC", false);
            let pristine = scene.sim().resolved_terrain().unwrap().clone();
            let rules = &scene.runtime.resources.rules;
            let kind = rules.object("TTNK").unwrap();
            let weapon = rules.weapon(kind.primary().unwrap()).unwrap();
            assert_eq!(
                (weapon.id.as_str(), weapon.burst, weapon.rof),
                ("TankBolt", 2, 50)
            );
            assert_eq!(&kind.burst_delays[..2], &[15, 45]);
            assert!(!kind.use_rof_as_burst_delays());
            let queue = |scene: &mut HeadlessScenario, command| {
                let sim = &mut scene.runtime.simulation;
                let owner = sim.entities().get(actor).unwrap().owner();
                sim.queue_command(CommandEnvelope::new(owner, sim.session.tick() + 1, command));
            };
            let attack = || Command::Attack {
                attacker_id: actor,
                target_id: target,
            };
            queue(&mut scene, attack());
            (0..240)
                .find(|_| {
                    tick_with_due_commands(&mut scene)
                        .fire_events
                        .iter()
                        .any(|event| event.attacker_id == actor)
                })
                .expect("actual TTNK Attack must fire");
            let first = scene.sim().entities().get(actor).unwrap();
            let first_frame = first.rearm_timer.start_frame();
            assert_eq!(
                (first.rearm_timer.duration(), first.saved_rearm_delay()),
                (50, 15)
            );
            assert_eq!(first.weapon_burst.index(), 1);
            queue(
                &mut scene,
                if guard {
                    Command::Guard {
                        entity_id: actor,
                        target: None,
                    }
                } else {
                    Command::Stop { entity_id: actor }
                },
            );
            for _ in 0..2 {
                assert!(
                    !tick_with_due_commands(&mut scene)
                        .fire_events
                        .iter()
                        .any(|event| event.attacker_id == actor)
                );
            }
            let stopped = scene.sim().entities().get(actor).unwrap();
            assert_eq!(
                stopped.weapon_burst.index(),
                1,
                "player order must retain mid-burst state"
            );
            assert_eq!(stopped.rearm_timer.start_frame(), first_frame);
            let bytes = save_scene(&scene, "TTNK mid-burst");
            scene.runtime.simulation = restored_retail(&scene, &pristine, &bytes);
            let loaded = scene.sim().entities().get(actor).unwrap();
            assert_eq!(
                (
                    loaded.rearm_timer.start_frame(),
                    loaded.rearm_timer.duration(),
                    loaded.saved_rearm_delay(),
                    loaded.weapon_burst.index()
                ),
                (first_frame, 50, 15, 1)
            );
            assert!(loaded.burst_rearm_remaining(scene.sim().session.binary_frame() as i32) > 0);
            queue(&mut scene, attack());
            let mut shots = 1;
            let mut previous = first_frame;
            for _ in 0..240 {
                let output = tick_with_due_commands(&mut scene);
                if !output
                    .fire_events
                    .iter()
                    .any(|event| event.attacker_id == actor)
                {
                    continue;
                }
                shots += 1;
                let fired = scene.sim().entities().get(actor).unwrap();
                let frame = fired.rearm_timer.start_frame();
                assert!(frame - previous >= if shots == 2 { 15 } else { 50 });
                if shots == 2 {
                    assert!(
                        frame - first_frame < 50,
                        "second shot must use short, not full reload"
                    );
                    assert_eq!(
                        (fired.weapon_burst.index(), fired.saved_rearm_delay()),
                        (0, 50)
                    );
                } else {
                    assert_eq!(
                        (fired.weapon_burst.index(), fired.saved_rearm_delay()),
                        (1, 15)
                    );
                }
                assert_eq!(fired.rearm_timer.duration(), 50);
                previous = frame;
                if shots == 3 {
                    break;
                }
            }
            assert_eq!(
                shots, 3,
                "restored Attack must complete a burst and start the next"
            );
        }
    }

    #[cfg(feature = "npatch")]
    #[test]
    #[ignore = "requires VERA20K_AYMOD_DIR; real AY TankBolt normal/elite and explicit-weapon colors"]
    fn aymod_tesla_bolt_colors_follow_the_birth_weapon() {
        use crate::sim::combat::electric_bolt;
        use crate::sim::command::{Command, CommandEnvelope};
        use crate::sim::world::LifecycleOutput;
        let ordinary = [[128, 160, 255], [128, 40, 255], [128, 160, 255]];
        let elite = [[160, 160, 255], [255, 192, 255], [160, 160, 255]];
        for (promoted, expected, direct_weapon, direct_colors) in [
            (false, ordinary, "TankBoltE", elite),
            (true, elite, "TankBolt", ordinary),
        ] {
            let (mut scenario, actor, target) = aymod_nearby_pair("TTNK", "APOC", false);
            let sim = &mut scenario.runtime.simulation;
            let entity = sim.substrate.entities.get_mut(actor).unwrap();
            if promoted {
                entity.veterancy.set_elite();
            }
            let owner = entity.owner();
            sim.queue_command(CommandEnvelope::new(
                owner,
                sim.session.tick() + 1,
                Command::Attack {
                    attacker_id: actor,
                    target_id: target,
                },
            ));
            let birth = (0..240)
                .find_map(|_| {
                    tick_with_due_commands(&mut scenario)
                        .lifecycle_outputs
                        .into_iter()
                        .find_map(|output| match output {
                            LifecycleOutput::ElectricBoltCreated(birth)
                                if birth.owner.is_some_and(|owner| owner.unit == actor) =>
                            {
                                Some(birth)
                            }
                            _ => None,
                        })
                })
                .expect("actual TTNK Attack must create its owned electric bolt");
            assert_eq!(birth.custom_colors, Some(expected));
            let runtime = &mut scenario.runtime;
            let (sim, rules) = (&mut runtime.simulation, &runtime.resources.rules);
            electric_bolt::create(
                sim,
                rules,
                actor,
                crate::sim::projectile::ProjectileTarget::Entity(target),
                rules.weapon(direct_weapon).unwrap(),
                birth.from,
                None,
            );
            let direct = sim
                .lifecycle_outputs
                .iter()
                .find_map(|output| match output {
                    LifecycleOutput::ElectricBoltCreated(birth) if birth.owner.is_none() => {
                        Some(birth)
                    }
                    _ => None,
                })
                .expect("explicit-weapon CreateElectricBolt must create a bolt");
            assert_eq!(
                direct.custom_colors,
                Some(direct_colors),
                "must not reselect the firer's own weapon"
            );
            assert!(
                !rules.weapon("TeslaFragmentT").unwrap().is_electric_bolt,
                "custom colors must not enable an unconfigured bolt effect"
            );
        }
    }

    #[cfg(feature = "npatch")]
    fn aymod_tanya_water_pair(target_type: &str, separation: u16) -> (HeadlessScenario, u64, u64) {
        let root = std::path::PathBuf::from(std::env::var_os("VERA20K_AYMOD_DIR").unwrap());
        let map = "重返迪拜.yrm";
        let mut launch = battle_session(
            map,
            &[(LaunchCountry::from_country_index(1), AiDifficulty::Hard)],
        );
        launch.local_slot_mut().country = LaunchCountry::from_country_index(0);
        launch.options.unit_count = 0;
        let mut scenario = load_session(&root, map, 0x00C0_FFEE, launch).unwrap();
        let runtime = &mut scenario.runtime;
        let (sim, rules) = (&mut runtime.simulation, &runtime.resources.rules);
        let site = {
            let terrain = sim.resolved_terrain().unwrap();
            terrain
                .cells()
                .iter()
                .find_map(|first| {
                    (0..=separation)
                        .all(|dx| {
                            first
                                .rx
                                .checked_add(dx)
                                .and_then(|x| terrain.cell(x, first.ry))
                                .is_some_and(|cell| {
                                    cell.land_type
                                        == crate::rules::terrain_rules::LandType::Water.as_index()
                                        && !cell.outside_playfield
                                        && !cell.bridge_facts.has_structural_bridge()
                                        && cell.level == first.level
                                        && sim.occupancy().get(cell.rx, cell.ry).is_none()
                                })
                        })
                        .then_some((first.rx, first.ry))
                })
                .expect("a continuous empty water lane in actual Return Dubai")
        };
        let owner_name = sim.interner.resolve(house_of(sim, "Americans")).to_owned();
        let enemy_name = sim.interner.resolve(house_of(sim, "Russians")).to_owned();
        let actor = sim
            .spawn_object("TANY", &owner_name, site.0, site.1, 0, rules)
            .unwrap();
        let target = sim
            .spawn_object(
                target_type,
                &enemy_name,
                site.0 + separation,
                site.1,
                0,
                rules,
            )
            .unwrap();
        (scenario, actor, target)
    }

    #[cfg(feature = "npatch")]
    #[test]
    #[ignore = "requires VERA20K_AYMOD_DIR; actual AY Tanya ordinary/elite naval weapons on Return Dubai"]
    fn aymod_tanya_uses_sapper_to_damage_mechanical_ships() {
        use crate::sim::combat::{TargetKind, combat_weapon};
        use crate::sim::command::{Command, CommandEnvelope};
        for (elite, expected) in [(false, "Sapper"), (true, "SapperE")] {
            let (mut scenario, actor, ship) = aymod_tanya_water_pair("HYD", 1);
            let runtime = &mut scenario.runtime;
            let (sim, rules) = (&mut runtime.simulation, &runtime.resources.rules);
            if elite {
                sim.substrate
                    .entities
                    .get_mut(actor)
                    .unwrap()
                    .veterancy
                    .set_elite();
            }
            let tanya = sim.entities().get(actor).unwrap();
            let kind = rules.object("TANY").unwrap();
            assert_eq!(kind.naval_targeting, 4);
            let target = TargetKind::Entity(ship);
            let (_, selected) = combat_weapon::select_weapon_against_in_query(
                rules,
                tanya,
                kind,
                &combat_weapon::attacker_facts(tanya, kind),
                Some(&target),
                sim,
                sim.interner(),
                Some(&crate::map::resolved_terrain::NativeCellQuery::isolated(
                    sim.resolved_terrain().unwrap(),
                )),
                Some(sim.house_alliances()),
            )
            .unwrap();
            assert_eq!(selected.unwrap().weapon.id, expected);
            let health = sim.entities().get(ship).unwrap().health.current();
            let owner = tanya.owner();
            sim.queue_command(CommandEnvelope::new(
                owner,
                sim.session.tick() + 1,
                Command::Attack {
                    attacker_id: actor,
                    target_id: ship,
                },
            ));
            let mut fired = false;
            let hit = (0..300).any(|_| {
                let output = tick_with_due_commands(&mut scenario);
                for event in output
                    .fire_events
                    .iter()
                    .filter(|event| event.attacker_id == actor)
                {
                    assert_eq!(scenario.sim().interner.resolve(event.weapon_id), expected);
                    assert_eq!(event.target, target);
                    fired = true;
                }
                scenario
                    .sim()
                    .entities()
                    .get(ship)
                    .is_none_or(|e| e.health.current() < health)
            });
            assert!(
                fired && hit,
                "actual {expected} Attack must fire and damage its naval target"
            );
        }
    }

    #[cfg(feature = "npatch")]
    #[test]
    #[ignore = "requires VERA20K_AYMOD_DIR; actual Tanya senses and attacks a saved cloaked submarine"]
    fn aymod_tanya_detects_a_cloaked_submarine_before_sapper_attack_after_load() {
        use crate::sim::combat::{
            TargetKind, fire_error::FireError, fire_error_world::FireSubject,
        };
        use crate::sim::command::{Command, CommandEnvelope};
        use crate::sim::mission::{MissionId, MissionType};
        use crate::sim::world::bridge_test_evidence::{restored_retail, save_scene};
        let (mut scene, actor, sub) = aymod_tanya_water_pair("SUB", 14);
        let pristine = scene.sim().resolved_terrain().unwrap().clone();
        let (owner, sub_cell, health) = {
            let runtime = &mut scene.runtime;
            let (sim, rules) = (&mut runtime.simulation, &runtime.resources.rules);
            let kind = rules.object("TANY").unwrap();
            assert!(kind.sensors && i32::from(kind.sensors_sight) < 12);
            assert_eq!(kind.naval_targeting, 4);
            assert!(rules.object("SUB").unwrap().underwater);
            for id in [actor, sub] {
                sim.mission_queue_exact(
                    id,
                    MissionId::from_known(MissionType::Sleep),
                    1,
                    sim.session.binary_frame(),
                    &crate::sim::mission::authority::LiveReadyInputProvider { rules },
                )
                .unwrap();
            }
            let victim = sim.entities().get(sub).unwrap();
            (
                sim.entities().get(actor).unwrap().owner(),
                (victim.position.rx(), victim.position.ry()),
                victim.health.current(),
            )
        };
        let sensed = |scene: &HeadlessScenario| {
            scene
                .sim()
                .fog
                .has_sensor_for_house(owner, sub_cell.0, sub_cell.1)
        };
        let error = |scene: &HeadlessScenario| {
            let sim = scene.sim();
            let rules = &scene.runtime.resources.rules;
            FireSubject {
                world: sim,
                rules,
                overlay_registry: Some(&scene.runtime.resources.overlay_registry),
                fog: Some(sim.fog.fire_view(sim.house_alliances())),
                firer: sim.entities().get(actor).unwrap(),
                obj: rules.object("TANY").unwrap(),
                target: Some(TargetKind::Entity(sub)),
                weapon_index: 1,
            }
            .fire_error(false)
        };
        assert!(
            (0..600).any(|_| {
                tick_with_due_commands(&mut scene);
                scene
                    .sim()
                    .entities()
                    .get(sub)
                    .unwrap()
                    .cloak
                    .as_ref()
                    .unwrap()
                    .is_fully_cloaked()
            }),
            "the submarine must submerge through its ordinary cloak updates"
        );
        assert!(!sensed(&scene));
        assert_eq!(
            error(&scene),
            FireError::Cant,
            "without range checks the unsensed submarine must still be rejected"
        );
        let bytes = save_scene(&scene, "AY unsensed submarine outside Tanya sensors");
        scene.runtime.simulation = restored_retail(&scene, &pristine, &bytes);
        assert!(!sensed(&scene));
        // Techno6FC24D..6FC29D returns Cant(6) for an unsensed target;
        // Cloaked(9) is the distinct firer's DecloakToFire refusal.
        assert_eq!(error(&scene), FireError::Cant);
        let tick = scene.sim().session.tick() + 1;
        scene.runtime.simulation.queue_command(CommandEnvelope::new(
            owner,
            tick,
            Command::Move {
                entity_id: actor,
                target_rx: sub_cell.0 - 3,
                target_ry: sub_cell.1,
            },
        ));
        assert!(
            (0..900).any(|_| {
                tick_with_due_commands(&mut scene);
                sensed(&scene)
            }),
            "normal swimming movement must publish Tanya's real sensor footprint"
        );
        assert!(
            scene
                .sim()
                .entities()
                .get(sub)
                .unwrap()
                .cloak
                .as_ref()
                .unwrap()
                .is_fully_cloaked()
        );
        let bytes = save_scene(&scene, "AY sensed submarine before Sapper attack");
        scene.runtime.simulation = restored_retail(&scene, &pristine, &bytes);
        assert!(sensed(&scene));
        assert_ne!(error(&scene), FireError::Cant);
        let tick = scene.sim().session.tick() + 1;
        scene.runtime.simulation.queue_command(CommandEnvelope::new(
            owner,
            tick,
            Command::Attack {
                attacker_id: actor,
                target_id: sub,
            },
        ));
        let mut fired = false;
        assert!(
            (0..600).any(|_| {
                let output = tick_with_due_commands(&mut scene);
                for event in output
                    .fire_events
                    .iter()
                    .filter(|event| event.attacker_id == actor)
                {
                    assert_eq!(scene.sim().interner.resolve(event.weapon_id), "Sapper");
                    assert_eq!(event.target, TargetKind::Entity(sub));
                    fired = true;
                }
                scene
                    .sim()
                    .entities()
                    .get(sub)
                    .is_none_or(|victim| victim.health.current() < health)
            }),
            "Tanya must close and damage the detected submarine with Sapper; fired={fired}"
        );
        assert!(fired);
    }

    #[cfg(feature = "npatch")]
    #[test]
    #[ignore = "requires VERA20K_AYMOD_DIR; Tanya armor fallback precedes hover and organic naval exceptions"]
    fn aymod_tanya_armor_fallback_precedes_hover_and_organic_naval_exceptions() {
        use crate::sim::combat::{TargetKind, combat_weapon};
        use crate::sim::command::{Command, CommandEnvelope};
        use crate::sim::mission::{MissionId, MissionType};
        for target_type in ["LCRF", "DLPH"] {
            let (mut scene, actor, target) = aymod_tanya_water_pair(target_type, 1);
            let (expected, health) = {
                let runtime = &mut scene.runtime;
                let (sim, rules) = (&mut runtime.simulation, &runtime.resources.rules);
                let victim_type = rules.object(target_type).unwrap();
                assert!(
                    victim_type.organic
                        || victim_type.speed_type == crate::rules::locomotor_type::SpeedType::Hover
                );
                let tanya = sim.entities().get(actor).unwrap();
                let kind = rules.object("TANY").unwrap();
                let target_facts = combat_weapon::techno_target_facts(
                    sim.entities().get(target).unwrap(),
                    victim_type,
                    sim.resolved_terrain(),
                    false,
                    rules,
                    sim.interner(),
                );
                assert_eq!(
                    combat_weapon::select_naval_targeting_weapon(kind, Some(&target_facts)),
                    0
                );
                let primary = rules.weapon(kind.primary().unwrap()).unwrap();
                let warhead = rules.warhead(primary.warhead.as_deref().unwrap()).unwrap();
                assert!(!warhead.interaction(victim_type.armor_id()).can_fire());
                let (slot, selected) = combat_weapon::select_weapon_against_in_query(
                    rules,
                    tanya,
                    kind,
                    &combat_weapon::attacker_facts(tanya, kind),
                    Some(&TargetKind::Entity(target)),
                    sim,
                    sim.interner(),
                    Some(&crate::map::resolved_terrain::NativeCellQuery::isolated(
                        sim.resolved_terrain().unwrap(),
                    )),
                    Some(sim.house_alliances()),
                )
                .unwrap();
                // SelectWeapon6F36E3..6F374F checks zero verses before the
                // naval selector6F3754. Actual AY HollowPoint2 has zero
                // against both heavy LCRF and light organic DLPH.
                assert_eq!(
                    slot, 1,
                    "{target_type}: zero primary verses wins before naval exceptions"
                );
                let expected = kind.secondary().unwrap().to_owned();
                assert_eq!(selected.unwrap().weapon.id, expected);
                let health = sim.entities().get(target).unwrap().health.current();
                let owner = tanya.owner();
                // Isolate the attack from a dolphin's own retaliation without
                // changing either weapon, verses or sensor flags.
                sim.mission_queue_exact(
                    target,
                    MissionId::from_known(MissionType::Sleep),
                    1,
                    sim.session.binary_frame(),
                    &crate::sim::mission::authority::LiveReadyInputProvider { rules },
                )
                .unwrap();
                sim.queue_command(CommandEnvelope::new(
                    owner,
                    sim.session.tick() + 1,
                    Command::Attack {
                        attacker_id: actor,
                        target_id: target,
                    },
                ));
                (expected, health)
            };
            let mut fired = false;
            assert!(
                (0..300).any(|_| {
                    let output = tick_with_due_commands(&mut scene);
                    for event in output
                        .fire_events
                        .iter()
                        .filter(|event| event.attacker_id == actor)
                    {
                        assert_eq!(scene.sim().interner.resolve(event.weapon_id), expected);
                        assert_eq!(event.target, TargetKind::Entity(target));
                        fired = true;
                    }
                    scene
                        .sim()
                        .entities()
                        .get(target)
                        .is_none_or(|victim| victim.health.current() < health)
                }),
                "Tanya must damage {target_type} using the authored armor fallback; fired={fired}"
            );
            assert!(fired);
        }
    }

    #[cfg(feature = "npatch")]
    #[test]
    #[ignore = "requires VERA20K_AYMOD_DIR; real AY ORCA airborne and ground Attack damage"]
    fn aymod_orca_attacks_airborne_and_ground_targets() {
        use crate::sim::command::{Command, CommandEnvelope};
        use crate::sim::movement::air_movement;
        for target_kind in ["ZEP", "MTNK"] {
            let (mut scenario, plane, target) = aymod_nearby_pair("ORCA", target_kind, false);
            let runtime = &mut scenario.runtime;
            let (sim, rules) = (&mut runtime.simulation, &runtime.resources.rules);
            let kind = rules.object("ORCA").unwrap();
            assert!(kind.npatch_aa_combat);
            let weapon = rules.weapon(kind.primary().unwrap()).unwrap();
            assert!(
                rules
                    .projectile(weapon.projectile.as_deref().unwrap())
                    .unwrap()
                    .aa
            );
            let entity = sim.substrate.entities.get_mut(target).unwrap();
            if target_kind == "ZEP" {
                let ground = i32::from(entity.position.z()) * 104;
                entity
                    .position
                    .set_exact_z_leptons_for_test(Some(ground + 500));
            }
            let health = entity.health.current();
            assert_eq!(
                air_movement::is_high_flying(
                    entity,
                    sim.resolved_terrain.as_ref(),
                    Some((rules, &sim.interner))
                ),
                target_kind == "ZEP"
            );
            let owner = sim.substrate.entities.get(plane).unwrap().owner();
            sim.queue_command(CommandEnvelope::new(
                owner,
                sim.session.tick() + 1,
                Command::Attack {
                    attacker_id: plane,
                    target_id: target,
                },
            ));
            let mut fired = false;
            let mut hit = false;
            for _ in 0..900 {
                tick_with_due_commands(&mut scenario);
                let sim = scenario.sim();
                fired |= sim
                    .entities()
                    .get(plane)
                    .is_some_and(|e| e.ammo().is_some_and(|ammo| ammo.current() < ammo.max()));
                hit = sim
                    .entities()
                    .get(target)
                    .is_none_or(|e| e.health.current() < health);
                if hit {
                    if target_kind == "ZEP" {
                        assert!(
                            sim.entities().get(target).is_some_and(|entity| {
                                air_movement::is_high_flying(
                                    entity,
                                    sim.resolved_terrain.as_ref(),
                                    Some((&scenario.runtime.resources.rules, &sim.interner)),
                                )
                            }),
                            "the hit must occur while the ZEP is still airborne"
                        );
                    }
                    break;
                }
            }
            assert!(
                fired && hit,
                "actual AY ORCA must fire and damage {target_kind}; fired={fired}, hit={hit}"
            );
        }
    }

    #[cfg(feature = "npatch")]
    #[test]
    #[ignore = "requires VERA20K_AYMOD_DIR; elite ORCA intercepts a moving airborne target after load"]
    fn aymod_orca_elite_intercepts_a_moving_air_target_after_load() {
        use crate::sim::combat::TargetKind;
        use crate::sim::command::{Command, CommandEnvelope};
        use crate::sim::world::bridge_test_evidence::{restored_retail, save_scene};
        let mut scene = aymod_battle(0);
        let pristine = scene.sim().resolved_terrain().unwrap().clone();
        let (plane, target, expected_weapon, health, initial_plane, initial_target) = {
            let runtime = &mut scene.runtime;
            let (sim, rules) = (&mut runtime.simulation, &runtime.resources.rules);
            let owner = house_of(sim, "Americans");
            let enemy = house_of(sim, "Russians");
            let plane = spawn_on_free_cells(
                sim,
                rules,
                owner,
                cells_nearest(aymod_anchor(sim, owner)),
                &["ORCA"],
            )[0];
            let at = sim.entities().get(plane).unwrap().position;
            let center = (i32::from(at.rx()), i32::from(at.ry()));
            let sites = cells_nearest(center)
                .into_iter()
                .filter(|&(x, y)| (x - center.0).abs() >= 9 && (y - center.1).abs() >= 9)
                .collect::<Vec<_>>();
            let target = spawn_on_free_cells(sim, rules, enemy, sites, &["ZEP"])[0];
            let kind = rules.object("ORCA").unwrap();
            assert!(kind.npatch_aa_combat);
            let weapon = rules.weapon(kind.elite_primary().unwrap()).unwrap();
            assert_ne!(kind.elite_primary(), kind.primary());
            assert!(
                rules
                    .projectile(weapon.projectile.as_deref().unwrap())
                    .unwrap()
                    .aa
            );
            let expected_weapon = weapon.id.clone();
            assert!(
                crate::sim::combat::object_distance_to(plane, &TargetKind::Entity(target), sim)
                    .unwrap()
                    > weapon.range_leptons + 4 * 256
            );
            sim.substrate
                .entities
                .get_mut(plane)
                .unwrap()
                .veterancy
                .set_elite();
            let victim = sim.substrate.entities.get_mut(target).unwrap();
            let ground = i32::from(victim.position.z()) * 104;
            victim
                .position
                .set_exact_z_leptons_for_test(Some(ground + 500));
            let health = victim.health.current();
            let initial_target = victim.position;
            let dest = cells_nearest((
                i32::from(initial_target.rx()),
                i32::from(initial_target.ry()),
            ))
            .into_iter()
            .find(|&(x, y)| {
                (x - i32::from(initial_target.rx()))
                    .abs()
                    .max((y - i32::from(initial_target.ry())).abs())
                    == 8
                    && u16::try_from(x)
                        .ok()
                        .zip(u16::try_from(y).ok())
                        .is_some_and(|(x, y)| {
                            sim.path_grid()
                                .is_some_and(|grid| grid.is_walkable_for_infantry(x, y))
                        })
            })
            .unwrap();
            // Supply the opponent's ordinary Move payload. This is target
            // motion, not a claim that the AI chose this tactical route.
            let enemy_name = sim.interner.resolve(enemy).to_owned();
            assert!(sim.apply_command(
                &enemy_name,
                &Command::Move {
                    entity_id: target,
                    target_rx: dest.0 as u16,
                    target_ry: dest.1 as u16,
                },
                Some(rules)
            ));
            sim.queue_command(CommandEnvelope::new(
                owner,
                sim.session.tick() + 1,
                Command::Attack {
                    attacker_id: plane,
                    target_id: target,
                },
            ));
            (plane, target, expected_weapon, health, at, initial_target)
        };
        let xy = |at: crate::sim::components::Position| (at.rx(), at.ry(), at.sub_x(), at.sub_y());
        assert!(
            (0..90).any(|_| {
                let output = tick_with_due_commands(&mut scene);
                assert!(
                    !output
                        .fire_events
                        .iter()
                        .any(|event| event.attacker_id == plane),
                    "the saved boundary must precede the first release"
                );
                let sim = scene.sim();
                let actor = sim.entities().get(plane).unwrap();
                actor.attack_target.target() == Some(TargetKind::Entity(target))
                    && xy(actor.position) != xy(initial_plane)
                    && xy(sim.entities().get(target).unwrap().position) != xy(initial_target)
            }),
            "the distant target must actually move while the ORCA retains its attack order"
        );
        let bytes = save_scene(&scene, "AY elite ORCA pursuing an airborne target");
        scene.runtime.simulation = restored_retail(&scene, &pristine, &bytes);
        let saved_target = scene.sim().entities().get(target).unwrap().position;
        assert_eq!(
            scene
                .sim()
                .entities()
                .get(plane)
                .unwrap()
                .attack_target
                .target(),
            Some(TargetKind::Entity(target))
        );
        let mut fired = false;
        assert!(
            (0..1200).any(|_| {
                let output = tick_with_due_commands(&mut scene);
                for event in output
                    .fire_events
                    .iter()
                    .filter(|event| event.attacker_id == plane)
                {
                    assert_eq!(
                        scene.sim().interner.resolve(event.weapon_id),
                        expected_weapon
                    );
                    assert_eq!(event.target, TargetKind::Entity(target));
                    fired = true;
                }
                scene
                    .sim()
                    .entities()
                    .get(target)
                    .is_some_and(|victim| victim.health.current() < health)
            }),
            "restored elite ORCA must intercept and damage the moving ZEP; fired={fired}"
        );
        assert!(fired);
        let sim = scene.sim();
        assert_ne!(
            xy(sim.entities().get(plane).unwrap().position),
            xy(initial_plane)
        );
        assert!(
            crate::sim::movement::air_movement::is_high_flying(
                sim.entities().get(target).unwrap(),
                sim.resolved_terrain(),
                Some((&scene.runtime.resources.rules, &sim.interner)),
            ),
            "the victim must still be airborne at impact"
        );
        let victim = sim.entities().get(target).unwrap();
        assert_ne!(xy(victim.position), xy(saved_target));
        assert!(
            victim.navigation.nav_com().is_some(),
            "the intercept must precede completion of the target's Move"
        );
    }

    #[cfg(feature = "npatch")]
    #[test]
    #[ignore = "requires VERA20K_AYMOD_DIR; real AY MTNK independent turret speed"]
    fn aymod_mtnk_keeps_independent_turret_speed_while_firing() {
        use crate::sim::command::{Command, CommandEnvelope};
        let (mut scenario, actor, target) = aymod_nearby_pair("MTNK", "APOC", false);
        let sim = &mut scenario.runtime.simulation;
        let entity = sim.entities().get(actor).unwrap();
        assert_eq!(entity.body_facing.rot_per_frame(), 7 << 8);
        assert_eq!(
            entity.barrel_facing.as_ref().unwrap().rot_per_frame(),
            9 << 8
        );
        let owner = entity.owner();
        let health = sim.entities().get(target).unwrap().health.current();
        sim.queue_command(CommandEnvelope::new(
            owner,
            sim.session.tick() + 1,
            Command::Attack {
                attacker_id: actor,
                target_id: target,
            },
        ));
        for _ in 0..180 {
            tick_with_due_commands(&mut scenario);
            let entity = scenario.sim().entities().get(actor).unwrap();
            assert_eq!(entity.body_facing.rot_per_frame(), 7 << 8);
            assert_eq!(
                entity.barrel_facing.as_ref().unwrap().rot_per_frame(),
                9 << 8,
                "aim/idle updates must not reset the turret to hull ROT"
            );
            if scenario
                .sim()
                .entities()
                .get(target)
                .is_none_or(|entity| entity.health.current() < health)
            {
                return;
            }
        }
        panic!("the actual MTNK Attack did not align and fire");
    }

    #[cfg(feature = "npatch")]
    fn aymod_ai_fortress_loading_team(
        isolate_team_creation: bool,
    ) -> (
        HeadlessScenario,
        crate::sim::intern::InternedId,
        u64,
        u64,
        Vec<u64>,
    ) {
        let root = std::path::PathBuf::from(std::env::var_os("VERA20K_AYMOD_DIR").unwrap());
        let (mut scene, _) = retail_battle_in(
            &root,
            |session| {
                session.local_slot_mut().country = LaunchCountry::from_country_index(1);
                session.options.unit_count = 0;
            },
            &[(LaunchCountry::from_country_index(0), AiDifficulty::Hard)],
        );
        let ai = house_of(scene.sim(), "Americans");
        assert!(
            !scene
                .sim()
                .houses
                .get(&ai)
                .unwrap()
                .event_dispatch_eligible()
        );
        assert!(
            (0..900).any(|_| {
                tick_with_due_commands(&mut scene);
                scene.sim().entities().values().any(|actor| {
                    actor.owner() == ai
                        && scene.sim().interner.resolve(actor.type_ref()) == "GACNST"
                })
            }),
            "normal AI MCV deployment must provide the friendly-base gather prerequisite"
        );
        let (team_id, carrier, riders) = {
            let runtime = &mut scene.runtime;
            let (sim, rules) = (&mut runtime.simulation, &runtime.resources.rules);
            if isolate_team_creation {
                // Native TAction74's existing latch isolates this component
                // from unrelated higher-priority teams, without altering INIs.
                sim.houses
                    .get_mut(&ai)
                    .unwrap()
                    .set_ai_triggers_active(false);
            }
            let team_type = sim.interner.get("07C32EFC-G").unwrap();
            let definition = sim.team_script_vm.team_type_definition(team_type).unwrap();
            assert_eq!(sim.interner.resolve(definition.script_id), "07C32C8C-G");
            let names = sim
                .team_script_vm
                .task_force_of(team_type)
                .unwrap()
                .entries
                .iter()
                .flat_map(|entry| {
                    std::iter::repeat_n(
                        sim.interner.resolve(entry.member_type.id).to_owned(),
                        usize::try_from(entry.count).unwrap(),
                    )
                })
                .collect::<Vec<_>>();
            assert_eq!(names.iter().filter(|name| *name == "BFRT").count(), 1);
            assert_eq!(names.iter().filter(|name| *name == "GGI").count(), 5);
            assert_eq!(names.iter().filter(|name| *name == "GHOST").count(), 1);
            assert_eq!(rules.object("BFRT").unwrap().passengers, 6);
            assert!(rules.object("BFRT").unwrap().open_topped);
            let (x, y) = sim.houses.get(&ai).unwrap().base_center().unwrap();
            let sites = cells_nearest((i32::from(x), i32::from(y)));
            // Supply exactly the authored TaskForce, not a new force count or
            // production policy. Recruitment and script14/43 do the boarding.
            let types = names.iter().map(String::as_str).collect::<Vec<_>>();
            let members = spawn_on_free_cells(sim, rules, ai, sites, &types);
            let carrier = members
                .iter()
                .copied()
                .find(|id| {
                    sim.interner
                        .resolve(sim.entities().get(*id).unwrap().type_ref())
                        == "BFRT"
                })
                .unwrap();
            let riders = members
                .into_iter()
                .filter(|&id| id != carrier)
                .collect::<Vec<_>>();
            let team_id = sim
                .create_team(team_type, Some(ai), rules)
                .or_else(|| {
                    sim.team_script_vm
                        .teams_in_order()
                        .find(|team| team.team_type_id() == Some(team_type) && team.owner() == ai)
                        .map(|team| team.id())
                })
                .unwrap();
            (team_id, carrier, riders)
        };
        (scene, ai, team_id, carrier, riders)
    }

    #[cfg(feature = "npatch")]
    #[test]
    #[ignore = "requires VERA20K_AYMOD_DIR; native-legal competing AI recruitment during BFRT loading"]
    fn aymod_ai_fortress_loading_observes_higher_priority_defense_recruitment() {
        let (mut scene, _, team_id, carrier, riders) = aymod_ai_fortress_loading_team(false);
        let team_type = scene
            .sim()
            .team_script_vm
            .team(team_id)
            .unwrap()
            .team_type_id()
            .unwrap();
        let priority = scene
            .sim()
            .team_script_vm
            .team_type_definition(team_type)
            .unwrap()
            .priority;
        let mut stolen = std::collections::BTreeSet::new();
        let mut receivers = std::collections::BTreeSet::new();
        assert!((0..1800).any(|_| {
            tick_with_due_commands(&mut scene);
            let sim = scene.sim();
            for &rider in &riders {
                if let Some((other, _)) = sim.team_script_vm.team_for_member(rider)
                    && other != team_id
                {
                    let other_type = sim.team_script_vm.team(other).unwrap().team_type_id().unwrap();
                    let definition = sim.team_script_vm.team_type_definition(other_type).unwrap();
                    assert!(definition.is_base_defense && definition.priority > priority,
                        "CanAdd6EA7C7's priority gate, not a passenger-link loss, explains this transfer");
                    stolen.insert(rider);
                    receivers.insert((sim.interner.resolve(other_type).to_owned(), definition.priority));
                }
            }
            let count = sim.entities().get(carrier).unwrap().passenger_role.passenger_count();
            !stolen.is_empty() && usize::try_from(count).unwrap() + stolen.len() == riders.len()
                && sim.team_script_vm.team(team_id).is_some_and(|team| team.current_action(&sim.team_script_vm) == 43)
        }), "the unrestricted fixture must retain the observed authored recruitment competition");
        eprintln!(
            "AY BFRT priority{priority}, competing defense {receivers:?}, reassigned {} riders; wait43 is not a boarding failure",
            stolen.len()
        );
    }

    #[cfg(feature = "npatch")]
    #[test]
    #[ignore = "requires VERA20K_AYMOD_DIR; authored AI fortress loading, save/fire/unload"]
    fn aymod_ai_fortress_team_loads_six_and_restores_passenger_fire_and_exit() {
        use crate::sim::command::Command;
        use crate::sim::mission::{MissionId, MissionType};
        use crate::sim::world::bridge_test_evidence::{restored_retail, save_scene};
        let (mut scene, ai, team_id, carrier, riders) = aymod_ai_fortress_loading_team(true);
        let pristine = scene.sim().resolved_terrain().unwrap().clone();
        let aboard =
            |scene: &HeadlessScenario| {
                riders.iter().all(|id| {
                    scene.sim().entities().get(*id).is_some_and(|actor| {
                        actor.passenger_role.open_transport_id() == Some(carrier)
                    })
                })
            };
        let mut loading_action = false;
        let mut full_action = false;
        // Action14 admits one radio-contact rider at a time; use the actual
        // walking cadence rather than shortening GGI speed or entry timers.
        let loaded = (0..3600).any(|_| {
            tick_with_due_commands(&mut scene);
            if let Some(team) = scene.sim().team_script_vm.team(team_id) {
                let action = team.current_action(&scene.sim().team_script_vm);
                loading_action |= action == 14;
                full_action |= action == 43;
                aboard(&scene) && team.cursor() >= 3
            } else {
                false
            }
        });
        assert!(
            loaded,
            "AI BFRT did not load: team={:?}; carrier={:?}; riders={:?}",
            scene
                .sim()
                .team_script_vm
                .team(team_id)
                .map(|team| (team.cursor(), team.members().collect::<Vec<_>>())),
            scene
                .sim()
                .entities()
                .get(carrier)
                .map(|actor| (&actor.mission, &actor.passenger_role)),
            riders
                .iter()
                .map(|id| scene.sim().entities().get(*id).map(|actor| (
                    &actor.mission,
                    &actor.position,
                    &actor.passenger_role,
                    scene.sim().team_script_vm.team_for_member(*id),
                    scene.sim().team_script_vm.team_to_rejoin(*id)
                )))
                .collect::<Vec<_>>()
        );
        assert!(
            loading_action && full_action,
            "the real script must pass load14 and wait43"
        );
        let cargo = scene
            .sim()
            .entities()
            .get(carrier)
            .unwrap()
            .passenger_role
            .cargo()
            .unwrap()
            .passengers()
            .to_vec();
        assert_eq!(cargo.len(), 6);
        assert_eq!(
            scene
                .sim()
                .team_script_vm
                .team(team_id)
                .unwrap()
                .members()
                .collect::<Vec<_>>(),
            [carrier]
        );
        let cursor = scene.sim().team_script_vm.team(team_id).unwrap().cursor();
        let bytes = save_scene(&scene, "AY AI six-passenger fortress");
        scene.runtime.simulation = restored_retail(&scene, &pristine, &bytes);
        assert!(aboard(&scene));
        assert_eq!(
            scene
                .sim()
                .entities()
                .get(carrier)
                .unwrap()
                .passenger_role
                .cargo()
                .unwrap()
                .passengers(),
            cargo.as_slice()
        );
        assert_eq!(
            scene.sim().team_script_vm.team(team_id).unwrap().cursor(),
            cursor
        );
        let (victim, foot_victim) = {
            let runtime = &mut scene.runtime;
            let (sim, rules) = (&mut runtime.simulation, &runtime.resources.rules);
            let at = sim.entities().get(carrier).unwrap().position;
            let center = (i32::from(at.rx()), i32::from(at.ry()));
            let sites = cells_nearest(center)
                .into_iter()
                .filter(|&(x, y)| (x - center.0).abs().max((y - center.1).abs()) == 3)
                .collect::<Vec<_>>();
            let enemy = house_of(sim, "Russians");
            let victim = spawn_on_free_cells(sim, rules, enemy, sites, &["BFRT"])[0];
            assert!(rules.object("BFRT").unwrap().omni_crush_resistant);
            let ghost_weapon = rules
                .weapon(rules.object("GHOST").unwrap().primary().unwrap())
                .unwrap();
            let ghost_wh = rules
                .warhead(ghost_weapon.warhead.as_deref().unwrap())
                .unwrap();
            assert!(
                !ghost_wh
                    .interaction(rules.object("BFRT").unwrap().armor_id())
                    .can_passively_acquire()
            );
            assert!(
                ghost_wh
                    .interaction(rules.object("E2").unwrap().armor_id())
                    .can_passively_acquire()
            );
            // HollowPoint's authored 1% vehicle verses forbids passive
            // acquisition. Give the SEAL a legal infantry target as well.
            let sites = cells_nearest(center)
                .into_iter()
                .filter(|&(x, y)| (x - center.0).abs().max((y - center.1).abs()) == 5)
                .collect::<Vec<_>>();
            let foot_victim = spawn_on_free_cells(sim, rules, enemy, sites, &["E2"])[0];
            sim.mission_queue_exact(
                victim,
                MissionId::from_known(MissionType::Sleep),
                1,
                sim.session.binary_frame(),
                &crate::sim::mission::authority::LiveReadyInputProvider { rules },
            )
            .unwrap();
            sim.mission_queue_exact(
                foot_victim,
                MissionId::from_known(MissionType::Sleep),
                1,
                sim.session.binary_frame(),
                &crate::sim::mission::authority::LiveReadyInputProvider { rules },
            )
            .unwrap();
            (victim, foot_victim)
        };
        let mut fired = std::collections::BTreeSet::new();
        assert!(
            (0..360).any(|_| {
                let output = tick_with_due_commands(&mut scene);
                fired.extend(
                    output
                        .fire_events
                        .iter()
                        .filter(|event| {
                            riders.contains(&event.attacker_id)
                                && [victim, foot_victim].iter().any(|&id| {
                                    event.target == crate::sim::combat::TargetKind::Entity(id)
                                })
                        })
                        .map(|event| event.attacker_id),
                );
                fired.len() == 6
            }),
            "all six restored AI passengers must actually fire: {fired:?}"
        );
        {
            let runtime = &mut scene.runtime;
            let owner = runtime.simulation.interner.resolve(ai).to_owned();
            // The waiter script does not author an unload action. Supply this
            // tactical intent at the real AI payload boundary, not fake input.
            assert!(runtime.simulation.apply_command(
                &owner,
                &Command::UnloadPassengers {
                    transport_id: carrier
                },
                Some(&runtime.resources.rules)
            ));
        }
        assert!(
            (0..600).any(|_| {
                tick_with_due_commands(&mut scene);
                let sim = scene.sim();
                sim.team_script_vm
                    .team(team_id)
                    .is_some_and(|team| team.current_action(&sim.team_script_vm) == 11)
                    && sim.entities().get(carrier).unwrap().mission.effective()
                        == MissionId::from_known(MissionType::AreaGuard)
                    && sim.entities().get(carrier).unwrap().mission.queued()
                        != MissionId::from_known(MissionType::Unload)
            }),
            "the authored waiter must resume its persistent AreaGuard order"
        );
        assert!(aboard(&scene));
        {
            let runtime = &mut scene.runtime;
            let (sim, rules) = (&mut runtime.simulation, &runtime.resources.rules);
            // DoMission6EDA25..6EDA71 spares Unload only for Guard(5),
            // not this script's AreaGuard(11). Explicitly disband through
            // TAction5's owner before testing a lasting external unload.
            let team_type = sim
                .team_script_vm
                .team(team_id)
                .unwrap()
                .team_type_id()
                .unwrap();
            sim.destroy_teams_of_type(team_type, rules, crate::sim::world::FrameEffects::default());
            assert!(sim.team_script_vm.team_for_member(carrier).is_none());
            assert_eq!(
                sim.entities()
                    .get(carrier)
                    .unwrap()
                    .passenger_role
                    .passenger_count(),
                6
            );
            let owner = sim.interner.resolve(ai).to_owned();
            assert!(sim.apply_command(
                &owner,
                &Command::UnloadPassengers {
                    transport_id: carrier
                },
                Some(rules)
            ));
        }
        assert!(
            (0..600).any(|_| {
                tick_with_due_commands(&mut scene);
                scene
                    .sim()
                    .entities()
                    .get(carrier)
                    .unwrap()
                    .passenger_role
                    .passenger_count()
                    == 0
            }),
            "a disbanded AI transport must release the six riders through ordinary Unload"
        );
        for &id in &riders {
            let rider = scene.sim().entities().get(id).unwrap();
            assert!(rider.health.current() > 0 && !rider.lifecycle.in_limbo());
            assert_eq!(rider.passenger_role.inside_transport_id(), None);
            assert_eq!(rider.passenger_role.open_transport_id(), None);
        }
        let bytes = save_scene(&scene, "AY AI fortress after unloading");
        scene.runtime.simulation = restored_retail(&scene, &pristine, &bytes);
        assert_eq!(
            scene
                .sim()
                .entities()
                .get(carrier)
                .unwrap()
                .passenger_role
                .passenger_count(),
            0
        );
        for id in riders {
            let rider = scene.sim().entities().get(id).unwrap();
            assert!(!rider.lifecycle.in_limbo());
            assert_eq!(rider.passenger_role.open_transport_id(), None);
        }
    }

    #[cfg(feature = "npatch")]
    #[test]
    #[ignore = "requires VERA20K_AYMOD_DIR; real AY sixth passenger fire coordinate"]
    fn aymod_sixth_fortress_passenger_reuses_first_firing_port() {
        use crate::sim::combat::fire_coord::{FireSource, fire_coordinate};
        use crate::sim::command::{Command, CommandEnvelope};
        let (mut scenario, carrier, first) = aymod_nearby_pair("BFRT", "E1", true);
        let mut riders = vec![first];
        {
            let runtime = &mut scenario.runtime;
            let sim = &mut runtime.simulation;
            let owner = sim.entities().get(carrier).unwrap().owner();
            let at = sim.entities().get(carrier).unwrap().position;
            let sites = cells_nearest((i32::from(at.rx()), i32::from(at.ry())));
            riders.extend(spawn_on_free_cells(
                sim,
                &runtime.resources.rules,
                owner,
                sites,
                &["E1"; 5],
            ));
            for &rider in &riders {
                sim.queue_command(CommandEnvelope::new(
                    owner,
                    sim.session.tick() + 1,
                    Command::clicked_mission(
                        rider,
                        crate::sim::mission::MissionType::Enter,
                        crate::sim::combat::TargetKind::Entity(carrier),
                    ),
                ));
            }
        }
        for _ in 0..600 {
            tick_with_due_commands(&mut scenario);
            if riders.iter().all(|id| {
                scenario
                    .sim()
                    .entities()
                    .get(*id)
                    .unwrap()
                    .passenger_role
                    .open_transport_id()
                    == Some(carrier)
            }) {
                break;
            }
        }
        let sim = scenario.sim();
        let cargo = sim
            .entities()
            .get(carrier)
            .unwrap()
            .passenger_role
            .cargo()
            .unwrap()
            .passengers();
        assert_eq!(cargo.len(), 6, "all six actual Enter missions must board");
        let coord = |id| {
            let entity = sim.entities().get(id).unwrap();
            fire_coordinate(
                sim,
                &scenario.runtime.resources.rules,
                &FireSource::of_entity(entity),
                sim.object_type(entity.type_ref(), &scenario.runtime.resources.rules)
                    .unwrap(),
                0,
                0,
                Default::default(),
            )
            .coord
        };
        assert_eq!(
            coord(cargo[5]),
            coord(cargo[0]),
            "NPAE174 cycles five ports; ART5/6 are not enabled"
        );
        assert_ne!(
            coord(cargo[4]),
            coord(cargo[0]),
            "the fifth port must not be collapsed too"
        );
    }

    #[cfg(feature = "npatch")]
    #[test]
    #[ignore = "requires VERA20K_AYMOD_DIR; actual absorbed passenger death and saved release"]
    fn aymod_reactor_passengers_follow_destruction_survival_flags_after_load() {
        use crate::sim::combat::{
            EntityDamageEvent, RAD_NO_ATTACKER, ReceiverCallFlags, TargetKind,
        };
        use crate::sim::command::{Command, CommandEnvelope};
        use crate::sim::mission::MissionType;
        use crate::sim::world::bridge_test_evidence::{restored_retail, save_scene};
        for (kind, no_survivor) in [("YAPOWR", false), ("YAPOWRAI", true)] {
            let (mut scene, building, rider) = aymod_nearby_pair(kind, "INIT", true);
            let pristine = scene.sim().resolved_terrain().unwrap().clone();
            let owner = scene.sim().entities().get(building).unwrap().owner();
            let rules = &scene.runtime.resources.rules;
            let cargo_buildings = rules
                .all_objects()
                .filter(|object| {
                    object.category == crate::rules::object_type::ObjectCategory::Building
                        && object.passengers > 0
                })
                .collect::<Vec<_>>();
            assert_eq!(cargo_buildings.len(), 2);
            assert!(
                cargo_buildings.iter().all(|object| {
                    ["YAPOWR", "YAPOWRAI"].contains(&object.id.as_str())
                        && object.infantry_absorb
                        && object.absorbs_passengers()
                }),
                "the pending nonabsorber true-Cargo residual is not an authored AY building profile"
            );
            let tick = scene.sim().session.tick() + 1;
            scene.runtime.simulation.queue_command(CommandEnvelope::new(
                owner,
                tick,
                Command::clicked_mission(rider, MissionType::Enter, TargetKind::Entity(building)),
            ));
            assert!(
                (0..600).any(|_| {
                    tick_with_due_commands(&mut scene);
                    scene
                        .sim()
                        .entities()
                        .get(rider)
                        .unwrap()
                        .passenger_role
                        .inside_transport_id()
                        == Some(building)
                }),
                "{kind}: ordinary Enter must load the real absorber"
            );
            assert!(
                scene
                    .sim()
                    .entities()
                    .get(building)
                    .unwrap()
                    .passenger_role
                    .occupants()
                    .is_empty()
            );
            let bytes = save_scene(&scene, "AY reactor with an absorbed infantryman");
            scene.runtime.simulation = restored_retail(&scene, &pristine, &bytes);
            {
                let runtime = &mut scene.runtime;
                let (sim, rules) = (&mut runtime.simulation, &runtime.resources.rules);
                assert_eq!(
                    sim.entities()
                        .get(building)
                        .unwrap()
                        .passenger_role
                        .cargo()
                        .unwrap()
                        .passengers(),
                    &[rider]
                );
                let damage = sim.entities().get(building).unwrap().health.current() + 1;
                let warhead = sim.interner.intern("Super");
                // DestructionEffects441F1B passes C4AppliedBy to
                // SpawnSurvivors442D90; IgnoreDefenses supplies NoSurvivor.
                sim.commit_direct_damage_receiver(
                    rules,
                    Some(&runtime.resources.overlay_registry),
                    EntityDamageEvent::direct_receiver(
                        building,
                        damage,
                        0,
                        RAD_NO_ATTACKER,
                        None,
                        warhead,
                        ReceiverCallFlags {
                            ignore_defenses: no_survivor,
                            arg6: false,
                        },
                    ),
                    Default::default(),
                );
                let actor = sim.entities().get(rider).unwrap();
                assert_eq!(
                    actor.is_object_alive(),
                    !no_survivor,
                    "{kind}: native absorbed-passenger survival gate"
                );
                if !no_survivor {
                    assert!(!actor.lifecycle.in_limbo());
                    assert_eq!(actor.owner(), owner);
                    assert_eq!(actor.passenger_role.inside_transport_id(), None);
                }
                assert!(sim.entities().get(building).is_none_or(|actor| {
                    actor
                        .passenger_role
                        .cargo()
                        .is_none_or(|cargo| cargo.is_empty())
                }));
            }
            tick_with_due_commands(&mut scene);
            let bytes = save_scene(&scene, "AY destroyed reactor passenger outcome");
            scene.runtime.simulation = restored_retail(&scene, &pristine, &bytes);
            if no_survivor {
                assert!(scene.sim().entities().get(rider).is_none());
            } else {
                let actor = scene.sim().entities().get(rider).unwrap();
                assert!(actor.is_object_alive() && !actor.lifecycle.in_limbo());
                assert_eq!(actor.passenger_role.inside_transport_id(), None);
            }
        }
    }

    #[cfg(feature = "npatch")]
    #[test]
    #[ignore = "requires VERA20K_AYMOD_DIR; actual AY hospitals power/Enter/heal/exit"]
    fn aymod_hospital_admits_heals_and_releases_wounded_infantry() {
        use crate::sim::combat::TargetKind;
        use crate::sim::command::{Command, CommandEnvelope};
        use crate::sim::mission::MissionType;
        for hospital_type in ["GAROBO", "NARADR"] {
            let (mut scenario, hospital, patient) = aymod_nearby_pair(hospital_type, "E1", true);
            let strength = {
                let runtime = &mut scenario.runtime;
                let (sim, rules) = (&mut runtime.simulation, &runtime.resources.rules);
                let object = rules.object(hospital_type).unwrap();
                assert!(object.hospital && !object.armory());
                let owner = sim.entities().get(hospital).unwrap().owner();
                let at = sim.entities().get(hospital).unwrap().position;
                spawn_on_free_cells(
                    sim,
                    rules,
                    owner,
                    cells_nearest((i32::from(at.rx()), i32::from(at.ry()))),
                    &["GAPOWR"],
                );
                let strength = rules.object("E1").unwrap().strength;
                sim.substrate
                    .entities
                    .get_mut(patient)
                    .unwrap()
                    .health
                    .set_for_test(strength - 1);
                eprintln!(
                    "AY {hospital_type} hospital: typeAmmo={} liveAmmo={:?} repairStep={} repairRate={}",
                    object.ammo,
                    sim.entities()
                        .get(hospital)
                        .unwrap()
                        .ammo()
                        .map(|a| a.current()),
                    rules.general.infantry_repair_step,
                    rules.general.infantry_repair_rate
                );
                strength
            };
            for _ in 0..3 {
                tick_with_due_commands(&mut scenario);
            }
            let sim = &mut scenario.runtime.simulation;
            assert_eq!(
                sim.building_operational_state(hospital, &scenario.runtime.resources.rules),
                Some(true)
            );
            let owner = sim.entities().get(patient).unwrap().owner();
            assert_eq!(
                sim.entry_click_action(
                    patient,
                    hospital,
                    crate::sim::world::EntryBaseAction::Select,
                    &scenario.runtime.resources.rules,
                    Default::default()
                ),
                Some(crate::sim::world::EntryClickAction::Enter),
                "ordinary wounded infantry must offer the real hospital entry action"
            );
            sim.queue_command(CommandEnvelope::new(
                owner,
                sim.session.tick() + 1,
                Command::SetBuildingOnLine {
                    entity_id: hospital,
                    on: false,
                },
            ));
            for _ in 0..3 {
                tick_with_due_commands(&mut scenario);
            }
            let sim = &mut scenario.runtime.simulation;
            assert_eq!(
                sim.building_operational_state(hospital, &scenario.runtime.resources.rules),
                Some(false)
            );
            assert_eq!(
                sim.entry_click_action(
                    patient,
                    hospital,
                    crate::sim::world::EntryBaseAction::Select,
                    &scenario.runtime.resources.rules,
                    Default::default()
                ),
                Some(crate::sim::world::EntryClickAction::NoEnter)
            );
            sim.queue_command(CommandEnvelope::new(
                owner,
                sim.session.tick() + 1,
                Command::clicked_mission(patient, MissionType::Enter, TargetKind::Entity(hospital)),
            ));
            for _ in 0..45 {
                tick_with_due_commands(&mut scenario);
                let patient = scenario.sim().entities().get(patient).unwrap();
                assert!(
                    !patient.in_limbo(),
                    "offline {hospital_type} admitted a patient"
                );
                assert_eq!(patient.health.current(), strength - 1);
            }
            let sim = &mut scenario.runtime.simulation;
            sim.queue_command(CommandEnvelope::new(
                owner,
                sim.session.tick() + 1,
                Command::SetBuildingOnLine {
                    entity_id: hospital,
                    on: true,
                },
            ));
            for _ in 0..3 {
                tick_with_due_commands(&mut scenario);
            }
            let sim = &mut scenario.runtime.simulation;
            assert_eq!(
                sim.building_operational_state(hospital, &scenario.runtime.resources.rules),
                Some(true)
            );
            sim.queue_command(CommandEnvelope::new(
                owner,
                sim.session.tick() + 1,
                Command::clicked_mission(patient, MissionType::Enter, TargetKind::Entity(hospital)),
            ));
            let mut entered = false;
            let mut completed = false;
            for _ in 0..600 {
                tick_with_due_commands(&mut scenario);
                let sim = scenario.sim();
                let entity = sim
                    .entities()
                    .get(patient)
                    .expect("the patient remains alive");
                entered |= entity.in_limbo();
                if entered && !entity.in_limbo() && entity.health.current() == strength {
                    completed = true;
                    break;
                }
            }
            let sim = scenario.sim();
            eprintln!(
                "AY {hospital_type} hospital entered={entered} completed={completed} patient={:?} cargo={:?}",
                sim.entities().get(patient).map(|p| (
                    p.health.current(),
                    p.in_limbo(),
                    p.mission.current()
                )),
                sim.entities()
                    .get(hospital)
                    .unwrap()
                    .passenger_role
                    .cargo()
                    .map(|cargo| cargo.count())
            );
            assert!(entered, "normal Enter never admitted the wounded patient");
            assert!(
                completed,
                "{hospital_type} did not heal and release the patient"
            );
            assert_eq!(
                sim.entry_click_action(
                    patient,
                    hospital,
                    crate::sim::world::EntryBaseAction::Select,
                    &scenario.runtime.resources.rules,
                    Default::default()
                ),
                Some(crate::sim::world::EntryClickAction::NoEnter)
            );
            assert_eq!(
                sim.entities()
                    .get(hospital)
                    .unwrap()
                    .ammo()
                    .unwrap()
                    .current(),
                -1
            );
            assert_eq!(
                sim.entities()
                    .get(hospital)
                    .unwrap()
                    .passenger_role
                    .cargo()
                    .unwrap()
                    .count(),
                0
            );
        }
    }

    #[cfg(feature = "npatch")]
    #[test]
    #[ignore = "requires VERA20K_AYMOD_DIR; actual AY garrison shot awards previous-index XP"]
    fn aymod_garrison_first_kill_awards_the_firing_occupant() {
        use crate::sim::command::{Command, CommandEnvelope};
        let (mut scenario, building, target) = aymod_nearby_pair("CAHSE01", "E2", false);
        let riders = {
            let runtime = &mut scenario.runtime;
            let (sim, rules) = (&mut runtime.simulation, &runtime.resources.rules);
            let object = rules.object("CAHSE01").unwrap();
            assert!(object.can_be_occupied && !object.trainable);
            assert!(rules.object("E1").unwrap().occupy_weapon.is_some());
            let owner = sim.entities().get(building).unwrap().owner();
            let at = sim.entities().get(building).unwrap().position;
            let riders = spawn_on_free_cells(
                sim,
                rules,
                owner,
                cells_nearest((i32::from(at.rx()), i32::from(at.ry()))),
                &["E1"; 2],
            );
            for &rider in &riders {
                assert!(crate::sim::passenger::add_garrison_occupant(
                    sim,
                    rules,
                    rider,
                    building,
                    Some(&runtime.resources.overlay_registry),
                    Default::default()
                ));
                assert_eq!(sim.entities().get(rider).unwrap().veterancy.raw().bits(), 0);
            }
            // A wounded target isolates the first actual shot, without changing
            // authored types/weapons or supplying a kill/experience callback.
            sim.substrate
                .entities
                .get_mut(target)
                .unwrap()
                .health
                .set_for_test(1);
            sim.queue_command(CommandEnvelope::new(
                owner,
                sim.session.tick() + 1,
                Command::Attack {
                    attacker_id: building,
                    target_id: target,
                },
            ));
            riders
        };
        let occupants = scenario
            .sim()
            .entities()
            .get(building)
            .unwrap()
            .passenger_role
            .occupants();
        let shooter = occupants.ids()[occupants.fire_index() as usize];
        let next = *riders.iter().find(|&&id| id != shooter).unwrap();
        for _ in 0..180 {
            tick_with_due_commands(&mut scenario);
            if scenario
                .sim()
                .entities()
                .get(target)
                .is_none_or(|e| e.health.current() <= 0)
            {
                break;
            }
        }
        let sim = scenario.sim();
        assert!(
            sim.entities()
                .get(target)
                .is_none_or(|e| e.health.current() <= 0)
        );
        assert!(sim.entities().get(shooter).unwrap().veterancy.raw().bits() > 0);
        assert_eq!(sim.entities().get(next).unwrap().veterancy.raw().bits(), 0);
        assert_eq!(
            sim.entities().get(building).unwrap().veterancy.raw().bits(),
            0
        );
    }

    #[cfg(feature = "npatch")]
    fn aymod_garrison_missile_in_flight(
        target_type: &str,
    ) -> (HeadlessScenario, u64, u64, Vec<u64>, u64) {
        use crate::sim::command::{Command, CommandEnvelope};
        use crate::sim::mission::{MissionId, MissionType};
        let mut scene = aymod_battle(0);
        let (building, target, old, replacement) = {
            let runtime = &mut scene.runtime;
            let (sim, rules) = (&mut runtime.simulation, &runtime.resources.rules);
            let owner = house_of(sim, "Americans");
            let enemy = house_of(sim, "Russians");
            let sites = cells_nearest(aymod_anchor(sim, owner));
            let building = spawn_on_free_cells(sim, rules, owner, sites, &["CAHSE01"])[0];
            let at = sim.entities().get(building).unwrap().position;
            let coord = crate::sim::movement::ground_pose::object_get_coords(
                sim.entities().get(building).unwrap(),
                sim.resolved_terrain(),
            );
            let center = (coord.x / 256, coord.y / 256);
            let far = cells_nearest(center)
                .into_iter()
                .filter(|&(x, y)| {
                    (x - center.0).abs().max((y - center.1).abs()) == 6
                        && u16::try_from(x)
                            .ok()
                            .zip(u16::try_from(y).ok())
                            .is_some_and(|(x, y)| sim.terrain_cell_level(x, y) == Some(at.z()))
                })
                .collect::<Vec<_>>();
            let target = spawn_on_free_cells(sim, rules, enemy, far, &[target_type])[0];
            let near = cells_nearest((i32::from(at.rx()), i32::from(at.ry())));
            let old = spawn_on_free_cells(sim, rules, owner, near.clone(), &["GGI"; 2]);
            let replacement = spawn_on_free_cells(sim, rules, owner, near, &["E1"])[0];
            for &rider in &old {
                assert!(crate::sim::passenger::add_garrison_occupant(
                    sim,
                    rules,
                    rider,
                    building,
                    Some(&runtime.resources.overlay_registry),
                    Default::default()
                ));
            }
            let weapon = rules
                .weapon(
                    rules
                        .object("GGI")
                        .unwrap()
                        .occupy_weapon
                        .as_deref()
                        .unwrap(),
                )
                .unwrap();
            assert_eq!(weapon.id, "OCMissileLauncher");
            assert!(
                !rules
                    .projectile(weapon.projectile.as_deref().unwrap())
                    .unwrap()
                    .inviso,
                "use an authored in-flight missile, not an instant gun"
            );
            sim.substrate
                .entities
                .get_mut(target)
                .unwrap()
                .health
                .set(1);
            sim.mission_queue_exact(
                target,
                MissionId::from_known(MissionType::Sleep),
                1,
                sim.session.binary_frame(),
                &crate::sim::mission::authority::LiveReadyInputProvider { rules },
            )
            .unwrap();
            // Keep the replacement from firing independently while outside.
            sim.mission_queue_exact(
                replacement,
                MissionId::from_known(MissionType::Sleep),
                1,
                sim.session.binary_frame(),
                &crate::sim::mission::authority::LiveReadyInputProvider { rules },
            )
            .unwrap();
            sim.queue_command(CommandEnvelope::new(
                owner,
                sim.session.tick() + 1,
                Command::Attack {
                    attacker_id: building,
                    target_id: target,
                },
            ));
            (building, target, old, replacement)
        };
        assert!(
            (0..180).any(|_| {
                let output = tick_with_due_commands(&mut scene);
                output.fire_events.iter().any(|event| {
                    event.attacker_id == building
                        && scene.sim().interner.resolve(event.weapon_id) == "OCMissileLauncher"
                })
            }),
            "actual occupied GGI must launch a missile"
        );
        assert!(
            scene.sim().entities().get(target).unwrap().health.current() > 0,
            "the shot must still be in flight when the player unloads"
        );
        (scene, building, target, old, replacement)
    }

    #[cfg(feature = "npatch")]
    #[test]
    #[ignore = "requires VERA20K_AYMOD_DIR; destroyed garrison survivors resume after player load"]
    fn aymod_garrison_destruction_releases_survivors_for_saved_reentry() {
        use crate::sim::combat::{EntityDamageEvent, ReceiverCallFlags, TargetKind};
        use crate::sim::command::{Command, CommandEnvelope};
        use crate::sim::mission::MissionType;
        use crate::sim::world::bridge_test_evidence::{restored_retail, save_scene};
        let (mut scene, building, attacker, riders, _) = aymod_garrison_missile_in_flight("APOC");
        let pristine = scene.sim().resolved_terrain().unwrap().clone();
        let (owner, shelter) = {
            let runtime = &mut scene.runtime;
            let (sim, rules) = (&mut runtime.simulation, &runtime.resources.rules);
            let actor = sim.entities().get(building).unwrap();
            let owner = actor.owner();
            let at = actor.position;
            let damage = actor.health.current() + 1;
            let center = (i32::from(at.rx()), i32::from(at.ry()));
            let sites = cells_nearest(center)
                .into_iter()
                .filter(|&(x, y)| (5..=7).contains(&(x - center.0).abs().max((y - center.1).abs())))
                .collect::<Vec<_>>();
            let shelter = spawn_on_free_cells(sim, rules, owner, sites, &["CAHSE01"])[0];
            let attacker_owner = sim.entities().get(attacker).unwrap().owner();
            let weapon = rules
                .weapon(rules.object("APOC").unwrap().primary().unwrap())
                .unwrap();
            let warhead = sim.interner.intern(weapon.warhead.as_deref().unwrap());
            // Enter the production lethal receiver, not the ejection helper.
            // Building442625..44263B calls SellBuilding(0,0) for survivors.
            let hit = EntityDamageEvent::direct_receiver(
                building,
                damage,
                0,
                attacker,
                Some(attacker_owner),
                warhead,
                ReceiverCallFlags {
                    ignore_defenses: true,
                    arg6: false,
                },
            );
            sim.commit_direct_damage_receiver(
                rules,
                Some(&runtime.resources.overlay_registry),
                hit,
                Default::default(),
            );
            assert!(sim.entities().get(building).is_none_or(|actor| {
                actor.health.current() <= 0 && actor.passenger_role.occupants().is_empty()
            }));
            for &id in &riders {
                let survivor = sim.entities().get(id).unwrap();
                assert!(survivor.is_object_alive() && !survivor.lifecycle.in_limbo());
                assert_eq!(survivor.owner(), owner);
                assert_eq!(survivor.passenger_role.inside_transport_id(), None);
                assert_eq!(survivor.passenger_role.open_transport_id(), None);
                let (w, h) = rules.object("CAHSE01").unwrap().foundation.dimensions();
                assert!(
                    survivor.position.rx() < at.rx()
                        || survivor.position.rx() >= at.rx() + w
                        || survivor.position.ry() < at.ry()
                        || survivor.position.ry() >= at.ry() + h,
                    "a live occupant must leave the destroyed foundation"
                );
            }
            (owner, shelter)
        };
        // Complete ordinary deferred retirement before serializing. The
        // already-launched missile is allowed to finish through its own owner.
        tick_with_due_commands(&mut scene);
        let bytes = save_scene(&scene, "AY destroyed garrison with live survivors");
        scene.runtime.simulation = restored_retail(&scene, &pristine, &bytes);
        for &id in &riders {
            let survivor = scene.sim().entities().get(id).unwrap();
            assert!(survivor.is_object_alive() && !survivor.lifecycle.in_limbo());
            assert_eq!(survivor.passenger_role.inside_transport_id(), None);
            let tick = scene.sim().session.tick() + 1;
            scene.runtime.simulation.queue_command(CommandEnvelope::new(
                owner,
                tick,
                // Occupier PerCell519675 uses Capture, not transport Enter.
                Command::clicked_mission(id, MissionType::Capture, TargetKind::Entity(shelter)),
            ));
        }
        assert!(
            (0..900).any(|_| {
                tick_with_due_commands(&mut scene);
                let occupants = scene
                    .sim()
                    .entities()
                    .get(shelter)
                    .unwrap()
                    .passenger_role
                    .occupants();
                riders.iter().all(|id| occupants.ids().contains(id))
            }),
            "restored survivors must walk and enter another building: {:?}",
            riders
                .iter()
                .map(|id| scene.sim().entities().get(*id).map(|actor| (
                    &actor.mission,
                    &actor.position,
                    &actor.passenger_role,
                    &actor.navigation
                )))
                .collect::<Vec<_>>()
        );
        let bytes = save_scene(&scene, "AY survivors after entering a new garrison");
        scene.runtime.simulation = restored_retail(&scene, &pristine, &bytes);
        let occupants = scene
            .sim()
            .entities()
            .get(shelter)
            .unwrap()
            .passenger_role
            .occupants();
        assert_eq!(occupants.ids().len(), riders.len());
        assert!(riders.iter().all(|id| occupants.ids().contains(id)));
        for id in riders {
            let survivor = scene.sim().entities().get(id).unwrap();
            assert!(survivor.is_object_alive() && survivor.lifecycle.in_limbo());
            assert_eq!(survivor.passenger_role.inside_transport_id(), Some(shelter));
        }
    }

    #[cfg(feature = "npatch")]
    #[test]
    #[ignore = "requires VERA20K_AYMOD_DIR; live garrison changes before a saved missile impact"]
    fn aymod_garrison_missile_uses_the_live_occupants_after_unload_and_save() {
        use crate::sim::combat::TargetKind;
        use crate::sim::command::{Command, CommandEnvelope};
        use crate::sim::mission::{MissionId, MissionType};
        use crate::sim::world::bridge_test_evidence::{restored_retail, save_scene};
        for replace in [false, true] {
            let (mut scene, building, target, old, replacement) =
                aymod_garrison_missile_in_flight("APOC");
            let pristine = scene.sim().resolved_terrain().unwrap().clone();
            {
                let sim = &mut scene.runtime.simulation;
                let owner = sim.entities().get(building).unwrap().owner();
                // Building ClickedAction443557..443562 sends target=self,
                // unlike the target-less convenience UnloadPassengers payload.
                sim.queue_command(CommandEnvelope::new(
                    owner,
                    sim.session.tick() + 1,
                    Command::MegaMission {
                        entity_id: building,
                        mission: MissionType::Unload,
                        target: Some(TargetKind::Entity(building)),
                        destination: None,
                    },
                ));
            }
            assert!((0..30).any(|_| {
                tick_with_due_commands(&mut scene);
                assert!(
                    scene.sim().entities().get(target).unwrap().health.current() > 0,
                    "the normal unload must precede impact"
                );
                scene
                    .sim()
                    .entities()
                    .get(building)
                    .unwrap()
                    .passenger_role
                    .occupants()
                    .is_empty()
            }));
            {
                let runtime = &mut scene.runtime;
                let (sim, rules) = (&mut runtime.simulation, &runtime.resources.rules);
                if replace {
                    // Enter the existing garrison admission leaf. Full walking
                    // admission is separate; do not force a path into a short
                    // missile window or edit the authored projectile speed.
                    assert!(crate::sim::passenger::add_garrison_occupant(
                        sim,
                        rules,
                        replacement,
                        building,
                        Some(&runtime.resources.overlay_registry),
                        Default::default()
                    ));
                }
                sim.mission_queue_exact(
                    building,
                    MissionId::from_known(MissionType::Sleep),
                    1,
                    sim.session.binary_frame(),
                    &crate::sim::mission::authority::LiveReadyInputProvider { rules },
                )
                .unwrap();
            }
            let occupants = scene
                .sim()
                .entities()
                .get(building)
                .unwrap()
                .passenger_role
                .occupants();
            assert_eq!(occupants.fire_index(), 0);
            assert_eq!(
                occupants.ids(),
                if replace {
                    std::slice::from_ref(&replacement)
                } else {
                    &[]
                }
            );
            let bytes = save_scene(&scene, "AY missile after changed garrison");
            scene.runtime.simulation = restored_retail(&scene, &pristine, &bytes);
            assert!(
                (0..180).any(|_| {
                    let output = tick_with_due_commands(&mut scene);
                    assert!(
                        !output
                            .fire_events
                            .iter()
                            .any(|event| event.attacker_id == building),
                        "only the original missile may kill; the replacement must not shoot"
                    );
                    scene
                        .sim()
                        .entities()
                        .get(target)
                        .is_none_or(|actor| actor.health.current() <= 0)
                }),
                "the already-fired missile must survive unload and validated load"
            );
            let sim = scene.sim();
            for rider in old {
                let rider = sim.entities().get(rider).unwrap();
                assert!(!rider.lifecycle.in_limbo());
                assert_eq!(
                    rider.veterancy.raw().bits(),
                    0,
                    "native NP credit uses the live list at impact, not a retained original shooter"
                );
            }
            assert_eq!(
                sim.entities()
                    .get(replacement)
                    .unwrap()
                    .veterancy
                    .raw()
                    .bits()
                    > 0,
                replace
            );
            assert_eq!(
                sim.entities().get(building).unwrap().veterancy.raw().bits(),
                0
            );
        }
    }

    #[cfg(feature = "npatch")]
    #[test]
    #[ignore = "requires VERA20K_AYMOD_DIR; two real garrison missiles overlap across different firing frames"]
    fn aymod_garrison_overlapping_missiles_use_the_latest_live_cursor_after_load() {
        aymod_garrison_overlapping_missiles_after_load(false);
    }

    #[cfg(feature = "npatch")]
    #[test]
    #[ignore = "requires VERA20K_AYMOD_DIR; a real pending missile survives its firing occupant's penetration death"]
    fn aymod_garrison_second_shooter_death_redirects_both_pending_kills_after_load() {
        aymod_garrison_overlapping_missiles_after_load(true);
    }

    #[cfg(feature = "npatch")]
    fn aymod_garrison_overlapping_missiles_after_load(retire_second: bool) {
        use crate::sim::command::{Command, CommandEnvelope};
        use crate::sim::mission::{MissionId, MissionType};
        use crate::sim::projectile::ProjectileTarget;
        use crate::sim::world::bridge_test_evidence::{restored_retail, save_scene};
        let (mut scene, building, first, riders, replacement) =
            aymod_garrison_missile_in_flight("E2");
        let pristine = scene.sim().resolved_terrain().unwrap().clone();
        let first_frame = scene.sim().session.binary_frame();
        let first_missile = scene
            .sim()
            .projectiles()
            .iter()
            .find_map(|(&id, p)| {
                (p.source_id() == building && p.target() == ProjectileTarget::Entity(first))
                    .then_some(id)
            })
            .unwrap();
        let second = {
            let runtime = &mut scene.runtime;
            let (sim, rules) = (&mut runtime.simulation, &runtime.resources.rules);
            let actor = sim.entities().get(building).unwrap();
            let owner = actor.owner();
            let level = actor.position.z();
            let at =
                crate::sim::movement::ground_pose::object_get_coords(actor, sim.resolved_terrain());
            let center = (at.x / 256, at.y / 256);
            let sites = cells_nearest(center)
                .into_iter()
                .filter(|&(x, y)| {
                    (x - center.0).abs().max((y - center.1).abs()) == 6
                        && u16::try_from(x)
                            .ok()
                            .zip(u16::try_from(y).ok())
                            .is_some_and(|(x, y)| sim.terrain_cell_level(x, y) == Some(level))
                })
                .collect::<Vec<_>>();
            let enemy = house_of(sim, "Russians");
            let second = spawn_on_free_cells(sim, rules, enemy, sites, &["E2"])[0];
            sim.substrate
                .entities
                .get_mut(second)
                .unwrap()
                .health
                .set(1);
            sim.mission_queue_exact(
                second,
                MissionId::from_known(MissionType::Sleep),
                1,
                sim.session.binary_frame(),
                &crate::sim::mission::authority::LiveReadyInputProvider { rules },
            )
            .unwrap();
            sim.queue_command(CommandEnvelope::new(
                owner,
                sim.session.tick() + 1,
                Command::Attack {
                    attacker_id: building,
                    target_id: second,
                },
            ));
            second
        };
        assert!((0..180).any(|_| {
            let output = tick_with_due_commands(&mut scene);
            assert!(
                scene
                    .sim()
                    .entities()
                    .get(first)
                    .is_some_and(|e| e.health.current() > 0),
                "the first missile must remain in flight until the next ordinary shot"
            );
            output.fire_events.iter().any(|e| e.attacker_id == building)
        }));
        assert!(scene.sim().session.binary_frame() > first_frame);
        let second_missile = scene
            .sim()
            .projectiles()
            .iter()
            .find_map(|(&id, p)| {
                (p.source_id() == building && p.target() == ProjectileTarget::Entity(second))
                    .then_some(id)
            })
            .unwrap();
        assert_ne!(first_missile, second_missile);
        assert!(scene.sim().projectiles().get(first_missile).is_some());
        assert_eq!(
            scene
                .sim()
                .entities()
                .get(building)
                .unwrap()
                .passenger_role
                .occupants()
                .fire_index(),
            0
        );
        if retire_second {
            use crate::sim::combat::{EntityDamageEvent, RAD_NO_ATTACKER, ReceiverCallFlags};
            let runtime = &mut scene.runtime;
            let (sim, resources) = (&mut runtime.simulation, &runtime.resources);
            let warhead = sim.interner.intern("AWPWH");
            // Actual NP penetration removes the last occupant (the second
            // missile's real shooter). Unlike bare UnInit, this producer owns
            // list erasure outside ScenarioInit; the live cursor stays zero.
            sim.commit_direct_damage_receiver(
                &resources.rules,
                Some(&resources.overlay_registry),
                EntityDamageEvent::direct_receiver(
                    building,
                    0,
                    0,
                    RAD_NO_ATTACKER,
                    None,
                    warhead,
                    ReceiverCallFlags {
                        ignore_defenses: false,
                        arg6: false,
                    },
                ),
                Default::default(),
            );
            let roster = sim
                .entities()
                .get(building)
                .unwrap()
                .passenger_role
                .occupants();
            assert_eq!(roster.ids(), &riders[..1]);
            assert_eq!(roster.fire_index(), 0);
            assert!(!sim.entities().get(riders[1]).unwrap().is_object_alive());
        }
        {
            let runtime = &mut scene.runtime;
            runtime
                .simulation
                .mission_queue_exact(
                    building,
                    MissionId::from_known(MissionType::Sleep),
                    1,
                    runtime.simulation.session.binary_frame(),
                    &crate::sim::mission::authority::LiveReadyInputProvider {
                        rules: &runtime.resources.rules,
                    },
                )
                .unwrap();
        }
        if retire_second {
            tick_with_due_commands(&mut scene);
            assert!(scene.sim().entities().get(riders[1]).is_none());
        }
        let bytes = save_scene(
            &scene,
            "AY two garrison missiles from distinct frames in flight",
        );
        scene.runtime.simulation = restored_retail(&scene, &pristine, &bytes);
        for id in [first_missile, second_missile] {
            assert!(scene.sim().projectiles().get(id).is_some());
        }
        let roster = scene
            .sim()
            .entities()
            .get(building)
            .unwrap()
            .passenger_role
            .occupants();
        assert_eq!(roster.fire_index(), 0);
        assert_eq!(
            roster.ids(),
            if retire_second {
                &riders[..1]
            } else {
                &riders[..]
            }
        );
        let xp = |scene: &HeadlessScenario, id| {
            scene
                .sim()
                .entities()
                .get(id)
                .unwrap()
                .veterancy
                .raw()
                .bits()
        };
        let recipient = if retire_second { riders[0] } else { riders[1] };
        let (mut dead, mut awards, mut previous) = ([false; 2], 0, 0);
        for _ in 0..240 {
            let output = tick_with_due_commands(&mut scene);
            assert!(!output.fire_events.iter().any(|e| e.attacker_id == building));
            let now = [first, second].map(|id| {
                scene
                    .sim()
                    .entities()
                    .get(id)
                    .is_none_or(|e| e.health.current() <= 0)
            });
            if now != dead {
                assert!(
                    xp(&scene, recipient) > previous,
                    "each original impact must award the current cursor-1 rider"
                );
                previous = xp(&scene, recipient);
                if !retire_second {
                    assert_eq!(
                        xp(&scene, riders[0]),
                        0,
                        "the first missile must not retain its historical shooter"
                    );
                }
                awards += 1;
                dead = now;
            }
            if dead == [true; 2] {
                break;
            }
        }
        assert_eq!(dead, [true; 2]);
        assert_eq!(
            awards, 2,
            "require two independently observed impact frames"
        );
        for id in [first_missile, second_missile] {
            assert!(scene.sim().projectiles().get(id).is_none());
        }
        for id in [building, replacement] {
            assert_eq!(xp(&scene, id), 0);
        }
    }

    #[cfg(feature = "npatch")]
    #[test]
    #[ignore = "requires VERA20K_AYMOD_DIR; partial garrison death, saved live cursor and refill"]
    fn aymod_garrison_tail_death_preserves_the_pending_shot_and_cursor_after_load() {
        use crate::sim::combat::{
            EntityDamageEvent, RAD_NO_ATTACKER, ReceiverCallFlags, TargetKind, combat_weapon,
        };
        use crate::sim::command::{Command, CommandEnvelope};
        use crate::sim::mission::{MissionId, MissionType};
        use crate::sim::world::bridge_test_evidence::{restored_retail, save_scene};
        let (mut scene, building, target, old, replacement) = aymod_garrison_missile_in_flight("APOC");
        let pristine = scene.sim().resolved_terrain().unwrap().clone();
        assert_eq!(
            scene
                .sim()
                .entities()
                .get(building)
                .unwrap()
                .passenger_role
                .occupants()
                .fire_index(),
            1
        );
        {
            let runtime = &mut scene.runtime;
            let (sim, rules) = (&mut runtime.simulation, &runtime.resources.rules);
            let hp = sim.entities().get(building).unwrap().health.current();
            let wh = rules.warhead("AWPWH").unwrap();
            assert_eq!(wh.bunker_penetration_level(), 1);
            assert!(
                wh.bunker_penetration_level()
                    > rules.object("CAHSE01").unwrap().bunker_protection_level()
            );
            let warhead = sim.interner.intern(&wh.id);
            // Enter the real NP damage prefix with zero ordinary damage.
            // Actual sniper firing is covered separately; this isolates the
            // roster mutation between one real missile's launch and impact.
            let hit = EntityDamageEvent::direct_receiver(
                building,
                0,
                0,
                RAD_NO_ATTACKER,
                None,
                warhead,
                ReceiverCallFlags {
                    ignore_defenses: false,
                    arg6: false,
                },
            );
            sim.commit_direct_damage_receiver(
                rules,
                Some(&runtime.resources.overlay_registry),
                hit,
                Default::default(),
            );
            assert_eq!(sim.entities().get(building).unwrap().health.current(), hp);
            assert!(
                sim.entities()
                    .get(old[1])
                    .is_none_or(|rider| !rider.is_object_alive())
            );
            let actor = sim.entities().get(building).unwrap();
            assert_eq!(actor.passenger_role.occupants().ids(), &old[..1]);
            assert_eq!(
                actor.passenger_role.occupants().fire_index(),
                1,
                "penetration/PointerExpired do not reset the live fire cursor"
            );
            assert_eq!(
                combat_weapon::firing_occupant_id(actor, rules.object("CAHSE01").unwrap()),
                None,
                "GetWeapon45274E falls back to the building's slots, not a wrapped survivor"
            );
            sim.mission_queue_exact(
                building,
                MissionId::from_known(MissionType::Sleep),
                1,
                sim.session.binary_frame(),
                &crate::sim::mission::authority::LiveReadyInputProvider { rules },
            )
            .unwrap();
        }
        // UnInit marks the object dead synchronously; physical retirement
        // belongs to the normal end-of-frame queue, not the damage callback.
        tick_with_due_commands(&mut scene);
        assert!(scene.sim().entities().get(old[1]).is_none());
        assert!(scene.sim().entities().get(target).unwrap().health.current() > 0);
        let bytes = save_scene(&scene, "AY partial garrison death with a pending missile");
        scene.runtime.simulation = restored_retail(&scene, &pristine, &bytes);
        let actor = scene.sim().entities().get(building).unwrap();
        assert_eq!(actor.passenger_role.occupants().ids(), &old[..1]);
        assert_eq!(
            actor.passenger_role.occupants().fire_index(),
            1,
            "load must preserve this valid native count-one/cursor-one state"
        );
        assert!(
            (0..180).any(|_| {
                let output = tick_with_due_commands(&mut scene);
                assert!(
                    !output
                        .fire_events
                        .iter()
                        .any(|event| event.attacker_id == building)
                );
                scene
                    .sim()
                    .entities()
                    .get(target)
                    .is_none_or(|victim| victim.health.current() <= 0)
            }),
            "the pending missile must still kill after partial death and restore"
        );
        let first_xp = scene.sim().entities().get(old[0]).unwrap().veterancy.raw();
        assert!(
            first_xp.bits() > 0,
            "NP previous live index is zero even though the current cursor is now out of the live list"
        );
        assert_eq!(
            scene
                .sim()
                .entities()
                .get(replacement)
                .unwrap()
                .veterancy
                .raw()
                .bits(),
            0
        );
        let next = {
            let runtime = &mut scene.runtime;
            let (sim, rules) = (&mut runtime.simulation, &runtime.resources.rules);
            assert!(crate::sim::passenger::add_garrison_occupant(
                sim,
                rules,
                replacement,
                building,
                Some(&runtime.resources.overlay_registry),
                Default::default()
            ));
            let actor = sim.entities().get(building).unwrap();
            assert_eq!(actor.passenger_role.occupants().fire_index(), 1);
            assert_eq!(
                combat_weapon::firing_occupant_id(actor, rules.object("CAHSE01").unwrap()),
                Some(replacement)
            );
            let coord =
                crate::sim::movement::ground_pose::object_get_coords(actor, sim.resolved_terrain());
            let center = (coord.x / 256, coord.y / 256);
            let sites = cells_nearest(center)
                .into_iter()
                .filter(|&(x, y)| (3..=4).contains(&(x - center.0).abs().max((y - center.1).abs())))
                .collect::<Vec<_>>();
            let enemy = house_of(sim, "Russians");
            let next = spawn_on_free_cells(sim, rules, enemy, sites, &["E2"])[0];
            sim.substrate.entities.get_mut(next).unwrap().health.set(1);
            sim.mission_queue_exact(
                next,
                MissionId::from_known(MissionType::Sleep),
                1,
                sim.session.binary_frame(),
                &crate::sim::mission::authority::LiveReadyInputProvider { rules },
            )
            .unwrap();
            let owner = sim.entities().get(building).unwrap().owner();
            sim.queue_command(CommandEnvelope::new(
                owner,
                sim.session.tick() + 1,
                Command::Attack {
                    attacker_id: building,
                    target_id: next,
                },
            ));
            next
        };
        let mut fired = false;
        assert!(
            (0..180).any(|_| {
                let output = tick_with_due_commands(&mut scene);
                for event in output
                    .fire_events
                    .iter()
                    .filter(|event| event.attacker_id == building)
                {
                    assert_eq!(event.target, TargetKind::Entity(next));
                    assert_eq!(scene.sim().interner.resolve(event.weapon_id), "UCPara");
                    fired = true;
                }
                scene
                    .sim()
                    .entities()
                    .get(next)
                    .is_none_or(|victim| victim.health.current() <= 0)
            }),
            "normal Attack must use the replacement in the still-current slot"
        );
        assert!(fired);
        assert!(
            scene
                .sim()
                .entities()
                .get(replacement)
                .unwrap()
                .veterancy
                .raw()
                .bits()
                > 0
        );
        assert_eq!(
            scene.sim().entities().get(old[0]).unwrap().veterancy.raw(),
            first_xp
        );
        assert_eq!(
            scene
                .sim()
                .entities()
                .get(building)
                .unwrap()
                .passenger_role
                .occupants()
                .fire_index(),
            0
        );
    }

    #[cfg(feature = "npatch")]
    #[test]
    #[ignore = "requires VERA20K_AYMOD_DIR; real AY sniper and MiG garrison penetration"]
    fn aymod_sniper_respects_bunker_protection_and_boris_sortie_clears_it() {
        use crate::sim::command::{Command, CommandEnvelope};
        // Battle setup uses the production garrison owner; ordinary Capture
        // approach/admission is separately exercised by the storage regression.
        fn occupy(scenario: &mut HeadlessScenario, building: u64) -> Vec<u64> {
            let runtime = &mut scenario.runtime;
            let sim = &mut runtime.simulation;
            let owner = sim.entities().get(building).unwrap().owner();
            let at = sim.entities().get(building).unwrap().position;
            let sites = cells_nearest((i32::from(at.rx()), i32::from(at.ry())));
            let riders =
                spawn_on_free_cells(sim, &runtime.resources.rules, owner, sites, &["E2"; 2]);
            for &rider in &riders {
                assert!(crate::sim::passenger::add_garrison_occupant(
                    sim,
                    &runtime.resources.rules,
                    rider,
                    building,
                    Some(&runtime.resources.overlay_registry),
                    Default::default()
                ));
            }
            riders
        }
        let attack = |scenario: &mut HeadlessScenario, actor, target| {
            let sim = &mut scenario.runtime.simulation;
            let owner = sim.entities().get(actor).unwrap().owner();
            sim.queue_command(CommandEnvelope::new(
                owner,
                sim.session.tick() + 1,
                Command::Attack {
                    attacker_id: actor,
                    target_id: target,
                },
            ));
        };
        let (mut scenario, sniper, city) = aymod_nearby_pair("SNIPE", "CAHSE01", false);
        for (name, level) in [
            ("AWPWH", 1),
            ("Virus", 1),
            ("VirusM", 1),
            ("RadBeamWarhead", 1),
            ("OCRadBeamWarhead", 1),
            ("MIGWH", -2),
        ] {
            assert_eq!(
                scenario
                    .runtime
                    .resources
                    .rules
                    .warhead(name)
                    .unwrap()
                    .bunker_penetration_level(),
                level
            );
        }
        let citizens = occupy(&mut scenario, city);
        attack(&mut scenario, sniper, city);
        for _ in 0..600 {
            tick_with_due_commands(&mut scenario);
            if scenario
                .sim()
                .entities()
                .get(city)
                .unwrap()
                .passenger_role
                .occupants()
                .ids()
                .len()
                < 2
            {
                break;
            }
        }
        assert_eq!(
            scenario
                .sim()
                .entities()
                .get(city)
                .unwrap()
                .passenger_role
                .occupants()
                .ids(),
            &[citizens[0]]
        );
        let bunker = {
            let runtime = &mut scenario.runtime;
            let sim = &mut runtime.simulation;
            let enemy = house_of(sim, "Russians");
            let at = sim.entities().get(city).unwrap().position;
            spawn_on_free_cells(
                sim,
                &runtime.resources.rules,
                enemy,
                cells_nearest((i32::from(at.rx()), i32::from(at.ry()))),
                &["NABNKR"],
            )[0]
        };
        let protected = occupy(&mut scenario, bunker);
        let before = scenario
            .sim()
            .entities()
            .get(bunker)
            .unwrap()
            .health
            .current();
        attack(&mut scenario, sniper, bunker);
        for _ in 0..600 {
            tick_with_due_commands(&mut scenario);
            if scenario
                .sim()
                .entities()
                .get(bunker)
                .unwrap()
                .health
                .current()
                < before
            {
                break;
            }
        }
        let building = scenario.sim().entities().get(bunker).unwrap();
        assert!(
            building.health.current() < before,
            "the actual sniper bullet must reach the protected building"
        );
        assert_eq!(
            building.passenger_role.occupants().ids(),
            protected.as_slice()
        );
        let boris = {
            let runtime = &mut scenario.runtime;
            let sim = &mut runtime.simulation;
            let owner = sim.entities().get(sniper).unwrap().owner();
            sim.queue_command(CommandEnvelope::new(
                owner,
                sim.session.tick() + 1,
                Command::Stop { entity_id: sniper },
            ));
            let at = sim.entities().get(bunker).unwrap().position;
            let sites = cells_nearest((i32::from(at.rx()), i32::from(at.ry())))
                .into_iter()
                .filter(|&(x, y)| {
                    (x - i32::from(at.rx())).abs() + (y - i32::from(at.ry())).abs() == 10
                })
                .collect::<Vec<_>>();
            spawn_on_free_cells(sim, &runtime.resources.rules, owner, sites, &["BORIS"])[0]
        };
        attack(&mut scenario, boris, bunker);
        for _ in 0..2400 {
            tick_with_due_commands(&mut scenario);
            if scenario
                .sim()
                .entities()
                .get(bunker)
                .is_none_or(|building| building.passenger_role.occupants().is_empty())
            {
                break;
            }
        }
        let building = scenario
            .sim()
            .entities()
            .get(bunker)
            .expect("penetration must precede ordinary destruction");
        assert!(building.health.current() > 0 && building.is_object_alive());
        assert!(
            building.passenger_role.occupants().is_empty(),
            "the real Boris/Maverick3 sortie must clear the protected garrison"
        );
        assert!(protected.iter().all(|id| {
            scenario
                .sim()
                .entities()
                .get(*id)
                .is_none_or(|actor| !actor.is_object_alive())
        }));
    }


    #[cfg(feature = "npatch")]
    #[test]
    #[ignore = "requires VERA20K_AYMOD_DIR; actual dolphin rescue, retained Sonic wave and save/load cleanup"]
    fn aymod_dolphin_rescues_a_grappled_destroyer_and_retires_its_saved_wave() {
        visit_aymod_dolphin_rescue_after_load(false, |_, _, _| {});
    }

    #[cfg(feature = "npatch")]
    #[test]
    #[ignore = "requires VERA20K_AYMOD_DIR; grabbed host owner transfer followed by real dolphin rescue"]
    fn aymod_dolphin_rescues_a_transferred_grappled_destroyer_after_load() {
        visit_aymod_dolphin_rescue_after_load(true, |_, _, _| {});
    }


    #[cfg(feature = "npatch")]
    #[test]
    #[ignore = "requires VERA20K_AYMOD_DIR; rookie and elite DESO deploy rescue"]
    fn aymod_radiation_deploy_releases_a_live_squid() {
        use crate::sim::command::{Command, CommandEnvelope};
        for elite in [false, true] {
            let (mut scene, squid, ship, land, anim) = aymod_grappled_ship_at_shore();
            let expected = if elite {
                "RadEruptionWeaponE"
            } else {
                "RadEruptionWeapon"
            };
            let actor = {
                let runtime = &mut scene.runtime;
                let (sim, rules) = (&mut runtime.simulation, &runtime.resources.rules);
                let weapon = rules.weapon(expected).unwrap();
                assert!(weapon.rad_level > 0);
                assert!(
                    rules
                        .warhead(weapon.warhead.as_deref().unwrap())
                        .unwrap()
                        .sonic
                );
                let owner = sim.entities().get(ship).unwrap().owner();
                let owner_name = sim.interner.resolve(owner).to_owned();
                let actor = sim
                    .spawn_object("DESO", &owner_name, land.0, land.1, 0, rules)
                    .unwrap();
                if elite {
                    // Rank is a fixture boundary here, not a promotion claim.
                    sim.substrate
                        .entities
                        .get_mut(actor)
                        .unwrap()
                        .veterancy
                        .set_elite();
                }
                sim.queue_command(CommandEnvelope::new(
                    owner,
                    sim.session.tick() + 1,
                    Command::ToggleInfantryDeploy { entity_id: actor },
                ));
                actor
            };
            let mut fired = false;
            let released = (0..360).any(|_| {
                let output = tick_with_due_commands(&mut scene);
                for event in output.fire_events.iter().filter(|e| e.attacker_id == actor) {
                    assert_eq!(scene.sim().interner.resolve(event.weapon_id), expected);
                    assert_eq!(
                        event.weapon_slot,
                        crate::sim::combat::combat_weapon::WeaponSlot::Secondary
                    );
                    assert_eq!(
                        event.target,
                        crate::sim::combat::TargetKind::Cell(land.0, land.1)
                    );
                    fired = true;
                }
                scene
                    .sim()
                    .entities()
                    .get(ship)
                    .is_some_and(|e| e.parasite.eater().is_none())
            });
            assert!(
                fired && released,
                "{expected}: normal deployment must rescue the host"
            );
            let sim = scene.sim();
            assert!(
                sim.radiation.site_at(land).is_some(),
                "actual deployment must create its field"
            );
            let host = sim.entities().get(ship).unwrap();
            assert!(
                sim.radiation
                    .cell_level((host.position.rx(), host.position.ry()))
                    > 0.0
            );
            assert!(
                host.health.current() > 0
                    && !host.parasite.is_paralyzed(sim.session.binary_frame())
            );
            let source = sim.entities().get(squid).unwrap();
            assert!(source.health.current() > 0 && !source.lifecycle.in_limbo());
            assert_eq!(source.parasite.class().unwrap().victim(), None);
            assert_eq!(source.parasite.class().unwrap().grapple_anim(), None);
            tick_with_due_commands(&mut scene);
            assert!(scene.sim().anim(anim).is_none());
        }
    }

    #[cfg(feature = "npatch")]
    #[test]
    #[ignore = "requires VERA20K_AYMOD_DIR; saved radiation field without a live shooter"]
    fn aymod_radiation_field_releases_a_live_squid_after_load() {
        use crate::sim::world::bridge_test_evidence::{restored_retail, save_scene};
        let (mut scene, squid, ship, _, anim) = aymod_grappled_ship_at_shore();
        let pristine = scene.sim().resolved_terrain().unwrap().clone();
        let center = {
            let runtime = &mut scene.runtime;
            let (sim, rules) = (&mut runtime.simulation, &runtime.resources.rules);
            let weapon = rules.weapon("RadEruptionWeapon").unwrap();
            let spread = crate::sim::combat::cell_spread::whole_cells(
                rules
                    .warhead(weapon.warhead.as_deref().unwrap())
                    .unwrap()
                    .cell_spread_f64,
            );
            let site_warhead = rules.warhead(&rules.radiation.site_warhead).unwrap();
            assert_eq!(site_warhead.id, "RadSite");
            assert!(site_warhead.sonic && site_warhead.radiation);
            let host = sim.entities().get(ship).unwrap();
            let center = (host.position.rx(), host.position.ry());
            // Enter the existing radiation producer boundary without a bullet
            // or direct damage. Only the periodic Foot AI may release the host.
            sim.radiation.apply_detonation(
                crate::sim::radiation::RadDetonation {
                    rx: center.0,
                    ry: center.1,
                    rad_level: weapon.rad_level,
                    spread,
                    rad_type: weapon.rad_type.clone(),
                    firing_house: None,
                    invoker: None,
                },
                sim.session.binary_frame(),
                sim.session.theater(),
                rules,
                sim.resolved_terrain.as_ref(),
                &mut sim.lighting_sources,
            );
            assert_eq!(
                sim.entities().get(ship).unwrap().parasite.eater(),
                Some(squid)
            );
            assert!(sim.radiation.cell_level(center) > 0.0);
            center
        };
        let bytes = save_scene(&scene, "AY pending periodic RadSite rescue");
        scene.runtime.simulation = restored_retail(&scene, &pristine, &bytes);
        assert!(scene.sim().radiation.cell_level(center) > 0.0);
        assert_eq!(
            scene.sim().entities().get(ship).unwrap().parasite.eater(),
            Some(squid)
        );
        let frames = scene
            .runtime
            .resources
            .rules
            .radiation
            .application_delay
            .max(1)
            + 1;
        let released = (0..frames).any(|_| {
            let output = tick_with_due_commands(&mut scene);
            assert!(!output.fire_events.iter().any(|e| e.attacker_id == ship));
            scene
                .sim()
                .entities()
                .get(ship)
                .is_some_and(|e| e.parasite.eater().is_none())
        });
        assert!(
            released,
            "saved RadSite must release through periodic Foot damage alone"
        );
        let sim = scene.sim();
        let source = sim.entities().get(squid).unwrap();
        assert!(source.health.current() > 0 && !source.lifecycle.in_limbo());
        assert_eq!(source.parasite.class().unwrap().victim(), None);
        let host = sim.entities().get(ship).unwrap();
        assert!(
            host.health.current() > 0 && !host.parasite.is_paralyzed(sim.session.binary_frame())
        );
        tick_with_due_commands(&mut scene);
        assert!(scene.sim().anim(anim).is_none());
    }

    #[cfg(feature = "npatch")]
    #[test]
    #[ignore = "requires VERA20K_AYMOD_DIR; live squid release, recapture and save/resume"]
    fn aymod_radiation_beam_releases_a_live_squid_from_its_ship() {
        use crate::rules::terrain_rules::LandType;
        use crate::sim::command::{Command, CommandEnvelope};
        use crate::sim::world::bridge_test_evidence::{restored_retail, save_scene};
        let (mut scene, squid, ship, land, held_anim) = aymod_grappled_ship_at_shore();
        let pristine = scene.sim().resolved_terrain().unwrap().clone();
        let bytes = save_scene(&scene, "AY live squid grapple");
        scene.runtime.simulation = restored_retail(&scene, &pristine, &bytes);
        assert_eq!(
            scene.sim().entities().get(ship).unwrap().parasite.eater(),
            Some(squid)
        );
        let state = scene
            .sim()
            .entities()
            .get(squid)
            .unwrap()
            .parasite
            .class()
            .unwrap();
        assert_eq!(state.victim(), Some(ship));
        assert_eq!(state.grapple_anim(), Some(held_anim));
        assert!(scene.sim().anim(held_anim).is_some());
        let beam = {
            let runtime = &mut scene.runtime;
            let (sim, rules) = (&mut runtime.simulation, &runtime.resources.rules);
            let owner = house_of(sim, "Americans");
            let wh = rules.warhead("RadBeamWarhead").unwrap();
            assert!(
                wh.radiation && wh.sonic,
                "AY uses the stock Sonic ejection prefix"
            );
            let owner_name = sim.interner.resolve(owner).to_owned();
            let actor = sim
                .spawn_object("DESO", &owner_name, land.0, land.1, 0, rules)
                .unwrap();
            sim.queue_command(CommandEnvelope::new(
                owner,
                sim.session.tick() + 1,
                Command::Attack {
                    attacker_id: actor,
                    target_id: ship,
                },
            ));
            actor
        };
        let mut fired = false;
        let released = (0..240).any(|_| {
            let output = tick_with_due_commands(&mut scene);
            for event in output.fire_events.iter().filter(|e| e.attacker_id == beam) {
                assert_eq!(
                    scene.sim().interner.resolve(event.weapon_id),
                    "RadBeamWeapon"
                );
                fired = true;
            }
            scene
                .sim()
                .entities()
                .get(ship)
                .is_some_and(|e| e.parasite.eater().is_none())
        });
        assert!(
            fired && released,
            "ordinary radiation Attack must release, not merely damage"
        );
        let sim = scene.sim();
        let source = sim
            .entities()
            .get(squid)
            .expect("ejection must not substitute squid death");
        assert!(source.health.current() > 0 && !source.lifecycle.in_limbo());
        assert_eq!(source.parasite.class().unwrap().victim(), None);
        assert_eq!(source.parasite.class().unwrap().grapple_anim(), None);
        let host = sim.entities().get(ship).unwrap();
        assert!(
            host.health.current() > 0 && !host.parasite.is_paralyzed(sim.session.binary_frame())
        );
        // The rescuer can now passively acquire and kill the released squid.
        // Sleep both defenders through the mission owner to isolate recapture.
        let runtime = &mut scene.runtime;
        for defender in [beam, ship] {
            runtime
                .simulation
                .mission_queue_exact(
                    defender,
                    crate::sim::mission::MissionId::from_known(
                        crate::sim::mission::MissionType::Sleep,
                    ),
                    1,
                    runtime.simulation.session.binary_frame(),
                    &crate::sim::mission::authority::LiveReadyInputProvider {
                        rules: &runtime.resources.rules,
                    },
                )
                .unwrap();
        }
        // Leave the hostile squid on its normal mission: release must not
        // permanently consume its ability to acquire and grapple again.
        let second_anim = (0..900)
            .find_map(|_| {
                let output = tick_with_due_commands(&mut scene);
                assert!(
                    !output
                        .fire_events
                        .iter()
                        .any(|e| e.attacker_id == beam || e.attacker_id == ship),
                    "rescuers fired during recapture isolation: {:?}",
                    output.fire_events
                );
                let sim = scene.sim();
                assert!(
                    sim.entities().get(squid).is_some(),
                    "released squid vanished at {}",
                    sim.session.binary_frame()
                );
                let state = sim.entities().get(squid)?.parasite.class()?;
                (state.victim() == Some(ship))
                    .then(|| state.grapple_anim())
                    .flatten()
            })
            .unwrap_or_else(|| {
                panic!(
                    "released AI squid did not recapture: squid={:?}, ship={:?}",
                    scene.sim().entities().get(squid).map(|e| (
                        e.health.current(),
                        &e.parasite,
                        &e.mission,
                        &e.attack_target
                    )),
                    scene.sim().entities().get(ship).map(|e| (
                        e.health.current(),
                        &e.parasite,
                        &e.mission,
                        &e.attack_target
                    ))
                )
            });
        assert_ne!(second_anim, held_anim);
        assert!(scene.sim().anim(held_anim).is_none());
        assert_eq!(
            scene.sim().entities().get(ship).unwrap().parasite.eater(),
            Some(squid)
        );
        let sim = &mut scene.runtime.simulation;
        let owner = sim.entities().get(beam).unwrap().owner();
        sim.queue_command(CommandEnvelope::new(
            owner,
            sim.session.tick() + 1,
            Command::Attack {
                attacker_id: beam,
                target_id: ship,
            },
        ));
        let mut fired_again = false;
        let released_again = (0..240).any(|_| {
            let output = tick_with_due_commands(&mut scene);
            fired_again |= output
                .fire_events
                .iter()
                .any(|event| event.attacker_id == beam);
            scene
                .sim()
                .entities()
                .get(ship)
                .is_some_and(|e| e.parasite.eater().is_none())
        });
        assert!(
            fired_again && released_again,
            "second normal beam must release the second grapple"
        );
        let bytes = save_scene(&scene, "AY released squid");
        scene.runtime.simulation = restored_retail(&scene, &pristine, &bytes);
        let sim = scene.sim();
        let source = sim.entities().get(squid).unwrap();
        assert!(source.health.current() > 0 && !source.lifecycle.in_limbo());
        assert_eq!(source.parasite.class().unwrap().victim(), None);
        assert_eq!(source.parasite.class().unwrap().grapple_anim(), None);
        let host = sim.entities().get(ship).unwrap();
        assert!(host.health.current() > 0 && host.parasite.eater().is_none());
        assert!(!host.parasite.is_paralyzed(sim.session.binary_frame()));
        // ExitUnit may place the squid on its old square. Do not order the
        // host onto that occupied square and mistake refusal for paralysis.
        let ship_goal =
            cells_nearest((i32::from(host.position.rx()), i32::from(host.position.ry())))
                .into_iter()
                .find_map(|(x, y)| {
                    let (x, y) = (u16::try_from(x).ok()?, u16::try_from(y).ok()?);
                    let cell = sim.resolved_terrain().unwrap().cell(x, y)?;
                    if cell.outside_playfield
                        || cell.bridge_facts.has_structural_bridge()
                        || cell.land_type != LandType::Water.as_index()
                        || sim.occupancy().get(x, y).is_some_and(|c| {
                            !c.is_empty_on(crate::sim::movement::locomotor::MovementLayer::Ground)
                        })
                    {
                        return None;
                    }
                    sim.foot_path_zone_precheck(
                        ship,
                        crate::sim::components::DriveCoord::cell(x, y, 0),
                        &scene.runtime.resources.rules,
                    )
                    .unwrap()
                    .then_some((x, y))
                })
                .expect("an empty reachable water cell for the released ship");
        let before = crate::sim::movement::ground_pose::position_world_xy(&host.position);
        let owner = host.owner();
        let order_tick = sim.session.tick() + 1;
        scene.runtime.simulation.queue_command(CommandEnvelope::new(
            owner,
            order_tick,
            Command::Move {
                entity_id: ship,
                target_rx: ship_goal.0,
                target_ry: ship_goal.1,
            },
        ));
        for _ in 0..60 {
            tick_with_due_commands(&mut scene);
            let sim = scene.sim();
            assert!(
                sim.anim(held_anim).is_none() && sim.anim(second_anim).is_none(),
                "old grapple animation must be collected"
            );
            if sim.entities().get(ship).is_some_and(|e| {
                crate::sim::movement::ground_pose::position_world_xy(&e.position) != before
            }) {
                return;
            }
        }
        panic!("released player ship never resumed ordinary movement toward {ship_goal:?}");
    }

    #[cfg(feature = "npatch")]
    #[test]
    #[ignore = "requires VERA20K_AYMOD_DIR; BFRT passive fire versus explicit crush approach"]
    fn aymod_fortress_passive_fire_does_not_charge_but_manual_attack_still_moves() {
        use crate::sim::command::{Command, CommandEnvelope};
        let (mut scene, actor, target) = aymod_nearby_pair("BFRT", "MTNK", false);
        {
            let runtime = &mut scene.runtime;
            let (sim, rules) = (&mut runtime.simulation, &runtime.resources.rules);
            let enemy = sim
                .interner
                .resolve(sim.entities().get(target).unwrap().owner())
                .to_owned();
            // Supplied opponent intent at the real payload boundary, not an
            // inert player event for an AI House. AI planning is not under test.
            assert!(sim.apply_command(
                &enemy,
                &Command::Attack {
                    attacker_id: target,
                    target_id: actor,
                },
                Some(rules)
            ));
        }
        let initial = {
            let sim = scene.sim();
            assert!(
                !scene
                    .runtime
                    .resources
                    .rules
                    .object("BFRT")
                    .unwrap()
                    .can_retaliate
            );
            crate::sim::movement::ground_pose::position_world_xy(
                &sim.entities().get(actor).unwrap().position,
            )
        };
        assert!(
            !scene
                .runtime
                .resources
                .rules
                .object("MTNK")
                .unwrap()
                .omni_crush_resistant
        );
        let before_hp = scene.sim().entities().get(actor).unwrap().health.current();
        let mut passive_fire = false;
        let mut took_hit = false;
        for _ in 0..90 {
            let output = tick_with_due_commands(&mut scene);
            passive_fire |= output.fire_events.iter().any(|e| e.attacker_id == actor);
            let fortress = scene.sim().entities().get(actor).unwrap();
            took_hit |= fortress.health.current() < before_hp;
            assert_eq!(
                crate::sim::movement::ground_pose::position_world_xy(&fortress.position),
                initial,
                "human passive acquisition must not become an OmniCrusher charge"
            );
        }
        let sim = scene.sim();
        let fortress = sim.entities().get(actor).unwrap();
        assert!(
            passive_fire,
            "actual passive scan must acquire and fire at the nearby enemy tank"
        );
        assert!(took_hit, "enemy actually hit the non-retaliating fortress");
        assert!(fortress.health.current() > 0 && fortress.passive_scan.acquired());
        let owner = fortress.owner();
        let order_tick = sim.session.tick() + 1;
        scene.runtime.simulation.queue_command(CommandEnvelope::new(
            owner,
            order_tick,
            Command::Attack {
                attacker_id: actor,
                target_id: target,
            },
        ));
        for _ in 0..120 {
            tick_with_due_commands(&mut scene);
            let fortress = scene
                .sim()
                .entities()
                .get(actor)
                .expect("manual approach keeps a live fortress");
            if crate::sim::movement::ground_pose::position_world_xy(&fortress.position) != initial {
                assert!(
                    !fortress.passive_scan.acquired(),
                    "explicit order clears the passive marker"
                );
                return;
            }
        }
        panic!(
            "explicit BFRT Attack must retain the native crush approach instead of disabling all movement"
        );
    }
    #[cfg(feature = "npatch")]
    #[test]
    #[ignore = "requires VERA20K_AYMOD_DIR; TTNK refuses automatic coil charging while SHK charges"]
    fn aymod_tesla_tank_does_not_charge_a_coil_but_the_trooper_does() {
        let mut scene = aymod_battle(0);
        let (tank, coil) = {
            let runtime = &mut scene.runtime;
            let (sim, rules) = (&mut runtime.simulation, &runtime.resources.rules);
            let owner = house_of(sim, "Americans");
            let sites = cells_nearest(aymod_anchor(sim, owner));
            spawn_on_free_cells(sim, rules, owner, sites, &["GAPOWR"]);
            let sites = cells_nearest(aymod_anchor(sim, owner));
            let coil = spawn_on_free_cells(sim, rules, owner, sites, &["TESLA"])[0];
            let at = sim.entities().get(coil).unwrap().position;
            let adjacent = crate::util::direction::DIRECTION_DELTAS
                .into_iter()
                .map(|(dx, dy)| {
                    (
                        i32::from(at.rx()) + i32::from(dx),
                        i32::from(at.ry()) + i32::from(dy),
                    )
                });
            let tank = spawn_on_free_cells(sim, rules, owner, adjacent, &["TTNK"])[0];
            assert!(rules.object("TESLA").unwrap().overpowerable);
            assert!(rules.object("TTNK").unwrap().secondary().is_none());
            (tank, coil)
        };
        for elite in [false, true] {
            if elite {
                scene
                    .runtime
                    .simulation
                    .substrate
                    .entities
                    .get_mut(tank)
                    .unwrap()
                    .veterancy
                    .set_elite();
            }
            for _ in 0..90 {
                tick_with_due_commands(&mut scene);
                let sim = scene.sim();
                assert!(
                    sim.entities()
                        .get(coil)
                        .unwrap()
                        .tesla_charge
                        .chargers()
                        .is_empty()
                );
                assert!(!sim.entities().get(tank).unwrap().tesla_charger.is_active());
            }
        }
        let trooper = {
            let runtime = &mut scene.runtime;
            let (sim, rules) = (&mut runtime.simulation, &runtime.resources.rules);
            assert_eq!(sim.building_operational_state(coil, rules), Some(true));
            let owner = sim.entities().get(coil).unwrap().owner();
            let at = sim.entities().get(coil).unwrap().position;
            let adjacent = crate::util::direction::DIRECTION_DELTAS
                .into_iter()
                .map(|(dx, dy)| {
                    (
                        i32::from(at.rx()) + i32::from(dx),
                        i32::from(at.ry()) + i32::from(dy),
                    )
                });
            spawn_on_free_cells(sim, rules, owner, adjacent, &["SHK"])[0]
        };
        let before_hp = scene.sim().entities().get(coil).unwrap().health.current();
        let mut fired = false;
        for _ in 0..240 {
            let output = tick_with_due_commands(&mut scene);
            fired |= output.fire_events.iter().any(|e| {
                e.attacker_id == trooper
                    && scene.sim().interner.resolve(e.weapon_id) == "AssaultBolt"
            });
            let charge = &scene.sim().entities().get(coil).unwrap().tesla_charge;
            assert!(!charge.chargers().contains(&tank));
            if fired && charge.chargers().contains(&trooper) && charge.is_overpowered() {
                assert_eq!(
                    scene.sim().entities().get(coil).unwrap().health.current(),
                    before_hp
                );
                return;
            }
        }
        panic!("real SHK Guard acquisition and AssaultBolt must charge the powered coil");
    }
    #[cfg(feature = "npatch")]
    #[test]
    #[ignore = "requires VERA20K_AYMOD_DIR; real AY Gattling kill promotion, hot save, Stop/Guard and resumed fire"]
    fn aymod_gattling_promotes_through_fire_and_resumes_after_hot_save() {
        use crate::sim::combat::veterancy::{LEVEL_ELITE, VeterancyRank};
        use crate::sim::command::{Command, CommandEnvelope};
        use crate::sim::world::bridge_test_evidence::{restored_retail, save_scene};
        for actor_type in ["YTNK", "YAGGUN"] {
            let (mut scene, actor, first_victim) = aymod_nearby_pair(actor_type, "APOC", false);
            {
                let runtime = &mut scene.runtime;
                let (sim, rules) = (&mut runtime.simulation, &runtime.resources.rules);
                if rules.object(actor_type).unwrap().powered {
                    let owner = sim.entities().get(actor).unwrap().owner();
                    let sites = cells_nearest(aymod_anchor(sim, owner));
                    spawn_on_free_cells(sim, rules, owner, sites, &["GAPOWR"]);
                }
            }
            let queue = |scene: &mut HeadlessScenario, command| {
                let sim = &mut scene.runtime.simulation;
                let owner = sim.entities().get(actor).unwrap().owner();
                sim.queue_command(CommandEnvelope::new(owner, sim.session.tick() + 1, command));
            };
            let spawn_near = |scene: &mut HeadlessScenario, kind: &str, enemy: bool| {
                let runtime = &mut scene.runtime;
                let (sim, rules) = (&mut runtime.simulation, &runtime.resources.rules);
                let owner = if enemy {
                    house_of(sim, "Russians")
                } else {
                    sim.entities().get(actor).unwrap().owner()
                };
                let at = sim.entities().get(actor).unwrap().position;
                spawn_on_free_cells(
                    sim,
                    rules,
                    owner,
                    cells_nearest((i32::from(at.rx()), i32::from(at.ry()))),
                    &[kind],
                )[0]
            };
            assert_eq!(
                scene
                    .sim()
                    .entities()
                    .get(actor)
                    .unwrap()
                    .veterancy
                    .rank_kind(),
                VeterancyRank::Rookie
            );
            // Only victim health is prepared. Every shot, death and XP award
            // runs through ordinary Attack frames and Record_The_Kill702D40.
            for round in 0..8 {
                let victim = if round == 0 {
                    first_victim
                } else {
                    spawn_near(&mut scene, "APOC", true)
                };
                scene
                    .runtime
                    .simulation
                    .substrate
                    .entities
                    .get_mut(victim)
                    .unwrap()
                    .health
                    .set_for_test(1);
                let before = scene.sim().entities().get(actor).unwrap().veterancy.raw();
                queue(
                    &mut scene,
                    Command::Attack {
                        attacker_id: actor,
                        target_id: victim,
                    },
                );
                let mut fired = false;
                let killed = (0..240).any(|_| {
                    let output = tick_with_due_commands(&mut scene);
                    fired |= output.fire_events.iter().any(|event| {
                        event.attacker_id == actor
                            && event.target == crate::sim::combat::TargetKind::Entity(victim)
                    });
                    scene
                        .sim()
                        .entities()
                        .get(victim)
                        .is_none_or(|entity| entity.health.current() <= 0)
                });
                assert!(
                    fired && killed,
                    "ordinary Gattling fire must kill the wounded victim"
                );
                let promoted = scene.sim().entities().get(actor).unwrap();
                assert_ne!(
                    promoted.veterancy.raw(),
                    before,
                    "actual weapon kills must award experience"
                );
                if promoted.veterancy.rank_kind() == VeterancyRank::Elite {
                    break;
                }
            }
            assert_eq!(
                scene
                    .sim()
                    .entities()
                    .get(actor)
                    .unwrap()
                    .veterancy
                    .rank_kind(),
                VeterancyRank::Elite
            );
            // A friendly unarmed building keeps the spin-up isolated from enemy
            // retaliation. No stage/value/rank field is set by this test.
            let target = spawn_near(&mut scene, "NACNST", false);
            queue(
                &mut scene,
                Command::Attack {
                    attacker_id: actor,
                    target_id: target,
                },
            );
            let hot = (0..1200).any(|_| {
                tick_with_due_commands(&mut scene);
                scene.sim().entities().get(actor).unwrap().gattling.stage() > 0
            });
            assert!(hot, "actual firing must advance the Gattling stage");
            let unit = scene.sim().entities().get(actor).unwrap();
            assert_eq!(unit.veterancy.rank_cache(), LEVEL_ELITE);
            let saved = (
                unit.veterancy.clone(),
                unit.gattling.stage(),
                unit.gattling.value(),
                unit.gattling.report_latch(),
            );
            let pristine = scene.sim().resolved_terrain().unwrap().clone();
            let bytes = save_scene(&scene, "AY promoted hot Gattling");
            scene.runtime.simulation = restored_retail(&scene, &pristine, &bytes);
            let loaded = scene.sim().entities().get(actor).unwrap();
            assert_eq!(
                (
                    &loaded.veterancy,
                    loaded.gattling.stage(),
                    loaded.gattling.value()
                ),
                (&saved.0, saved.1, saved.2)
            );
            // Native TechnoLoad70BF61/70C20E clears the process-local report
            // latch, so a future charge restarts its sound loop after loading.
            assert!(saved.3);
            assert!(!loaded.gattling.report_latch());
            let rules = &scene.runtime.resources.rules;
            let table = &rules.object(actor_type).unwrap().gattling_stages;
            assert!(table.rate_down() > 0);
            let cooldown =
                saved.2 / table.rate_down() + rules.general.guard_area_targeting_delay + 64;
            queue(&mut scene, Command::Stop { entity_id: actor });
            for _ in 0..2 {
                tick_with_due_commands(&mut scene);
            }
            queue(
                &mut scene,
                Command::Guard {
                    entity_id: actor,
                    target: None,
                },
            );
            let cold = (0..cooldown).any(|_| {
                tick_with_due_commands(&mut scene);
                let unit = scene.sim().entities().get(actor).unwrap();
                unit.gattling.stage() == 0 && unit.gattling.value() == 0
            });
            assert!(cold, "normal Stop/Guard must wind the loaded Gattling down");
            assert_eq!(
                scene.sim().entities().get(actor).unwrap().veterancy,
                saved.0
            );
            let victim = spawn_near(&mut scene, "APOC", true);
            queue(
                &mut scene,
                Command::Attack {
                    attacker_id: actor,
                    target_id: victim,
                },
            );
            let mut resumed = false;
            for _ in 0..240 {
                let stage = scene.sim().entities().get(actor).unwrap().gattling.stage();
                let expected = scene
                    .runtime
                    .resources
                    .rules
                    .object(actor_type)
                    .unwrap()
                    .elite_weapon_at((stage * 2) as usize)
                    .unwrap()
                    .to_owned();
                let output = tick_with_due_commands(&mut scene);
                if let Some(fire) = output
                    .fire_events
                    .iter()
                    .find(|event| event.attacker_id == actor)
                {
                    assert_eq!(scene.sim().interner.resolve(fire.weapon_id), expected);
                    resumed = true;
                    break;
                }
            }
            assert!(
                resumed,
                "{actor_type}: promoted Gattling never resumed fire after Stop/Guard and load"
            );
        }
    }

    /// AY's elite initial fire rates are authored in EliteWeapon1, not an
    /// NPatch stage jump. Keep the native6F3436 stage-pair selection intact.
    #[cfg(feature = "npatch")]
    #[test]
    #[ignore = "requires VERA20K_AYMOD_DIR; real AY elite Gattling first-shot rates"]
    fn aymod_elite_gattling_first_shots_use_authored_faster_weapons() {
        use crate::sim::command::{Command, CommandEnvelope};
        for (actor_type, expected, reference) in [
            ("YTNK", "AGGattlingE", "AGGattling2"),
            ("YAGGUN", "AGGattCannE", "AGGattCann3"),
        ] {
            let (mut scenario, actor, target) = aymod_nearby_pair(actor_type, "APOC", false);
            {
                let runtime = &mut scenario.runtime;
                let (sim, rules) = (&mut runtime.simulation, &runtime.resources.rules);
                let owner = sim.entities().get(actor).unwrap().owner();
                // The actual powered cannon needs a real plant and House power pass.
                let sites = cells_nearest(aymod_anchor(sim, owner));
                spawn_on_free_cells(sim, rules, owner, sites, &["GAPOWR"]);
                let kind = rules.object(actor_type).unwrap();
                let first = rules.weapon(kind.weapon_at(0).unwrap()).unwrap();
                let elite = rules.weapon(expected).unwrap();
                assert_eq!(elite.rof, rules.weapon(reference).unwrap().rof);
                assert!(
                    elite.rof < first.rof,
                    "authored elite rate exceeds the base first slot"
                );
                let entity = sim.substrate.entities.get_mut(actor).unwrap();
                assert_eq!(entity.gattling.stage(), 0);
                assert_eq!(entity.gattling.value(), 0);
                // Rank is the explicit fixture boundary; kill-to-promotion is separate.
                entity.veterancy.set_elite();
                sim.queue_command(CommandEnvelope::new(
                    owner,
                    sim.session.tick() + 1,
                    Command::Attack {
                        attacker_id: actor,
                        target_id: target,
                    },
                ));
            }
            let first = (0..240)
                .find_map(|_| {
                    tick_with_due_commands(&mut scenario)
                        .fire_events
                        .into_iter()
                        .find(|event| event.attacker_id == actor)
                })
                .expect("actual elite Gattling Attack must produce its first shot");
            assert_eq!(scenario.sim().interner.resolve(first.weapon_id), expected);
            assert_eq!(
                scenario
                    .sim()
                    .entities()
                    .get(actor)
                    .unwrap()
                    .gattling
                    .stage(),
                0,
                "faster first fire is an elite weapon definition, not stage promotion"
            );
        }
    }
    #[cfg(feature = "npatch")]
    #[test]
    #[ignore = "requires VERA20K_AYMOD_DIR; player infantry enters a same-House red bunker"]
    fn aymod_red_bunker_admits_infantry_and_keeps_the_garrison() {
        use crate::sim::command::{Command, CommandEnvelope};
        let (mut scene, bunker, infantry) = aymod_nearby_pair("NABNKR", "E1", true);
        let runtime = &mut scene.runtime;
        let (sim, rules) = (&mut runtime.simulation, &runtime.resources.rules);
        let object = rules.object("NABNKR").unwrap();
        assert!(object.can_be_occupied && sim.object_tech_level(object) >= 0);
        assert!(rules.general.condition_red > 0.1);
        sim.substrate
            .entities
            .get_mut(bunker)
            .unwrap()
            .health
            .set_for_test((object.strength / 10).max(1));
        assert!(crate::sim::passenger::can_enter_transport(
            sim, rules, infantry, bunker
        ));
        let owner = sim.entities().get(infantry).unwrap().owner();
        assert_eq!(sim.entities().get(bunker).unwrap().owner(), owner);
        sim.queue_command(CommandEnvelope::new(
            owner,
            sim.session.tick() + 1,
            Command::CaptureBuilding {
                engineer_id: infantry,
                target_building_id: bunker,
            },
        ));
        let entered = (0..360).any(|_| {
            tick_with_due_commands(&mut scene);
            scene
                .sim()
                .entities()
                .get(bunker)
                .unwrap()
                .passenger_role
                .occupants()
                .ids()
                .contains(&infantry)
        });
        assert!(
            entered,
            "actual Capture order must reach garrison admission"
        );
        for _ in 0..60 {
            tick_with_due_commands(&mut scene);
        }
        let sim = scene.sim();
        assert!(
            sim.entities()
                .get(bunker)
                .unwrap()
                .passenger_role
                .occupants()
                .ids()
                .contains(&infantry)
        );
        assert_eq!(
            sim.entities()
                .get(infantry)
                .unwrap()
                .passenger_role
                .inside_transport_id(),
            Some(bunker)
        );
        assert!(sim.entities().get(bunker).unwrap().health.current() > 0);
    }

    #[cfg(feature = "npatch")]
    fn aymod_nearby_pair(
        actor_type: &str,
        target_type: &str,
        friendly: bool,
    ) -> (HeadlessScenario, u64, u64) {
        let mut scenario = aymod_battle(0);
        let runtime = &mut scenario.runtime;
        let (sim, rules) = (&mut runtime.simulation, &runtime.resources.rules);
        let owner = house_of(sim, "Americans");
        let enemy = house_of(sim, "Russians");
        let sites = cells_nearest(aymod_anchor(sim, owner));
        let actor = spawn_on_free_cells(sim, rules, owner, sites, &[actor_type])[0];
        let at = sim.entities().get(actor).unwrap().position;
        let sites = cells_nearest((i32::from(at.rx()), i32::from(at.ry())))
            .into_iter()
            .filter(|&(x, y)| {
                let distance = (x - i32::from(at.rx()))
                    .abs()
                    .max((y - i32::from(at.ry())).abs());
                (1..=3).contains(&distance)
                    && u16::try_from(x)
                        .ok()
                        .zip(u16::try_from(y).ok())
                        .is_some_and(|(x, y)| sim.terrain_cell_level(x, y) == Some(at.z()))
            })
            .collect::<Vec<_>>();
        let target = spawn_on_free_cells(
            sim,
            rules,
            if friendly { owner } else { enemy },
            sites,
            &[target_type],
        )[0];
        (scenario, actor, target)
    }

    #[cfg(feature = "npatch")]
    fn aymod_kill_for_experience(scenario: &mut HeadlessScenario, killer: u64) {
        use crate::sim::combat::{EntityDamageEvent, ReceiverCallFlags};
        let runtime = &mut scenario.runtime;
        let (sim, resources) = (&mut runtime.simulation, &runtime.resources);
        let enemy = house_of(sim, "Russians");
        let sites = cells_nearest(aymod_anchor(sim, enemy));
        let victim = spawn_on_free_cells(sim, &resources.rules, enemy, sites, &["APOC"])[0];
        let owner = sim.entities().get(killer).unwrap().owner();
        let kind = sim
            .object_type(
                sim.entities().get(killer).unwrap().type_ref(),
                &resources.rules,
            )
            .unwrap();
        let weapon = resources.rules.weapon(kind.primary().unwrap()).unwrap();
        let warhead = sim.interner.intern(weapon.warhead.as_deref().unwrap());
        let hit = EntityDamageEvent::direct_receiver(
            victim,
            sim.entities().get(victim).unwrap().health.current(),
            0,
            killer,
            Some(owner),
            warhead,
            ReceiverCallFlags {
                ignore_defenses: true,
                arg6: true,
            },
        );
        sim.commit_direct_damage_receiver(
            &resources.rules,
            Some(&resources.overlay_registry),
            hit,
            Default::default(),
        );
        assert!(
            sim.entities()
                .get(victim)
                .is_none_or(|entity| entity.health.current() <= 0 || entity.death.is_dying())
        );
    }

    #[cfg(feature = "npatch")]
    #[test]
    #[ignore = "requires VERA20K_AYMOD_DIR; real AY spy boarding and permanent ownership"]
    fn aymod_spy_attack_consumes_boarder_and_keeps_stolen_tank() {
        use crate::sim::command::{Command, CommandEnvelope};
        let (mut scenario, spy, tank) = aymod_nearby_pair("SPY", "MTNK", false);
        let sim = &mut scenario.runtime.simulation;
        let owner = sim.entities().get(spy).unwrap().owner();
        sim.queue_command(CommandEnvelope::new(
            owner,
            sim.session.tick() + 1,
            Command::Attack {
                attacker_id: spy,
                target_id: tank,
            },
        ));
        for _ in 0..400 {
            tick_with_due_commands(&mut scenario);
            if scenario
                .sim()
                .entities()
                .get(tank)
                .is_some_and(|entity| entity.owner() == owner)
            {
                break;
            }
        }
        let stolen = scenario
            .sim()
            .entities()
            .get(tank)
            .expect("the stolen tank survives");
        assert_eq!(
            stolen.owner(),
            owner,
            "the authored SpyJack attack must steal the tank"
        );
        assert!(stolen.mind_control.permanent());
        assert!(stolen.mind_control.controller().is_none());
        assert!(
            scenario.sim().entities().get(spy).is_none(),
            "boarding consumes the real spy, not an invisible limbo remainder"
        );
        for _ in 0..60 {
            tick_with_due_commands(&mut scenario);
        }
        assert_eq!(scenario.sim().entities().get(tank).unwrap().owner(), owner);
    }

    #[cfg(feature = "npatch")]
    #[test]
    #[ignore = "requires VERA20K_AYMOD_DIR; real AY elite Yuri permanent building control"]
    fn aymod_elite_yuri_keeps_captured_building_after_controller_removal() {
        use crate::sim::command::{Command, CommandEnvelope};
        let (mut scenario, yuri, building) = aymod_nearby_pair("YURIPR", "GAPOWR", false);
        for _ in 0..240 {
            tick_with_due_commands(&mut scenario);
            if scenario
                .sim()
                .entities()
                .get(building)
                .unwrap()
                .mission
                .effective()
                .known()
                != Some(crate::sim::mission::MissionType::Construction)
            {
                break;
            }
        }
        let sim = &mut scenario.runtime.simulation;
        let owner = sim.entities().get(yuri).unwrap().owner();
        sim.substrate
            .entities
            .get_mut(yuri)
            .unwrap()
            .veterancy
            .set_elite();
        sim.queue_command(CommandEnvelope::new(
            owner,
            sim.session.tick() + 1,
            Command::Attack {
                attacker_id: yuri,
                target_id: building,
            },
        ));
        for _ in 0..400 {
            tick_with_due_commands(&mut scenario);
            if scenario
                .sim()
                .entities()
                .get(building)
                .is_some_and(|entity| entity.owner() == owner)
            {
                break;
            }
        }
        let captured = scenario.sim().entities().get(building).unwrap();
        assert_eq!(captured.owner(), owner);
        assert!(captured.mind_control.permanent());
        assert!(captured.mind_control.controller().is_none());
        aymod_remove_shooter(&mut scenario, yuri);
        for _ in 0..60 {
            tick_with_due_commands(&mut scenario);
        }
        assert_eq!(
            scenario.sim().entities().get(building).unwrap().owner(),
            owner
        );
    }

    #[cfg(feature = "npatch")]
    #[test]
    #[ignore = "requires VERA20K_AYMOD_DIR; real AY controlled kill experience routing"]
    fn aymod_yuri_receives_controlled_kill_experience_and_elite_falls_back() {
        use crate::sim::command::{Command, CommandEnvelope};
        let (mut scenario, yuri, tank) = aymod_nearby_pair("YURI", "MTNK", false);
        let sim = &mut scenario.runtime.simulation;
        let owner = sim.entities().get(yuri).unwrap().owner();
        sim.queue_command(CommandEnvelope::new(
            owner,
            sim.session.tick() + 1,
            Command::Attack {
                attacker_id: yuri,
                target_id: tank,
            },
        ));
        for _ in 0..400 {
            tick_with_due_commands(&mut scenario);
            if scenario
                .sim()
                .entities()
                .get(tank)
                .is_some_and(|entity| entity.owner() == owner)
            {
                break;
            }
        }
        assert_eq!(
            scenario
                .sim()
                .entities()
                .get(tank)
                .unwrap()
                .mind_control
                .controller(),
            Some(yuri)
        );
        aymod_kill_for_experience(&mut scenario, tank);
        assert!(
            scenario
                .sim()
                .entities()
                .get(yuri)
                .unwrap()
                .veterancy
                .raw()
                .bits()
                > 0
        );
        assert_eq!(
            scenario
                .sim()
                .entities()
                .get(tank)
                .unwrap()
                .veterancy
                .raw()
                .bits(),
            0
        );
        scenario
            .runtime
            .simulation
            .substrate
            .entities
            .get_mut(yuri)
            .unwrap()
            .veterancy
            .set_elite();
        aymod_kill_for_experience(&mut scenario, tank);
        assert!(
            scenario
                .sim()
                .entities()
                .get(tank)
                .unwrap()
                .veterancy
                .raw()
                .bits()
                > 0
        );
    }

    #[cfg(feature = "npatch")]
    #[test]
    #[ignore = "requires VERA20K_AYMOD_DIR; real AY boarding and TrainPassengers kill credit"]
    fn aymod_fortress_passenger_trains_before_carrier_then_elite_falls_back() {
        use crate::sim::command::{Command, CommandEnvelope};
        let (mut scenario, carrier, passenger) = aymod_nearby_pair("BFRT", "E1", true);
        let sim = &mut scenario.runtime.simulation;
        let owner = sim.entities().get(passenger).unwrap().owner();
        sim.queue_command(CommandEnvelope::new(
            owner,
            sim.session.tick() + 1,
            Command::clicked_mission(
                passenger,
                crate::sim::mission::MissionType::Enter,
                crate::sim::combat::TargetKind::Entity(carrier),
            ),
        ));
        for _ in 0..400 {
            tick_with_due_commands(&mut scenario);
            if scenario
                .sim()
                .entities()
                .get(passenger)
                .is_some_and(|entity| entity.passenger_role.inside_transport_id() == Some(carrier))
            {
                break;
            }
        }
        assert_eq!(
            scenario
                .sim()
                .entities()
                .get(passenger)
                .unwrap()
                .passenger_role
                .open_transport_id(),
            Some(carrier)
        );
        aymod_kill_for_experience(&mut scenario, passenger);
        assert!(
            scenario
                .sim()
                .entities()
                .get(passenger)
                .unwrap()
                .veterancy
                .raw()
                .bits()
                > 0
        );
        assert_eq!(
            scenario
                .sim()
                .entities()
                .get(carrier)
                .unwrap()
                .veterancy
                .raw()
                .bits(),
            0
        );
        scenario
            .runtime
            .simulation
            .substrate
            .entities
            .get_mut(passenger)
            .unwrap()
            .veterancy
            .set_elite();
        aymod_kill_for_experience(&mut scenario, passenger);
        assert!(
            scenario
                .sim()
                .entities()
                .get(carrier)
                .unwrap()
                .veterancy
                .raw()
                .bits()
                > 0
        );
    }

    #[cfg(feature = "npatch")]
    #[test]
    #[ignore = "requires VERA20K_AYMOD_DIR; carrier child kill, saved regeneration and relaunch"]
    fn aymod_carrier_child_keeps_experience_and_regenerates_after_real_load() {
        use crate::sim::combat::{EntityDamageEvent, RAD_NO_ATTACKER, ReceiverCallFlags};
        use crate::sim::command::{Command, CommandEnvelope};
        use crate::sim::mission::{MissionId, MissionType};
        use crate::sim::world::bridge_test_evidence::{restored_retail, save_scene};
        let root = std::path::PathBuf::from(std::env::var_os("VERA20K_AYMOD_DIR").unwrap());
        let map = "\u{91cd}\u{8fd4}\u{8fea}\u{62dc}.yrm";
        let mut launch = battle_session(
            map,
            &[(LaunchCountry::from_country_index(1), AiDifficulty::Hard)],
        );
        launch.local_slot_mut().country = LaunchCountry::from_country_index(0);
        launch.options.unit_count = 0;
        let mut scene = load_session(&root, map, 0x00C0_FFEE, launch).unwrap();
        let pristine = scene.sim().resolved_terrain().unwrap().clone();
        let (carrier, victim, target_cell, children) = {
            let runtime = &mut scene.runtime;
            let (sim, rules) = (&mut runtime.simulation, &runtime.resources.rules);
            let terrain = sim.resolved_terrain().unwrap();
            let free = |cell: &crate::map::resolved_terrain::ResolvedTerrainCell| {
                !cell.outside_playfield
                    && !cell.bridge_facts.has_structural_bridge()
                    && sim.occupancy().get(cell.rx, cell.ry).is_none()
            };
            let (water, land) = terrain
                .cells()
                .iter()
                .find_map(|cell| {
                    if !free(cell)
                        || cell.land_type != crate::rules::terrain_rules::LandType::Water.as_index()
                    {
                        return None;
                    }
                    (-3i16..=3)
                        .flat_map(|dx| (-3i16..=3).map(move |dy| (dx, dy)))
                        .find_map(|(dx, dy)| {
                            let land = terrain.cell(
                                cell.rx.checked_add_signed(dx)?,
                                cell.ry.checked_add_signed(dy)?,
                            )?;
                            (free(land)
                                && land.level == cell.level
                                && land.land_type
                                    == crate::rules::terrain_rules::LandType::Clear.as_index())
                            .then_some(((cell.rx, cell.ry), (land.rx, land.ry)))
                        })
                })
                .expect("a clear land target near a legal water carrier site");
            let owner = house_of(sim, "Americans");
            let enemy = house_of(sim, "Russians");
            let owner_name = sim.interner.resolve(owner).to_owned();
            let enemy_name = sim.interner.resolve(enemy).to_owned();
            let carrier = sim
                .spawn_object("CARRIER", &owner_name, water.0, water.1, 0, rules)
                .unwrap();
            let victim = sim
                .spawn_object("APOC", &enemy_name, land.0, land.1, 0, rules)
                .unwrap();
            sim.substrate
                .entities
                .get_mut(victim)
                .unwrap()
                .health
                .set(1);
            sim.mission_queue_exact(
                victim,
                MissionId::from_known(MissionType::Sleep),
                1,
                sim.session.binary_frame(),
                &crate::sim::mission::authority::LiveReadyInputProvider { rules },
            )
            .unwrap();
            assert!(rules.object("HORNET").unwrap().trainable);
            let parent = sim.entities().get(carrier).unwrap();
            assert_eq!(parent.veterancy.raw().bits(), 0);
            let children = parent
                .spawn_manager
                .as_ref()
                .unwrap()
                .slots()
                .iter()
                .map(|slot| slot.spawn().unwrap())
                .collect::<Vec<_>>();
            assert_eq!(
                children.len(),
                rules.object("CARRIER").unwrap().spawns_number as usize
            );
            (carrier, victim, land, children)
        };
        let attack = |scene: &mut HeadlessScenario, target_id| {
            let sim = &mut scene.runtime.simulation;
            let owner = sim.entities().get(carrier).unwrap().owner();
            sim.queue_command(CommandEnvelope::new(
                owner,
                sim.session.tick() + 1,
                Command::Attack {
                    attacker_id: carrier,
                    target_id,
                },
            ));
        };
        attack(&mut scene, victim);
        let mut child_fired = false;
        assert!(
            (0..1200).any(|_| {
                let output = tick_with_due_commands(&mut scene);
                child_fired |= output.fire_events.iter().any(|event| {
                    children.contains(&event.attacker_id)
                        && event.target == crate::sim::combat::TargetKind::Entity(victim)
                });
                scene
                    .sim()
                    .entities()
                    .get(carrier)
                    .unwrap()
                    .veterancy
                    .raw()
                    .bits()
                    > 0
            }),
            "normal Hornet weapons must kill and award the carrier"
        );
        assert!(child_fired);
        let parent_xp = scene.sim().entities().get(carrier).unwrap().veterancy.raw();
        assert!(
            children.iter().any(|child| scene
                .sim()
                .entities()
                .get(*child)
                .is_some_and(|child| child.veterancy.raw().bits() > parent_xp.bits())),
            "a trainable Hornet must keep its own larger award, not be overwritten by carrier XP"
        );
        let slot = 0;
        let lost = children[slot];
        let regen_rate = {
            let runtime = &mut scene.runtime;
            let (sim, rules) = (&mut runtime.simulation, &runtime.resources.rules);
            let owner = sim.entities().get(carrier).unwrap().owner();
            sim.queue_command(CommandEnvelope::new(
                owner,
                sim.session.tick() + 1,
                Command::Stop { entity_id: carrier },
            ));
            // Explicit fatal damage at the production receiver boundary. A
            // healthy child's UnInit is not a death: PointerExpired6B7C60
            // deliberately preserves a living, untracked reusable child.
            let hp = sim.entities().get(lost).unwrap().health.current();
            let warhead = sim.interner.intern(
                rules
                    .weapon("HornetBomb")
                    .unwrap()
                    .warhead
                    .as_deref()
                    .unwrap(),
            );
            let hit = EntityDamageEvent::direct_receiver(
                lost,
                hp,
                0,
                RAD_NO_ATTACKER,
                None,
                warhead,
                ReceiverCallFlags {
                    ignore_defenses: true,
                    arg6: true,
                },
            );
            sim.commit_direct_damage_receiver(
                rules,
                Some(&runtime.resources.overlay_registry),
                hit,
                Default::default(),
            );
            let manager = sim
                .entities()
                .get(carrier)
                .unwrap()
                .spawn_manager
                .as_ref()
                .unwrap();
            assert_eq!(
                manager.regen_rate(),
                rules.object("CARRIER").unwrap().spawn_regen_rate
            );
            manager.regen_rate()
        };
        assert!(
            (0..300).any(|_| {
                tick_with_due_commands(&mut scene);
                scene
                    .sim()
                    .entities()
                    .get(carrier)
                    .unwrap()
                    .spawn_manager
                    .as_ref()
                    .unwrap()
                    .slots()[slot]
                    .spawn()
                    .is_none()
            }),
            "the child's real death must release its manager slot"
        );
        for _ in 0..20 {
            tick_with_due_commands(&mut scene);
        }
        let pending = scene
            .sim()
            .entities()
            .get(carrier)
            .unwrap()
            .spawn_manager
            .clone()
            .unwrap();
        assert!(pending.slots()[slot].spawn().is_none());
        assert!(
            pending.slots()[slot]
                .timer()
                .remaining(scene.sim().session.binary_frame() as i32)
                > 0
        );
        let bytes = save_scene(&scene, "AY carrier pending Hornet regeneration");
        scene.runtime.simulation = restored_retail(&scene, &pristine, &bytes);
        assert_eq!(
            scene
                .sim()
                .entities()
                .get(carrier)
                .unwrap()
                .spawn_manager
                .as_ref(),
            Some(&pending)
        );
        let replacement = (0..regen_rate + 60)
            .find_map(|_| {
                tick_with_due_commands(&mut scene);
                scene
                    .sim()
                    .entities()
                    .get(carrier)?
                    .spawn_manager
                    .as_ref()?
                    .slots()[slot]
                    .spawn()
            })
            .expect(
                "the actual configured regeneration timer must produce a replacement after load",
            );
        assert_ne!(replacement, lost);
        let child = scene.sim().entities().get(replacement).unwrap();
        assert_eq!(child.spawn_owner_id.owner(), Some(carrier));
        assert_eq!(child.veterancy.raw(), parent_xp);
        let next = {
            let runtime = &mut scene.runtime;
            let (sim, rules) = (&mut runtime.simulation, &runtime.resources.rules);
            let enemy = house_of(sim, "Russians");
            let enemy_name = sim.interner.resolve(enemy).to_owned();
            let target = sim
                .spawn_object("APOC", &enemy_name, target_cell.0, target_cell.1, 0, rules)
                .unwrap();
            sim.mission_queue_exact(
                target,
                MissionId::from_known(MissionType::Sleep),
                1,
                sim.session.binary_frame(),
                &crate::sim::mission::authority::LiveReadyInputProvider { rules },
            )
            .unwrap();
            target
        };
        attack(&mut scene, next);
        assert!(
            (0..900).any(|_| {
                let output = tick_with_due_commands(&mut scene);
                output.fire_events.iter().any(|event| {
                    event.attacker_id == replacement
                        && event.target == crate::sim::combat::TargetKind::Entity(next)
                })
            }),
            "the regenerated Hornet must relaunch and fire through the ordinary carrier Attack"
        );
    }

    /// Whether `entity` aims at an object of another house.
    fn aims_at_another_house(
        sim: &Simulation,
        entity: &crate::sim::game_entity::GameEntity,
    ) -> bool {
        match entity.attack_target.target() {
            Some(crate::sim::combat::TargetKind::Entity(id)) => sim
                .entities()
                .get(id)
                .is_some_and(|target| target.owner() != entity.owner()),
            _ => false,
        }
    }

    /// A retail skirmish: the local house idles while three computer houses
    /// build their bases, create teams from their AI triggers and build what
    /// the teams need. Every 1500 frames it prints each computer house's
    /// teams (with the script action each is on), choices and Foot objects;
    /// each house must have created a team.
    #[test]
    #[ignore = "requires RA2_DIR with installed retail RA2/YR assets"]
    fn retail_skirmish_computer_houses_build_their_teams() {
        use crate::map::entities::EntityCategory;

        let (mut scenario, computers) = retail_skirmish();
        let mut created = std::collections::BTreeMap::new();
        for frame in 1..=6_000u32 {
            scenario.tick();
            let sim = scenario.sim();
            created.extend(
                sim.team_script_vm
                    .teams_in_order()
                    .map(|team| (team.id(), team.owner())),
            );
            if frame % 1500 != 0 {
                continue;
            }
            for &owner in &computers {
                let house = &sim.houses[&owner];
                let (teams, members) = sim
                    .team_script_vm
                    .teams_in_order()
                    .filter(|team| team.owner() == owner)
                    .fold((0, 0), |(teams, members), team| {
                        (teams + 1, members + team.member_count())
                    });
                let foot = sim
                    .entities()
                    .values()
                    .filter(|entity| {
                        entity.owner() == owner && entity.category() != EntityCategory::Structure
                    })
                    .count();
                let buildings = sim
                    .entities()
                    .values()
                    .filter(|entity| {
                        entity.owner() == owner && entity.category() == EntityCategory::Structure
                    })
                    .count();
                eprintln!(
                    "frame {frame} {}: {buildings} buildings, {foot} foot, {teams} live teams \
                     ({members} members), choices {:?}",
                    sim.interner.resolve(owner),
                    house.ai_unit_choices(),
                );
            }
            for team in sim.team_script_vm.teams_in_order() {
                eprintln!(
                    "  team {} of {}: action {} at line {}, formed {}, {} members",
                    team.id(),
                    sim.interner.resolve(team.owner()),
                    team.current_action(&sim.team_script_vm),
                    team.cursor(),
                    team.formed(),
                    team.member_count(),
                );
            }
            eprintln!("frame {frame}: {} teams created so far", created.len());
        }
        for owner in computers {
            assert!(
                created.values().any(|&creator| creator == owner),
                "{} created no team",
                scenario.sim().interner.resolve(owner)
            );
        }
    }

    /// Spawns `types` for the first house of `country`, in order, on free
    /// infantry cells within 15 cells of its base centre.
    fn spawn_beside_base(
        sim: &mut Simulation,
        rules: &crate::rules::ruleset::RuleSet,
        country: &str,
        types: &[&str],
    ) -> (crate::sim::intern::InternedId, Vec<u64>) {
        let house = house_of(sim, country);
        let (bx, by) = sim.houses[&house]
            .base_center()
            .expect("the base has a centre");
        let cells = (0..900).map(|n| (i32::from(bx) + n % 30 - 15, i32::from(by) + n / 30 - 15));
        let spawned = spawn_on_free_cells(sim, rules, house, cells, types);
        (house, spawned)
    }




    /// Two retail Yuri TeamTypes whose members are spawned beside the Yuri
    /// house's base once the bases stand: "Yuri Infantry - M1" (`05FFB46C-G`,
    /// script `0,4 / 49 / 0,1`) and "Yuri Engineers" (`08B95EFC-G`,
    /// `54 / 0,6 / 49 / 0,9 / 0,2`). Action 0's scan finds the first team
    /// another house's infantry, which its members attack. For the second it
    /// prints whether an engineer aimed at another house's building: in the
    /// first run that team disbanded before frame 1000, not yet explained.
    #[test]
    #[ignore = "requires RA2_DIR with installed retail RA2/YR assets"]
    fn retail_yuri_teams_attack() {
        use crate::map::entities::EntityCategory;

        let (mut scenario, _) = retail_skirmish();
        for _ in 0..1500 {
            scenario.tick();
        }
        let runtime = &mut scenario.runtime;
        let (sim, resources) = (&mut runtime.simulation, &runtime.resources);
        let (yuri, spawned) = spawn_beside_base(
            sim,
            &resources.rules,
            "YuriCountry",
            &[
                "INIT",
                "INIT",
                "INIT",
                "INIT",
                "INIT",
                "YENGINEER",
                "YENGINEER",
                "YENGINEER",
            ],
        );
        let factory_scan = crate::sim::world::direct_greatest_threat(
            sim,
            &resources.rules,
            None,
            spawned[5],
            crate::sim::combat::ScanMission::Quarry {
                mask: 0x1000,
                only_target_house_enemy: false,
            },
        );
        eprintln!(
            "a Yuri engineer's quarry-6 scan: {:?}",
            factory_scan.map(|id| sim
                .interner
                .resolve(sim.entities().get(id).unwrap().type_ref()))
        );
        let teams: Vec<u64> = ["05FFB46C-G", "08B95EFC-G"]
            .into_iter()
            .map(|team_type| {
                let id = sim
                    .interner
                    .get(team_type)
                    .expect("retail TeamType interned");
                sim.create_team(id, Some(yuri), &resources.rules)
                    .expect("under the TeamType's Max=")
            })
            .collect();
        let initiate = sim.interner.get("INIT").expect("INIT interned");
        let engineer = sim.interner.get("YENGINEER").expect("YENGINEER interned");
        let aims_at_a_building = |sim: &Simulation, id: u64| {
            let Some(entity) = sim.entities().get(id) else {
                return false;
            };
            aims_at_another_house(sim, entity)
                && matches!(
                    entity.attack_target.target(),
                    Some(crate::sim::combat::TargetKind::Entity(target))
                        if sim.entities().get(target).is_some_and(|target| {
                            target.category() == EntityCategory::Structure
                        })
                )
        };
        let (mut infantry_attacked, mut engineer_attacked) = (false, false);
        for frame in 1..=3_000u32 {
            scenario.tick();
            let sim = scenario.sim();
            infantry_attacked |= spawned.iter().any(|&id| {
                sim.entities().get(id).is_some_and(|entity| {
                    entity.type_ref() == initiate && aims_at_another_house(sim, entity)
                })
            });
            engineer_attacked |= spawned.iter().any(|&id| {
                sim.entities()
                    .get(id)
                    .is_some_and(|entity| entity.type_ref() == engineer)
                    && aims_at_a_building(sim, id)
            });
            let report = if frame <= 1000 {
                frame % 50 == 0
            } else {
                frame % 500 == 0
            };
            if report {
                for &team_id in &teams {
                    let Some(team) = sim.team_script_vm.team(team_id) else {
                        eprintln!("frame {frame}: team {team_id} is gone");
                        continue;
                    };
                    let members: Vec<String> = team
                        .members()
                        .filter_map(|id| sim.entities().get(id))
                        .map(|entity| {
                            format!(
                                "{}@({},{}) m{} aims away {}",
                                sim.interner.resolve(entity.type_ref()),
                                entity.position.rx(),
                                entity.position.ry(),
                                entity.mission.effective().raw(),
                                aims_at_another_house(sim, entity),
                            )
                        })
                        .collect();
                    eprintln!(
                        "frame {frame}: team {team_id} action {} at line {}, formed {}: {members:?}",
                        team.current_action(&sim.team_script_vm),
                        team.cursor(),
                        team.formed(),
                    );
                }
            }
            if infantry_attacked && engineer_attacked {
                break;
            }
        }
        eprintln!("an engineer aimed at another house's building: {engineer_attacked}");
        assert!(
            infantry_attacked,
            "no Initiate aimed at another house's object"
        );
    }

    /// Two retail transport TeamTypes, each in a duel against an idle local
    /// house that starts with only its MCV (`UnitCount=0`), deployed on the
    /// first frame, so that undefended construction yard is the team's
    /// least-threat target (mode 0):
    /// - "Soviet Engineer vs Ally" (`0C143EFC-G`: an HTK and five SENGINEER,
    ///   `58,131086 / 14 / 43 / 47,2 / 8,0 / 49 / 46,131074`), Russia against
    ///   America;
    /// - "Allied Tanya vs Soviet Con" (`0AD4D7BC-G`: a Night Hawk, Tanya,
    ///   three GIs and a Guardian GI, `TransportsReturnOnUnload=yes`,
    ///   `58,7 / 14 / 43 / 47,25 / 8,2 / 46,131097 / 49`), France against
    ///   Russia.
    ///
    /// Each team's members are spawned beside the computer base once it
    /// stands. The infantry board the transport, leaving the team (actions
    /// 14 and 43); it carries them to the yard (47) and unloads them, and
    /// they rejoin the team (8). Five riders fill the Night Hawk, so action
    /// 14 finishes on its no-transport arm without archiving a cell (only a
    /// transport with a seat left reaches `Set_ArchiveTarget @ 0x0070C610`),
    /// and the unload sends it out of the team with no destination.
    #[test]
    #[ignore = "requires RA2_DIR with installed retail RA2/YR assets"]
    fn retail_transport_teams_load_and_unload() {
        use crate::sim::command::{Command, CommandEnvelope};
        use crate::skirmish_launch::{AiDifficulty, LaunchCountry};

        for (local, opponent, country, team_type, types) in [
            (
                LaunchCountry::America,
                LaunchCountry::Russia,
                "Russians",
                "0C143EFC-G",
                [
                    "HTK",
                    "SENGINEER",
                    "SENGINEER",
                    "SENGINEER",
                    "SENGINEER",
                    "SENGINEER",
                ],
            ),
            (
                LaunchCountry::Russia,
                LaunchCountry::France,
                "French",
                "0AD4D7BC-G",
                ["SHAD", "TANY", "E1", "E1", "E1", "GGI"],
            ),
        ] {
            let (mut scenario, _) = retail_battle(
                |session| {
                    session.local_slot_mut().country = local;
                    session.options.unit_count = 0;
                },
                &[(opponent, AiDifficulty::Hard)],
            );
            let sim = scenario.sim();
            let human = sim
                .houses
                .iter()
                .find(|(_, house)| house.is_controlled_by_human(true))
                .map(|(&id, _)| id)
                .expect("the local house");
            let mcv = sim
                .substrate
                .entities
                .iter_sorted()
                .find(|(_, entity)| entity.owner() == human)
                .map(|(id, _)| id)
                .expect("the local MCV");
            let deploy = CommandEnvelope::new(
                human,
                sim.session.tick() + 1,
                Command::DeployMcv { entity_id: mcv },
            );
            scenario
                .runtime
                .advance_frame(
                    &[deploy],
                    SIM_TICK_MS,
                    crate::sim::world::TickLane::Ordinary,
                    crate::sim::world::FrameEffects::default(),
                )
                .expect("simulation frame failed; prior world mutations remain");
            for _ in 0..1500 {
                scenario.tick();
            }
            let runtime = &mut scenario.runtime;
            let (sim, resources) = (&mut runtime.simulation, &runtime.resources);
            let yards: Vec<String> = sim
                .substrate
                .entities
                .iter_sorted()
                .filter(|(_, entity)| entity.owner() == human)
                .map(|(_, entity)| sim.interner.resolve(entity.type_ref()).to_string())
                .collect();
            eprintln!("{team_type}: the local house owns {yards:?}");
            let (house, spawned) = spawn_beside_base(sim, &resources.rules, country, &types);
            let id = sim
                .interner
                .get(team_type)
                .expect("retail TeamType interned");
            // The computer house may already have made the team; then it
            // recruits the new members.
            let team_id = sim
                .create_team(id, Some(house), &resources.rules)
                .or_else(|| {
                    sim.team_script_vm
                        .teams_in_order()
                        .find(|team| team.team_type_id() == Some(id) && team.owner() == house)
                        .map(|team| team.id())
                })
                .expect("a team of the TeamType");
            let (transport, infantry) = (spawned[0], &spawned[1..]);
            // The first frame all the infantry were aboard the transport and
            // its archive then, the first frame after that they were all
            // members again, and the first frame the transport was not, with
            // its NavCom then.
            let (mut loaded, mut archive, mut unloaded) = (None, None, None);
            let mut transport_left = None;
            let mut present = (true, vec![true; spawned.len()]);
            for frame in 1..=6_000u32 {
                scenario.tick();
                let sim = scenario.sim();
                let team = sim.team_script_vm.team(team_id);
                let aboard = infantry.iter().all(|&id| {
                    sim.entities().get(id).is_some_and(|entity| {
                        entity.passenger_role.inside_transport_id() == Some(transport)
                    })
                });
                if loaded.is_none() && aboard {
                    loaded = Some(frame);
                }
                if loaded.is_some() && unloaded.is_none() {
                    archive = sim
                        .entities()
                        .get(transport)
                        .and_then(|entity| entity.archive_target());
                }
                let rejoined = team.is_some_and(|team| {
                    infantry
                        .iter()
                        .all(|&id| team.members().any(|member| member == id))
                });
                if loaded.is_some() && unloaded.is_none() && rejoined {
                    unloaded = Some(frame);
                }
                let now = (
                    team.is_some(),
                    spawned
                        .iter()
                        .map(|&id| sim.entities().get(id).is_some())
                        .collect::<Vec<_>>(),
                );
                let changed = now != present;
                present = now;
                if frame % 250 == 0 || unloaded == Some(frame) || changed {
                    let members: Vec<String> = spawned
                        .iter()
                        .map(|&id| {
                            let Some(entity) = sim.entities().get(id) else {
                                return format!("{id}:gone");
                            };
                            format!(
                                "{}@({},{}) hp{} m{} {} nav {:?}{}",
                                sim.interner.resolve(entity.type_ref()),
                                entity.position.rx(),
                                entity.position.ry(),
                                entity.health.current(),
                                entity.mission.effective().raw(),
                                match sim.team_script_vm.team_for_member(id) {
                                    Some((member_team, _)) if member_team == team_id => "in",
                                    Some(_) => "in another team",
                                    None => "out",
                                },
                                entity.navigation.nav_com(),
                                if entity.passenger_role.is_inside_transport() {
                                    " aboard"
                                } else {
                                    ""
                                },
                            )
                        })
                        .collect();
                    match team {
                        Some(team) => eprintln!(
                            "frame {frame}: {team_type} action {} at line {}: {members:?}",
                            team.current_action(&sim.team_script_vm),
                            team.cursor(),
                        ),
                        None => eprintln!("frame {frame}: {team_type} is gone: {members:?}"),
                    }
                }
                if let Some(done) = unloaded {
                    if transport_left.is_none()
                        && !team
                            .is_some_and(|team| team.members().any(|member| member == transport))
                    {
                        transport_left = Some((
                            frame,
                            sim.entities()
                                .get(transport)
                                .and_then(|entity| entity.navigation.nav_com()),
                        ));
                    }
                    if frame >= done + 60 {
                        break;
                    }
                }
            }
            eprintln!(
                "{team_type}: aboard at {loaded:?}, transport archive {archive:?}, \
                 rejoined after unloading at {unloaded:?}, transport left with its \
                 NavCom at {transport_left:?}"
            );
            assert!(loaded.is_some(), "{team_type}: the infantry never boarded");
            assert!(
                unloaded.is_some(),
                "{team_type}: the infantry never rejoined"
            );
            if team_type == "0AD4D7BC-G" {
                assert_eq!(archive, None, "the full Night Hawk archived a cell");
                assert_eq!(
                    transport_left.map(|(_, nav_com)| nav_com),
                    Some(None),
                    "the Night Hawk left its team with no destination"
                );
            } else {
                assert_eq!(transport_left, None, "the HTK left its team");
            }
        }
    }

    /// Computer engineers capture once hit on Guard, in two XMP03T4 duels
    /// of a Hard Russia against an idle America whose MCV deploys on frame 1:
    /// - a lone Soviet Engineer by the yard, inside an American GI's range:
    ///   shot on Guard, it queues Hunt (`0x00518BF2`), and Mission_Hunt's
    ///   engineer arm (`0x004D53BA`) sends it away from the GI to capture
    ///   what its scan finds;
    /// - "Soviet Engineer vs Ally" (`0C143EFC-G`, an HTK and five Soviet
    ///   Engineers) against three Rhino Tanks by the yard: unloaded, the
    ///   engineers stand on Guard until action 46 gives them Attack, and one
    ///   hit on Guard hunts and captures.
    ///
    /// Each prints when an engineer first hunted and first took Capture, and
    /// must take the American Construction Yard.
    #[test]
    #[ignore = "requires RA2_DIR with installed retail RA2/YR assets"]
    fn retail_computer_engineers_capture_once_hit() {
        use crate::sim::command::{Command, CommandEnvelope};
        use crate::sim::mission::MissionType;
        use crate::skirmish_launch::{AiDifficulty, LaunchCountry};

        for lone in [true, false] {
            let (mut scenario, _) = retail_battle(
                |session| {
                    session.local_slot_mut().country = LaunchCountry::America;
                    session.options.unit_count = 0;
                },
                &[(LaunchCountry::Russia, AiDifficulty::Hard)],
            );
            let sim = scenario.sim();
            let human = sim
                .houses
                .iter()
                .find(|(_, house)| house.is_controlled_by_human(true))
                .map(|(&id, _)| id)
                .expect("the local house");
            let mcv = sim
                .substrate
                .entities
                .iter_sorted()
                .find(|(_, entity)| entity.owner() == human)
                .map(|(id, _)| id)
                .expect("the local MCV");
            let deploy = CommandEnvelope::new(
                human,
                sim.session.tick() + 1,
                Command::DeployMcv { entity_id: mcv },
            );
            scenario
                .runtime
                .advance_frame(
                    &[deploy],
                    SIM_TICK_MS,
                    crate::sim::world::TickLane::Ordinary,
                    crate::sim::world::FrameEffects::default(),
                )
                .expect("simulation frame failed; prior world mutations remain");
            for _ in 0..1500 {
                scenario.tick();
            }
            let runtime = &mut scenario.runtime;
            let (sim, resources) = (&mut runtime.simulation, &runtime.resources);
            let yard = sim
                .substrate
                .entities
                .iter_sorted()
                .find(|(_, entity)| entity.owner() == human)
                .map(|(id, _)| id)
                .expect("the local Construction Yard");
            let at = |sim: &Simulation, id| {
                let entity = sim.entities().get(id).expect("spawned");
                (
                    i32::from(entity.position.rx()),
                    i32::from(entity.position.ry()),
                )
            };
            let (yx, yy) = at(sim, yard);
            let russians = house_of(sim, "Russians");
            let engineers = if lone {
                let engineer = spawn_on_free_cells(
                    sim,
                    &resources.rules,
                    russians,
                    cells_nearest((yx - 2, yy - 2)),
                    &["SENGINEER"],
                );
                let (ex, ey) = at(sim, engineer[0]);
                spawn_on_free_cells(
                    sim,
                    &resources.rules,
                    human,
                    cells_nearest((ex - 3, ey - 2)),
                    &["E1"],
                );
                engineer
            } else {
                let (house, spawned) = spawn_beside_base(
                    sim,
                    &resources.rules,
                    "Russians",
                    &[
                        "HTK",
                        "SENGINEER",
                        "SENGINEER",
                        "SENGINEER",
                        "SENGINEER",
                        "SENGINEER",
                    ],
                );
                let id = sim
                    .interner
                    .get("0C143EFC-G")
                    .expect("retail TeamType interned");
                // The computer house may already have made the team; then it
                // recruits the new members.
                sim.create_team(id, Some(house), &resources.rules)
                    .or_else(|| {
                        sim.team_script_vm
                            .teams_in_order()
                            .find(|team| team.team_type_id() == Some(id) && team.owner() == house)
                            .map(|team| team.id())
                    })
                    .expect("a team of the TeamType");
                spawn_on_free_cells(
                    sim,
                    &resources.rules,
                    human,
                    cells_nearest((yx + 3, yy + 3)),
                    &["MTNK", "MTNK", "MTNK"],
                );
                spawned[1..].to_vec()
            };
            let label = if lone { "lone engineer" } else { "0C143EFC-G" };
            let (mut hunted, mut capturing, mut captured) = (None, None, None);
            for frame in 1..=4_000u32 {
                scenario.tick();
                let sim = scenario.sim();
                for &id in &engineers {
                    let Some(entity) = sim.entities().get(id) else {
                        continue;
                    };
                    let current = entity.mission.current().known();
                    if hunted.is_none()
                        && (current == Some(MissionType::Hunt)
                            || entity.mission.queued().known() == Some(MissionType::Hunt))
                    {
                        hunted = Some((frame, id));
                    }
                    if capturing.is_none() && current == Some(MissionType::Capture) {
                        capturing = Some((frame, id));
                    }
                }
                if sim
                    .entities()
                    .get(yard)
                    .is_some_and(|entity| entity.owner() == russians)
                {
                    captured = Some(frame);
                    break;
                }
            }
            eprintln!(
                "{label}: hunted at {hunted:?}, took Capture at {capturing:?}, \
                 captured the yard at {captured:?}"
            );
            assert!(hunted.is_some(), "{label}: no engineer hunted");
            assert!(capturing.is_some(), "{label}: no engineer took Capture");
            assert!(captured.is_some(), "{label}: the yard was not captured");
        }
    }

    /// "Yuri Armor - E1" (`05FC195C-G`: three Lashers and a Gattling Tank,
    /// the "Yuri Unit Hunter" script `54 / 61 / 0,5 / 49 / 0,4 / 0,1`) for a
    /// Normal Yuri against an idle America on XMP03T4. Once the base stands,
    /// a Yuri Tank Bunker is placed on the nearest open ground beside it and
    /// the team's members are spawned there. Action 61 sends the first
    /// member that the bunker admits into it, and the team goes on past it.
    /// Then the stock "Yuri Chaos - M1" (`07C0304C-G`, one Chaos Drone, the
    /// same script) reaches action 61 beside a second, free bunker: the
    /// drone (`Bunkerable=no`) stays in its team and the team goes on.
    #[test]
    #[ignore = "requires RA2_DIR with installed retail RA2/YR assets"]
    fn retail_yuri_armor_team_mans_a_tank_bunker() {
        use crate::sim::movement::locomotor::MovementLayer;
        use crate::skirmish_launch::{AiDifficulty, LaunchCountry};

        let (mut scenario, _) =
            retail_battle(|_| {}, &[(LaunchCountry::Yuri, AiDifficulty::Normal)]);
        for _ in 0..1500 {
            scenario.tick();
        }
        let runtime = &mut scenario.runtime;
        let (sim, resources) = (&mut runtime.simulation, &runtime.resources);
        let yuri = house_of(sim, "YuriCountry");
        let (bx, by) = sim.houses[&yuri]
            .base_center()
            .expect("the base has a centre");
        // A 2x2 foundation inside a free 4x4 square, so a tank can reach it.
        let open = |sim: &Simulation, rx: i32, ry: i32| {
            (-1..3).all(|dy| {
                (-1..3).all(|dx| {
                    let (Ok(x), Ok(y)) = (u16::try_from(rx + dx), u16::try_from(ry + dy)) else {
                        return false;
                    };
                    sim.path_grid()
                        .is_some_and(|grid| grid.is_walkable_for_infantry(x, y))
                        && sim
                            .substrate
                            .occupancy
                            .get(x, y)
                            .is_none_or(|cell| cell.is_empty_on(MovementLayer::Ground))
                })
            })
        };
        let (rx, ry) = cells_nearest((i32::from(bx), i32::from(by)))
            .into_iter()
            .find(|&(rx, ry)| open(&*sim, rx, ry))
            .and_then(|(rx, ry)| Some((u16::try_from(rx).ok()?, u16::try_from(ry).ok()?)))
            .expect("open ground beside the base");
        let z = sim.terrain_cell_level(rx, ry).unwrap_or(0);
        let owner = sim.interner.resolve(yuri).to_string();
        let bunker = sim
            .spawn_object_at_height("NATBNK", &owner, rx, ry, 0, z, &resources.rules)
            .expect("the bunker spawns");
        let (_, members) = spawn_beside_base(
            sim,
            &resources.rules,
            "YuriCountry",
            &["LTNK", "LTNK", "LTNK", "YTNK"],
        );
        let team_type = sim
            .interner
            .get("05FC195C-G")
            .expect("retail TeamType interned");
        // The computer house may already have made the team; then it
        // recruits the new members.
        let team = sim
            .create_team(team_type, Some(yuri), &resources.rules)
            .or_else(|| {
                sim.team_script_vm
                    .teams_in_order()
                    .find(|team| team.team_type_id() == Some(team_type) && team.owner() == yuri)
                    .map(|team| team.id())
            })
            .expect("a team of the TeamType");
        let (mut seeking, mut installed, mut passed) = (None, None, None);
        for frame in 1..=3_000u32 {
            scenario.tick();
            let sim = scenario.sim();
            if seeking.is_none() {
                seeking = members
                    .iter()
                    .find(|&&id| {
                        sim.entities()
                            .get(id)
                            .is_some_and(|entity| entity.radio_contacts.slot(0) == Some(bunker))
                    })
                    .map(|&id| (frame, id));
            }
            if installed.is_none() {
                installed = sim
                    .entities()
                    .get(bunker)
                    .and_then(|entity| entity.bunker.occupant())
                    .filter(|occupant| members.contains(occupant))
                    .map(|occupant| (frame, occupant));
            }
            let line = sim
                .team_script_vm
                .team(team)
                .map(|team| (team.cursor(), team.current_action(&sim.team_script_vm)));
            if passed.is_none() && line.is_none_or(|(cursor, _)| cursor >= 2) {
                passed = Some((frame, line));
            }
            if frame % 250 == 0 {
                eprintln!(
                    "frame {frame}: team line {line:?}, seeking {seeking:?}, installed {installed:?}"
                );
            }
            if installed.is_some() && passed.is_some() {
                break;
            }
        }
        eprintln!(
            "bunker at ({rx},{ry}): sought at {seeking:?}, manned at {installed:?}, \
             the team passed action 61 at {passed:?}"
        );
        assert!(installed.is_some(), "no team member manned the bunker");
        assert!(passed.is_some(), "the team stayed on action 61");

        let runtime = &mut scenario.runtime;
        let (sim, resources) = (&mut runtime.simulation, &runtime.resources);
        let (fx, fy) = cells_nearest((i32::from(bx), i32::from(by)))
            .into_iter()
            .find(|&(rx, ry)| open(&*sim, rx, ry))
            .and_then(|(rx, ry)| Some((u16::try_from(rx).ok()?, u16::try_from(ry).ok()?)))
            .expect("more open ground beside the base");
        let z = sim.terrain_cell_level(fx, fy).unwrap_or(0);
        let free = sim
            .spawn_object_at_height("NATBNK", &owner, fx, fy, 0, z, &resources.rules)
            .expect("the second bunker spawns");
        spawn_beside_base(sim, &resources.rules, "YuriCountry", &["CAOS"]);
        let team_type = sim
            .interner
            .get("07C0304C-G")
            .expect("retail TeamType interned");
        let team = sim
            .create_team(team_type, Some(yuri), &resources.rules)
            .or_else(|| {
                sim.team_script_vm
                    .teams_in_order()
                    .find(|team| team.team_type_id() == Some(team_type) && team.owner() == yuri)
                    .map(|team| team.id())
            })
            .expect("a team of the TeamType");
        let (mut on_61, mut past) = (None, None);
        for frame in 1..=3_000u32 {
            scenario.tick();
            let sim = scenario.sim();
            let Some(line) = sim.team_script_vm.team(team) else {
                break;
            };
            let members: Vec<u64> = line.members().collect();
            if line.cursor() == 1 {
                on_61 = Some((frame, members));
            } else if line.cursor() >= 2 && on_61.is_some() {
                let held = sim.entities().get(free).is_none_or(|bunker| {
                    bunker.bunker.occupant().is_some() || bunker.radio_contacts.slot(0).is_some()
                });
                past = Some((frame, members, held));
                break;
            }
        }
        eprintln!("Chaos Drone team: on action 61 {on_61:?}, past it {past:?}");
        let (_, on_61) = on_61.expect("the Chaos Drone team never reached action 61");
        let (_, members, held) = past.expect("the Chaos Drone team stayed on action 61");
        assert!(
            !on_61.is_empty(),
            "the team reached action 61 without a drone"
        );
        assert_eq!(members, on_61, "a drone left its team at action 61");
        assert!(!held, "the second bunker was taken");
    }

    #[cfg(feature = "npatch")]
    #[test]
    #[ignore = "requires VERA20K_AYMOD_DIR; ordered actual AY deployment prerequisites"]
    fn aymod_siege_chopper_deploy_hooks_are_unset_in_every_battle_layer() {
        use crate::rules::retail_sources::{RetailRulesSources, select_mode_ini};
        let root = std::path::PathBuf::from(std::env::var_os("VERA20K_AYMOD_DIR").unwrap());
        let assets = AssetManager::new(&root, MediaArchiveMode::STOCK_DIGITAL).unwrap();
        let sources = RetailRulesSources::select(&assets).unwrap();
        let map = crate::map::source::load_map_by_name_or_path_with_assets(
            &root,
            "重返迪拜.yrm",
            &assets,
        )
        .unwrap()
        .map;
        let mode = select_mode_ini(&assets, Some("MPBattleMD.ini")).unwrap();
        for (name, ini) in [
            ("RULESMD", Some(&sources.rulesmd.ini)),
            ("LANGRULE", sources.langrule.as_ref().map(|s| &s.ini)),
            ("MPBattleMD", mode.as_ref()),
            ("Return Dubai", Some(&map.ini)),
        ] {
            let Some(ini) = ini else { continue };
            let section = ini.section_or_empty("SCHP");
            // Original ctor: DeployFacing710F93=-1, TrueConversion7113A7=0,
            // DeployToFire747127=0. Prove absence per physical pass instead of
            // interpreting a merged empty value as a native retained default.
            for key in ["DeployFacing", "TrueConversion", "DeployToFire"] {
                assert!(
                    !section.is_present(key),
                    "{name} SCHP unexpectedly authors {key}"
                );
            }
        }
        let audio = crate::rules::audio_sources::AudioDefinitions::select(&assets);
        let mut owner = sources
            .into_owner(
                std::sync::Arc::clone(audio.sounds()),
                std::sync::Arc::clone(audio.eva()),
            )
            .unwrap();
        let (rules, _, _, _) = owner
            .load_scenario(
                crate::rules::process_owner::NativeScenarioRulesPrefix::NonCampaign(mode.as_ref()),
                &map.ini,
            )
            .unwrap()
            .into_parts();
        let schp = rules.object("SCHP").unwrap();
        assert!(schp.is_simple_deployer && schp.deploy_to_land && !schp.balloon_hover);
        assert!(schp.deploys_into.is_none());
        assert_eq!(schp.unloading_class.as_deref(), Some("SCHD"));
        assert!(
            schp.deploy_fire,
            "DeployFire is not TrueConversion or DeployToFire"
        );
        let miner = rules.object("SMIN").unwrap();
        assert!(!miner.is_simple_deployer);
        assert_eq!(miner.deploys_into.as_deref(), Some("YAREFN"));
        assert_eq!(rules.object("YAREFN").unwrap().deploy_facing, 0);
    }

    #[cfg(feature = "npatch")]
    #[test]
    #[ignore = "requires VERA20K_AYMOD_DIR; actual AY Siege Chopper gather/deploy diagnostic"]
    fn aymod_siege_chopper_team_gather_and_deploy_boundaries() {
        use crate::sim::mission::MissionType;
        use crate::sim::movement::jumpjet_movement::jumpjet_flight::STATE_HOLD;
        let root = std::path::PathBuf::from(std::env::var_os("VERA20K_AYMOD_DIR").unwrap());
        let map = "重返迪拜.yrm";
        let launch = battle_session(
            map,
            &[(LaunchCountry::from_country_index(1), AiDifficulty::Hard)],
        );
        let mut scenario = load_session(&root, map, 0x00C0_FFEE, launch).unwrap();
        for _ in 0..1500 {
            scenario.tick();
        }
        let (owner, choppers, team) = {
            let (sim, resources) = (
                &mut scenario.runtime.simulation,
                &scenario.runtime.resources,
            );
            let object = resources.rules.object("SCHP").unwrap();
            eprintln!(
                "AY SCHP: simple={} deploy_to_land={} balloon={} deploys_into={:?}",
                object.is_simple_deployer,
                object.deploy_to_land,
                object.balloon_hover,
                object.deploys_into
            );
            assert!(object.is_simple_deployer && object.deploy_to_land);
            assert!(!object.balloon_hover && object.deploys_into.is_none());
            let (owner, choppers) =
                spawn_beside_base(sim, &resources.rules, "Russians", &["SCHP", "SCHP"]);
            let type_id = sim.interner.get("07C3092C-G").unwrap();
            let team = sim
                .create_team(type_id, Some(owner), &resources.rules)
                .unwrap();
            (owner, choppers, team)
        };
        let mut held = None;
        for frame in 1..=1500 {
            scenario.tick();
            let sim = scenario.sim();
            let line = sim.team_script_vm.team(team).expect("the team lives");
            assert_eq!(sim.interner.resolve(line.script_id()), "07C48DBC-G");
            assert!(line.cursor() <= 0, "the team unexpectedly left regroup");
            let both = choppers.iter().all(|&id| {
                let entity = sim.entities().get(id).unwrap();
                let at = crate::sim::movement::ground_pose::position_world_coord(&entity.position);
                line.members().any(|member| member == id)
                    && entity.mission.current().known() == Some(MissionType::Move)
                    && entity.navigation.nav_com().is_some()
                    && entity
                        .locomotor
                        .as_ref()
                        .and_then(|l| l.jumpjet_runtime())
                        .is_some_and(|j| {
                            j.phase() == STATE_HOLD
                                && j.moving()
                                && (j.destination().x, j.destination().y) == (at.x, at.y)
                        })
            });
            match held {
                None if both => {
                    held = Some(frame);
                }
                Some(_) => assert!(both, "a member left its arrived hover hold"),
                None => {}
            }
            if frame == 1500 {
                eprintln!(
                    "AY SCHP owner={owner:?} team={team} cursor={} held_since={held:?}",
                    line.cursor()
                );
                assert_eq!(
                    line.cursor(),
                    0,
                    "native54BE09/6EBDA2 keeps the gather action"
                );
                assert!(held.is_some(), "no actual jumpjet hold was observed");
            }
        }
        // Component boundary only: skip the demonstrated native gather hold
        // to inspect the authored action9; this is NOT an autonomous team pass.
        scenario
            .runtime
            .simulation
            .team_script_vm
            .seek_line_for_test(team, 2);
        let (mut unloading, mut deployed, mut attacking) = ([false; 2], [false; 2], false);
        for _ in 0..600 {
            scenario.tick();
            let sim = scenario.sim();
            for (index, &id) in choppers.iter().enumerate() {
                let entity = sim.entities().get(id).unwrap();
                unloading[index] |= entity.mission.current().known() == Some(MissionType::Unload);
                deployed[index] |= entity.is_deployed();
            }
            attacking |= choppers.iter().all(|&id| {
                sim.entities().get(id).unwrap().mission.current().known()
                    == Some(MissionType::Attack)
            });
        }
        eprintln!(
            "AY SCHP action9 component: unloading={unloading:?}, deployed={deployed:?}, attacking={attacking}"
        );
        assert!(
            unloading.into_iter().all(|seen| seen),
            "action9 did not unload both members"
        );
        assert!(
            !deployed.into_iter().any(|seen| seen),
            "the component unexpectedly deployed a member"
        );
        assert!(
            attacking,
            "authored attack actions never followed deployment request"
        );
    }

    /// "Soviet SChop vs All" (`07C3092C-G`: two Siege Choppers, script
    /// `54 / 53 / 47,131072 / 9 / 0,9 / 49 / 0,2 / 0,1`) for a Normal Soviet
    /// AI against an idle America on XMP03T4, with two Siege Choppers spawned
    /// beside the standing base for the team.
    ///
    /// The team holds on action 54, as natively. Each chopper reaches the
    /// gather point and hovers there: a `DeployToLand=` simple deployer's
    /// State 2 holds without clearing the moving byte (`0x0054BE09..0x0054BE2F`).
    /// So `Coordinate_Move` keeps its NavCom (`Is_Moving` at `0x006EBDA2`) and
    /// counts it unarrived (`0x006EBF01..0x006EBF13`); only a `BalloonHover=`
    /// member is let off (`0x006EBED5..0x006EBEFB`). Ares bug 895814 reports
    /// this of the game: the AI's Siege Choppers run their script only once an
    /// enemy is within range, and `BalloonHover=yes` cures it. That release is
    /// not reproduced here.
    ///
    /// Put on line 3 instead, as action 6 would, the team's action 9 orders
    /// each chopper (`IsSimpleDeployer=`) to Unload, and they start to land.
    /// Action 9 is done at once, and `Coordinate_Attack` queues Attack even
    /// during Unload (`0x006EB79A..0x006EB7CB`): Unit vt+0x4E4 (`0x00736D50`)
    /// keeps only a `DeployToFire=` unit, or a computer one deploying into a
    /// `TickTank=` building, on Unload, and the Siege Chopper is neither. Queue_Mission
    /// (`0x005B35E0`) does not force commencement. A descending jumpjet reports
    /// moving (`0x0054D0D0`), so Unit readiness (`0x00744270`) can keep the
    /// queued Attack waiting until landing; cancelling Unload need not be airborne.
    #[test]
    #[ignore = "requires RA2_DIR with installed retail RA2/YR assets"]
    fn retail_soviet_siege_chopper_team_holds_then_attacks_undeployed() {
        use crate::sim::mission::MissionType;
        use crate::sim::movement::jumpjet_movement::jumpjet_flight::{STATE_DESCEND, STATE_HOLD};
        use crate::skirmish_launch::{AiDifficulty, LaunchCountry};

        let (mut scenario, _) =
            retail_battle(|_| {}, &[(LaunchCountry::Russia, AiDifficulty::Normal)]);
        for _ in 0..1500 {
            scenario.tick();
        }
        let runtime = &mut scenario.runtime;
        let (sim, resources) = (&mut runtime.simulation, &runtime.resources);
        let (soviet, choppers) =
            spawn_beside_base(sim, &resources.rules, "Russians", &["SCHP", "SCHP"]);
        let team_type = sim
            .interner
            .get("07C3092C-G")
            .expect("retail TeamType interned");
        let team = sim
            .create_team(team_type, Some(soviet), &resources.rules)
            .or_else(|| {
                sim.team_script_vm
                    .teams_in_order()
                    .find(|team| team.team_type_id() == Some(team_type) && team.owner() == soviet)
                    .map(|team| team.id())
            })
            .expect("a team of the TeamType");
        // Hovering on Move at its destination in State 2, with a NavCom and
        // the moving byte set (State 1 passes through State 2 at lift-off);
        // the team recruits the choppers on its first frames.
        let holding = |sim: &Simulation, id: u64| {
            sim.entities().get(id).is_some_and(|entity| {
                let at = crate::sim::movement::ground_pose::position_world_coord(&entity.position);
                entity.mission.current().known() == Some(MissionType::Move)
                    && entity.navigation.nav_com().is_some()
                    && entity
                        .locomotor
                        .as_ref()
                        .and_then(|locomotor| locomotor.jumpjet_runtime())
                        .is_some_and(|jumpjet| {
                            let to = jumpjet.destination();
                            jumpjet.phase() == STATE_HOLD
                                && jumpjet.moving()
                                && (to.x, to.y) == (at.x, at.y)
                        })
            })
        };
        let mut held = None;
        for frame in 1..=1_500u32 {
            scenario.tick();
            let sim = scenario.sim();
            let line = sim.team_script_vm.team(team).expect("the team lives");
            assert!(
                line.cursor() <= 0,
                "the team left action 54 at frame {frame}"
            );
            let both = choppers
                .iter()
                .all(|&id| line.members().any(|member| member == id) && holding(sim, id));
            match held {
                None if both => held = Some(frame),
                Some(since) => assert!(
                    both,
                    "a chopper left the team or its hold at frame {frame} (held from {since})"
                ),
                None => {}
            }
        }
        let since = held.expect("the choppers never held at the gather point");
        let sim = scenario.sim();
        let cells: Vec<_> = choppers
            .iter()
            .filter_map(|&id| sim.entities().get(id))
            .map(|entity| {
                (
                    (entity.position.rx(), entity.position.ry()),
                    entity.navigation.nav_com(),
                )
            })
            .collect();
        eprintln!("Siege Chopper team: held on action 54 from frame {since}: {cells:?}");
        assert!(since <= 1_000, "the choppers held only from frame {since}");

        scenario
            .runtime
            .simulation
            .team_script_vm
            .seek_line_for_test(team, 3);
        // The phase of each chopper's jumpjet, its height, whether it is
        // deployed and its mission.
        let state = |sim: &Simulation, id: u64| {
            let entity = sim.entities().get(id).expect("the chopper lives");
            (
                entity
                    .locomotor
                    .as_ref()
                    .and_then(|locomotor| locomotor.jumpjet_runtime())
                    .map(|jumpjet| jumpjet.phase()),
                crate::sim::movement::air_movement::current_fly_height(
                    entity,
                    sim.resolved_terrain.as_ref(),
                ),
                entity.is_deployed(),
                entity.mission.current().known(),
            )
        };
        let (mut on_9, mut past, mut attacking) = (None, None, None);
        let mut attack_waited_for_landing = false;
        for frame in 1..=600u32 {
            scenario.tick();
            let sim = scenario.sim();
            let states: Vec<_> = choppers.iter().map(|&id| state(sim, id)).collect();
            attack_waited_for_landing |= choppers.iter().zip(&states).any(|(&id, state)| {
                state.0 == Some(STATE_DESCEND)
                    && state.1 > 0
                    && state.3 == Some(MissionType::Unload)
                    && sim.entities().get(id).is_some_and(|entity| {
                        entity.mission.queued().known() == Some(MissionType::Attack)
                    })
            });
            assert!(
                states.iter().all(|state| !state.2),
                "a chopper deployed at frame {frame}: {states:?}"
            );
            let cursor = sim.team_script_vm.team(team).map(|team| team.cursor());
            if on_9.is_none() && cursor == Some(3) {
                on_9 = Some((frame, states.clone()));
            }
            if past.is_none() && on_9.is_some() && cursor.is_none_or(|cursor| cursor >= 4) {
                past = Some(frame);
            }
            if states
                .iter()
                .all(|state| state.3 == Some(MissionType::Attack))
            {
                attacking = Some((frame, cursor, states));
                break;
            }
        }
        eprintln!(
            "Siege Chopper team: on action 9 {on_9:?}, past it at {past:?}, attacking at \
             {attacking:?}"
        );
        let (_, unloading) = on_9.expect("the team never ran action 9");
        assert!(
            unloading.iter().all(|state| {
                state.0 == Some(STATE_DESCEND) && state.3 == Some(MissionType::Unload)
            }),
            "action 9 did not set both choppers landing to unload: {unloading:?}"
        );
        assert!(past.is_some(), "the team stayed on action 9");
        attacking.expect("no attack order took the choppers off Unload");
        assert!(
            attack_waited_for_landing,
            "a queued Attack never waited on the descending member's readiness"
        );
    }

    /// "Soviet MCV" (`043F83EC-G`: one SMCV, script `043F23CC-G`
    /// `54 / 9 / 49`, which the Hard "Soviet MCV - H" trigger makes for a
    /// house without a Construction Yard) for a Hard Soviet AI against an
    /// idle America on XMP03T4. Once the base stands its yard is removed, and
    /// an SMCV is spawned beside the base for the team. The MCV becomes a
    /// yard again, it leaves the team as it does, and the team is done with
    /// its TeamType's `Max=` slot free. The house's own MCV routes act on a
    /// team member too (UnitClass::AI's Hunt queue has no team check), so
    /// the deploy may be theirs: the test prints the MCV's orders.
    /// [`retail_soviet_mcv_team_deploys_its_mcv_by_action_9`] isolates
    /// action 9.
    #[test]
    #[ignore = "requires RA2_DIR with installed retail RA2/YR assets"]
    fn retail_hard_soviet_mcv_team_rebuilds_its_yard() {
        use crate::sim::mission::MissionType;
        use crate::skirmish_launch::{AiDifficulty, LaunchCountry};

        let (mut scenario, _) =
            retail_battle(|_| {}, &[(LaunchCountry::Russia, AiDifficulty::Hard)]);
        for _ in 0..1500 {
            scenario.tick();
        }
        let yards = |sim: &Simulation, owner| -> Vec<u64> {
            sim.entities()
                .values()
                .filter(|entity| {
                    !entity.death.is_dying()
                        && entity.owner() == owner
                        && sim.interner.resolve(entity.type_ref()) == "NACNST"
                })
                .map(crate::sim::game_entity::GameEntity::stable_id)
                .collect()
        };
        let runtime = &mut scenario.runtime;
        let (sim, resources) = (&mut runtime.simulation, &runtime.resources);
        let soviet = house_of(sim, "Russians");
        let lost = yards(sim, soviet);
        assert!(!lost.is_empty(), "the base has a yard");
        for yard in &lost {
            sim.uninit_with_rules(*yard, &resources.rules);
        }
        let (_, spawned) = spawn_beside_base(sim, &resources.rules, "Russians", &["SMCV"]);
        let mcv = spawned[0];
        let team_type = sim
            .interner
            .get("043F83EC-G")
            .expect("retail TeamType interned");
        let team = sim
            .create_team(team_type, Some(soviet), &resources.rules)
            .or_else(|| {
                sim.team_script_vm
                    .teams_in_order()
                    .find(|team| team.team_type_id() == Some(team_type) && team.owner() == soviet)
                    .map(|team| team.id())
            })
            .expect("a team of the TeamType");
        let (mut joined, mut on_9, mut rebuilt, mut done) = (None, None, None, None);
        let mut orders: Vec<(u32, Option<MissionType>, Option<MissionType>)> = Vec::new();
        for frame in 1..=3_000u32 {
            scenario.tick();
            let sim = scenario.sim();
            let line = sim.team_script_vm.team(team);
            if joined.is_none() && line.is_some_and(|line| line.members().any(|id| id == mcv)) {
                joined = Some(frame);
            }
            if on_9.is_none() && line.is_some_and(|line| line.cursor() == 1) {
                on_9 = Some((frame, line.map(|line| line.member_count())));
            }
            if let Some(entity) = sim
                .entities()
                .get(mcv)
                .filter(|entity| !entity.death.is_dying())
            {
                let order = (
                    entity.mission.current().known(),
                    entity.mission.queued().known(),
                );
                if orders.last().is_none_or(|last| (last.1, last.2) != order) {
                    orders.push((frame, order.0, order.1));
                }
            }
            if rebuilt.is_none() && !yards(sim, soviet).is_empty() {
                rebuilt = Some(frame);
            }
            if done.is_none() && line.is_none_or(|line| line.cursor() >= 2) {
                done = Some(frame);
            }
            if rebuilt.is_some() && done.is_some() {
                break;
            }
        }
        let sim = scenario.sim();
        let live = sim
            .team_script_vm
            .teams_in_order()
            .filter(|line| line.team_type_id() == Some(team_type) && line.owner() == soviet)
            .count();
        eprintln!(
            "MCV team: joined at {joined:?}, on action 9 {on_9:?}, yard rebuilt at {rebuilt:?}, \
             done at {done:?}, {live} left of the type; MCV orders (frame, current, queued) \
             {orders:?}"
        );
        assert!(joined.is_some(), "the MCV never joined the team");
        assert!(rebuilt.is_some(), "no yard was rebuilt");
        assert!(done.is_some(), "the team stayed on its script");
        assert_eq!(live, 0, "the TeamType's Max= slot is still taken");
    }

    /// The "Soviet MCV" team (`043F83EC-G`) of a Hard Soviet AI on XMP03T4,
    /// with an SMCV spawned beside the standing base for it and the house's
    /// `AutoBaseBuilding` latch cleared, so neither UnitClass::AI's Hunt
    /// queue nor Mission_Guard's Unload arm deploys the MCV. Put on line 1,
    /// action 9 asks the yard to stand at the MCV's own cell and orders it
    /// to Unload; Mission_Unload deploys it, it leaves the team as a yard, and
    /// the team, empty, is done.
    #[test]
    #[ignore = "requires RA2_DIR with installed retail RA2/YR assets"]
    fn retail_soviet_mcv_team_deploys_its_mcv_by_action_9() {
        use crate::sim::mission::{MissionId, MissionType};
        use crate::skirmish_launch::{AiDifficulty, LaunchCountry};

        let (mut scenario, _) =
            retail_battle(|_| {}, &[(LaunchCountry::Russia, AiDifficulty::Hard)]);
        for _ in 0..1500 {
            scenario.tick();
        }
        let yards = |sim: &Simulation, owner| -> usize {
            sim.entities()
                .values()
                .filter(|entity| {
                    !entity.death.is_dying()
                        && entity.owner() == owner
                        && sim.interner.resolve(entity.type_ref()) == "NACNST"
                })
                .count()
        };
        let runtime = &mut scenario.runtime;
        let (sim, resources) = (&mut runtime.simulation, &runtime.resources);
        let soviet = house_of(sim, "Russians");
        let standing = yards(sim, soviet);
        let house = sim.houses.get_mut(&soviet).expect("the Soviet house");
        house.set_ai_activation_for_test(crate::sim::house_state::HouseAiActivationLatches {
            auto_base_building: false,
            ..house.ai_activation()
        });
        // The first cell near the base where a yard can stand from every
        // origin within a cell of it: the MCV's own cell (action 9's test),
        // Deploy's (-1, -1), and a cell of drift.
        let yard = resources.rules.object("NACNST").expect("retail NACNST");
        let (bx, by) = sim.houses[&soviet]
            .base_center()
            .expect("the base has a centre");
        let mut around: Vec<(i32, i32)> = (0..1681).map(|n| (n % 41 - 20, n / 41 - 20)).collect();
        around.sort_by_key(|&(dx, dy)| (dx * dx + dy * dy, dy, dx));
        let (rx, ry) = around
            .into_iter()
            .map(|(dx, dy)| (i32::from(bx) + dx, i32::from(by) + dy))
            .find(|&(rx, ry)| {
                (-1..=1).all(|dy| {
                    (-1..=1).all(|dx| {
                        let (Ok(x), Ok(y)) = (i16::try_from(rx + dx), i16::try_from(ry + dy))
                        else {
                            return false;
                        };
                        crate::sim::build_site::can_place_building_at(
                            sim,
                            &resources.rules,
                            Some(&resources.overlay_registry),
                            yard,
                            (x, y),
                            None,
                            None,
                            None,
                        )
                    })
                })
            })
            .and_then(|(rx, ry)| Some((u16::try_from(rx).ok()?, u16::try_from(ry).ok()?)))
            .expect("room for a yard near the base");
        let z = sim.terrain_cell_level(rx, ry).unwrap_or(0);
        let owner = sim.interner.resolve(soviet).to_string();
        let mcv = sim
            .spawn_object_at_height("SMCV", &owner, rx, ry, 0, z, &resources.rules)
            .expect("the MCV spawns");
        let team_type = sim
            .interner
            .get("043F83EC-G")
            .expect("retail TeamType interned");
        let team = sim
            .create_team(team_type, Some(soviet), &resources.rules)
            .or_else(|| {
                sim.team_script_vm
                    .teams_in_order()
                    .find(|team| team.team_type_id() == Some(team_type) && team.owner() == soviet)
                    .map(|team| team.id())
            })
            .expect("a team of the TeamType");
        // A full team forms and starts its script over (`0x006E91AD..
        // 0x006E91F5`), so it is put on line 1 once formed.
        let mut joined = None;
        for frame in 1..=60u32 {
            scenario.tick();
            if scenario
                .sim()
                .team_script_vm
                .team(team)
                .is_some_and(|line| line.formed() && line.members().any(|id| id == mcv))
            {
                joined = Some(frame);
                break;
            }
        }
        assert!(joined.is_some(), "the MCV never joined the team");
        scenario
            .runtime
            .simulation
            .team_script_vm
            .seek_line_for_test(team, 1);
        let (mut ordered, mut deployed, mut done) = (None, None, None);
        for frame in 1..=1_500u32 {
            scenario.tick();
            let sim = scenario.sim();
            let line = sim.team_script_vm.team(team);
            if ordered.is_none()
                && line.is_some_and(|line| line.cursor() == 1)
                && sim.entities().get(mcv).is_some_and(|entity| {
                    entity.mission.effective() == MissionId::from_known(MissionType::Unload)
                })
            {
                ordered = Some(frame);
            }
            if deployed.is_none() && yards(sim, soviet) > standing {
                deployed = Some(frame);
            }
            if done.is_none() && line.is_none_or(|line| line.cursor() >= 2) {
                done = Some(frame);
            }
            if deployed.is_some() && done.is_some() {
                break;
            }
        }
        eprintln!(
            "MCV team, house routes off, MCV at ({rx},{ry}): joined at {joined:?}, Unload on \
             action 9 at {ordered:?}, deployed at {deployed:?}, done at {done:?}"
        );
        assert!(
            ordered.is_some(),
            "action 9 never ordered the MCV to Unload"
        );
        assert!(deployed.is_some(), "the MCV did not deploy");
        assert!(done.is_some(), "the team stayed on its script");
    }


    /// A Normal Soviet house's Rhino, in no team, shot by a GI beside it:
    /// `UnitClass::ReceiveDamage`'s computer-owner response gives it the GI
    /// as its NavCom and queues Move (`combat::unit_damage_response`), it
    /// drives off its cell at the GI, and crushes it.
    #[test]
    #[ignore = "requires RA2_DIR with installed retail RA2/YR assets"]
    fn retail_computer_rhino_runs_at_the_gi_that_shoots_it() {
        use crate::sim::command::{Command, CommandEnvelope};
        use crate::sim::components::NavTargetRef;
        use crate::sim::mission::{MissionId, MissionType};
        use crate::skirmish_launch::{AiDifficulty, LaunchCountry};

        let (mut scenario, _) =
            retail_battle(|_| {}, &[(LaunchCountry::Russia, AiDifficulty::Normal)]);
        for _ in 0..60 {
            scenario.tick();
        }
        let runtime = &mut scenario.runtime;
        let (sim, rules) = (&mut runtime.simulation, &runtime.resources.rules);
        let (_, spawned) = spawn_beside_base(sim, rules, "Russians", &["HTNK"]);
        let rhino = spawned[0];
        let start = {
            let entity = sim.entities().get(rhino).expect("the Rhino");
            (entity.position.rx(), entity.position.ry())
        };
        let america = house_of(sim, "Americans");
        let gi = spawn_on_free_cells(
            sim,
            rules,
            america,
            cells_nearest((i32::from(start.0), i32::from(start.1))),
            &["E1"],
        )[0];
        let tick = sim.session.tick();
        sim.queue_command(CommandEnvelope::new(
            america,
            tick + 1,
            Command::Attack {
                attacker_id: gi,
                target_id: rhino,
            },
        ));
        let full = sim.entities().get(rhino).unwrap().health.current();
        let move_mission = MissionId::from_known(MissionType::Move);
        let (mut hit, mut charged, mut moved, mut died) = (None, None, None, None);
        let (mut gi_cell, mut crushed) = (None, false);
        for frame in 1..=600u32 {
            let output = tick_with_due_commands(&mut scenario);
            let sim = scenario.sim();
            let entity = sim.entities().get(rhino).expect("the Rhino lives");
            if hit.is_none() && entity.health.current() < full {
                assert!(
                    sim.team_script_vm.team_for_member(rhino).is_none(),
                    "the Rhino joined a team before the hit"
                );
                hit = Some(frame);
            }
            if charged.is_none()
                && entity.navigation.nav_com() == Some(NavTargetRef::object(gi))
                && (entity.mission.queued() == move_mission
                    || entity.mission.current() == move_mission)
            {
                charged = Some(frame);
            }
            if charged.is_some()
                && moved.is_none()
                && (entity.position.rx(), entity.position.ry()) != start
            {
                moved = Some(frame);
            }
            match sim
                .entities()
                .get(gi)
                .filter(|gi| !gi.death.is_dying() && gi.health.current() > 0)
            {
                Some(gi) => gi_cell = Some((gi.position.rx(), gi.position.ry())),
                None => {
                    died = Some(frame);
                    // The per-cell crush plays the victim's `CrushSound=` at
                    // the crusher's Location, in the victim's own cell.
                    crushed = output.sound_events.iter().any(|event| {
                        matches!(
                            event,
                            crate::sim::world::SimSoundEvent::EntityCrushed { world, .. }
                                if gi_cell
                                    == Some((world.to_cell_sub_z().0, world.to_cell_sub_z().1))
                        )
                    });
                    break;
                }
            }
        }
        eprintln!(
            "hit {hit:?}, charged {charged:?}, moved {moved:?}, GI died {died:?}, crushed {crushed}"
        );
        assert!(hit.is_some(), "the GI never hit the Rhino");
        assert!(
            charged >= hit,
            "the Rhino never took the GI as its NavCom on Move"
        );
        assert!(moved.is_some(), "the Rhino never left its cell");
        assert!(died.is_some(), "the GI survived");
        assert!(crushed, "the GI died, but not under the Rhino");
    }

    /// A Normal Soviet house's Rhino sent to Attack a GI beside it drives over
    /// it: `UnitClass::Approach_Target`'s computer crush arm gives it the GI
    /// as its destination (`movement::foot_approach::ApproachCrush`).
    #[test]
    #[ignore = "requires RA2_DIR with installed retail RA2/YR assets"]
    fn retail_computer_rhino_drives_over_the_gi_it_attacks() {
        use crate::sim::combat::TargetKind;
        use crate::sim::components::NavTargetRef;
        use crate::sim::mission::{MissionId, MissionType};
        use crate::skirmish_launch::{AiDifficulty, LaunchCountry};

        let (mut scenario, _) =
            retail_battle(|_| {}, &[(LaunchCountry::Russia, AiDifficulty::Normal)]);
        for _ in 0..60 {
            scenario.tick();
        }
        let runtime = &mut scenario.runtime;
        let (sim, rules) = (&mut runtime.simulation, &runtime.resources.rules);
        let (_, spawned) = spawn_beside_base(sim, rules, "Russians", &["HTNK"]);
        let rhino = spawned[0];
        let start = {
            let entity = sim.entities().get(rhino).expect("the Rhino");
            (entity.position.rx(), entity.position.ry())
        };
        let america = house_of(sim, "Americans");
        let gi = spawn_on_free_cells(
            sim,
            rules,
            america,
            cells_nearest((i32::from(start.0), i32::from(start.1))),
            &["E1"],
        )[0];
        sim.assign_target_represented(
            rhino,
            Some(TargetKind::Entity(gi)),
            rules,
            None,
            crate::sim::world::FrameEffects::default(),
        )
        .expect("the Rhino takes the GI as its target");
        let now = sim.session.binary_frame();
        sim.substrate
            .mission_assign_exact(rhino, MissionId::from_known(MissionType::Attack), now)
            .expect("the Rhino takes Attack");
        let _ = sim.substrate.mission_commence_exact(rhino, now);
        let (mut charged, mut died, mut gi_cell, mut crushed) = (None, None, None, false);
        for frame in 1..=600u32 {
            let output = tick_with_due_commands(&mut scenario);
            let sim = scenario.sim();
            let entity = sim.entities().get(rhino).expect("the Rhino lives");
            if charged.is_none() && entity.navigation.nav_com() == Some(NavTargetRef::object(gi)) {
                charged = Some(frame);
            }
            match sim
                .entities()
                .get(gi)
                .filter(|gi| !gi.death.is_dying() && gi.health.current() > 0)
            {
                Some(gi) => gi_cell = Some((gi.position.rx(), gi.position.ry())),
                None => {
                    died = Some(frame);
                    crushed = output.sound_events.iter().any(|event| {
                        matches!(
                            event,
                            crate::sim::world::SimSoundEvent::EntityCrushed { world, .. }
                                if gi_cell
                                    == Some((world.to_cell_sub_z().0, world.to_cell_sub_z().1))
                        )
                    });
                    break;
                }
            }
        }
        // Its first Attack tick, the frame after Commence, takes the charge.
        assert_eq!(charged, Some(1), "the Rhino's first Attack tick");
        assert!(died.is_some(), "the GI survived");
        assert!(crushed, "the GI died, but not under the Rhino");
    }

    /// The local Allied player's Spy sent into a Normal Soviet house's Tesla
    /// Reactor walks in and leaves the game; the Soviet power blacks out for
    /// `SpyPowerBlackout=` frames, and the sim announces it for the local
    /// player's EVA (`sim::spy_infiltration`).
    #[test]
    #[ignore = "requires RA2_DIR with installed retail RA2/YR assets"]
    fn retail_player_spy_blacks_out_a_computer_power_plant() {
        use crate::sim::command::{Command, CommandEnvelope};
        use crate::sim::spy_infiltration::InfiltrationEffect;
        use crate::sim::world::SimSoundEvent;
        use crate::skirmish_launch::{AiDifficulty, LaunchCountry};

        let (mut scenario, _) =
            retail_battle(|_| {}, &[(LaunchCountry::Russia, AiDifficulty::Normal)]);
        let soviet = house_of(scenario.sim(), "Russians");
        let mut reactor = None;
        for _ in 0..6000 {
            scenario.tick();
            let sim = scenario.sim();
            reactor = sim
                .entities()
                .values()
                .find(|entity| {
                    entity.owner() == soviet
                        && !entity.death.is_dying()
                        && !entity.lifecycle.in_limbo()
                        && sim.interner.resolve(entity.type_ref()) == "NAPOWR"
                })
                .map(|entity| entity.stable_id());
            if reactor.is_some() {
                break;
            }
        }
        let reactor = reactor.expect("a Tesla Reactor");
        // Past its build-up.
        for _ in 0..300 {
            scenario.tick();
        }
        let runtime = &mut scenario.runtime;
        let (sim, rules) = (&mut runtime.simulation, &runtime.resources.rules);
        let at = {
            let entity = sim.entities().get(reactor).expect("the reactor stands");
            (
                i32::from(entity.position.rx()),
                i32::from(entity.position.ry()),
            )
        };
        let america = house_of(sim, "Americans");
        let cells = cells_nearest(at)
            .into_iter()
            .filter(|&(rx, ry)| (rx - at.0).abs().max((ry - at.1).abs()) >= 3);
        let spy = spawn_on_free_cells(sim, rules, america, cells, &["SPY"])[0];
        let tick = sim.session.tick();
        sim.queue_command(CommandEnvelope::new(
            america,
            tick + 1,
            Command::CaptureBuilding {
                engineer_id: spy,
                target_building_id: reactor,
            },
        ));
        let mut announced = None;
        for _ in 0..600 {
            let output = tick_with_due_commands(&mut scenario);
            announced = announced.or(output.sound_events.iter().find_map(|event| match event {
                SimSoundEvent::BuildingInfiltrated {
                    owner,
                    infiltrator,
                    effect,
                    ..
                } => Some((*owner, *infiltrator, *effect)),
                _ => None,
            }));
            if scenario.sim().entities().get(spy).is_none() {
                break;
            }
        }
        let sim = scenario.sim();
        assert_eq!(
            announced,
            Some((soviet, america, InfiltrationEffect::Power)),
            "the spy never went in"
        );
        assert!(sim.entities().get(spy).is_none(), "the spy stayed");
        let frame = sim.session.binary_frame();
        assert!(sim.power_states[&soviet].blackout_remaining(frame) > 0);
    }

    /// "Allied Spy vs Soviet Power" (`09B5091C-G`: one SPY, the script
    /// `0,4 / 47,131081 / 46,9 / 47,131083 / 46,11`) for a Normal American
    /// house against the idle local Soviet house, which is given a Tesla
    /// Reactor beside its MCV, guarded by a Sentry Gun and a Conscript. The
    /// team's spy is spawned twelve cells from the reactor, towards its own
    /// house's MCV. Action 0 (attack infantry) takes the flat walk over every
    /// object (`0x006F8FE0`) and finds the Conscript, which the `MakeupKit`
    /// (`Range=-2`) reaches from anywhere: its `MakesDisguise=` shot dresses
    /// the spy as a Soviet Conscript, and the `FireOnce=` shot ends the
    /// action. Action 47 brings it beside the reactor past guards that take
    /// it for their own, and action 46 hands it the reactor:
    /// `Assign_Target`'s infiltration arm heads it there, Mission_Attack's
    /// computer arm puts it on Capture, and it walks in and blacks the local
    /// house out.
    #[test]
    #[ignore = "requires RA2_DIR with installed retail RA2/YR assets"]
    fn retail_computer_spy_team_infiltrates_a_tesla_reactor() {
        use crate::sim::mission::MissionType;
        use crate::sim::spy_infiltration::InfiltrationEffect;
        use crate::sim::world::SimSoundEvent;
        use crate::skirmish_launch::{AiDifficulty, LaunchCountry};

        let (mut scenario, _) = retail_battle(
            |session| {
                session.local_slot_mut().country = LaunchCountry::Russia;
                session.options.unit_count = 0;
            },
            &[(LaunchCountry::America, AiDifficulty::Normal)],
        );
        let runtime = &mut scenario.runtime;
        let (sim, rules) = (&mut runtime.simulation, &runtime.resources.rules);
        let soviet = house_of(sim, "Russians");
        let america = house_of(sim, "Americans");
        let at = |sim: &Simulation, id| {
            let entity = sim.entities().get(id).expect("spawned");
            (
                i32::from(entity.position.rx()),
                i32::from(entity.position.ry()),
            )
        };
        let mcv = sim
            .substrate
            .entities
            .iter_sorted()
            .find(|(_, entity)| entity.owner() == soviet)
            .map(|(id, _)| id)
            .expect("the local MCV");
        let (mx, my) = at(sim, mcv);
        let reactor = spawn_on_free_cells(
            sim,
            rules,
            soviet,
            cells_nearest((mx + 5, my + 5)),
            &["NAPOWR"],
        )[0];
        let (rx, ry) = at(sim, reactor);
        spawn_on_free_cells(
            sim,
            rules,
            soviet,
            cells_nearest((rx, ry)),
            &["NALASR", "E2"],
        );
        // Twelve cells out towards the American MCV. The nearest cells the
        // other way lie in no zone (0xFFFF), and Evaluate_Candidate's zone
        // test, which only DWORD -1 skips (`0x006F7E36`), then rejects every
        // quarry of a spy standing there.
        let american_mcv = sim
            .substrate
            .entities
            .iter_sorted()
            .find(|(_, entity)| entity.owner() == america)
            .map(|(id, _)| id)
            .expect("the American MCV");
        let (ax, ay) = at(sim, american_mcv);
        let reach = (ax - rx).abs().max((ay - ry).abs()).max(1);
        let toward = (rx + (ax - rx) * 12 / reach, ry + (ay - ry) * 12 / reach);
        let cells = cells_nearest(toward)
            .into_iter()
            .filter(|&(x, y)| (x - rx).abs().max((y - ry).abs()) >= 10);
        let spy = spawn_on_free_cells(sim, rules, america, cells, &["SPY"])[0];
        let id = sim
            .interner
            .get("09B5091C-G")
            .expect("retail TeamType interned");
        // The computer house may already have made the team; then it
        // recruits the new spy.
        sim.create_team(id, Some(america), rules)
            .or_else(|| {
                sim.team_script_vm
                    .teams_in_order()
                    .find(|team| team.team_type_id() == Some(id) && team.owner() == america)
                    .map(|team| team.id())
            })
            .expect("a team of the TeamType");
        let (mut capturing, mut announced, mut hurt) = (None, None, false);
        for frame in 1..=2_000u32 {
            let output = tick_with_due_commands(&mut scenario);
            announced = announced.or(output.sound_events.iter().find_map(|event| match event {
                SimSoundEvent::BuildingInfiltrated {
                    owner,
                    infiltrator,
                    effect,
                    ..
                } => Some((frame, *owner, *infiltrator, *effect)),
                _ => None,
            }));
            if let Some(entity) = scenario.sim().entities().get(spy) {
                hurt |= entity.health.current() < 100;
                if capturing.is_none()
                    && entity.mission.current().known() == Some(MissionType::Capture)
                {
                    let worn = entity.disguise.as_ref().filter(|d| d.is_disguised());
                    capturing = Some((frame, worn.map(|d| (d.type_id(), d.house()))));
                }
            }
            if announced.is_some() {
                break;
            }
        }
        eprintln!("09B5091C-G: took Capture at {capturing:?}, went in: {announced:?}");
        let conscript = scenario.sim().interner.get("E2");
        assert_eq!(
            capturing.map(|(_, worn)| worn),
            Some(Some((conscript, Some(soviet)))),
            "the spy took Capture dressed as a Soviet Conscript"
        );
        assert!(!hurt, "a guard shot the disguised spy");
        assert_eq!(
            announced.map(|(_, owner, infiltrator, effect)| (owner, infiltrator, effect)),
            Some((soviet, america, InfiltrationEffect::Power)),
            "the spy never went in"
        );
        let sim = scenario.sim();
        assert!(sim.power_states[&soviet].blackout_remaining(sim.session.binary_frame()) > 0);
    }

    /// A Normal American house's Tanya (retail `[TANY]`: `C4=yes`,
    /// `Secondary=Sapper`) handed a Soviet Tesla Reactor eight cells away
    /// with Attack, as a team's action 46 does. Mission_Attack's C4 arm heads
    /// her in on Sabotage, she plants on the first reactor cell she reaches,
    /// and retail `C4Delay=.03` sets the charge off 26 frames later. The
    /// reactor dies; she lives.
    #[test]
    #[ignore = "requires RA2_DIR with installed retail RA2/YR assets"]
    fn retail_computer_tanya_plants_c4_on_a_tesla_reactor() {
        use crate::sim::combat::TargetKind;
        use crate::sim::mission::{MissionId, MissionType};
        use crate::sim::world::FrameEffects;
        use crate::skirmish_launch::{AiDifficulty, LaunchCountry};

        let (mut scenario, _) = retail_battle(
            |session| {
                session.local_slot_mut().country = LaunchCountry::Russia;
                session.options.unit_count = 0;
            },
            &[(LaunchCountry::America, AiDifficulty::Normal)],
        );
        let runtime = &mut scenario.runtime;
        let (sim, rules) = (&mut runtime.simulation, &runtime.resources.rules);
        let soviet = house_of(sim, "Russians");
        let america = house_of(sim, "Americans");
        let at = |sim: &Simulation, id| {
            let entity = sim.entities().get(id).expect("spawned");
            (
                i32::from(entity.position.rx()),
                i32::from(entity.position.ry()),
            )
        };
        let mcv_of = |sim: &Simulation, house| {
            sim.substrate
                .entities
                .iter_sorted()
                .find(|(_, entity)| entity.owner() == house)
                .map(|(id, _)| id)
                .expect("the house's MCV")
        };
        let (mx, my) = at(sim, mcv_of(sim, soviet));
        let reactor = spawn_on_free_cells(
            sim,
            rules,
            soviet,
            cells_nearest((mx + 5, my + 5)),
            &["NAPOWR"],
        )[0];
        let (rx, ry) = at(sim, reactor);
        // Out towards the American MCV: some nearer cells the other way lie
        // in no zone (see the spy team's test).
        let (ax, ay) = at(sim, mcv_of(sim, america));
        let reach = (ax - rx).abs().max((ay - ry).abs()).max(1);
        let toward = (rx + (ax - rx) * 8 / reach, ry + (ay - ry) * 8 / reach);
        let cells = cells_nearest(toward)
            .into_iter()
            .filter(|&(x, y)| (x - rx).abs().max((y - ry).abs()) >= 6);
        let tanya = spawn_on_free_cells(sim, rules, america, cells, &["TANY"])[0];
        let now = sim.session.binary_frame();
        sim.substrate
            .mission_assign_exact(tanya, MissionId::from_known(MissionType::Attack), now)
            .unwrap();
        sim.assign_target_represented(
            tanya,
            Some(TargetKind::Entity(reactor)),
            rules,
            None,
            FrameEffects::default(),
        )
        .unwrap();
        let (mut sabotage, mut planted, mut died) = (None, None, None);
        for frame in 1..=1_500u32 {
            tick_with_due_commands(&mut scenario);
            let sim = scenario.sim();
            if sabotage.is_none()
                && sim.entities().get(tanya).is_some_and(|tanya| {
                    tanya.mission.current().known() == Some(MissionType::Sabotage)
                })
            {
                sabotage = Some(frame);
            }
            if planted.is_none()
                && let Some(pending) = sim
                    .entities()
                    .get(reactor)
                    .and_then(|reactor| reactor.pending_c4_detonation.pending())
            {
                planted = Some((frame, pending));
            }
            if sim
                .entities()
                .get(reactor)
                .is_none_or(|reactor| reactor.death.is_dying())
            {
                died = Some(frame);
                break;
            }
        }
        eprintln!("TANY: Sabotage at {sabotage:?}, planted {planted:?}, reactor died {died:?}");
        assert!(sabotage.is_some(), "Tanya never took Sabotage");
        let (plant_frame, pending) = planted.expect("Tanya planted no charge");
        assert_eq!(pending.source_entity_id(), Some(tanya));
        assert_eq!(pending.timer().duration(), 26);
        assert_eq!(died, Some(plant_frame + 26));
        assert!(
            scenario
                .sim()
                .entities()
                .get(tanya)
                .is_some_and(|tanya| !tanya.death.is_dying() && tanya.health.current() > 0),
            "Tanya died"
        );
    }

    /// A Normal Soviet house's War Miner out on the ore with a load, brought
    /// to half strength and shot by a Grizzly: its refinery answers its HELLO
    /// and it queues Enter (`combat::unit_damage_response`), then docks.
    #[test]
    #[ignore = "requires RA2_DIR with installed retail RA2/YR assets"]
    fn retail_damaged_computer_miner_heads_for_its_refinery() {
        use crate::sim::command::{Command, CommandEnvelope};
        use crate::sim::mission::{MissionId, MissionType};
        use crate::skirmish_launch::{AiDifficulty, LaunchCountry};

        let (mut scenario, _) =
            retail_battle(|_| {}, &[(LaunchCountry::Russia, AiDifficulty::Normal)]);
        let soviet = house_of(scenario.sim(), "Russians");
        let harvest = MissionId::from_known(MissionType::Harvest);
        let mut miner = None;
        for _ in 0..6000 {
            scenario.tick();
            let sim = scenario.sim();
            miner = sim
                .entities()
                .values()
                .find(|entity| {
                    entity.owner() == soviet
                        && !entity.death.is_dying()
                        && sim.interner.resolve(entity.type_ref()) == "HARV"
                        && entity.radio_contacts.is_empty()
                        && entity.mission.current() == harvest
                        && entity
                            .miner
                            .as_ref()
                            .is_some_and(|miner| miner.cargo().len() >= 5)
                })
                .map(|entity| entity.stable_id());
            if miner.is_some() {
                break;
            }
        }
        let miner = miner.expect("a loaded War Miner on the ore");
        let runtime = &mut scenario.runtime;
        let (sim, rules) = (&mut runtime.simulation, &runtime.resources.rules);
        let strength = rules.object("HARV").expect("retail HARV").strength;
        let at = {
            let entity = sim.substrate.entities.get_mut(miner).unwrap();
            entity.health.set_for_test(strength / 2);
            (
                i32::from(entity.position.rx()),
                i32::from(entity.position.ry()),
            )
        };
        let america = house_of(sim, "Americans");
        let cells = cells_nearest(at)
            .into_iter()
            .filter(|&(rx, ry)| (rx - at.0).abs().max((ry - at.1).abs()) >= 2);
        let tank = spawn_on_free_cells(sim, rules, america, cells, &["MTNK"])[0];
        let tick = sim.session.tick();
        sim.queue_command(CommandEnvelope::new(
            america,
            tick + 1,
            Command::Attack {
                attacker_id: tank,
                target_id: miner,
            },
        ));
        let enter = MissionId::from_known(MissionType::Enter);
        let (mut entered, mut docked) = (None, None);
        for frame in 1..=900u32 {
            tick_with_due_commands(&mut scenario);
            let sim = scenario.sim();
            let entity = sim.entities().get(miner).expect("the miner lives");
            let refinery = entity.radio_contacts.slot(0).filter(|&dock| {
                sim.entities().get(dock).is_some_and(|dock| {
                    dock.owner() == soviet && sim.interner.resolve(dock.type_ref()) == "NAREFN"
                })
            });
            if entered.is_none()
                && refinery.is_some()
                && (entity.mission.queued() == enter || entity.mission.current() == enter)
            {
                entered = Some(frame);
            }
            if entered.is_some() && entity.tether.partner().is_some() {
                docked = Some(frame);
                break;
            }
        }
        eprintln!("entered {entered:?}, docked {docked:?}");
        assert!(entered.is_some(), "the miner never headed for its refinery");
        assert!(docked.is_some(), "the miner never docked");
    }

    /// A Normal Soviet house on XMP03T4 whose refineries and War Miners are
    /// gone needs no rebuild a credit above a refinery's price, and does a
    /// credit below it (`sim::ai_build_need`). There, the strategy tick's
    /// Manage_Build_Queue sells the last sellable building of its base plan,
    /// which pays, and chooses a refinery. The yard stands, nothing is sent
    /// to hunt, and the yard builds the refinery, which brings a War Miner
    /// back.
    #[test]
    #[ignore = "requires RA2_DIR with installed retail RA2/YR assets"]
    fn retail_broke_computer_house_sells_to_rebuild_its_refinery() {
        use crate::map::entities::EntityCategory;
        use crate::sim::ai_build_need::MANAGE_CALLS;
        use crate::sim::mission::{MissionId, MissionType};
        use crate::skirmish_launch::{AiDifficulty, LaunchCountry};

        let (mut scenario, _) =
            retail_battle(|_| {}, &[(LaunchCountry::Russia, AiDifficulty::Normal)]);
        let soviet = house_of(scenario.sim(), "Russians");
        // The house's live objects of type `name`, on the map or, with
        // `limbo`, in limbo too (a factory's product).
        let of_type = |sim: &Simulation, name: &str, limbo: bool| -> Vec<u64> {
            sim.entities()
                .values()
                .filter(|entity| {
                    !entity.death.is_dying()
                        && (limbo || !entity.lifecycle.in_limbo())
                        && entity.owner() == soviet
                        && sim.interner.resolve(entity.type_ref()) == name
                })
                .map(crate::sim::game_entity::GameEntity::stable_id)
                .collect()
        };
        let mut standing = None;
        for frame in 1..=9000u32 {
            scenario.tick();
            let sim = scenario.sim();
            let economy = ["NAREFN", "HARV"]
                .iter()
                .all(|name| of_type(sim, name, true).len() == of_type(sim, name, false).len());
            if economy
                && ["NAREFN", "NAWEAP", "HARV"]
                    .iter()
                    .all(|name| !of_type(sim, name, false).is_empty())
            {
                standing = Some(frame);
                break;
            }
        }
        assert!(
            standing.is_some(),
            "the base never stood with a refinery, a war factory and a miner"
        );
        let runtime = &mut scenario.runtime;
        let (sim, rules) = (&mut runtime.simulation, &runtime.resources.rules);
        let lost: Vec<u64> = ["NAREFN", "HARV"]
            .iter()
            .flat_map(|name| of_type(&*sim, name, false))
            .collect();
        for id in lost {
            sim.uninit_with_rules(id, rules);
        }
        let price = sim.cost_of(
            soviet,
            rules.object("NAREFN").expect("retail NAREFN"),
            rules,
        );
        // A yard's house can make money above the refinery's price, and must
        // rebuild below it.
        for (credits, need) in [(price + 1, false), (price - 1, true)] {
            sim.houses
                .get_mut(&soviet)
                .unwrap()
                .economy_mut()
                .set_credits_for_test(credits);
            assert_eq!(
                crate::sim::ai_build_need::check_build_need(sim, rules, soviet),
                need,
                "{credits} credits"
            );
        }
        let refinery = rules.building_type_index("NAREFN").expect("retail NAREFN");
        let (selling, construction) = (
            MissionId::from_known(MissionType::Selling),
            MissionId::from_known(MissionType::Construction),
        );
        MANAGE_CALLS.set(Some(0));
        let (mut managed, mut chosen, mut rebuilt, mut mined) = (None, None, None, None);
        let mut sold = std::collections::BTreeMap::new();
        for frame in 1..=9000u32 {
            scenario.tick();
            let sim = scenario.sim();
            if managed.is_none() && MANAGE_CALLS.get().is_some_and(|calls| calls > 0) {
                managed = Some(frame);
            }
            for entity in sim.entities().values() {
                if entity.owner() == soviet
                    && entity.category() == EntityCategory::Structure
                    && entity.mission.current() == selling
                {
                    sold.entry(entity.stable_id()).or_insert_with(|| {
                        (frame, sim.interner.resolve(entity.type_ref()).to_string())
                    });
                }
            }
            if chosen.is_none()
                && managed.is_some()
                && sim.houses[&soviet].ai_production().building_choice() == refinery
            {
                chosen = Some(frame);
            }
            if rebuilt.is_none()
                && of_type(sim, "NAREFN", false).iter().any(|&id| {
                    sim.entities()
                        .get(id)
                        .is_some_and(|building| building.mission.current() != construction)
                })
            {
                rebuilt = Some(frame);
            }
            if mined.is_none() && !of_type(sim, "HARV", false).is_empty() {
                mined = Some(frame);
            }
            if rebuilt.is_some() && mined.is_some() {
                break;
            }
        }
        let calls = MANAGE_CALLS.take().unwrap_or(0);
        let sim = scenario.sim();
        let hunting = sim.houses[&soviet].strategy_emergency().all_to_hunt_bias();
        let yards = of_type(sim, "NACNST", false).len();
        eprintln!(
            "base standing at {standing:?}; Manage_Build_Queue first at {managed:?} ({calls} \
             calls), refinery chosen at {chosen:?}, standing at {rebuilt:?}, miner back at \
             {mined:?}; {yards} yards, hunting {hunting}; sold (frame, type) {:?}",
            sold.values().collect::<Vec<_>>()
        );
        assert!(managed.is_some(), "Manage_Build_Queue never ran");
        assert!(!sold.is_empty(), "the house sold nothing");
        assert!(
            sold.values().all(|(_, name)| name != "NACNST") && yards > 0,
            "the house sold its yard"
        );
        assert!(!hunting, "the house sent everything to hunt");
        assert!(chosen.is_some(), "the house never chose a refinery");
        assert!(rebuilt.is_some(), "no refinery was rebuilt");
        assert!(mined.is_some(), "no War Miner came back");
    }
}

#[cfg(test)]
mod launch_special_flags_tests {
    use super::*;

    /// The nonzero game mode's session-flags replacement (`0x00687C16..
    /// 0x00687C29`) for Scenario bit `0x800`: `PregameSetup` stores the
    /// session's Harvester Truce byte (`0x00A8B31D`) in the staging word
    /// (`0x005E7495..0x005E74D4`), and the launch copies it over the map's
    /// flags. A launched session with the truce on carries the bit; the
    /// default skirmish session — whose dialog clears the byte
    /// (`0x006AD89D`) — leaves it clear.
    #[test]
    #[ignore = "requires RA2_DIR with installed retail RA2/YR assets"]
    fn headless_launch_harvester_truce_sets_the_scenario_bit() {
        let root = std::path::PathBuf::from(
            std::env::var("RA2_DIR").expect("set RA2_DIR to the retail RA2/YR install directory"),
        );
        let mut truce = battle_session(
            "XMP03T4.MAP",
            &[(
                crate::skirmish_launch::LaunchCountry::America,
                crate::skirmish_launch::AiDifficulty::Easy,
            )],
        );
        truce.options.harvester_truce = true;
        let scene = load_session(&root, "XMP03T4.MAP", 0x1234_5678, truce)
            .expect("launch with Harvester Truce on");
        assert!(
            scene.sim().session.harvester_immune(),
            "the session's truce byte replaces the map's bit"
        );

        let plain = load_session(
            &root,
            "XMP03T4.MAP",
            0x1234_5678,
            battle_session(
                "XMP03T4.MAP",
                &[(
                    crate::skirmish_launch::LaunchCountry::America,
                    crate::skirmish_launch::AiDifficulty::Easy,
                )],
            ),
        )
        .expect("launch with the default options");
        assert!(
            !plain.sim().session.harvester_immune(),
            "the default skirmish session leaves the bit clear"
        );
    }
}
