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
fn npatch_transport_award_uses_the_shooters_controller_not_the_recipient() {
    for control_passenger in [false, true] {
        let (mut sim, rules) = fixture("");
        let controller = spawn(&mut sim, &rules, "YURI", "Americans", 10);
        let carrier = spawn(
            &mut sim,
            &rules,
            "BFRT",
            if control_passenger {
                "Americans"
            } else {
                "Russians"
            },
            12,
        );
        let passenger = spawn(
            &mut sim,
            &rules,
            "GI",
            if control_passenger {
                "Russians"
            } else {
                "Americans"
            },
            14,
        );
        assert!(sim.capture_unit(
            controller,
            if control_passenger {
                passenger
            } else {
                carrier
            },
            &rules,
            None,
            FrameEffects::default()
        ));
        let rider = sim.substrate.entities.get_mut(passenger).unwrap();
        rider.veterancy.set_elite();
        rider.passenger_role.set_inside_transport(carrier, true);
        let target = spawn(&mut sim, &rules, "TARGET", "Russians", 20);
        kill(&mut sim, &rules, passenger, target, KillCallback::Terminal);
        let diverted = crate::rules::NPATCH_ENABLED && control_passenger;
        assert_eq!(
            sim.entities()
                .get(controller)
                .unwrap()
                .veterancy
                .raw()
                .bits()
                > 0,
            diverted,
            "RecordKill C2C208 reads the original shooter's +2C0"
        );
        assert_eq!(
            sim.entities().get(carrier).unwrap().veterancy.raw().bits() > 0,
            !diverted,
            "the ordinary carrier recipient must not supply the controller"
        );
    }
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
fn garrison_award_costs_the_occupant_through_the_victims_country() {
    let (mut sim, rules) = fixture(
        "[BuildingTypes]\n0=BUNKER\n[BUNKER]\nStrength=1000\nCanBeOccupied=yes\nCanOccupyFire=yes\nMaxNumberOccupants=1\n\
         [Countries]\n0=Americans\n1=Russians\n[Americans]\nCostInfantryMult=.25\n",
    );
    let owner = sim.interner.get("Americans").unwrap();
    sim.houses
        .get_mut(&owner)
        .unwrap()
        .project_country_mults(&rules, &sim.interner);
    let bunker = spawn(&mut sim, &rules, "BUNKER", "Americans", 10);
    let occupant = spawn(&mut sim, &rules, "GI", "Americans", 12);
    sim.substrate
        .entities
        .get_mut(bunker)
        .unwrap()
        .passenger_role
        .occupants_mut()
        .add(occupant);
    sim.substrate
        .entities
        .get_mut(occupant)
        .unwrap()
        .passenger_role
        .set_inside_transport(bunker, false);
    assert!(!rules.object("BUNKER").unwrap().trainable);
    let victim = spawn(&mut sim, &rules, "TARGET", "Russians", 20);
    kill(&mut sim, &rules, bunker, victim, KillCallback::Terminal);
    //702FD4 uses the victim's full infantry cost. The killer's .25
    // factor would incorrectly make this single kill elite instead.
    assert_eq!(sim.entities().get(occupant).unwrap().veterancy.rank(), 100);
}

#[test]
fn npatch_aircraft_spawn_keeps_its_award_and_also_trains_the_parent() {
    let (mut sim, rules) = fixture(
        "[AircraftTypes]\n0=CHILD\n[CHILD]\nStrength=100\nCost=100\nTrainable=yes\nSpawned=yes\n\
         [BFRT]\nCost=400\n[Countries]\n0=Americans\n1=Russians\n\
         [Americans]\nCostUnitsMult=.25\nCostAircraftMult=.5\n",
    );
    let owner = sim.interner.get("Americans").unwrap();
    sim.houses
        .get_mut(&owner)
        .unwrap()
        .project_country_mults(&rules, &sim.interner);
    let parent = spawn(&mut sim, &rules, "BFRT", "Americans", 10);
    let child = spawn(&mut sim, &rules, "CHILD", "Americans", 12);
    sim.substrate
        .entities
        .get_mut(child)
        .unwrap()
        .spawn_owner_id
        .set_for_test(Some(parent));
    let victim = spawn(&mut sim, &rules, "TARGET", "Russians", 20);
    kill(&mut sim, &rules, child, victim, KillCallback::Terminal);
    let child_xp = sim.entities().get(child).unwrap().veterancy.raw();
    let parent_xp = sim.entities().get(parent).unwrap().veterancy.raw();
    assert!(
        child_xp.bits() > 0,
        "the child's own award must not be redirected"
    );
    assert_eq!(parent_xp.bits() > 0, crate::rules::NPATCH_ENABLED);
    if crate::rules::NPATCH_ENABLED {
        assert!(
            child_xp.bits() > parent_xp.bits(),
            "costlier parent earns its own smaller award"
        );
        assert_eq!(
            sim.entities().get(parent).unwrap().veterancy.rank(),
            100,
            "recipient costs use the killer/parent house, not the victim's country"
        );
    }
}
