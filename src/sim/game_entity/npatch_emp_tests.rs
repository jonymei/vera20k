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
        "[Countries]\n0=Americans\n1=Russians\n\
         [VehicleTypes]\n0=TTNK\n1=TANK\n2=ORGANIC\n\
         [InfantryTypes]\n0=PERSON\n1=CYB\n[BuildingTypes]\n0=TOWER\n1=POWER\n2=FACTORY\n\
         [TTNK]\nStrength=300\nPrimary=TankBolt\nSpeed=5\n\
         [TANK]\nStrength=1000\nArmor=heavy\nSpeed=5\nLocomotor={4A582741-9839-11D1-B709-00A024DDAFD1}\nCost=700\nTechLevel=1\nOwner=Americans,Russians\n\
         [ORGANIC]\nStrength=100\nOrganic=yes\nSpeed=5\n\
         [PERSON]\nStrength=100\nSpeed=5\n\
         [CYB]\nStrength=300\nSpeed=5\nSpeedType=Foot\nCyborg=yes\nLocomotor={4A582744-9839-11D1-B709-00A024DDAFD1}\n\
         [TOWER]\nStrength=500\nArmor=concrete\nPowered=yes\nPower=-10\nRadar=yes\nPrimary=TankBolt\n\
         [POWER]\nStrength=500\nPower=200\n\
         [FACTORY]\nStrength=1000\nPowered=yes\nPower=-10\nFactory=UnitType\nOwner=Americans,Russians\n\
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

fn paid_factory_scene() -> (Simulation, RuleSet, u64, u64) {
    let (mut sim, rules, source, _) = scene();
    sim.spawn_object("POWER", "Russians", 18, 20, 0, &rules)
        .unwrap();
    let factory = sim
        .spawn_object("FACTORY", "Russians", 20, 20, 0, &rules)
        .unwrap();
    sim.entities_mut_for_test()
        .get_mut(factory)
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
    let owner = sim.entities().get(factory).unwrap().owner();
    sim.houses
        .get_mut(&owner)
        .unwrap()
        .economy_mut()
        .set_credits_for_test(10_000);
    assert!(crate::sim::production::enqueue_by_type(
        &mut sim,
        &rules,
        "Russians",
        "TANK",
        FrameEffects::default(),
        true,
    ));
    (sim, rules, source, factory)
}

#[test]
fn emp_system_hold_preserves_paid_head_and_manual_off_until_power_restores() {
    use crate::sim::production::{ProductionCategory, step_factories};
    let (mut sim, rules, source, factory) = paid_factory_scene();
    let owner = sim.entities().get(factory).unwrap().owner();
    let category = ProductionCategory::Vehicle;
    let before = sim.production.factories().view(owner, category).unwrap();
    let head = before.object.unwrap().clone();
    let progress = before.progress;
    let wallet = sim.houses[&owner].economy().clone();
    hit(&mut sim, &rules, source, factory, 2);
    assert!(
        sim.production
            .factories()
            .view(owner, category)
            .unwrap()
            .suspended
    );
    for frame in 1..=1000 {
        sim.session.set_binary_frame_for_test(frame);
        step_factories(&mut sim);
    }
    let held = sim.production.factories().view(owner, category).unwrap();
    assert_eq!(held.object, Some(&head));
    assert_eq!(held.progress, progress);
    assert_eq!(*sim.houses[&owner].economy(), wallet);
    assert!(sim.go_building_offline(factory, &rules, FrameEffects::default()));
    for _ in 0..2 {
        sim.tick_ares_emp(factory, &rules, FrameEffects::default());
    }
    assert!(
        sim.production
            .factories()
            .view(owner, category)
            .unwrap()
            .suspended
    );
    assert!(sim.go_building_online(factory, &rules, FrameEffects::default()));
    sim.update_ares_emp_operability(factory, &rules, FrameEffects::default());
    let resumed = sim.production.factories().view(owner, category).unwrap();
    assert!(!resumed.suspended);
    assert_eq!(resumed.object, Some(&head));
    assert_eq!(resumed.progress, progress);
    hit(&mut sim, &rules, source, factory, 1);
    sim.power_states
        .map_mut_for_test()
        .get_mut(&owner)
        .unwrap()
        .set_total_output_for_test(0);
    sim.tick_ares_emp(factory, &rules, FrameEffects::default());
    assert!(
        sim.production
            .factories()
            .view(owner, category)
            .unwrap()
            .suspended
    );
    sim.power_states
        .map_mut_for_test()
        .get_mut(&owner)
        .unwrap()
        .set_total_output_for_test(200);
    sim.update_ares_emp_operability(factory, &rules, FrameEffects::default());
    let resumed = sim.production.factories().view(owner, category).unwrap();
    assert!(!resumed.suspended);
    assert_eq!(resumed.object, Some(&head));
}

#[test]
fn emp_expiry_resumes_system_hold_but_not_player_hold() {
    use crate::sim::production::{ProductionCategory, suspend_production};
    let (mut sim, rules, source, factory) = paid_factory_scene();
    let owner = sim.entities().get(factory).unwrap().owner();
    let category = ProductionCategory::Vehicle;
    let head = sim
        .production
        .factories()
        .view(owner, category)
        .unwrap()
        .object
        .unwrap()
        .clone();
    hit(&mut sim, &rules, source, factory, 1);
    assert!(
        sim.production
            .factories()
            .view(owner, category)
            .unwrap()
            .suspended
    );
    sim.tick_ares_emp(factory, &rules, FrameEffects::default());
    let resumed = sim.production.factories().view(owner, category).unwrap();
    assert!(!resumed.suspended);
    assert_eq!(resumed.object, Some(&head));
    assert!(suspend_production(
        &sim.interner,
        &mut sim.production,
        &sim.session,
        "Russians",
        category
    ));
    hit(&mut sim, &rules, source, factory, 1);
    sim.tick_ares_emp(factory, &rules, FrameEffects::default());
    assert!(
        sim.production
            .factories()
            .view(owner, category)
            .unwrap()
            .suspended
    );
}

#[cfg(not(feature = "ares"))]
#[test]
fn native_building_emp_saved_power_wait_preserves_the_paid_head_without_provider_latch() {
    use crate::sim::production::ProductionCategory;
    let (mut sim, rules, source, factory) = paid_factory_scene();
    let owner = sim.entities().get(factory).unwrap().owner();
    let category = ProductionCategory::Vehicle;
    let head = sim
        .production
        .factories()
        .view(owner, category)
        .unwrap()
        .object
        .unwrap()
        .clone();
    hit(&mut sim, &rules, source, factory, 1);
    assert!(sim.go_building_offline(factory, &rules, FrameEffects::default()));
    sim.tick_ares_emp(factory, &rules, FrameEffects::default());
    let entity = sim.entities().get(factory).unwrap();
    assert_eq!(entity.emp_remaining(), 0);
    assert!(entity.emp_factory_recovery_pending() && !entity.deactivated.is_offline());
    let bytes = crate::sim::snapshot::GameSnapshot::save(
        &sim,
        0,
        rules.simulation_config_hash(),
        "NP factory power wait",
        0,
    );
    let mut loaded = crate::sim::snapshot::GameSnapshot::load(&bytes)
        .unwrap()
        .sim;
    loaded.restore_after_snapshot_load().unwrap();
    loaded.resolve_type_handles(&rules);
    let entity = loaded.entities().get(factory).unwrap();
    assert!(!entity.building_online() && entity.emp_factory_recovery_pending());
    assert!(!entity.deactivated.is_offline());
    let held = loaded.production.factories().view(owner, category).unwrap();
    assert!(held.suspended);
    assert_eq!(held.object, Some(&head));
    assert!(loaded.go_building_online(factory, &rules, FrameEffects::default()));
    loaded.update_ares_emp_operability(factory, &rules, FrameEffects::default());
    assert!(
        !loaded
            .entities()
            .get(factory)
            .unwrap()
            .emp_factory_recovery_pending()
    );
    let resumed = loaded.production.factories().view(owner, category).unwrap();
    assert!(!resumed.suspended);
    assert_eq!(resumed.object, Some(&head));
}

#[cfg(not(feature = "ares"))]
#[test]
fn native_building_emp_radar_refresh_skips_disabled_providers_in_acquisition_order() {
    use crate::sim::mission::{MissionId, MissionType};
    let (mut sim, rules, source, _) = scene();
    sim.spawn_object("POWER", "Russians", 18, 20, 0, &rules)
        .unwrap();
    let first = sim
        .spawn_object("TOWER", "Russians", 20, 20, 0, &rules)
        .unwrap();
    let second = sim
        .spawn_object("TOWER", "Russians", 22, 20, 0, &rules)
        .unwrap();
    for id in [first, second] {
        sim.entities_mut_for_test()
            .get_mut(id)
            .unwrap()
            .mission
            .set_current_for_test(MissionId::from_known(MissionType::Guard), 0);
    }
    crate::sim::power_system::tick_power_states(
        &mut sim.power_states,
        &mut sim.substrate.entities,
        &rules,
        &sim.interner,
        0,
    );
    let owner = sim.entities().get(first).unwrap().owner();
    let available = |sim: &Simulation| {
        crate::sim::radar::has_radar_for_owner(sim, &rules, sim.interner.resolve(owner))
    };
    assert!(available(&sim));
    hit(&mut sim, &rules, source, first, 3);
    assert!(
        available(&sim),
        "fresh refresh precedes the new counter write"
    );
    hit(&mut sim, &rules, source, first, 3);
    assert!(
        available(&sim),
        "C2BCB8 rejects the EMP provider via508F08 and scans the healthy second provider"
    );
    hit(&mut sim, &rules, source, second, 3);
    hit(&mut sim, &rules, source, second, 3);
    assert!(!available(&sim), "both eligible providers now carry EMP");
}

#[test]
fn emp_does_not_hold_a_paid_head_with_another_online_producer() {
    use crate::sim::production::ProductionCategory;
    let (mut sim, rules, source, factory) = paid_factory_scene();
    let second = sim
        .spawn_object("FACTORY", "Russians", 24, 20, 0, &rules)
        .unwrap();
    sim.entities_mut_for_test()
        .get_mut(second)
        .unwrap()
        .mission
        .set_current_for_test(
            crate::sim::mission::MissionId::from_known(crate::sim::mission::MissionType::Guard),
            0,
        );
    let owner = sim.entities().get(factory).unwrap().owner();
    let head = sim
        .production
        .factories()
        .view(owner, ProductionCategory::Vehicle)
        .unwrap()
        .object
        .unwrap()
        .clone();
    hit(&mut sim, &rules, source, factory, 2);
    let running = sim
        .production
        .factories()
        .view(owner, ProductionCategory::Vehicle)
        .unwrap();
    assert!(!running.suspended);
    assert_eq!(running.object, Some(&head));
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
fn emp_refuses_drive_destination_until_expiry_without_erasing_foot_orders() {
    use crate::sim::components::NavTargetRef;
    use crate::sim::movement::track_process::TrackFamily;
    let (mut sim, rules, source, target) = scene();
    assert_eq!(
        sim.entities()
            .get(target)
            .unwrap()
            .locomotor
            .as_ref()
            .unwrap()
            .active_kind(),
        crate::rules::locomotor_type::LocomotorKind::Drive
    );
    hit(&mut sim, &rules, source, target, 3);
    let requested = NavTargetRef::cell(16, 20);
    sim.set_unit_destination(target, requested, &rules, true, FrameEffects::default());
    let entity = sim.entities().get(target).unwrap();
    assert_eq!(
        entity.navigation.nav_com(),
        Some(requested),
        "Foot4D9510 publishes before Drive Move_To can refuse"
    );
    assert!(
        entity
            .locomotor
            .as_ref()
            .unwrap()
            .track_destination(TrackFamily::Drive)
            .is_none(),
        "Drive4AFD4B must refuse Move_To while EMP is positive"
    );
    for frame in 1..=3 {
        sim.session.set_binary_frame_for_test(frame);
        sim.object_ai_visit_one(target, Some(&rules), Default::default());
    }
    assert_eq!(sim.entities().get(target).unwrap().emp_remaining(), 0);
    sim.set_unit_destination(
        target,
        NavTargetRef::cell(17, 20),
        &rules,
        true,
        FrameEffects::default(),
    );
    assert!(
        sim.entities()
            .get(target)
            .unwrap()
            .locomotor
            .as_ref()
            .unwrap()
            .track_destination(TrackFamily::Drive)
            .is_some()
    );
}

#[test]
fn emp_walk_preserves_head_but_stops_paid_motion_until_expiry() {
    use crate::sim::{
        components::DriveCoord,
        movement::{ground_pose, set_walk_destination_coord},
    };
    let (mut sim, rules, source, _) = scene();
    let target = sim
        .spawn_object("CYB", "Russians", 12, 20, 0, &rules)
        .unwrap();
    hit(&mut sim, &rules, source, target, 3);
    assert_eq!(sim.entities().get(target).unwrap().emp_remaining(), 3);
    let entity = sim.substrate.entities.get_mut(target).unwrap();
    let start = ground_pose::position_world_coord(&entity.position);
    let goal = DriveCoord {
        x: start.x + 256,
        ..start
    };
    set_walk_destination_coord(entity, goal, None);
    assert!(
        entity
            .locomotor
            .as_ref()
            .unwrap()
            .walk_destination()
            .is_none(),
        "Walk75ACBD must refuse a new destination while under EMP"
    );
    // Supply a retained head separately from Move_To; test the paid Process
    // gate, not first-hit Deactivate's command cleanup.
    let head = DriveCoord {
        x: start.x + 17,
        ..start
    };
    let loco = entity.locomotor.as_mut().unwrap();
    loco.set_walk_destination(Some(goal));
    loco.set_step_head(Some(head));
    loco.begin_walk_motion();
    entity
        .navigation
        .path_runtime_mut()
        .set_scold_latch_for_test(1);
    sim.process_ground_locomotor_stats_for_test(target, Some(&rules), None)
        .unwrap();
    let entity = sim.substrate.entities.get_mut(target).unwrap();
    assert_eq!(ground_pose::position_world_coord(&entity.position), start);
    assert_eq!(entity.locomotor.as_ref().unwrap().step_head(), Some(head));
    assert_eq!(
        entity.locomotor.as_ref().unwrap().walk_animation_moving(),
        Some(false)
    );
    assert_eq!(entity.navigation.path_runtime().scold_latch_raw(), 0);
    // The existing <17-lepton completion branch precedes the EMP query.
    entity
        .locomotor
        .as_mut()
        .unwrap()
        .set_step_head(Some(DriveCoord {
            x: start.x + 16,
            ..start
        }));
    sim.process_ground_locomotor_stats_for_test(target, Some(&rules), None)
        .unwrap();
    assert!(
        sim.entities()
            .get(target)
            .unwrap()
            .locomotor
            .as_ref()
            .unwrap()
            .step_head()
            .is_none()
    );
    for frame in 1..=3 {
        sim.session.set_binary_frame_for_test(frame);
        sim.object_ai_visit_one(target, Some(&rules), Default::default());
    }
    assert_eq!(sim.entities().get(target).unwrap().emp_remaining(), 0);
    let entity = sim.substrate.entities.get_mut(target).unwrap();
    set_walk_destination_coord(entity, goal, None);
    assert_eq!(
        entity.locomotor.as_ref().unwrap().walk_destination(),
        Some(goal)
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
    assert_eq!(victim.deactivated.is_offline(), cfg!(feature = "ares"));
    assert!(!sim.entities().get(source).unwrap().deactivated.is_offline());
}

#[test]
fn emp_refresh_expiry_and_owner_change_keep_separate_provider_and_emp_state() {
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
    assert_eq!(
        sim.entities().get(target).unwrap().deactivated.is_offline(),
        cfg!(feature = "ares")
    );
    assert_eq!(
        sim.entities().get(target).unwrap().emp_remaining(),
        3,
        "provider reactivation must not clear live EMP"
    );
    assert!(!sim.techno_player_controllable(target, &rules));
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
    // Pure NP Building B951C6 stores EMP without provider Deactivate;
    // Ares/mixed builds deliberately retain their separate lifecycle.
    assert_eq!(
        sim.entities().get(tower).unwrap().deactivated.is_offline(),
        cfg!(feature = "ares")
    );
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
    assert_eq!(
        sim.entities().get(tower).unwrap().deactivated.is_offline(),
        cfg!(feature = "ares")
    );
    assert!(sim.go_building_online(tower, &rules, FrameEffects::default()));
    sim.update_ares_emp_operability(tower, &rules, FrameEffects::default());
    assert!(!sim.entities().get(tower).unwrap().deactivated.is_offline());
    hit(&mut sim, &rules, source, tower, 2);
    sim.uninit_with_context(tower, UninitContext::with_rules(&rules));
    sim.flush_pending_delete();
    sim.tick_ares_emp(tower, &rules, FrameEffects::default());
    assert!(sim.entities().get(tower).is_none());
}

#[test]
fn snapshot_retains_emp_remaining_and_manual_power_intent_through_expiry() {
    let (mut sim, rules, source, _) = scene();
    sim.spawn_object("POWER", "Russians", 18, 20, 0, &rules)
        .unwrap();
    let online = sim
        .spawn_object("TOWER", "Russians", 20, 20, 0, &rules)
        .unwrap();
    let manual_off = sim
        .spawn_object("TOWER", "Russians", 22, 20, 0, &rules)
        .unwrap();
    for id in [online, manual_off] {
        sim.entities_mut_for_test()
            .get_mut(id)
            .unwrap()
            .mission
            .set_current_for_test(
                crate::sim::mission::MissionId::from_known(crate::sim::mission::MissionType::Guard),
                0,
            );
    }
    crate::sim::power_system::tick_power_states(
        &mut sim.power_states,
        &mut sim.substrate.entities,
        &rules,
        &sim.interner,
        0,
    );
    for id in [online, manual_off] {
        hit(&mut sim, &rules, source, id, 3);
        sim.tick_ares_emp(id, &rules, FrameEffects::default());
    }
    assert!(sim.go_building_offline(manual_off, &rules, FrameEffects::default()));
    let saved = crate::sim::snapshot::GameSnapshot::save(
        &sim,
        0,
        rules.simulation_config_hash(),
        "NP EMP manual power",
        0,
    );
    let mut loaded = crate::sim::snapshot::GameSnapshot::load(&saved)
        .unwrap()
        .sim;
    loaded.restore_after_snapshot_load().unwrap();
    loaded.resolve_type_handles(&rules);

    for id in [online, manual_off] {
        let entity = loaded.entities().get(id).unwrap();
        assert_eq!(entity.emp_remaining(), 2);
        assert_eq!(entity.building_online(), id == online);
        assert_eq!(entity.deactivated.is_offline(), cfg!(feature = "ares"));
        assert_eq!(
            entity.building_art.stuff_enabled(),
            !cfg!(feature = "ares") && id == online
        );
        assert_eq!(loaded.building_operational_state(id, &rules), Some(false));
    }
    assert!(!loaded.go_building_online(manual_off, &rules, FrameEffects::default()));
    for id in [online, manual_off] {
        loaded.tick_ares_emp(id, &rules, FrameEffects::default());
        assert_eq!(loaded.entities().get(id).unwrap().emp_remaining(), 1);
        assert_eq!(
            loaded.entities().get(id).unwrap().deactivated.is_offline(),
            cfg!(feature = "ares")
        );
        loaded.tick_ares_emp(id, &rules, FrameEffects::default());
        let entity = loaded.entities().get(id).unwrap();
        assert_eq!(entity.emp_remaining(), 0);
        assert_eq!(entity.building_online(), id == online);
        assert_eq!(
            entity.deactivated.is_offline(),
            cfg!(feature = "ares") && id == manual_off
        );
        assert_eq!(entity.building_art.stuff_enabled(), id == online);
        assert_eq!(
            loaded.building_operational_state(id, &rules),
            Some(id == online)
        );
    }
    assert!(loaded.go_building_online(manual_off, &rules, FrameEffects::default()));
    loaded.update_ares_emp_operability(manual_off, &rules, FrameEffects::default());
    let entity = loaded.entities().get(manual_off).unwrap();
    assert!(entity.building_online() && !entity.deactivated.is_offline());
    assert!(entity.building_art.stuff_enabled());
}

#[test]
fn emp_expiry_and_later_house_power_respect_the_selected_stuff_lifecycle() {
    let (mut sim, rules, source, _) = scene();
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
    let owner = sim.entities().get(tower).unwrap().owner();
    hit(&mut sim, &rules, source, tower, 1);
    assert_eq!(
        sim.entities()
            .get(tower)
            .unwrap()
            .building_art
            .stuff_enabled(),
        !cfg!(feature = "ares")
    );
    sim.power_states
        .map_mut_for_test()
        .get_mut(&owner)
        .unwrap()
        .set_total_output_for_test(0);
    sim.tick_ares_emp(tower, &rules, FrameEffects::default());
    assert_eq!(sim.entities().get(tower).unwrap().emp_remaining(), 0);
    assert_eq!(
        sim.entities().get(tower).unwrap().deactivated.is_offline(),
        cfg!(feature = "ares")
    );
    assert_eq!(
        sim.entities()
            .get(tower)
            .unwrap()
            .building_art
            .stuff_enabled(),
        !cfg!(feature = "ares")
    );
    sim.power_states
        .map_mut_for_test()
        .get_mut(&owner)
        .unwrap()
        .set_total_output_for_test(200);
    sim.update_ares_emp_operability(tower, &rules, FrameEffects::default());
    assert!(!sim.entities().get(tower).unwrap().deactivated.is_offline());
    assert!(
        sim.entities()
            .get(tower)
            .unwrap()
            .building_art
            .stuff_enabled(),
        "the shared light and powered-animation gate must recover too"
    );
}

#[cfg(not(feature = "ares"))]
fn drive_provider_scene() -> (Simulation, RuleSet, u64, u64, u64) {
    use crate::rules::{art_data::ArtRegistry, ini_parser::IniFile, sound_ini::SoundRegistry};
    let ini = arena_fixture::clear_ground_ini(
        "[Countries]\n0=Americans\n1=Russians\n[General]\nEMPulseSparkles=EMPFX\n\
         [VehicleTypes]\n0=DRON\n1=SOURCE\n[BuildingTypes]\n0=CENTER\n\
         [DRON]\nStrength=300\nSpeed=5\nSpeedType=Track\nPoweredUnit=yes\nArmor=heavy\n\
         Locomotor={4A582741-9839-11D1-B709-00A024DDAFD1}\nActivateSound=On\nDeactivateSound=Off\n\
         [SOURCE]\nStrength=100\n[CENTER]\nStrength=500\nPowersUnit=DRON\n\
         [Warheads]\n0=EMPulsWH\n[EMPulsWH]\nEMEffect=yes\nAffectsAllies=no\n\
         [Animations]\n0=EMPFX\n[CombatDamage]\nDefaultSparkSystem=SparkSys\n\
         [ParticleSystems]\n0=SparkSys\n[SparkSys]\nBehavesLike=Spark\nHoldsWhat=Spark\nLifetime=5\n\
         [Particles]\n0=Spark\n[Spark]\nBehavesLike=Spark\n",
    );
    let art = IniFile::from_str("[EMPFX]\nLoopCount=-1\nRate=1\nEnd=4\n");
    let mut rules = RuleSet::from_ini_with_fixed_art_for_test(&ini, &art).unwrap();
    let sounds = SoundRegistry::from_ini(&IniFile::from_str("[SoundList]\n0=On\n1=Off\n"));
    rules.bind_type_sound_references(&ini, &sounds);
    let mut art = ArtRegistry::from_ini(&art);
    art.bind_anim_frame_count_for_test("EMPFX", 4);
    rules.install_art_data(art);
    let mut sim = arena_fixture::rival_arena(&rules, 31);
    sim.intern_rule_type_ids(&rules);
    sim.resolve_type_handles(&rules);
    let source = sim
        .spawn_object("SOURCE", "Americans", 6, 20, 0, &rules)
        .unwrap();
    let target = sim
        .spawn_object("DRON", "Russians", 10, 20, 0, &rules)
        .unwrap();
    let center = sim
        .spawn_object("CENTER", "Russians", 20, 20, 0, &rules)
        .unwrap();
    sim.powered_center_edge(center, true, &rules, Default::default());
    sim.sound_events.take_for_test();
    (sim, rules, source, target, center)
}

#[cfg(not(feature = "ares"))]
#[test]
fn npatch_drive_hit_and_equal_rehit_preserve_provider_mission_and_selection() {
    use crate::sim::{
        components::NavTargetRef,
        mission::{MissionId, MissionType},
    };
    let (mut sim, rules, source, target, _) = drive_provider_scene();
    sim.set_unit_destination(
        target,
        NavTargetRef::cell(15, 20),
        &rules,
        true,
        Default::default(),
    );
    sim.substrate
        .entities
        .get_mut(target)
        .unwrap()
        .mission
        .set_current_for_test(MissionId::from_known(MissionType::Attack), 0);
    sim.substrate
        .entities
        .get_mut(target)
        .unwrap()
        .selection
        .select();
    let mut expected_rng = sim.scenario_rng.clone();
    expected_rng.next_range_i32_inclusive(0, 25);
    hit(&mut sim, &rules, source, target, 3);
    assert_eq!(
        sim.scenario_rng.logical_state(),
        expected_rng.logical_state()
    );
    let victim = sim.entities().get(target).unwrap();
    assert!(
        !victim.deactivated.is_offline(),
        "EMP must not acquire the provider latch"
    );
    assert!(victim.selection.is_selected());
    assert_eq!(victim.mission.current().known(), Some(MissionType::Attack));
    assert!(victim.navigation.nav_com().is_none());
    assert!(victim.locomotor.as_ref().unwrap().is_powered());
    let first = sim
        .anims()
        .filter(|(_, a)| a.owner_entity() == Some(target))
        .count();
    assert_eq!(first, 1);
    sim.set_unit_destination(
        target,
        NavTargetRef::cell(16, 20),
        &rules,
        true,
        Default::default(),
    );
    hit(&mut sim, &rules, source, target, 3);
    assert!(
        sim.entities()
            .get(target)
            .unwrap()
            .navigation
            .nav_com()
            .is_none(),
        "equal rehit clears newly published Foot orders"
    );
    assert_eq!(
        sim.anims()
            .filter(|(_, a)| a.owner_entity() == Some(target))
            .count(),
        2
    );
    assert_eq!(
        sim.sound_events.take_for_test().len(),
        1,
        "only the first online hit plays Off"
    );
    // Reuse an executed spark-frame vector to test NPAE70FBD0's EMP
    // disjunct, rather than inventing another RNG/phase implementation.
    let vectors: serde_json::Value =
        serde_json::from_str(crate::test_fixture::text("tools/robot_offline_oracle.json")).unwrap();
    let row = vectors["sparks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["name"] == "offline, a spark frame, seed 1")
        .unwrap();
    let victim = sim.substrate.entities.get_mut(target).unwrap();
    victim.position.put_world_xy([
        row["location"][0].as_i64().unwrap() as i32,
        row["location"][1].as_i64().unwrap() as i32,
    ]);
    victim
        .position
        .set_exact_z_leptons_for_test(Some(row["location"][2].as_i64().unwrap() as i32));
    victim.set_native_unique_id_for_test(row["id"].as_i64().unwrap() as i32);
    sim.session
        .set_binary_frame_for_test(row["frame"].as_u64().unwrap() as u32);
    sim.scenario_rng = crate::sim::rng::SimRng::new(row["seed"].as_u64().unwrap());
    sim.deactivated_unit_sparks(target, &rules);
    let systems: Vec<_> = sim.particle_systems().iter().collect();
    assert_eq!(
        systems.len(),
        1,
        "EMP alone must reach the native disabled spark arm"
    );
    let at = systems[0].1.coords();
    assert_eq!(
        serde_json::json!([at.x, at.y, at.z]),
        row["expected"]["systems"][0]["coords"]
    );
    let rng = sim.scenario_rng.logical_state();
    hit(&mut sim, &rules, source, target, 2);
    assert_eq!(
        sim.scenario_rng.logical_state(),
        rng,
        "weaker hit is rejected before animation allocation"
    );
    assert_eq!(
        sim.anims()
            .filter(|(_, a)| a.owner_entity() == Some(target))
            .count(),
        2
    );
    for _ in 0..3 {
        sim.tick_ares_emp(target, &rules, Default::default());
    }
    assert!(
        sim.anims()
            .filter(|(_, a)| a.owner_entity() == Some(target))
            .all(|(_, a)| a.runtime().loop_remaining == 0)
    );
}

#[cfg(not(feature = "ares"))]
#[test]
fn npatch_drive_provider_returns_during_emp_then_saved_expiry_powers_on_without_hunt() {
    use crate::sim::{components::NavTargetRef, mission::MissionType, snapshot::GameSnapshot};
    let (mut sim, rules, source, target, center) = drive_provider_scene();
    sim.powered_center_edge(center, false, &rules, Default::default());
    hit(&mut sim, &rules, source, target, 2);
    sim.set_unit_destination(
        target,
        NavTargetRef::cell(16, 20),
        &rules,
        true,
        Default::default(),
    );
    sim.powered_center_edge(center, true, &rules, Default::default());
    let victim = sim.entities().get(target).unwrap();
    assert!(
        !victim.deactivated.is_offline(),
        "provider latch clears even while EMP is positive"
    );
    assert!(victim.navigation.nav_com().is_none());
    assert!(!victim.locomotor.as_ref().unwrap().is_powered());
    let bytes = GameSnapshot::save(&sim, 0, 0, "NP Drive overlap", 0);
    sim = GameSnapshot::load(&bytes).unwrap().sim;
    sim.restore_after_snapshot_load().unwrap();
    sim.resolve_type_handles(&rules);
    sim.sound_events.take_for_test();
    for _ in 0..2 {
        sim.tick_ares_emp(target, &rules, Default::default());
    }
    let victim = sim.entities().get(target).unwrap();
    assert_eq!(victim.emp_remaining(), 0);
    assert!(!victim.deactivated.is_offline());
    assert!(victim.locomotor.as_ref().unwrap().is_powered());
    assert_ne!(victim.mission.queued().known(), Some(MissionType::Hunt));
    assert_eq!(sim.sound_events.take_for_test().len(), 1);
    assert!(
        sim.anims()
            .filter(|(_, a)| a.owner_entity() == Some(target))
            .all(|(_, a)| a.runtime().loop_remaining == 0)
    );
}

#[cfg(not(feature = "ares"))]
#[test]
fn npatch_drive_expiry_respects_provider_and_legacy_policy_transition() {
    let (mut sim, rules, source, target, center) = drive_provider_scene();
    sim.powered_center_edge(center, false, &rules, Default::default());
    hit(&mut sim, &rules, source, target, 2);
    sim.sound_events.take_for_test();
    for _ in 0..2 {
        sim.tick_ares_emp(target, &rules, Default::default());
    }
    let victim = sim.entities().get(target).unwrap();
    assert_eq!(victim.emp_remaining(), 0);
    assert!(victim.deactivated.is_offline());
    assert!(!victim.locomotor.as_ref().unwrap().is_powered());
    assert!(sim.sound_events.take_for_test().is_empty());
    // A positive-height compatibility hit followed by a ground rehit must
    // not reinterpret its synthetic offline latch as a missing provider.
    let (mut sim, rules, source, target, _) = drive_provider_scene();
    let ground = crate::sim::movement::ground_pose::object_location(
        sim.entities().get(target).unwrap(),
        sim.resolved_terrain.as_ref(),
    )
    .z;
    sim.substrate
        .entities
        .get_mut(target)
        .unwrap()
        .position
        .set_exact_z_leptons_for_test(Some(ground + 128));
    hit(&mut sim, &rules, source, target, 3);
    assert!(sim.entities().get(target).unwrap().deactivated.is_offline());
    sim.substrate
        .entities
        .get_mut(target)
        .unwrap()
        .position
        .set_exact_z_leptons_for_test(Some(ground));
    hit(&mut sim, &rules, source, target, 3);
    for _ in 0..3 {
        sim.tick_ares_emp(target, &rules, Default::default());
    }
    assert!(
        !sim.entities().get(target).unwrap().deactivated.is_offline(),
        "legacy EMP ownership must not be converted into a permanent provider latch"
    );
}
