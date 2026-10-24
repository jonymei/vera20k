//! NPatch manual power input; the simulation's BuildingOnline latch is authoritative.

use crate::app::AppState;
use crate::map::entities::EntityCategory;
use crate::rules::ini_parser::IniFile;
use crate::rules::ruleset::RuleSet;
use crate::sim::command::Command;
use crate::sim::intern::InternedId;
use crate::sim::world::Simulation;
use crate::ui::sidebar::gadget_flash::SelectionMode;

// NPExt174 ReadBool5295F0 at C2A362..C2A377 reads UseTogglePower.
// Policy: use the frozen startup UISettings root, false when absent; never
// reopen a file or let a map silently enable a process UI capability.
fn enabled_in_root(root: &IniFile) -> bool {
    crate::rules::NPATCH_ENABLED
        && root
            .section_or_empty("UISettings")
            .read_bool("UseTogglePower", false)
}

fn enabled(state: &AppState) -> bool {
    state
        .process_assets
        .native_rules()
        .is_some_and(|owner| enabled_in_root(owner.selected_rules_root()))
}

pub(super) fn toggle_mode(state: &mut AppState) {
    if !enabled(state) || state.match_state.input.planning.active() {
        return;
    }
    let owned_buildings = state
        .match_state
        .sim_runtime
        .as_ref()
        .and_then(|runtime| {
            let sim = &runtime.simulation;
            sim.houses().get(&sim.session().current_house()?)
        })
        .map_or(0, |house| house.tracking().buildings());
    let placement = state.armed_building_type().is_some();
    if state
        .match_state
        .match_presentation
        .sidebar_gadget_state
        .toggle_selection_mode(SelectionMode::Power, placement, owned_buildings)
    {
        state.match_state.input.targeting_mode = None;
        crate::app::input::dispatch::unselect_all(state);
    }
}

/// Shared hover/release admission; reject enemies, dead/limbo objects and
/// non-toggleable/non-powered types before queuing the existing event.
fn command_for_target(
    sim: &Simulation,
    rules: &RuleSet,
    owner: InternedId,
    id: u64,
) -> Option<Command> {
    let entity = sim.entities().get(id)?;
    if entity.owner() != owner
        || entity.category() != EntityCategory::Structure
        || !entity.lifecycle.object_alive()
        || entity.lifecycle.in_limbo()
    {
        return None;
    }
    let object = rules.object(sim.interner().resolve(entity.type_ref()))?;
    let on = !entity.building_online();
    if !object.toggle_power || object.firestorm_wall || !sim.can_set_building_online(id, on, rules)
    {
        return None;
    }
    Some(Command::SetBuildingOnLine { entity_id: id, on })
}

pub(super) fn command_at_cursor(state: &AppState) -> Option<Command> {
    if !enabled(state) {
        return None;
    }
    let runtime = state.match_state.sim_runtime.as_ref()?;
    let sim = &runtime.simulation;
    let owner = super::commands::preferred_local_owner(state)?;
    let point = crate::app::match_runtime::sim_tick::screen_point_to_world(
        state,
        state.match_state.input.cursor_x,
        state.match_state.input.cursor_y,
    );
    let target = super::commands::visible_object_under_point(state, point.0, point.1)?;
    command_for_target(
        sim,
        &runtime.resources.rules,
        sim.interner().get(&owner)?,
        target,
    )
}

pub(super) fn click(state: &mut AppState) {
    if let Some(command) = command_at_cursor(state)
        && let Some(owner) = super::commands::preferred_local_owner(state)
    {
        super::commands::schedule_command(state, &owner, command);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::input::hotkeys::{HotkeyBindings, HotkeyCommand};
    use crate::app::input::state::{MatchInputState, TacticalPointerMode};
    use crate::app::types::CursorFeedbackKind;
    use crate::render::cursor_atlas::CursorId;
    use crate::sim::house_state::HouseState;
    use crate::ui::sidebar::gadget_flash::SidebarGadgetState;

    #[test]
    fn manual_power_reads_root_and_preserves_hotkey_choices() {
        assert!(!enabled_in_root(&IniFile::from_str(
            "[General]\nUseTogglePower=yes\n"
        )));
        assert_eq!(
            enabled_in_root(&IniFile::from_str("[UISettings]\nUseTogglePower=yes\n")),
            crate::rules::NPATCH_ENABLED
        );
        if !crate::rules::NPATCH_ENABLED {
            return;
        }
        let mut keys = HotkeyBindings::from_ini_bytes(None);
        assert_eq!(keys.command_at(86), Some(HotkeyCommand::TogglePower));
        assert_eq!(
            keys.resolve_event(
                &winit::keyboard::Key::Character("v".into()),
                winit::keyboard::KeyLocation::Standard,
                winit::keyboard::ModifiersState::empty(),
            ),
            crate::app::input::hotkeys::HotkeyResolution::Command(HotkeyCommand::TogglePower),
        );
        keys.assign(HotkeyCommand::TogglePower, 65).unwrap();
        let saved = keys.to_ini_string();
        assert_eq!(
            HotkeyBindings::reload_from_ini_bytes(Some(saved.as_bytes())).command_at(65),
            Some(HotkeyCommand::TogglePower)
        );
        keys.assign(HotkeyCommand::TogglePower, 0).unwrap();
        let saved = keys.to_ini_string();
        assert_eq!(
            HotkeyBindings::reload_from_ini_bytes(Some(saved.as_bytes()))
                .first_key(HotkeyCommand::TogglePower),
            None
        );
        let occupied = HotkeyBindings::from_ini_bytes(Some(b"[Hotkey]\nGuardObject=86\n"));
        assert_eq!(occupied.command_at(86), Some(HotkeyCommand::GuardObject));
    }

    #[test]
    fn manual_power_mode_uses_power_cursor_and_existing_cancel_owner() {
        let mut sidebar = SidebarGadgetState::default();
        let mut input = MatchInputState::new(HotkeyBindings::default());
        sidebar.repair_mode_on = true;
        sidebar.place_beacon_mode_on = true;
        assert!(sidebar.toggle_selection_mode(SelectionMode::Power, false, 1));
        assert!(!sidebar.repair_mode_on && !sidebar.sell_mode_on && !sidebar.place_beacon_mode_on);
        assert_eq!(
            input.tactical_pointer_mode(&sidebar, true),
            TacticalPointerMode::Power
        );
        assert_eq!(
            super::super::cursor::cursor_id_for_feedback(CursorFeedbackKind::PowerMode(true)),
            Some(CursorId::TogglePower)
        );
        assert_eq!(
            super::super::cursor::cursor_id_for_feedback(CursorFeedbackKind::PowerMode(false)),
            Some(CursorId::NoTogglePower)
        );
        assert!(input.cancel_pointer_mode(&mut sidebar));
        assert!(!sidebar.power_mode_on);
        assert!(sidebar.toggle_selection_mode(SelectionMode::Power, false, 1));
        assert!(sidebar.toggle_selection_mode(SelectionMode::Sell, false, 1));
        assert!(!sidebar.power_mode_on && sidebar.sell_mode_on);
        assert!(sidebar.toggle_selection_mode(SelectionMode::Power, false, 1));
        crate::app::presentation::sidebar_gadgets::exit_sell_mode_without_buildings(
            &mut sidebar,
            &Simulation::new(),
            None,
            false,
        );
        assert!(
            !sidebar.power_mode_on,
            "losing the local house retires the mode"
        );
    }

    #[test]
    fn fresh_world_reset_retires_power_even_when_the_incoming_house_has_buildings() {
        let mut sim = Simulation::new();
        let owner = sim.intern("Local");
        let mut house = HouseState::new(owner, 0, None, true, 1000, 10);
        house.tracking_mut_for_test().set_buildings_for_test(1);
        sim.houses_mut_for_test().register(house);
        let mut sidebar = SidebarGadgetState::default();
        sidebar.command_bar_closed = true;
        assert!(sidebar.toggle_selection_mode(SelectionMode::Power, false, 1));
        let mut input = MatchInputState::new(HotkeyBindings::default());

        // The common fresh/saved world-install boundary clears outgoing modes
        // before saved routes can restore their supported incoming local view.
        input.reset_targeting_for_new_world(&mut sidebar);
        crate::app::presentation::sidebar_gadgets::exit_sell_mode_without_buildings(
            &mut sidebar,
            &sim,
            Some(owner),
            false,
        );

        assert!(
            !sidebar.power_mode_on
                && !sidebar.repair_mode_on
                && !sidebar.sell_mode_on
                && !sidebar.place_beacon_mode_on
        );
        assert_eq!(
            input.tactical_pointer_mode(&sidebar, true),
            TacticalPointerMode::Ordinary
        );
        assert!(
            sidebar.command_bar_closed,
            "world replacement must retain display preferences"
        );
    }

    #[test]
    fn manual_power_commands_reuse_online_latch_and_reassess_drain() {
        let rules = RuleSet::from_ini(&IniFile::from_str(
            "[BuildingTypes]\n0=RADAR\n1=LOCKED\n2=PLANT\n\
             [RADAR]\nStrength=100\nPower=-50\nPowered=yes\nTogglePower=yes\n\
             [LOCKED]\nStrength=100\nPower=-10\nTogglePower=no\n\
             [PLANT]\nStrength=100\nPower=100\nTogglePower=yes\n",
        ))
        .unwrap();
        let mut sim = Simulation::new();
        let owner = sim.intern("Local");
        let enemy = sim.intern("Other");
        for (i, house) in [owner, enemy].into_iter().enumerate() {
            sim.houses_mut_for_test().register(HouseState::new(
                house,
                i as u8,
                None,
                i == 0,
                1000,
                10,
            ));
        }
        sim.session_mut_for_test()
            .set_current_house_for_test(Some(owner));
        let radar = sim.spawn_object("RADAR", "Local", 2, 2, 0, &rules).unwrap();
        let locked = sim
            .spawn_object("LOCKED", "Local", 4, 2, 0, &rules)
            .unwrap();
        let plant = sim.spawn_object("PLANT", "Local", 6, 2, 0, &rules).unwrap();
        let foreign = sim.spawn_object("RADAR", "Other", 8, 2, 0, &rules).unwrap();
        assert!(command_for_target(&sim, &rules, owner, foreign).is_none());
        assert!(command_for_target(&sim, &rules, owner, locked).is_none());
        assert!(command_for_target(&sim, &rules, owner, plant).is_none());
        sim.assess_power_for_test(&rules);
        let before = sim.power_states()[&owner].total_drain();
        let off = command_for_target(&sim, &rules, owner, radar).unwrap();
        super::super::commands::schedule_command_in_sim(&mut sim, "Local", off.clone()).unwrap();
        assert_eq!(sim.pending_command_snapshot()[0].payload, off);
        let issued = super::super::commands::encode_ordinary_local_command(
            &sim,
            crate::sim::command::CommandEnvelope::new(owner, sim.session().tick(), off.clone()),
        )
        .unwrap();
        let Command::SynchronizedRecord { record } = issued.payload else {
            panic!("manual power must use the existing native command transport");
        };
        assert_eq!(
            record.opcode(),
            crate::sim::command::BUILDING_OFF_LINE_OPCODE
        );
        let decoded = sim
            .decode_native_command_record(&record, issued.execute_tick)
            .unwrap();
        assert_eq!(decoded.payload, off);
        let mut runtime = crate::sim::runtime::SimRuntime {
            simulation: sim,
            resources: crate::sim::runtime::SimResources::new(rules.clone()),
        };
        runtime
            .advance_frame(
                &[],
                crate::headless_scenario::SIM_TICK_MS,
                crate::sim::world::TickLane::Ordinary,
                crate::sim::world::FrameEffects::default(),
            )
            .expect("ordinary frame executes the power record");
        let mut sim = runtime.simulation;
        assert!(!sim.entities().get(radar).unwrap().building_online());
        sim.assess_power_for_test(&rules);
        assert_eq!(sim.power_states()[&owner].total_drain(), before - 50);
        let on = command_for_target(&sim, &rules, owner, radar).unwrap();
        assert!(sim.apply_command("Local", &on, Some(&rules)));
        assert!(sim.entities().get(radar).unwrap().building_online());
        sim.assess_power_for_test(&rules);
        assert_eq!(sim.power_states()[&owner].total_drain(), before);
        sim.entities_mut_for_test()
            .get_mut(radar)
            .unwrap()
            .set_owner_for_test(enemy);
        assert!(command_for_target(&sim, &rules, owner, radar).is_none());
        sim.entities_mut_for_test()
            .get_mut(radar)
            .unwrap()
            .set_owner_for_test(owner);
        sim.entities_mut_for_test()
            .get_mut(radar)
            .unwrap()
            .lifecycle
            .set_object_alive_for_test(false);
        assert!(command_for_target(&sim, &rules, owner, radar).is_none());
        sim.entities_mut_for_test()
            .get_mut(radar)
            .unwrap()
            .lifecycle
            .set_object_alive_for_test(true);
        sim.entities_mut_for_test()
            .get_mut(radar)
            .unwrap()
            .lifecycle
            .set_in_limbo_for_test(true);
        assert!(command_for_target(&sim, &rules, owner, radar).is_none());
    }
}
