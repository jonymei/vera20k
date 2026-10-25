# AYMod extension compatibility

This is the current compatibility ledger, not a whole-mod certification.
Unfinished rows remain requirements; parsing a key or loading a match does not
close them.

## Target and scope

The inspected AYMod executable identifies itself as **NPAE #0011 Ext #174 by
LH**, or **NPatch Autumn Edition Extended #174**. Its version resource, referenced
version strings and installed jumps establish that identity. NPSE 2.0 #359 is not
an interchangeable reference for its readers. There is no positive evidence that
this installation loads Ares, Syringe or Phobos; absence of a DLL alone is not a
proof that injection is impossible.

The local MOD baseline is commit `9f8ccbd`, including its four maintained AIMD
corrections. The installed clean local build is `f1d141c7a`, built with the optimized
`fast` profile and `npatch`, with default/dev features disabled. Ares implementations can identify reusable
owners, but enabling Ares is not a substitute for porting the NPAE input and
behavior.

The main inventory is `rulesmd.ini`, followed through its registered types,
weapons, projectiles, warheads and referenced assets. ART, AI, sounds, EVA, UI,
themes, mode overrides and scenario layers are also in scope where they affect
these features. Ordinary retail keys must be distinguished from extension keys,
changed retail behavior, and always-active binary patches with no enabling key.

## Completion criteria

A used feature closes only when all applicable stages have evidence:

1. Its actual selected source, section, exact key and effective value are known.
2. The matching NPAE reader, constructor/default, layer order and post-pass are
   established, rather than borrowed from a similar Ares or NPSE feature.
3. Its caller and state chain is traced through admission, effects, consumers,
   updates and cleanup. Player, AI and bypass routes stay distinct where native
   behavior distinguishes them.
4. The existing authoritative Rust owners implement and connect that chain.
5. Focused production-path regression tests protect the repaired behavior.
6. An affected real MOD scenario demonstrates the player-visible outcome.

The target is all used extension mechanisms and their necessary dependencies.
Neither a percentage of source lines nor a fixed list of guessed extension
names is the coverage denominator. Discovery remains open until the full key
inventory, referenced data and author-described no-key patches are reconciled.

## Selected sources and inventory boundary

| Data | Current selected source | Inspection boundary |
| --- | --- | --- |
| RULESMD | Maintained loose `rulesmd.ini` | Primary inventory and production accessor queries |
| ARTMD | `expandmd99.mix` | Fixed ART; BFRT passenger FLH differences identified |
| AIMD | Maintained loose `aimd.ini` | Four recorded corrections; original snapshot remains separate |
| LANGRULE | Absent | No layer in the inspected installation |
| SoundMD | `expandmd01.mix` | Bytes match the supplied Steam selected source |
| EVAMD | `ra2md.mix -> localmd.mix` | Bytes match the supplied Steam selected source |
| UIMD | Maintained loose `Uimd.ini` | Advanced command button lists; not a toggle-power implementation |
| MPModesMD | Maintained loose `mpmodesmd.ini` | Nine distinct referenced override files extracted |
| Mode overrides | Existing selected archives | Those nine files match the supplied Steam files; this is not nine-mode runtime validation |
| ThemeMD | `expandmd99.mix` | Selected source identified; not music playback certification |
| Scenario | `XMP03T4.MAP` from `multimd.mix` | Battle mode 1 production queries and earlier runtime smoke only |

The first main-rules pass found 25,068 physical section/key pairs and 2,374
distinct key strings. These include registry indices and dynamic color names,
not 2,374 extension fields. The initial production query set contains 207
section/key queries across 81 key names and 112 sections. It is an accessor
inventory, not 207 validated gameplay mechanisms. Other scenarios, modes,
constructor-driven behavior and same-name patched retail keys remain outside
that query set's proof.

The full key-string difference from the supplied Steam RULESMD contains 67
strings: 20 registry/color data names and 47 field candidates. All 47 were kept
in the inventory, including unclassified names, instead of filtering to a
preselected extension whitelist. Absence from Steam data does not establish
absence from its executable or prove that a field belongs to NPatch.

## Confirmed gaps

| Feature | AYMod use | Current engine boundary | Status |
| --- | --- | --- | --- |
| Alternative technology branches | Seven `Prerequisite2` entries | `d997311e6` reads both alternative lists through the native parser and retries complete AND groups as OR alternatives in the shared CanBuild owner, including early-failure routes. Mixed-extension policy is documented | Implemented; production CanBuild/direct consumers and maintained AYMod alternative-tech scenario pass |
| Naval factory exclusion | Three `NegativePrerequisite` entries naming the respective shipyards | `d997311e6` checks the existing on-map House counters at the shared success/build-limit join, including override/skip-tech routes; preserves ordinary AI early success. Invalid generic negatives are safely ignored by explicit policy | Implemented; production counter regression and maintained AYMod shipyard Unlimbo/Limbo scenario pass |
| Repeated AI base candidates | Eight factories use `AIBasePlanningCount=2` | `d997311e6` reads the NPAE spelling/default into the existing signed-byte candidate-repeat owner. Same-pass NPAE-key precedence and later-pass replacement by either spelling are explicit compatibility policy | Implemented; actual AYMod eight-factory count reader and existing candidate-repeat/AI chooser regressions pass |
| Manual building power toggle | `[UISettings] UseTogglePower=yes` | `f2b8f5946..22d05aac1` connects frozen-root input, configurable default V, Power/NoPower cursor and own-building admission to existing Event01/02 and BuildingOnline owners. Fresh-world modes and outgoing held-mouse gestures are cleared; active EMP blocks On | Implemented; input/event/power regressions, real AYMod paid-factory EMP interaction, and GUI barracks Off/On transitions pass |
| EMP weapons | `EMEffect=yes` warheads and referenced weapons/projectiles | `5bc1290c2..dcce945a7` connects actual detonations and AYMod's retained Tesla impact-child producer to the shared EMP/deactivation/operational owners. Duration, susceptibility and mixed Ares precedence are explicit playable policies, not asserted NPAE parity | Implemented; actual AYMod TTNK Attack/expiry and paid-factory recovery scenarios pass; native parity and balance are not certified |
| Passenger experience | BFRT/APOC `TrainPassengers=yes` | Local reward candidate routes open-topped kill XP to a trainable passenger before elite, then falls back to the carrier; spawned-child awards and inheritance reuse existing kill/spawn owners. This is documented compatibility policy | Implemented; real AYMod BFRT Enter, passenger kill credit and elite-to-carrier fallback pass; spawned inheritance regressions pass |
| Controller experience | `MindControllerGainExperienceRatio=100%` | Local reward candidate shares conserved kill XP with a valid non-elite controller through the existing award owner; unavailable/elite controllers leave XP with the ordinary recipient. Ratio bounds are explicit policy | Implemented; real AYMod Yuri Attack/control, authored XP share and elite-controller fallback pass |
| Hero bounty | `GetsBounty` and the bounty multiplier family | Local reward candidate uses pre-kill collector rank and victim cost/rank multipliers; hostile terminal kills pay once through existing KillRecord/Economy, with registered BountyAnim through AnimClass. Defaults/clamps are explicit policy | Implemented; real AYMod Tanya kill pays spendable House credits; reward/dedup/non-NP regressions pass; animation appearance not separately certified |
| Bunker protection/penetration | Protected bunker/power plants and sniper/radiation/MiG warheads | No matching NPAE admission and victim-effect chain established | Missing |
| Permanent control | `IsPermaControlled` on control warheads | Local control candidate reads the NP warhead key and reuses permanent-capture ownership without a temporary manager link. SpyJack boarding consumes its spy on success and reuses parasite release on refusal/miss | Implemented; real AYMod Spy boarding/consumption and elite Yuri building capture persist after controller removal; focused refusal/miss regressions pass |
| Independent turret speed | Fourteen `TurretROT` entries | Current ordinary turret rotation uses `ROT`; the stock-only no-key rationale does not cover this NPAE reader | Missing |
| Electric bolt colors | Authored `EBoltColor1/2/3` overrides | No matching reader/render consumer established | Missing |
| Weapon target restrictions | `AACombat`, `AntiNaval`, `AntiUnderwater` | Existing projectile AA/AG and naval rules do not implement these weapon-level entries | Missing; native semantics still to trace |
| Other warhead policies | `IsFlatExplosion`, `DoesntAffectEnemies` | No corresponding NP input/effect policy established; Ares house filters are not the same input | Missing; native semantics still to trace |
| Per-superweapon lighting | Psychic Dominator `Lighting` and `LightRed/Green/Ambient/Ground/Level` | Ordinary map/SW lighting does not establish these NP overrides | Missing; native and rendered results still to trace |
| Extension shell controls | `ShowTechLevelSlider`, `UseDropDownColors`, `MPC01..MPC16` color data | No configuration-to-UI chain established for the NP fields; dynamic color names are data, not separate rule fields | Missing/unverified |
| Power-dependent drones | ROBO/DRON/CAOS `PoweredUnit` and GAROBO/NARADR/NAPSIS `PowersUnit` | Existing retail robot behavior does not certify all three MOD type/provider pairs | Dedicated native/wiring/runtime audit pending |
| Six-seat firing origins | BFRT `AlternateFLH5/6` in fixed ART | Authored origins confirmed; indexing and final rendered firing origins not certified | Dedicated consumer/runtime audit pending |

These are initial findings, not the complete feature list. Ordinary `Hospital`,
`BurstDelay0..3`, cloak/sensor keys and related existing mechanisms require their
own classification and affected-path evidence; they must not be marked missing
merely because the MOD uses them differently from retail data.

The implemented rows describe local delivery candidates, not whole-mod certification.
Reward/control is committed as `dd3756a77`;
the technology, power and EMP commits are now combined on the local integration
branch. Some individual mixed-feature paths have
focused coverage, but combined-delivery feature combinations, save/load and real
AYMod window/gameplay acceptance remain unverified. No row above is closed by
implementation or a green focused test alone.

The combined local candidate passes seven opt-in scenarios against the maintained
AYMod rules and assets through the headless scenario runtime:

- A real TTNK Attack delivers EMP to an APOC; actual frames expire it after the
  shooter is removed.
- EMP suspends a paid MTNK factory head without losing the object or its cost;
  manual Off survives expiry, and On resumes real production progress.
- A real Tanya kill receiver increases its House's spendable credits.
- Actual Enter boarding into a BFRT trains its passenger; an elite passenger's
  next kill trains the carrier instead.
- Actual Yuri Attack captures a tank; its kills train the controller at the
  authored ratio, and an elite controller leaves subsequent XP with the tank.
- Actual Spy Attack consumes the boarder and permanently retains the tank.
- Actual elite Yuri Attack permanently captures a building; removing Yuri does
  not release it.

The maintained technology/shipyard regression also passes through production
CanBuild and building Unlimbo/Limbo counters. Eight combined NPatch EMP tests
cover production identity/cash/progress, other online producers, user holds,
low-power recovery and saved EMP/manual-power intent. The production fix keeps
physical factory lookup separate from online admission and notifies the existing
queue owner on EMP transitions. No second hold state was added.

The seven scenarios above are headless production-path checks, not whole-mod
rendering certification. Computer Use additionally exercised normal menus,
skirmish launch, base deployment, power-plant/barracks construction and loading
the original QA save. The saved terrain source-to-LAT variant restoration defect
is repaired without rewriting the saved Cell fields or bypassing layout checks.
Actual GUI V/click targeting produced Barracks Off and then On commands; the
next click observed the prior command's committed online latch, not merely an
input receipt.

The apparent black field was reproduced from an actual camera that had scrolled
outside the explored area. Re-centering with the pointer inside the tactical
view restores the base; no shroud bypass or speculative palette fix was made.
The windowed camera owner now stops edge/coast scrolling on pointer exit while
retaining in-window coast and captured right-drag behavior. Its focused regression
fails before the fix and the 26-test camera module passes. Earlier startup white
screens have not recurred on subsequent normal launches; no startup fix is claimed.
The clean build was reloaded through the normal save-game menu after removing
all temporary input/render diagnostics; its original QA battle renders normally.

These selected five delivery families do not close the other missing mechanisms
in this ledger. Full balance, every mode/map combination and every visual effect
are not certified. Documented player-friendly NPatch policies remain intentional
rather than being presented as exact NPAE arithmetic.

## Initial data families

The 207 main queries were grouped as follows. Counts are query instances, not
the count of every occurrence of those keys in the MOD, and not runtime test
coverage. Three additional probes checked an absent `AlternatePrerequisite`
on GATECH and BFRT's two added ART firing origins.

| Family | Query instances | Required mechanism boundary |
| --- | ---: | --- |
| Supporting references | 39 | Registered types, weapons, projectiles and warheads |
| Retail or reactivated fields | 55 | EMP, power providers, cloak, sensors and prerequisite groups |
| AI planning and legacy AI | 12 | Counted candidates and `[AI] Build*` call reachability |
| Experience | 3 | Passenger training and controller sharing |
| Turret and burst timing | 20 | Independent turret rate versus existing retail burst delays |
| Alternative/negative prerequisites | 10 | Seven alternate groups and three shipyard exclusions |
| Garrison penetration | 9 | Six warhead levels and three building protection levels |
| Bounty | 22 | Recipient/rank flags, values, multipliers and animation |
| Other warhead effects | 6 | Flat explosion and house filtering |
| Permanent control | 2 | Implemented; real AYMod Spy boarding/consumption and elite Yuri building capture persist after controller removal; focused refusal/miss regressions pass |
| Hospitals | 2 | GAROBO/NARADR entry, treatment and departure |
| Country/UI | 4 | Country range, power toggle, tech slider and colors |
| Weapon targeting | 5 | Air/naval/underwater weapon policies |
| Dominator lighting | 6 | Per-superweapon lighting activation and restoration |
| Electric bolt colors | 12 | Three weapons with custom-color flags and RGB triplets |

Author-described changes without a distinct new key remain additional inventory
items: spawned/mother experience, building veterancy, crate effects and inherited
bug fixes must be reconciled with actual native consumers. An unchanged sound,
mode or UI file does not exclude a binary patch changing its interpretation.

The candidate Tesla EMP path is TTNK -> TankBolt/TankBoltE ->
ElectricbounceT/ElectricbounceTE -> TankBoltEMP -> EMPulsWH. The parent
projectiles also author `Airburst=no`; the static `AirburstWeapon` reference
alone does not prove that the EMP child fires. Preserve this reachability
question instead of treating the reference graph as an executed attack.

Likewise, `GetsBounty` rank flags are authored on TANY/BORIS/BPLN, whereas the
`Bounty` rank values are on TANY/BORIS/YURIPR. Their beneficiary/victim roles
must come from the native kill chain, not from an assumed shared switch.

## Native chains established so far

### AIBasePlanningCount

NPAE `TechnoType` constructor extension `0xC2EF82` initializes byte `+0x5A2`
to 1. Reader hook `0x7138C7 -> 0xC2EF3E` calls the original ReadInt at
`0x5276D0` with `AIBasePlanningCount` and the retained byte as default, then
stores AL at `0xC2EF52`. Base-plan collection hook
`0x5056C1 -> 0xC2D8C1` increments a byte counter and uses signed CMP/JL to
re-enter the same candidate at `0x505544`.

The existing `sim/base_plan_generation.rs` owner already has the corresponding
signed-byte, at-least-one repetition behavior for NPSE. Repair the actual reader
and default/layer semantics; do not add a second planner or rename MOD data to
the other version's spelling. Any policy for simultaneously authored keys must
be explicit, not falsely attributed to native behavior.

### Prerequisite2 and NegativePrerequisite

NPAE reader `0xC27010` uses the original prerequisite parser `0x4770E0` for
`AlternatePrerequisite`, `Prerequisite2` and `NegativePrerequisite`, storing the
lists in the existing extension object at `+0x6C`, `+0xB4` and `+0x88`.

`HouseClass::CanBuild @ 0x4F7870` has a patched failure exit
`0x4F7F46 -> 0xC27E80`. It tries each nonempty alternate list by returning to
the original per-member AND checks at `0x4F7BF1`. Complete lists are alternatives
(OR), not additional requirements (AND). The fallback is reached from more than
ordinary prerequisite failure: the `TechLevel=-1` branch
`0x4F794A..0x4F7961` and a stolen-Allied-tech rejection also reach it. Adding an
OR only after the current Rust prerequisite check would preserve the wrong
earlier refusals.

The success join `0x4F7F9E -> 0xC27EED` checks each negative entry against the
House on-map BuildingType counter (`+0x5550`, `0x49FAE0`), refusing when any
count is positive. It also covers `skip_tech`, prerequisite overrides and Secret
Lab shortcuts. An ordinary non-human AI reaches its earlier success return
`0x4F7BE5`, bypassing these prerequisite/negative checks. Menu, sidebar and AI
chooser callers all use this existing CanBuild owner, but with different flags.

Negative generic-group codes are not ordinary group queries: the native counter
does not reject negative indices safely. AYMod's three entries name concrete
shipyards, so that undefined domain is not needed to reproduce them. No native
execution or whole-match equivalence is claimed by these instruction reads.

## Next closure

Finish the inventory classification, then prioritize the technology-tree chain,
manual power control and EMP by player visibility and frequency. Each repair
must extend the existing owner, include its necessary lifecycle consumers, and
get focused regression plus affected MOD runtime verification. Keep source,
installed-build and actual configuration state separate. Do not mark the ledger
complete while any used mechanism is missing, partial or unverified.
