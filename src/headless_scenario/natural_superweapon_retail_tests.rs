use super::battle_test_evidence::{
    aymod_anchor, cells_nearest, house_of, spawn_on_free_cells, tick_with_due_commands,
};
use super::*;
use crate::sim::mission::{MissionId, MissionType};
use crate::sim::scenario_session::ScenarioLightingProfile;
use crate::sim::superweapon::{invulnerability, psychic_dominator};
use crate::sim::world::{
    SimSoundEvent,
    bridge_test_evidence::{restored_retail, save_scene},
};

#[test]
#[ignore = "requires VERA20K_AYMOD_DIR; AI naturally builds and charges its Dominator, captures and restores lighting across saves"]
fn aymod_natural_dominator_build_charge_capture_and_recovery() {
    let root = std::path::PathBuf::from(std::env::var_os("VERA20K_AYMOD_DIR").unwrap());
    let map = "XMP03T4.MAP";
    let mut launch = battle_session(
        map,
        &[(LaunchCountry::from_country_index(2), AiDifficulty::Hard)],
    );
    launch.options.unit_count = 0;
    launch.options.short_game = false;
    launch.options.super_weapons = true;
    let mut scene = load_session(&root, map, 0x00C0_FFEE, launch).unwrap();
    let ai = house_of(scene.sim(), "YuriCountry");
    let human = scene.sim().session.current_house().unwrap();
    let sw = scene.sim().interner.get("PsychicDominatorSpecial").unwrap();
    let provider_type = scene.sim().interner.get("YAPPET").unwrap();
    assert!(!scene.sim().super_weapons[&ai][&sw].is_active());
    let targets = {
        let runtime = &mut scene.runtime;
        let (sim, rules) = (&mut runtime.simulation, &runtime.resources.rules);
        let at = aymod_anchor(sim, human);
        let enemy_at = aymod_anchor(sim, ai);
        // Only the idle human observer is protected. The AI's economy,
        // prerequisites, production, teams, power and charge remain untouched.
        let observer = spawn_on_free_cells(sim, rules, human, cells_nearest(at), &["GACNST"])[0];
        let now = sim.session.binary_frame();
        invulnerability::apply_invulnerability(
            sim.substrate.entities.get_mut(observer).unwrap(),
            now,
            90000,
            invulnerability::InvulnKind::IronCurtain,
        );
        let remote = sim
            .resolved_terrain()
            .unwrap()
            .cells()
            .iter()
            .filter(|cell| {
                !cell.outside_playfield
                    && (i32::from(cell.rx) - at.0)
                        .abs()
                        .max((i32::from(cell.ry) - at.1).abs())
                        > 25
                    && (i32::from(cell.rx) - enemy_at.0)
                        .abs()
                        .max((i32::from(cell.ry) - enemy_at.1).abs())
                        > 25
                    && sim
                        .path_grid()
                        .unwrap()
                        .is_walkable_for_infantry(cell.rx, cell.ry)
            })
            .min_by_key(|cell| {
                (i32::from(cell.rx) - at.0).pow(2) + (i32::from(cell.ry) - at.1).pow(2)
            })
            .map(|cell| (i32::from(cell.rx), i32::from(cell.ry)))
            .unwrap();
        let targets = spawn_on_free_cells(sim, rules, human, cells_nearest(remote), &["MTNK"; 24]);
        for &id in &targets {
            sim.mission_queue_exact(
                id,
                MissionId::from_known(MissionType::Sleep),
                1,
                sim.session.binary_frame(),
                &crate::sim::mission::authority::LiveReadyInputProvider { rules },
            )
            .unwrap();
        }
        targets
    };
    let pristine = scene.sim().resolved_terrain().unwrap().clone();
    let map_profile = scene.sim().session.lighting().dominator();
    let mut provider = None;
    let mut previous_balance = None;
    let mut paid = false;
    let mut construction = false;
    let mut first_charge = None;
    let mut charge_saved = false;
    let mut launched = false;
    let mut effect_saved = false;
    let mut eligible_at_launch = Vec::new();
    let mut captured = Vec::new();
    let mut finished = false;
    for _ in 0..60000 {
        let output = tick_with_due_commands(&mut scene);
        assert!(output.tick.frame_committed);
        let sim = scene.sim();
        for factory in sim.production().factories().iter_insertion_ordered() {
            if factory.owner() == ai
                && let Some(object) = factory
                    .object()
                    .filter(|object| object.type_id == provider_type)
            {
                let id = object.entity_id.unwrap();
                if let Some(old) = provider {
                    assert_eq!(old, id, "follow the original paid facility");
                }
                provider = Some(id);
                if previous_balance.is_some_and(|before| factory.balance() < before) {
                    paid = true;
                }
                previous_balance = Some(factory.balance());
            }
        }
        construction |= provider
            .and_then(|id| sim.entities().get(id))
            .is_some_and(|entity| {
                !entity.lifecycle.in_limbo()
                    && entity.mission.effective().known() == Some(MissionType::Construction)
            });
        let instance = &sim.super_weapons[&ai][&sw];
        if instance.is_active() && first_charge.is_none() {
            assert!(
                paid && construction,
                "only the naturally paid, placed and constructed provider may grant the weapon"
            );
            assert!(!instance.is_ready() && instance.charge_duration() > 0);
            first_charge = Some((instance.charge_start_tick(), instance.charge_duration()));
        }
        let launch_event = output.sound_events.iter().any(|event| matches!(event,
            SimSoundEvent::SuperWeaponLaunched { owner, sw_type, .. } if *owner == ai && *sw_type == sw));
        if launch_event
            && psychic_dominator::active(sim)
            && sim.psychic_dominator().owner() == Some(ai)
        {
            assert!(
                charge_saved,
                "the natural charge must pass the saved partial-charge boundary"
            );
            let (start, duration) = first_charge.unwrap();
            assert!(
                i64::from(sim.session.binary_frame()) - i64::from(start) >= i64::from(duration)
            );
            assert!(
                sim.session
                    .lighting()
                    .profile_lighting_enabled(ScenarioLightingProfile::Dominator)
            );
            assert!(sim.session.lighting().alternate_rgb().is_some());
            eligible_at_launch = targets
                .iter()
                .copied()
                .filter(|id| {
                    sim.entities()
                        .get(*id)
                        .is_some_and(|entity| entity.owner() == human)
                })
                .collect();
            launched = true;
        }
        captured = eligible_at_launch
            .iter()
            .copied()
            .filter(|id| {
                sim.entities().get(*id).is_some_and(|entity| {
                    entity.owner() == ai
                        && entity.mind_control.permanent()
                        && entity.mind_control.controller().is_none()
                })
            })
            .collect();
        let remaining = instance.charge_remaining(sim.session.binary_frame() as i32);
        if !charge_saved
            && instance.is_active()
            && !instance.is_ready()
            && remaining > 0
            && remaining < instance.charge_duration() / 2
        {
            let clock = (instance.charge_start_tick(), instance.charge_duration());
            let bytes = save_scene(
                &scene,
                "AY naturally paid Dominator during its first full charge",
            );
            scene.runtime.simulation = restored_retail(&scene, &pristine, &bytes);
            let restored = &scene.sim().super_weapons[&ai][&sw];
            assert_eq!(
                (restored.charge_start_tick(), restored.charge_duration()),
                clock
            );
            assert!(!restored.is_ready());
            charge_saved = true;
        } else if launched
            && !effect_saved
            && !captured.is_empty()
            && psychic_dominator::tints_scenario(sim)
        {
            let tint = sim.session.lighting().alternate_rgb();
            let bytes = save_scene(
                &scene,
                "AY natural Dominator captured targets while its dedicated tint remains active",
            );
            scene.runtime.simulation = restored_retail(&scene, &pristine, &bytes);
            assert_eq!(scene.sim().session.lighting().alternate_rgb(), tint);
            for &id in &captured {
                assert!(
                    scene
                        .sim()
                        .entities()
                        .get(id)
                        .unwrap()
                        .mind_control
                        .permanent()
                );
            }
            effect_saved = true;
        } else if launched && effect_saved && !psychic_dominator::active(sim) {
            assert_eq!(
                sim.session.lighting().selected_profile(),
                ScenarioLightingProfile::Normal
            );
            assert_eq!(sim.session.lighting().alternate_rgb(), None);
            assert_eq!(sim.session.lighting().dominator(), map_profile);
            assert_eq!(
                sim.session.lighting().current_ambient(),
                sim.session.lighting().normal().ambient_percent
            );
            assert!(!captured.is_empty());
            assert!(instance.is_active() && !instance.is_ready() && remaining > 0);
            finished = true;
            break;
        }
    }
    assert!(
        finished,
        "natural Dominator chain incomplete: provider={provider:?}, paid={paid}, construction={construction}, charge={first_charge:?}, saved={charge_saved}, launch={launched}, effect_saved={effect_saved}, captured={}, AI buildings={:?}",
        captured.len(),
        scene
            .sim()
            .entities()
            .values()
            .filter(|entity| entity.owner() == ai
                && entity.category() == crate::map::entities::EntityCategory::Structure)
            .map(|entity| scene.sim().interner.resolve(entity.type_ref()))
            .collect::<Vec<_>>()
    );
    eprintln!(
        "natural AY Dominator: paid provider, complete charge, permanent captures={}, dedicated tint restored and ended; tick={}",
        captured.len(),
        scene.sim().session.tick()
    );
}
