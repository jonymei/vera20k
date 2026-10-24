use crate::rules::ruleset::RuleSet;
use crate::sim::{
    arena_fixture,
    projectile::{
        ProjectileCoord, ProjectileDetonation, ProjectileDetonationReason, ProjectilePayload,
        ProjectileTarget,
    },
    world::{FrameEffects, Simulation, UninitContext},
};

fn scene() -> (Simulation, RuleSet, u64, u64) {
    let ini = arena_fixture::clear_ground_ini(
        "[VehicleTypes]\n0=TTNK\n1=TANK\n2=ORGANIC\n\
         [InfantryTypes]\n0=PERSON\n[BuildingTypes]\n0=TOWER\n1=POWER\n\
         [TTNK]\nStrength=300\nPrimary=TankBolt\nSpeed=5\n\
         [TANK]\nStrength=1000\nArmor=heavy\nSpeed=5\n\
         [ORGANIC]\nStrength=100\nOrganic=yes\nSpeed=5\n\
         [PERSON]\nStrength=100\nSpeed=5\n\
         [TOWER]\nStrength=500\nArmor=concrete\nPowered=yes\nPower=-10\nPrimary=TankBolt\n\
         [POWER]\nStrength=500\nPower=200\n\
         [TankBolt]\nDamage=1\nROF=50\nRange=6\nProjectile=ElectricbounceT\nWarhead=ElectricT\n\
         [TankBoltEMP]\nDamage=3\nROF=50\nRange=1\nProjectile=InvisibleSpecial\nWarhead=EMPulsWH\n\
         [ElectricbounceT]\nInviso=yes\nImage=none\nCluster=1\nAirburst=no\nAirburstWeapon=TankBoltEMP\n\
         [InvisibleSpecial]\nInviso=yes\nImage=none\n\
         [Warheads]\n0=ElectricT\n1=EMPulsWH\n\
         [ElectricT]\nVerses=100%,100%,100%,100%,100%,100%,100%,100%,100%,100%,100%\n\
         [EMPulsWH]\nEMEffect=yes\nAffectsAllies=no\nVerses=100%,100%,100%,100%,100%,100%,100%,100%,100%,100%,100%\n",
    );
    let rules = RuleSet::from_ini(&ini).unwrap();
    let mut sim = arena_fixture::rival_arena(&rules, 31);
    sim.intern_rule_type_ids(&rules);
    sim.resolve_type_handles(&rules);
    let source = sim
        .spawn_object("TTNK", "Americans", 6, 20, 0, &rules)
        .unwrap();
    let target = sim
        .spawn_object("TANK", "Russians", 10, 20, 0, &rules)
        .unwrap();
    (sim, rules, source, target)
}

fn hit(sim: &mut Simulation, rules: &RuleSet, source: u64, target: u64, duration: i32) {
    sim.apply_npatch_emp_detonation(
        ProjectileCoord::new(2688, 5248, 0),
        Some(target),
        Some(source),
        duration,
        rules.warhead("EMPulsWH").unwrap(),
        rules,
        FrameEffects::default(),
    );
}

#[test]
fn tesla_impact_child_with_airburst_no_reaches_the_emp_receiver() {
    let (mut sim, rules, source, target) = scene();
    let payload = ProjectilePayload::new(
        1,
        sim.interner.intern("ElectricT"),
        Some(sim.interner.intern("ElectricbounceT")),
        false,
    );
    let packet_target = ProjectileTarget::Entity(target);
    let parent = sim.construct_projectile(source, packet_target, payload, &rules);
    let impact = ProjectileCoord::new(2688, 5248, 0);
    sim.projectiles
        .get_mut(parent)
        .unwrap()
        .set_position_for_test(impact);
    let request = ProjectileDetonation {
        projectile_id: parent,
        source_id: source,
        target: packet_target,
        impact,
        payload,
        reason: ProjectileDetonationReason::ReachedTarget,
    };
    let mut run = crate::sim::combat::world_receiver::ReceiverRun::default();
    crate::sim::combat::world_receiver::commit_projectiles(
        &mut sim,
        &mut run,
        &[request],
        &rules,
        None,
    );
    run.finish();
    let children: Vec<_> = sim
        .projectiles
        .iter()
        .filter(|(id, _)| **id != parent)
        .map(|(id, _)| *id)
        .collect();
    assert_eq!(
        children.len(),
        1,
        "NP impact payload is one child, not a nine-cell fan"
    );
    sim.retire_non_entity_object(parent, UninitContext::with_rules(&rules));
    for frame in 1..=5 {
        sim.session.set_binary_frame_for_test(frame);
        sim.visit_combat_tail(children[0], &rules, None, FrameEffects::default());
        sim.flush_pending_delete();
    }
    let victim = sim.entities().get(target).unwrap();
    assert!(
        victim.emp_remaining() > 0,
        "actual child detonation must stun the tank"
    );
    assert!(victim.deactivated.is_offline());
    assert!(!sim.entities().get(source).unwrap().deactivated.is_offline());
}

#[test]
fn emp_refresh_expiry_and_owner_change_keep_one_authoritative_latch() {
    let (mut sim, rules, source, target) = scene();
    hit(&mut sim, &rules, source, target, 3);
    assert_eq!(sim.entities().get(target).unwrap().emp_remaining(), 3);
    sim.session.set_binary_frame_for_test(1);
    sim.object_ai_visit_one(target, Some(&rules), Default::default());
    hit(&mut sim, &rules, source, target, 3);
    assert_eq!(
        sim.entities().get(target).unwrap().emp_remaining(),
        3,
        "refresh is not stacking"
    );
    let new_owner = sim.interner.intern("Americans");
    sim.change_owner_with_rules(target, new_owner, &rules, None, FrameEffects::default());
    sim.techno_reactivate(target, &rules, FrameEffects::default());
    assert!(
        sim.entities().get(target).unwrap().deactivated.is_offline(),
        "capture/reactivation cannot clear live EMP"
    );
    for frame in 2..=4 {
        sim.session.set_binary_frame_for_test(frame);
        sim.object_ai_visit_one(target, Some(&rules), Default::default());
    }
    let victim = sim.entities().get(target).unwrap();
    assert_eq!(victim.emp_remaining(), 0);
    assert!(!victim.deactivated.is_offline());
    assert_eq!(victim.owner(), new_owner);
}

#[test]
fn emp_spares_organic_and_allied_units_but_disables_a_powered_tower() {
    let (mut sim, rules, source, _) = scene();
    for (kind, owner, x) in [
        ("ORGANIC", "Russians", 12),
        ("PERSON", "Russians", 14),
        ("TANK", "Americans", 16),
    ] {
        let id = sim.spawn_object(kind, owner, x, 20, 0, &rules).unwrap();
        hit(&mut sim, &rules, source, id, 3);
        assert_eq!(
            sim.entities().get(id).unwrap().emp_remaining(),
            0,
            "{kind}/{owner}"
        );
    }
    sim.spawn_object("POWER", "Russians", 18, 20, 0, &rules)
        .unwrap();
    let tower = sim
        .spawn_object("TOWER", "Russians", 20, 20, 0, &rules)
        .unwrap();
    sim.entities_mut_for_test()
        .get_mut(tower)
        .unwrap()
        .mission
        .set_current_for_test(
            crate::sim::mission::MissionId::from_known(crate::sim::mission::MissionType::Guard),
            0,
        );
    crate::sim::power_system::tick_power_states(
        &mut sim.power_states,
        &mut sim.substrate.entities,
        &rules,
        &sim.interner,
        0,
    );
    assert_eq!(sim.building_operational_state(tower, &rules), Some(true));
    hit(&mut sim, &rules, source, tower, 2);
    assert!(sim.entities().get(tower).unwrap().deactivated.is_offline());
    for _ in 0..2 {
        sim.tick_ares_emp(tower, &rules, FrameEffects::default());
    }
    assert_eq!(sim.entities().get(tower).unwrap().emp_remaining(), 0);
    assert!(!sim.entities().get(tower).unwrap().deactivated.is_offline());
    hit(&mut sim, &rules, source, tower, 2);
    assert!(sim.go_building_offline(tower, &rules, FrameEffects::default()));
    for _ in 0..2 {
        sim.tick_ares_emp(tower, &rules, FrameEffects::default());
    }
    assert!(!sim.entities().get(tower).unwrap().building_online());
    assert!(sim.entities().get(tower).unwrap().deactivated.is_offline());
    assert!(sim.go_building_online(tower, &rules, FrameEffects::default()));
    sim.update_ares_emp_operability(tower, &rules, FrameEffects::default());
    assert!(!sim.entities().get(tower).unwrap().deactivated.is_offline());
    hit(&mut sim, &rules, source, tower, 2);
    sim.uninit_with_context(tower, UninitContext::with_rules(&rules));
    sim.flush_pending_delete();
    sim.tick_ares_emp(tower, &rules, FrameEffects::default());
    assert!(sim.entities().get(tower).is_none());
}
