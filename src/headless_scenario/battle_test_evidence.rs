//! Shared physical Battle fixtures for core and offscreen client regressions.
//! All spawning, commands, saves and frames delegate to production owners.

use super::*;

/// The first house of `country`.
pub(super) fn house_of(sim: &Simulation, country: &str) -> crate::sim::intern::InternedId {
    let country_id = sim.interner.get(country).expect("retail country interned");
    sim.houses
        .iter()
        .find(|(_, house)| house.country() == Some(country_id))
        .map(|(&id, _)| id)
        .expect("a house of the country")
}

/// Spawns `types` for `house`, in order, each on the next of `cells`
/// that is a free infantry cell.
pub(super) fn spawn_on_free_cells(
    sim: &mut Simulation,
    rules: &crate::rules::ruleset::RuleSet,
    house: crate::sim::intern::InternedId,
    cells: impl IntoIterator<Item = (i32, i32)>,
    types: &[&str],
) -> Vec<u64> {
    use crate::sim::movement::locomotor::MovementLayer;

    let owner = sim.interner.resolve(house).to_string();
    let free: Vec<(u16, u16)> = cells
        .into_iter()
        .filter_map(|(rx, ry)| {
            let (rx, ry) = (u16::try_from(rx).ok()?, u16::try_from(ry).ok()?);
            let walkable = sim
                .path_grid()
                .is_some_and(|grid| grid.is_walkable_for_infantry(rx, ry));
            let empty = sim
                .substrate
                .occupancy
                .get(rx, ry)
                .is_none_or(|cell| cell.is_empty_on(MovementLayer::Ground));
            (walkable && empty).then_some((rx, ry))
        })
        .collect();
    let mut free = free.into_iter();
    types
        .iter()
        .map(|type_name| {
            free.by_ref()
                .find_map(|(rx, ry)| {
                    let z = sim.terrain_cell_level(rx, ry).unwrap_or(0);
                    sim.spawn_object_at_height(type_name, &owner, rx, ry, 0, z, rules)
                })
                .expect("a free cell")
        })
        .collect()
}

/// The cells within 10 of `centre` on both axes, nearest first.
pub(super) fn cells_nearest(centre: (i32, i32)) -> Vec<(i32, i32)> {
    let mut offsets: Vec<(i32, i32)> = (0..441).map(|n| (n % 21 - 10, n / 21 - 10)).collect();
    offsets.sort_by_key(|&(dx, dy)| (dx * dx + dy * dy, dy, dx));
    offsets
        .into_iter()
        .map(|(dx, dy)| (centre.0 + dx, centre.1 + dy))
        .collect()
}

/// Advances one frame with the commands now due, as `tick` does with none.
pub(super) fn tick_with_due_commands(
    scenario: &mut HeadlessScenario,
) -> crate::sim::world::SimFrameOutput {
    let commands = scenario.runtime.simulation.take_due_commands();
    scenario
        .runtime
        .advance_frame(
            &commands,
            SIM_TICK_MS,
            crate::sim::world::TickLane::Ordinary,
            crate::sim::world::FrameEffects::default(),
        )
        .expect("simulation frame failed")
}

#[cfg(feature = "npatch")]
pub(super) fn aymod_grappled_ship_at_shore() -> (HeadlessScenario, u64, u64, (u16, u16), u64) {
    use crate::rules::terrain_rules::LandType;
    let root = std::path::PathBuf::from(std::env::var_os("VERA20K_AYMOD_DIR").unwrap());
    let map = "\u{91cd}\u{8fd4}\u{8fea}\u{62dc}.yrm";
    let launch = battle_session(
        map,
        &[(LaunchCountry::from_country_index(1), AiDifficulty::Hard)],
    );
    let mut scene = load_session(&root, map, 0x00C0_FFEE, launch).unwrap();
    let (squid, ship, land) = {
        let runtime = &mut scene.runtime;
        let (sim, rules) = (&mut runtime.simulation, &runtime.resources.rules);
        let terrain = sim.resolved_terrain().unwrap();
        let free = |cell: &crate::map::resolved_terrain::ResolvedTerrainCell| {
            !cell.outside_playfield
                && !cell.bridge_facts.has_structural_bridge()
                && sim.occupancy().get(cell.rx, cell.ry).is_none()
        };
        let (water, other, land) = terrain
            .cells()
            .iter()
            .find_map(|cell| {
                let other = terrain.cell(cell.rx.checked_add(1)?, cell.ry)?;
                if ![cell, other].into_iter().all(|c| {
                    free(c) && c.land_type == LandType::Water.as_index() && c.level == cell.level
                }) {
                    return None;
                }
                let land = (-3i16..=3)
                    .flat_map(|dx| (-3i16..=3).map(move |dy| (dx, dy)))
                    .find_map(|(dx, dy)| {
                        let land = terrain.cell(
                            cell.rx.checked_add_signed(dx)?,
                            cell.ry.checked_add_signed(dy)?,
                        )?;
                        (free(land)
                            && land.land_type == LandType::Clear.as_index()
                            && land.level == cell.level)
                            .then_some((land.rx, land.ry))
                    })?;
                Some(((cell.rx, cell.ry), (other.rx, other.ry), land))
            })
            .expect("a legal clear shore beside two free water cells");
        // Player-owned host, hostile squid; the later friendly DESO can rescue it.
        let owner = house_of(sim, "Russians");
        let enemy = house_of(sim, "Americans");
        let owner_name = sim.interner.resolve(owner).to_owned();
        let enemy_name = sim.interner.resolve(enemy).to_owned();
        let squid = sim
            .spawn_object("SQD", &owner_name, other.0, other.1, 0, rules)
            .unwrap();
        let ship = sim
            .spawn_object("DEST", &enemy_name, water.0, water.1, 0, rules)
            .unwrap();
        // Execute_DoList64C4FB excludes an ordinary AI House. Let its
        // actual Guard scan acquire the player ship; do not queue fake input.
        assert!(!sim.houses.get(&owner).unwrap().event_dispatch_eligible());
        (squid, ship, land)
    };
    let mut grabbed = false;
    let held_anim = (0..240)
        .find_map(|_| {
            let output = tick_with_due_commands(&mut scene);
            for event in output.fire_events.iter().filter(|e| e.attacker_id == squid) {
                assert_eq!(scene.sim().interner.resolve(event.weapon_id), "SquidGrab");
                grabbed = true;
            }
            let sim = scene.sim();
            let state = sim.entities().get(squid)?.parasite.class()?;
            (state.victim() == Some(ship))
                .then(|| state.grapple_anim())
                .flatten()
        })
        .expect("actual SquidGrab must attach and create a live grapple animation");
    assert!(grabbed);
    assert_eq!(
        scene.sim().entities().get(ship).unwrap().parasite.eater(),
        Some(squid)
    );
    (scene, squid, ship, land, held_anim)
}

#[cfg(feature = "npatch")]
/// Visit the actual shot, restored wave, each following frame and final retirement.
/// The callback is read-only and cannot replace simulation geometry or combat.
pub fn visit_aymod_dolphin_rescue_after_load(
    transfer_host: bool,
    mut visit: impl FnMut(&str, &HeadlessScenario, u64),
) {
    use crate::sim::command::{Command, CommandEnvelope};
    use crate::sim::mission::{MissionId, MissionType};
    use crate::sim::world::bridge_test_evidence::{restored_retail, save_scene};
    let (mut scene, squid, ship, land, anim) = aymod_grappled_ship_at_shore();
    if transfer_host {
        let country = scene
            .runtime
            .resources
            .rules
            .country_index("Americans")
            .unwrap();
        let recipient = scene.add_human_house(
            "RescueRecipient",
            LaunchCountry::from_country_index(u32::from(country.0)),
        );
        // Exercise the shared FootChangeOwner4DBED0 -> Techno7014A0
        // receiver, not raw entity or parasite-link writes. Full capture
        // weapon admission is outside this ownership-lifecycle fixture.
        let runtime = &mut scene.runtime;
        // This is a normal short-game launch. Give the new participant
        // a real base so defeat cleanup cannot masquerade as release.
        spawn_on_free_cells(
            &mut runtime.simulation,
            &runtime.resources.rules,
            recipient,
            cells_nearest((i32::from(land.0), i32::from(land.1))),
            &["GAPOWR"],
        );
        runtime.simulation.change_owner_with_rules(
            ship,
            recipient,
            &runtime.resources.rules,
            Some(&runtime.resources.overlay_registry),
            Default::default(),
        );
        assert_eq!(
            runtime.simulation.entities().get(ship).unwrap().owner(),
            recipient
        );
    }
    let expected_owner = scene.sim().entities().get(ship).unwrap().owner();
    let pristine = scene.sim().resolved_terrain().unwrap().clone();
    let bytes = save_scene(&scene, "AY live squid before dolphin rescue");
    scene.runtime.simulation = restored_retail(&scene, &pristine, &bytes);
    assert_eq!(
        scene.sim().entities().get(ship).unwrap().owner(),
        expected_owner
    );
    assert_eq!(
        scene.sim().entities().get(ship).unwrap().parasite.eater(),
        Some(squid)
    );
    assert_eq!(
        scene
            .sim()
            .entities()
            .get(squid)
            .unwrap()
            .parasite
            .class()
            .unwrap()
            .victim(),
        Some(ship)
    );
    assert!(scene.sim().anim(anim).is_some());
    let dolphin = {
        let runtime = &mut scene.runtime;
        let (sim, rules) = (&mut runtime.simulation, &runtime.resources.rules);
        let host = sim.entities().get(ship).unwrap();
        let owner = host.owner();
        let at = host.position;
        let sites = cells_nearest((i32::from(at.rx()), i32::from(at.ry())))
            .into_iter()
            .filter(|&(x, y)| {
                (2..=4).contains(
                    &(x - i32::from(at.rx()))
                        .abs()
                        .max((y - i32::from(at.ry())).abs()),
                )
            });
        let dolphin = spawn_on_free_cells(sim, rules, owner, sites, &["DLPH"])[0];
        let weapon = rules
            .weapon(rules.object("DLPH").unwrap().primary().unwrap())
            .unwrap();
        assert!(
            weapon.is_sonic
                && rules
                    .warhead(weapon.warhead.as_deref().unwrap())
                    .unwrap()
                    .sonic
        );
        sim.queue_command(CommandEnvelope::new(
            owner,
            sim.session.tick() + 1,
            Command::Attack {
                attacker_id: dolphin,
                target_id: ship,
            },
        ));
        dolphin
    };
    let mut fired = false;
    let mut wave = None;
    assert!(
        (0..240).any(|_| {
            let output = tick_with_due_commands(&mut scene);
            fired |= output.fire_events.iter().any(|e| e.attacker_id == dolphin);
            wave = scene.sim().waves().iter().find_map(|(&id, w)| {
                (w.owner_id() == Some(dolphin) && w.wave_type() == 0).then_some(id)
            });
            scene
                .sim()
                .entities()
                .get(ship)
                .unwrap()
                .parasite
                .eater()
                .is_none()
        }),
        "ordinary friendly dolphin Attack must release the grabbed destroyer"
    );
    assert!(
        fired,
        "release preceded dolphin firing after transfer={transfer_host}: host={:?} squid={:?} dolphin={:?}",
        scene
            .sim()
            .entities()
            .get(ship)
            .map(|e| (&e.mission, &e.parasite, e.owner(), &e.health)),
        scene
            .sim()
            .entities()
            .get(squid)
            .map(|e| (&e.mission, &e.parasite, e.owner(), &e.health)),
        scene
            .sim()
            .entities()
            .get(dolphin)
            .map(|e| (&e.mission, &e.attack_target, e.owner()))
    );
    let wave = wave.expect("the actual shot must leave a live Sonic WaveClass");
    for id in [dolphin, ship, squid] {
        let runtime = &mut scene.runtime;
        let actor = runtime.simulation.entities().get(id).unwrap();
        assert!(actor.is_object_alive() && !actor.lifecycle.in_limbo());
        // Hold further combat through the mission owner: retirement must
        // concern this exact wave, not a replacement from another shot.
        runtime
            .simulation
            .mission_queue_exact(
                id,
                MissionId::from_known(MissionType::Sleep),
                1,
                runtime.simulation.session.binary_frame(),
                &crate::sim::mission::authority::LiveReadyInputProvider {
                    rules: &runtime.resources.rules,
                },
            )
            .unwrap();
    }
    assert_eq!(
        scene
            .sim()
            .entities()
            .get(squid)
            .unwrap()
            .parasite
            .class()
            .unwrap()
            .victim(),
        None
    );
    assert!(
        !scene
            .sim()
            .entities()
            .get(ship)
            .unwrap()
            .parasite
            .is_paralyzed(scene.sim().session.binary_frame())
    );
    // Foot ReceiveDamage4D734F..4D7371 clears the Sonic shooter's target
    // after ExitUnit. Wave762AF0 then decays from that real target loss;
    // never retain a fake target just to prolong the drawing fixture.
    assert!(
        scene
            .sim()
            .entities()
            .get(dolphin)
            .unwrap()
            .attack_target
            .target()
            .is_none()
    );
    visit("shot", &scene, wave);
    let bytes = save_scene(
        &scene,
        "AY dolphin rescue with original Sonic wave in flight",
    );
    scene.runtime.simulation = restored_retail(&scene, &pristine, &bytes);
    assert!(scene.sim().waves().iter().any(|(&id, _)| id == wave));
    visit("loaded", &scene, wave);
    let mut drew_geometry = false;
    for _ in 0..120 {
        visit("advance", &scene, wave);
        for (_, w) in scene.sim().waves().iter().filter(|(id, _)| **id == wave) {
            drew_geometry |= w.sonic_draw_geometry([0, 0]).is_some();
        }
        let output = tick_with_due_commands(&mut scene);
        assert!(
            !output
                .fire_events
                .iter()
                .any(|e| [dolphin, ship, squid].contains(&e.attacker_id))
        );
    }
    assert!(
        drew_geometry,
        "saved actual Sonic must still expose its production draw polygon"
    );
    visit("retired", &scene, wave);
    assert!(!scene.sim().waves().iter().any(|(&id, _)| id == wave));
    assert!(scene.sim().anim(anim).is_none());
    assert!(
        scene
            .sim()
            .entities()
            .get(ship)
            .unwrap()
            .parasite
            .eater()
            .is_none()
    );
    assert_eq!(
        scene
            .sim()
            .entities()
            .get(squid)
            .unwrap()
            .parasite
            .class()
            .unwrap()
            .grapple_anim(),
        None
    );
}
