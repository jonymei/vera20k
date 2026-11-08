//! `TechnoClass::GetROF @ 0x006FCFA0` (vtable `+0x318` in every Techno-family
//! vtable; no class overrides it): the rearm FireAt stores after a shot. The
//! C4 plant, the gunner hand-over and `UnitClass::PerCellProcess` ask it too.
//!
//! Native execution: `tools/spatial_oracle/techno_rearm.py` runs the original
//! body under Unicorn over every arm, and [`tests`] replays those rows,
//! including the Scenario RNG state after each call.

use crate::rules::weapon_type::WeaponType;
use crate::sim::rng::SimRng;
use crate::util::native_x87::{
    MaskedX87Chop53 as X, MaskedX87Ordering, NativeF32Bits, NativeF64Bits,
};

/// The rearm countdown, `TechnoClass+0x2EC` (a CDTimerClass whose duration is
/// `+0x2F4`). It belongs to the object, not to its target: `Assign_Target @
/// 0x006FCDB0` never touches it, so a new target does not reload the weapon.
///
/// Writers, from a scan of every `+0x2EC..+0x2F4` operand in the image:
/// - The constructor starts it at the current frame with no duration
///   (`0x006F2E81`, [`Self::new`]).
/// - `TechnoClass::FireAt`, every launched shot, with GetROF's value (the
///   drawn gap mid-burst, the full reload after the burst's last shot),
///   halved for a berserk firer (`0x006FF274..0x006FF2CB`). Its DiskLaser
///   path stores the value unhalved, fires the disk laser and returns
///   (`0x006FE4A4..0x006FE4EF`). Ported (`world_fire::emit_admitted_fire`,
///   through `GameEntity::rearm_after_fire`); DiskLaser returns before the
///   ordinary Bullet and report/muzzle/reveal tails.
/// - `AircraftClass::Drop_Payload` restarts it with no duration
///   (`0x00415E88`). Ported (the paradrop success arm).
/// - A Prism support beam starts the supporter's downtime,
///   {Frame, `PrismSupportDelay=`} (`0x0044ACD0..0x0044ACDC`). Ported
///   (`building_missions::process_delayed_fire`).
/// - A C4 plant arms the planter with `GetROF(1)`
///   (`InfantryClass::Per_Cell_Process 0x0051A564`/`0x0051A624`). Ported
///   (`sim::c4_plant`).
/// - `UnitClass::ReceiveGunner`/`RemoveGunner` read the donor's active rearm
///   (`0x0074643A`, `0x00746502`), increment the receiving owner's burst,
///   and restart its timer with its own GetROF7/0 (`0x00746499`,
///   `0x00746574`; `passenger::rearm_transferred_gunner`). No charge-delay
///   write or burst remainder accompanies these callers.
/// - RESIDUAL, with its mechanism:
///   - `UnitClass::PerCellProcess` arms a unit that stops with no NavCom
///     and no path with `GetROF(1) / 4` when its type has
///     `MobileFire=no` (`0x0073ADCA..0x0073AE18`). Dormant: no retail
///     type sets it (the constructor's default is yes, `0x00711150`).
///
/// Readers: GetFireError answers Rearm while it runs (`0x006FC94F`);
/// `CanAutoCloak @ 0x006FBDC0` waits for it; `FootClass::Mission_Guard`
/// returns its remaining frames as the next delay and draws nothing
/// (`0x004D52A9`); the Prism walk skips a tower while it runs
/// (`0x0044B3AE`); `TechnoClass::Compute_CRC` folds it (`0x0070C362`).
/// RESIDUAL:
/// - `AircraftClass::Mission_Guard` falls into the same Foot body
///   (`0x0041A92B`), but VERA's aircraft Guard is not a port of it.
/// - `CanDeploySlashUnload @ 0x00700D50` reads it only for a type with
///   `UndeployDelay > -1` (`0x00700DF9..0x00700E23`); that type refuses
///   deployment while it runs. Stock GI/GGI keep the reader default-1.
///
/// The charge-turret model index reads this with the saved +2F8 duration
/// (6FA540); `game_entity::gunner` owns that presentation state.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RearmTimer {
    timer: crate::sim::timer::CdTimer,
}

impl RearmTimer {
    /// The constructor's start: the current frame, no duration (`0x006F2E81`).
    pub(crate) const fn new(frame: i32) -> Self {
        Self {
            timer: crate::sim::timer::CdTimer::started(frame, 0),
        }
    }

    /// The countdown as a timer.
    pub const fn timer(self) -> crate::sim::timer::CdTimer {
        self.timer
    }

    /// The frame it was last started.
    pub const fn start_frame(self) -> i32 {
        self.timer.start_frame()
    }

    /// Its duration (`+0x2F4`).
    pub const fn duration(self) -> i32 {
        self.timer.duration()
    }

    /// The frames left at `current_frame`.
    pub fn remaining(self, current_frame: i32) -> i32 {
        self.timer.remaining(current_frame)
    }

    /// Start it: the writers listed on [`RearmTimer`].
    pub(crate) fn start(&mut self, current_frame: i32, duration: i32) {
        self.timer.start(current_frame, duration);
    }

    /// Install a fixture's countdown.
    #[cfg(any(test, feature = "test-support"))]
    pub fn set_for_test(&mut self, timer: crate::sim::timer::CdTimer) {
        self.timer = timer;
    }
}

/// The firer's live particle systems GetROF tests against the weapon's flags.
/// FireAt creates the fired weapon's systems (`0x006FF15B..0x006FF26E`) before
/// it calls GetROF, so inside FireAt a flagged weapon always has its own.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct LiveParticles {
    /// `+0x308`, tested with `UseSparkParticles=` (WeaponType `+0x12A`).
    pub spark: bool,
    /// `+0x304`, tested with `UseFireParticles=` (`+0x129`).
    pub fire: bool,
    /// `+0x314`, tested with `IsRailgun=` (`+0x12D`).
    pub railgun: bool,
}

/// Everything GetROF reads besides the Scenario RNG.
#[derive(Debug, Clone, Copy)]
pub(crate) struct RofQuery<'a> {
    /// A building's `Ammo` (`+0x2FC`); `None` for every other class.
    pub building_ammo: Option<i32>,
    /// GetWeapon (vt+0x3F8) at the asked index; `None` for an empty slot.
    pub weapon: Option<&'a WeaponType>,
    pub live: LiveParticles,
    /// The burst index (`+0x3B8`) as the caller left it.
    pub burst_index: i32,
    /// A Unit's `BurstDelay0..3=`; `None` for every other class.
    pub unit_burst_delays: Option<[i32; 4]>,
    /// The owner's ROF bias (`House+0x1A8`).
    pub house_rof: NativeF64Bits,
    /// The rank holds the ROF ability: a veteran with the type's veteran byte
    /// (`+0x2A0`), or an elite with either byte (`+0x2A0`, `+0x2B2`).
    pub rof_ability: bool,
    /// `[General] VeteranROF=` (`Rules+0x690`).
    pub veteran_rof: f64,
    /// An occupied building that can fire (vt+0x400), with its occupant count
    /// (vt+0x408).
    pub occupants: Option<i32>,
    /// A non-building inside a tank bunker (`+0x2E4`).
    pub bunkered: bool,
    /// `[CombatDamage] OccupyROFMultiplier=` (`Rules+0xF44`).
    pub occupy_rof_multiplier: f32,
    /// `[CombatDamage] BunkerROFMultiplier=` (`Rules+0xF50`).
    pub bunker_rof_multiplier: f32,
}

/// GetROF6FCFA0 returns EAX plus, under NPAE174, the EDX burst interval.
/// Ammo/empty-slot early returns have no established EDX semantic result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RofDurations {
    pub reload: i32,
    pub burst_delay: Option<i32>,
}

/// `TechnoClass::GetROF @ 0x006FCFA0`, arm by arm:
/// - a building with more than one Ammo returns 1 (`0x006FCFA9..0x006FCFBE`),
///   and so does an empty weapon slot (`0x006FCFD4`), with no draw;
/// - `IsSonic=`, or a weapon whose spark, fire or railgun system is live,
///   returns the raw `ROF=` (`0x006FCFE0..0x006FD036` → `0x006FD1FA`);
/// - mid-burst (burst index below `Burst=`, signed, `0x006FD05A`) a Unit's
///   `BurstDelay{index-1}=` for index 1..=4 when it is not -1, else
///   `RandomRanged(3, 5)` (`0x006FD084..0x006FD094`);
/// - otherwise `RandomRanged(0, 2)` first, then `ftol(ROF * house + r)`
///   (`FILD; FMUL qword; FIADD`, `0x006FD09E..0x006FD0CF`), then
///   `ftol(rof * VeteranROF)` for the ROF ability (`0x006FD0E2..0x006FD14C`),
///   then an occupied building divides by its occupants (signed IDIV) and by
///   `OccupyROFMultiplier=` when that is above zero (`0x006FD150..0x006FD1AD`),
///   and a bunkered unit divides by `BunkerROFMultiplier=` when that is not
///   zero or NaN (`0x006FD1B1..0x006FD1EF`). No clamp follows.
///
/// NPAE174 removes the Sonic/particle early returns, continues full ROF
/// after a burst delay, and consumes but does not add jitter6FD0CB..CE.
/// The capability argument lets stock native fixtures exercise this same
/// owner; production passes NPATCH_ENABLED, never a per-entity policy.
/// Every draw is Scenario `RandomRanged`; all x87 steps run under the
/// process control word (53-bit, chop).
pub(crate) fn get_rof(query: &RofQuery<'_>, rng: &mut SimRng, npatch: bool) -> RofDurations {
    if query.building_ammo.is_some_and(|ammo| ammo > 1) {
        return RofDurations {
            reload: 1,
            burst_delay: None,
        };
    }
    let Some(weapon) = query.weapon else {
        return RofDurations {
            reload: 1,
            burst_delay: None,
        };
    };
    if !npatch
        && (weapon.is_sonic
            || (weapon.use_spark_particles && query.live.spark)
            || (weapon.use_fire_particles && query.live.fire)
            || (weapon.is_railgun && query.live.railgun))
    {
        return RofDurations {
            reload: weapon.rof,
            burst_delay: Some(weapon.rof),
        };
    }
    let mut burst_delay = None;
    if query.burst_index < weapon.burst {
        let explicit = query.unit_burst_delays.and_then(|delays| {
            (1..=4)
                .contains(&query.burst_index)
                .then(|| delays[(query.burst_index - 1) as usize])
                .filter(|&delay| delay != -1)
        });
        let delay = explicit.unwrap_or_else(|| rng.next_range_u32_inclusive(3, 5) as i32);
        if !npatch {
            return RofDurations {
                reload: delay,
                burst_delay: Some(delay),
            };
        }
        // NPAE1746FD075/6FD094 retain the interval and continue full ROF.
        burst_delay = Some(delay);
    }
    let jitter = rng.next_range_u32_inclusive(0, 2) as i32;
    let scaled = X::mul(X::load_i32(weapon.rof), X::load_f64(query.house_rof));
    //6FD0CB..CE are NOPs in NPAE174: keep the draw, omit FIADD only.
    let mut rof = X::ftol_i32_low_masked(if npatch {
        scaled
    } else {
        X::add(scaled, X::load_i32(jitter))
    });
    if query.rof_ability {
        rof = X::ftol_i32_low_masked(X::mul(
            X::load_i32(rof),
            X::load_f64(NativeF64Bits::from_bits(query.veteran_rof.to_bits())),
        ));
    }
    let zero = X::load_i32(0);
    if let Some(occupants) = query.occupants {
        if occupants > 0 {
            rof = rof.wrapping_div(occupants);
        }
        let multiplier = X::load_f32(NativeF32Bits::from_bits(
            query.occupy_rof_multiplier.to_bits(),
        ));
        if X::compare(multiplier, zero) == MaskedX87Ordering::Greater {
            rof = X::ftol_i32_low_masked(X::div(X::load_i32(rof), multiplier));
        }
    }
    if query.bunkered {
        let multiplier = X::load_f32(NativeF32Bits::from_bits(
            query.bunker_rof_multiplier.to_bits(),
        ));
        if !matches!(
            X::compare(multiplier, zero),
            MaskedX87Ordering::Equal | MaskedX87Ordering::Unordered
        ) {
            rof = X::ftol_i32_low_masked(X::div(X::load_i32(rof), multiplier));
        }
    }
    RofDurations {
        reload: rof,
        burst_delay: Some(burst_delay.unwrap_or(rof)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::ini_parser::IniFile;

    #[cfg(feature = "npatch")]
    #[test]
    fn npatch_full_reload_and_short_interval_match_native_outputs() {
        let rows: Vec<serde_json::Value> = serde_json::from_str(crate::test_fixture::text(
            "tools/spatial_oracle/techno_rearm_npatch.json",
        ))
        .unwrap();
        for row in rows {
            let input = &row["input"];
            assert_eq!(input["class"], 1, "these controls supply a Unit");
            let ini = IniFile::from_str(&format!(
                "[W]\nROF={}\nBurst={}\nIsSonic={}\nUseSparkParticles={}\n",
                input["rof"],
                input["burst"],
                input["direct"].as_i64().unwrap_or(0) != 0,
                input["spark"][0].as_i64().unwrap_or(0) != 0
            ));
            let weapon = WeaponType::from_ini_section("W", ini.section("W").unwrap());
            let query = RofQuery {
                building_ammo: None,
                weapon: Some(&weapon),
                live: LiveParticles {
                    spark: input["spark"][1].as_i64().unwrap_or(0) != 0,
                    ..Default::default()
                },
                burst_index: input["burst_index"].as_i64().unwrap() as i32,
                unit_burst_delays: Some(
                    input["unit_delays"].as_array().map_or([-1; 4], |values| {
                        std::array::from_fn(|n| values[n].as_i64().unwrap() as i32)
                    }),
                ),
                house_rof: NativeF64Bits::from_bits(
                    u64::from_str_radix(
                        input["house_bits"].as_str().unwrap_or("3ff0000000000000"),
                        16,
                    )
                    .unwrap(),
                ),
                rof_ability: input["veteran_ability"].as_i64().unwrap_or(0) != 0
                    && input["veterancy"].as_f64().unwrap_or(0.0) >= 1.0,
                veteran_rof: f64::from_bits(0x3fe3333340000000),
                occupants: None,
                bunkered: false,
                occupy_rof_multiplier: 1.0,
                bunker_rof_multiplier: 1.0,
            };
            let mut rng = SimRng::new(input["seed"].as_u64().unwrap());
            assert_eq!(rng.native_state_hex(), row["output"]["rng_before"]);
            let duration = get_rof(&query, &mut rng, true);
            let mut actor = crate::sim::game_entity::GameEntity::test_default_of_category(
                1,
                "TANK",
                "H",
                10,
                10,
                crate::map::entities::EntityCategory::Unit,
            );
            actor.rearm_after_fire(0, duration.reload, duration.burst_delay.unwrap());
            let retained = serde_json::to_value(&actor).unwrap();
            assert_eq!(
                i64::from(actor.rearm_timer.duration()),
                row["output"]["full_rof"],
                "{input}"
            );
            assert_eq!(
                retained["turret_selection"]["charge_turret_delay"], row["output"]["burst_delay"],
                "{input}"
            );
            assert_eq!(
                rng.native_state_hex(),
                row["output"]["rng_after"],
                "{input}"
            );
            if let Some(writers) = row["output"]["fire_writers"].as_array() {
                for writer in writers {
                    let expected = if writer["disk"].as_bool().unwrap() {
                        RofDurations {
                            reload: duration.reload,
                            burst_delay: Some(duration.reload),
                        }
                    } else {
                        crate::sim::combat::world_fire::fireat_rearm_frames(
                            duration,
                            writer["berserk"].as_bool().unwrap(),
                            true,
                        )
                    };
                    actor.rearm_after_fire(200, expected.reload, expected.burst_delay.unwrap());
                    assert_eq!(
                        i64::from(actor.rearm_timer.duration()),
                        writer["reload"],
                        "{writer}"
                    );
                    assert_eq!(
                        i64::from(actor.saved_rearm_delay()),
                        writer["saved_delay"],
                        "{writer}"
                    );
                    assert_eq!(
                        i64::from(actor.rearm_timer.start_frame()),
                        writer["start"],
                        "{writer}"
                    );
                }
            }
        }
    }

    /// `tools/spatial_oracle/techno_rearm.py` runs the original GetROF over
    /// every arm (supplied class, weapon slot, garrison and occupant queries;
    /// the original RNG, veterancy predicates and ftol execute) and records the
    /// value and the Scenario RNG state before and after. Every row replays
    /// through [`get_rof`] from the same seed.
    #[test]
    fn get_rof_matches_the_original() {
        let rows: Vec<serde_json::Value> = serde_json::from_str(crate::test_fixture::text(
            "tools/spatial_oracle/techno_rearm.json",
        ))
        .unwrap();
        assert_eq!(rows.len(), 535);
        for row in &rows {
            let input = &row["input"];
            let int = |key: &str, default: i64| input[key].as_i64().unwrap_or(default) as i32;
            let hex = |key: &str, default: &str| {
                u64::from_str_radix(input[key].as_str().unwrap_or(default), 16).unwrap()
            };
            let pair = |key: &str| {
                input[key]
                    .as_array()
                    .map_or((false, false), |pair| (pair[0] != 0, pair[1] != 0))
            };
            let class = int("class", 15);
            let (spark, spark_live) = pair("spark");
            let (fire, fire_live) = pair("fire");
            let (railgun, railgun_live) = pair("railgun");
            let ini = IniFile::from_str(&format!(
                "[W]\nROF={}\nBurst={}\nIsSonic={}\nUseSparkParticles={}\n\
                 UseFireParticles={}\nIsRailgun={}\n",
                int("rof", 50),
                int("burst", 1),
                if int("direct", 0) != 0 { "yes" } else { "no" },
                if spark { "yes" } else { "no" },
                if fire { "yes" } else { "no" },
                if railgun { "yes" } else { "no" },
            ));
            let weapon = WeaponType::from_ini_section("W", ini.section("W").unwrap());
            let veterancy = input["veterancy"].as_f64().unwrap_or(0.0) as f32;
            let veteran = (1.0..2.0).contains(&veterancy);
            let elite = veterancy >= 2.0;
            let veteran_byte = int("veteran_ability", 0) != 0;
            let elite_byte = int("elite_ability", 0) != 0;
            let delays = input["unit_delays"].as_array().map_or([-1; 4], |delays| {
                std::array::from_fn(|index| delays[index].as_i64().unwrap() as i32)
            });
            let query = RofQuery {
                building_ammo: (class == 6).then(|| int("building_ammo", 0)),
                weapon: (!input["no_weapon"].as_bool().unwrap_or(false)).then_some(&weapon),
                live: LiveParticles {
                    spark: spark_live,
                    fire: fire_live,
                    railgun: railgun_live,
                },
                burst_index: int("burst_index", 1),
                unit_burst_delays: (class == 1).then_some(delays),
                house_rof: NativeF64Bits::from_bits(hex("house_bits", "3ff0000000000000")),
                rof_ability: (veteran && veteran_byte) || (elite && (veteran_byte || elite_byte)),
                veteran_rof: f64::from_bits(hex("veteran_bits", "3fe3333340000000")),
                occupants: (int("garrison", 0) != 0).then(|| int("occupants", 0)),
                bunkered: int("bunker", 0) != 0 && class != 6,
                occupy_rof_multiplier: f32::from_bits(hex("occupy_bits", "3f800000") as u32),
                bunker_rof_multiplier: f32::from_bits(hex("bunker_bits", "3f800000") as u32),
            };
            let mut rng = SimRng::new(input["seed"].as_u64().unwrap_or(31));
            assert_eq!(
                rng.native_state_hex(),
                row["rng_before"].as_str().unwrap(),
                "{input}"
            );
            assert_eq!(
                i64::from(get_rof(&query, &mut rng, false).reload),
                row["output"].as_i64().unwrap(),
                "{input}"
            );
            assert_eq!(
                rng.native_state_hex(),
                row["rng_after"].as_str().unwrap(),
                "{input}"
            );
        }
    }
}
