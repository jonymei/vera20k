use super::*;
use crate::rules::ini_parser::IniFile;
use crate::sim::house_state::HouseState;
use crate::sim::world::{FrameEffects, Simulation, UninitContext};

fn fixture(extra: &str) -> (Simulation, RuleSet) {
    let mut ini = IniFile::from_str(
        "[General]\nVeteranRatio=3\nMindControllerGainExperienceRatio=100%\n\
         [InfantryTypes]\n0=GI\n1=YURI\n\
         [VehicleTypes]\n0=BFRT\n1=TARGET\n\
         [Warheads]\n0=MC\n[MC]\nMindControl=yes\n\
         [Control]\nDamage=1\nWarhead=MC\nRange=8\nROF=30\n\
         [GI]\nStrength=100\nCost=100\nTrainable=yes\nGetsBounty=yes\n\
         [YURI]\nStrength=100\nCost=100\nTrainable=yes\nPrimary=Control\n\
         [BFRT]\nStrength=500\nCost=100\nTrainable=yes\nOpenTopped=yes\nTrainPassengers=yes\n\
         [TARGET]\nStrength=100\nCost=400\nTrainable=yes\n",
    );
    ini.merge(&IniFile::from_str(extra));
    let rules = RuleSet::from_ini(&ini).unwrap();
    let mut sim = Simulation::new();
    for (name, human) in [("Americans", true), ("Russians", false)] {
        let owner = sim.interner.intern(name);
        sim.houses
            .register(HouseState::new(owner, 0, None, human, 0, 10));
    }
    (sim, rules)
}

fn spawn(sim: &mut Simulation, rules: &RuleSet, kind: &str, owner: &str, x: u16) -> u64 {
    sim.spawn_object_at_height(kind, owner, x, 10, 0, 0, rules)
        .unwrap()
}

fn kill(sim: &mut Simulation, rules: &RuleSet, killer: u64, victim: u64, callback: KillCallback) {
    sim.record_the_kill(
        victim,
        KillSource::Techno(Some(killer)),
        callback,
        UninitContext::with_rules(rules),
    );
}

#[test]
fn npatch_passenger_trains_before_carrier_and_elite_passes_credit_back() {
    let (mut sim, rules) = fixture("");
    let carrier = spawn(&mut sim, &rules, "BFRT", "Americans", 10);
    let passenger = spawn(&mut sim, &rules, "GI", "Americans", 12);
    let target = spawn(&mut sim, &rules, "TARGET", "Russians", 20);
    sim.substrate
        .entities
        .get_mut(passenger)
        .unwrap()
        .passenger_role
        .set_inside_transport(carrier, true);
    kill(&mut sim, &rules, passenger, target, KillCallback::Terminal);
    let trained = sim
        .substrate
        .entities
        .get(passenger)
        .unwrap()
        .veterancy
        .rank();
    assert_eq!(trained > 0, crate::rules::NPATCH_ENABLED);
    let carrier_rank = sim
        .substrate
        .entities
        .get(carrier)
        .unwrap()
        .veterancy
        .rank();
    assert_eq!(carrier_rank > 0, !crate::rules::NPATCH_ENABLED);
    sim.substrate
        .entities
        .get_mut(passenger)
        .unwrap()
        .veterancy
        .set_elite();
    let target = spawn(&mut sim, &rules, "TARGET", "Russians", 22);
    kill(&mut sim, &rules, passenger, target, KillCallback::Terminal);
    assert!(
        sim.substrate
            .entities
            .get(carrier)
            .unwrap()
            .veterancy
            .rank()
            > 0
    );
}

#[test]
fn npatch_bounty_only_pays_hostile_terminal_kill_once() {
    let (mut sim, rules) = fixture("[CombatDamage]\nBountyCostMult=.25\n");
    let killer = spawn(&mut sim, &rules, "GI", "Americans", 10);
    let victim = spawn(&mut sim, &rules, "TARGET", "Russians", 20);
    let owner = sim.interner.get("Americans").unwrap();
    let money = sim.houses[&owner].economy().available_money();
    kill(&mut sim, &rules, killer, victim, KillCallback::OwnerChange);
    assert_eq!(sim.houses[&owner].economy().available_money(), money);
    // A live owner-change callback books vanilla statistics; a fresh victim
    // represents the actual terminal event without reusing that accounting bit.
    let victim = spawn(&mut sim, &rules, "TARGET", "Russians", 22);
    kill(&mut sim, &rules, killer, victim, KillCallback::Terminal);
    let expected = money + if crate::rules::NPATCH_ENABLED { 100 } else { 0 };
    assert_eq!(sim.houses[&owner].economy().available_money(), expected);
    kill(&mut sim, &rules, killer, victim, KillCallback::Terminal);
    assert_eq!(sim.houses[&owner].economy().available_money(), expected);
    let friendly = spawn(&mut sim, &rules, "TARGET", "Americans", 24);
    kill(&mut sim, &rules, killer, friendly, KillCallback::Terminal);
    assert_eq!(sim.houses[&owner].economy().available_money(), expected);
}

#[test]
fn npatch_controller_receives_authored_share_and_elite_controller_does_not_waste_it() {
    let (mut sim, rules) = fixture("");
    let controller = spawn(&mut sim, &rules, "YURI", "Americans", 10);
    let controlled = spawn(&mut sim, &rules, "GI", "Russians", 12);
    assert!(sim.capture_unit(
        controller,
        controlled,
        &rules,
        None,
        FrameEffects::default()
    ));
    let victim = spawn(&mut sim, &rules, "TARGET", "Russians", 20);
    kill(&mut sim, &rules, controlled, victim, KillCallback::Terminal);
    assert_eq!(
        sim.substrate
            .entities
            .get(controller)
            .unwrap()
            .veterancy
            .rank()
            > 0,
        crate::rules::NPATCH_ENABLED
    );
    assert_eq!(
        sim.substrate
            .entities
            .get(controlled)
            .unwrap()
            .veterancy
            .rank()
            > 0,
        !crate::rules::NPATCH_ENABLED
    );
    sim.substrate
        .entities
        .get_mut(controller)
        .unwrap()
        .veterancy
        .set_elite();
    let victim = spawn(&mut sim, &rules, "TARGET", "Russians", 22);
    kill(&mut sim, &rules, controlled, victim, KillCallback::Terminal);
    assert!(
        sim.substrate
            .entities
            .get(controlled)
            .unwrap()
            .veterancy
            .rank()
            > 0
    );
}

#[test]
fn npatch_zero_cost_controller_does_not_swallow_experience() {
    let (mut sim, rules) = fixture("[YURI]\nCost=0\n");
    let controller = spawn(&mut sim, &rules, "YURI", "Americans", 10);
    let controlled = spawn(&mut sim, &rules, "GI", "Russians", 12);
    assert!(sim.capture_unit(
        controller,
        controlled,
        &rules,
        None,
        FrameEffects::default()
    ));
    let victim = spawn(&mut sim, &rules, "TARGET", "Russians", 20);
    kill(&mut sim, &rules, controlled, victim, KillCallback::Terminal);
    assert!(
        sim.substrate
            .entities
            .get(controlled)
            .unwrap()
            .veterancy
            .rank()
            > 0
    );
}

#[test]
fn npatch_trainable_spawn_pays_parent_and_synchronizes_live_children() {
    use crate::sim::spawn_manager::{SpawnManagerState, SpawnSlot, SpawnSlotState};
    use crate::sim::timer::CdTimer;
    let (mut sim, rules) = fixture("[GI]\nSpawned=yes\nMissileSpawn=yes\n");
    let parent = spawn(&mut sim, &rules, "BFRT", "Americans", 10);
    let child = spawn(&mut sim, &rules, "GI", "Americans", 12);
    let sibling = spawn(&mut sim, &rules, "GI", "Americans", 14);
    let child_type = sim.interner.get("GI").unwrap();
    sim.substrate
        .entities
        .get_mut(child)
        .unwrap()
        .spawn_owner_id
        .set_for_test(Some(parent));
    sim.substrate
        .entities
        .get_mut(parent)
        .unwrap()
        .spawn_manager = Some(SpawnManagerState::for_test(
        child_type,
        vec![
            SpawnSlot::for_test(
                Some(child),
                SpawnSlotState::ReadyDocked,
                CdTimer::default(),
                true,
            ),
            SpawnSlot::for_test(
                Some(sibling),
                SpawnSlotState::ReadyDocked,
                CdTimer::default(),
                true,
            ),
        ],
    ));
    let victim = spawn(&mut sim, &rules, "TARGET", "Russians", 20);
    kill(&mut sim, &rules, child, victim, KillCallback::Terminal);
    let rank = sim.entities().get(parent).unwrap().veterancy.rank();
    assert_eq!(rank > 0, crate::rules::NPATCH_ENABLED);
    if crate::rules::NPATCH_ENABLED {
        assert_eq!(sim.entities().get(child).unwrap().veterancy.rank(), rank);
        assert_eq!(sim.entities().get(sibling).unwrap().veterancy.rank(), rank);
    }
}
