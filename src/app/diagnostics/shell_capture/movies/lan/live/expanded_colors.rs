//! Real LAN options exchange through the ordinary setup action owner.
//! No received roster, ready bit, launch slot or render mapping is injected.
use super::*;

impl LiveLanCapture {
    pub(super) fn expanded_color_step(&mut self, state: &mut AppState, frame: u32) -> Result<bool> {
        let host = self.host();
        let wanted = if host { 15 } else { 14 };
        let peer_wanted = if host { 14 } else { 15 };
        let session = &state.match_state.session;
        let launch = session.network_setup().context("expanded LAN setup")?;
        let row = launch
            .local_player()
            .raw_slot_index
            .context("local raw row")?;
        let local_matches = !launch.local_player().slot.color_random
            && launch.local_player().slot.color_index == wanted;
        let peer_matches = launch.humans.iter().any(|human| {
            human.name == if host { GUEST_NAME } else { HOST_NAME }
                && !human.slot.color_random
                && human.slot.color_index == peer_wanted
        });
        let control = setup_control_at_row(state, SetupColumn::Color, usize::from(row))?
            .context("visible expanded LAN color control")?;
        let view = setup::view(state).context("expanded LAN view")?;
        let combo = view.combo(control).context("expanded LAN color choices")?;
        if !local_matches {
            ensure!(
                combo.visible
                    && combo.enabled
                    && combo
                        .choices
                        .iter()
                        .any(|choice| choice.value == i32::from(wanted)),
                "last two AY colors are unavailable through the normal LAN control"
            );
            setup::apply_action(
                state,
                SetupAction::Select {
                    control,
                    value: i32::from(wanted),
                },
            );
            self.event(
                frame,
                "selected expanded LAN color through ordinary local combo action",
            );
            return Ok(false);
        }
        if !peer_matches {
            return Ok(false);
        }
        ensure!(
            !combo
                .choices
                .iter()
                .any(|choice| choice.value == i32::from(peer_wanted)),
            "received occupied color remains selectable by the other participant"
        );
        let snapshot = json!({"after_kick":self.kick_observed,
            "local_color":wanted,"received_peer_color":peer_wanted,
            "occupied_peer_color_available":false});
        if self
            .color_observations
            .last()
            .is_none_or(|last| last["state"] != snapshot)
        {
            self.color_observations
                .push(json!({"render_frame":frame,"state":snapshot}));
            self.event(
                frame,
                "received real peer expanded color; occupied choice excluded",
            );
        }
        Ok(true)
    }
}

pub(super) fn runtime_colors(state: &AppState) -> Result<Value> {
    let session = &state.match_state.session;
    let launch = session
        .network_setup()
        .context("expanded running network setup")?;
    let rules = state.rules().context("expanded running rules")?;
    let mut rows = Vec::new();
    for (source, human) in launch.humans.iter().enumerate() {
        let expected = if human.name == HOST_NAME {
            "MPC16"
        } else {
            "MPC15"
        };
        let owner = session
            .network_source_owner(source)
            .context("expanded source House")?;
        let color = state
            .match_state
            .match_presentation
            .house_color_map
            .get(&owner)
            .context("actual loaded House render color")?;
        let scheme = rules
            .color_schemes
            .get(usize::from(color.0))
            .context("actual loaded render scheme")?;
        ensure!(
            scheme.name.eq_ignore_ascii_case(expected),
            "received expanded choice did not reach actual render scheme: {}",
            scheme.name
        );
        rows.push(json!({"source":source,"name":human.name,"house":owner,
            "launch_color":human.slot.color_index,"render_scheme_entry":color.0,
            "render_scheme_name":scheme.name}));
    }
    Ok(json!(rows))
}
