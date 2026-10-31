use super::battle_test_evidence::{aymod_anchor, house_of, tick_with_due_commands};
use super::*;
use crate::map::entities::EntityCategory;
use crate::sim::combat::TargetKind;
use crate::sim::world::bridge_test_evidence::{restored_retail, save_scene};

#[test]
#[ignore = "requires VERA20K_AYMOD_DIR; natural AI navy on Isle of War reaches a real human shipyard"]
fn aymod_natural_navy_leaves_port_and_damages_shipyard() {
    let root = std::path::PathBuf::from(std::env::var_os("VERA20K_AYMOD_DIR").unwrap());
    let map = "XMP02T2.MAP";
    let mut launch = battle_session(
        map,
        &[(LaunchCountry::from_country_index(0), AiDifficulty::Hard)],
    );
    launch.local_slot_mut().country = LaunchCountry::from_country_index(1);
    launch.options.unit_count = 0;
    launch.options.short_game = false;
    let mut scene = load_session(&root, map, 0x00C0_FFEE, launch).unwrap();
    let ai = house_of(scene.sim(), "Americans");
    let human = scene.sim().session.current_house().unwrap();
    let human_at = aymod_anchor(scene.sim(), human);
    let origin = aymod_anchor(scene.sim(), ai);
    // Supply a real unarmed enemy target, not AI facilities, members, money,
    // team requests or tactical orders. Placement still uses normal admission.
    let target = {
        let runtime = &mut scene.runtime;
        let (sim, rules) = (&mut runtime.simulation, &runtime.resources.rules);
        let mut water = sim
            .resolved_terrain()
            .unwrap()
            .cells()
            .iter()
            .filter(|cell| {
                !cell.outside_playfield
                    && cell.land_type == crate::rules::terrain_rules::LandType::Water.as_index()
            })
            .map(|cell| (cell.rx, cell.ry))
            .collect::<Vec<_>>();
        water.sort_by_key(|&(x, y)| {
            (i32::from(x) - human_at.0).pow(2) + (i32::from(y) - human_at.1).pow(2)
        });
        let name = sim.interner.resolve(human).to_owned();
        water
            .into_iter()
            .find_map(|(x, y)| sim.spawn_object("NAYARD", &name, x, y, 0, rules))
            .expect("legal human shipyard target in actual water")
    };
    let hp = scene.sim().entities().get(target).unwrap().health.current();
    let pristine = scene.sim().resolved_terrain().unwrap().clone();
    let mut balances = std::collections::BTreeMap::<u64, i32>::new();
    let mut paid = std::collections::BTreeSet::new();
    let mut births = std::collections::BTreeMap::new();
    let mut traveling_team = None;
    let mut foreign_shots = Vec::new();
    let mut fired = None;
    let reached = (0..24000).any(|_| {
        let output = tick_with_due_commands(&mut scene);
        let sim = scene.sim();
        for factory in sim
            .production()
            .factories()
            .iter_insertion_ordered()
            .into_iter()
            .filter(|factory| factory.owner() == ai)
        {
            if let Some(id) = factory.object().and_then(|object| object.entity_id) {
                let before = balances.entry(id).or_insert(factory.balance());
                if factory.balance() < *before {
                    paid.insert(id);
                }
                *before = factory.balance();
            }
        }
        for actor in sim.entities().values().filter(|actor| {
            actor.owner() == ai && paid.contains(&actor.stable_id()) && !actor.lifecycle.in_limbo()
        }) {
            births
                .entry(actor.stable_id())
                .or_insert((actor.position.rx(), actor.position.ry()));
        }
        let traveling = sim.entities().values().find_map(|actor| {
            let &(x, y) = births.get(&actor.stable_id())?;
            let (team, _) = sim.team_script_vm.team_for_member(actor.stable_id())?;
            let definition = scene
                .runtime
                .resources
                .rules
                .object(sim.interner.resolve(actor.type_ref()))?;
            let target_at = sim.entities().get(target)?.position;
            (definition.naval
                && sim.team_script_vm.team(team)?.formed()
                && actor
                    .position
                    .rx()
                    .abs_diff(x)
                    .max(actor.position.ry().abs_diff(y))
                    > 6
                && actor
                    .position
                    .rx()
                    .abs_diff(target_at.rx())
                    .max(actor.position.ry().abs_diff(target_at.ry()))
                    > 12
                && crate::sim::movement::motion_query::is_moving(actor) == Some(true))
            .then_some(team)
        });
        for event in output.fire_events {
            if event.target != TargetKind::Entity(target) {
                continue;
            }
            let Some(actor) = sim.entities().get(event.attacker_id) else {
                continue;
            };
            let object = scene
                .runtime
                .resources
                .rules
                .object(sim.interner.resolve(actor.type_ref()))
                .unwrap();
            if actor.owner() == ai
                && actor.category() == EntityCategory::Unit
                && object.naval
                && paid.contains(&event.attacker_id)
            {
                fired = Some((
                    event.attacker_id,
                    actor.position.rx(),
                    actor.position.ry(),
                    sim.team_script_vm.team_for_member(event.attacker_id),
                ));
            } else {
                foreign_shots.push(sim.interner.resolve(actor.type_ref()).to_owned());
            }
        }
        if traveling_team.is_none()
            && let Some(team) = traveling
        {
            let members = scene
                .sim()
                .team_script_vm
                .team(team)
                .unwrap()
                .members()
                .collect::<Vec<_>>();
            let bytes = save_scene(
                &scene,
                "AY naturally paid navy traveling toward human shipyard",
            );
            scene.runtime.simulation = restored_retail(&scene, &pristine, &bytes);
            assert_eq!(
                scene
                    .sim()
                    .team_script_vm
                    .team(team)
                    .unwrap()
                    .members()
                    .collect::<Vec<_>>(),
                members
            );
            traveling_team = Some(team);
        }
        fired.is_some()
            && scene
                .sim()
                .entities()
                .get(target)
                .is_none_or(|entity| entity.health.current() < hp)
    });
    assert!(
        reached,
        "natural navy did not reach the real shipyard: origin={origin:?}, target={:?}, fired={fired:?}, paid={}, AI entities={:?}, teams={:?}",
        scene
            .sim()
            .entities()
            .get(target)
            .map(|entity| (entity.position, entity.health.current())),
        paid.len(),
        scene
            .sim()
            .entities()
            .values()
            .filter(|entity| entity.owner() == ai
                && entity.is_ai_alive()
                && !entity.lifecycle.in_limbo())
            .map(|entity| scene.sim().interner.resolve(entity.type_ref()))
            .collect::<Vec<_>>(),
        scene
            .sim()
            .team_script_vm
            .teams_in_order()
            .filter(|team| team.owner() == ai)
            .map(|team| (
                team.team_type_id()
                    .map(|id| scene.sim().interner.resolve(id)),
                team.formed(),
                team.cursor(),
                team.members().count()
            ))
            .collect::<Vec<_>>()
    );
    let (_, x, y, team) = fired.unwrap();
    assert!(
        foreign_shots.is_empty(),
        "non-naval fire would confound target damage: {foreign_shots:?}"
    );
    assert_eq!(
        team.map(|(id, _)| id),
        traveling_team,
        "the originally traveling saved team must reach combat"
    );
    assert!(
        team.is_some(),
        "the paid navy shooter must belong to an actual autonomous team"
    );
    assert!(
        (i32::from(x) - origin.0)
            .abs()
            .max((i32::from(y) - origin.1).abs())
            > 10,
        "the navy must leave its original base rather than shoot at a supplied adjacent target"
    );
    let (actor, _, _, _) = fired.unwrap();
    let born = births[&actor];
    assert!(
        x.abs_diff(born.0).max(y.abs_diff(born.1)) > 6,
        "the actual shooter must travel away from its own port exit"
    );
    let remaining = scene.sim().entities().get(target).unwrap().health.current();
    assert!(remaining > 0 && remaining < hp);
    let bytes = save_scene(
        &scene,
        "AY natural navy reached combat and damaged the real shipyard",
    );
    scene.runtime.simulation = restored_retail(&scene, &pristine, &bytes);
    let mut new_fire = false;
    assert!(
        (0..1800).any(|_| {
            let output = tick_with_due_commands(&mut scene);
            for event in output
                .fire_events
                .iter()
                .filter(|event| event.target == TargetKind::Entity(target))
            {
                let source = scene.sim().entities().get(event.attacker_id).unwrap();
                let kind = scene
                    .runtime
                    .resources
                    .rules
                    .object(scene.sim().interner.resolve(source.type_ref()))
                    .unwrap();
                assert!(
                    source.owner() == ai && kind.naval && paid.contains(&event.attacker_id),
                    "non-naval fire after restore would confound resumed damage"
                );
            }
            new_fire |= output.fire_events.iter().any(|event| {
                event.target == TargetKind::Entity(target)
                    && paid.contains(&event.attacker_id)
                    && scene
                        .sim()
                        .team_script_vm
                        .team_for_member(event.attacker_id)
                        .map(|(id, _)| id)
                        == traveling_team
            });
            new_fire
                && scene
                    .sim()
                    .entities()
                    .get(target)
                    .is_none_or(|entity| entity.health.current() < remaining)
        }),
        "the original saved fleet must resume firing and inflict new damage after combat restore"
    );
    let team = scene
        .sim()
        .team_script_vm
        .team(traveling_team.unwrap())
        .unwrap();
    eprintln!(
        "natural AY navy: TeamType={}, shooter={}, tick={}, target HP {} -> {} -> {:?}; two full restores",
        scene.sim().interner.resolve(team.team_type_id().unwrap()),
        scene
            .sim()
            .interner
            .resolve(scene.sim().entities().get(actor).unwrap().type_ref()),
        scene.sim().session.tick(),
        hp,
        remaining,
        scene
            .sim()
            .entities()
            .get(target)
            .map(|entity| entity.health.current())
    );
}
