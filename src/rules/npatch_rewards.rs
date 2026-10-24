//! AYMod reward compatibility. Defaults and sharing policy are deliberate
//! player-facing choices where NPAE's complete arithmetic is not established.

use crate::rules::ini_parser::{IniFile, IniSection};
use crate::util::fixed_math::SimFixed;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct NpatchRewards {
    controller_share: SimFixed,
    bounty_factors: [SimFixed; 3],
    pub(crate) bounty_anim: Option<String>,
}

impl Default for NpatchRewards {
    fn default() -> Self {
        Self {
            controller_share: SimFixed::ZERO,
            bounty_factors: [SimFixed::ZERO; 3],
            bounty_anim: None,
        }
    }
}

impl NpatchRewards {
    pub(crate) fn read(ini: &IniFile) -> Self {
        if !super::NPATCH_ENABLED {
            return Self::default();
        }
        let general = ini.section_or_empty("General");
        let combat = ini.section_or_empty("CombatDamage");
        Self {
            controller_share: multiplier(general, "MindControllerGainExperienceRatio", 0.0, 1.0),
            bounty_factors: [
                multiplier(combat, "BountyCostMult", 0.25, 16.0),
                multiplier(combat, "VeteranBountyCostMult", 0.5, 16.0),
                multiplier(combat, "EliteBountyCostMult", 1.0, 16.0),
            ],
            bounty_anim: combat
                .read_name("BountyAnim", 0x80)
                .filter(|name| !crate::rules::ini_parser::is_native_none_type_name(name))
                .map(str::to_owned),
        }
    }

    pub(crate) fn controller_points(&self, points: i32) -> i32 {
        ((i64::from(points.max(0)) * i64::from(self.controller_share.to_bits())) >> 16) as i32
    }

    pub(crate) fn bounty(&self, cost: i32, victim_rank: u16, victim: &NpatchTypeRewards) -> i32 {
        let tier = tier(victim_rank);
        let amount = i128::from(cost.max(0))
            * i128::from(self.bounty_factors[tier].to_bits())
            * i128::from(victim.bounty[tier].to_bits());
        (amount >> 32).clamp(0, i128::from(i32::MAX)) as i32
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct NpatchTypeRewards {
    pub(crate) train_passengers: bool,
    collects_bounty: [bool; 3],
    bounty: [SimFixed; 3],
}

impl Default for NpatchTypeRewards {
    fn default() -> Self {
        Self {
            train_passengers: false,
            collects_bounty: [false; 3],
            bounty: [SimFixed::ONE; 3],
        }
    }
}

impl NpatchTypeRewards {
    pub(crate) fn read(section: &IniSection) -> Self {
        if !super::NPATCH_ENABLED {
            return Self::default();
        }
        let gets = section.read_bool("GetsBounty", false);
        let veteran_gets = section.read_bool("VeteranGetsBounty", gets);
        let base = multiplier(section, "Bounty", 1.0, 16.0);
        let veteran = multiplier(section, "VeteranBounty", base.to_num(), 16.0);
        Self {
            train_passengers: section.read_bool("TrainPassengers", false),
            collects_bounty: [
                gets,
                veteran_gets,
                section.read_bool("EliteGetsBounty", veteran_gets),
            ],
            bounty: [
                base,
                veteran,
                multiplier(section, "EliteBounty", veteran.to_num(), 16.0),
            ],
        }
    }

    pub(crate) fn collects_bounty(&self, rank: u16) -> bool {
        self.collects_bounty[tier(rank)]
    }
}

fn tier(rank: u16) -> usize {
    if rank >= 200 {
        2
    } else if rank >= 100 {
        1
    } else {
        0
    }
}

fn multiplier(section: &IniSection, key: &str, default: f64, maximum: f64) -> SimFixed {
    let value = section.read_double(key, default);
    SimFixed::from_num(if value.is_finite() {
        value.clamp(0.0, maximum)
    } else {
        default
    })
}
