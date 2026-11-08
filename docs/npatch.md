# Optional NPatch compatibility

NPatch is a long-standing binary patch for Yuri's Revenge that predates the DLL-based Ares and
Phobos extensions. VERA20K implements the supported behavior in Rust. Enable it explicitly:

```sh
cargo run --release --bin vera20k --features npatch
```

The feature compiles to `rules::NPATCH_ENABLED`. A build without it ignores the extension keys
and keeps stock behavior, like a build without `ares` or `phobos`.

NPatch generations are not one interchangeable reader contract. AYMod uses NPAE
#0011 Extended #174, including `AIBasePlanningCount`, not NPSE #359's
`AIPlanningCount`. The [AYMod compatibility ledger](aymod-compatibility.md)
separates actual data, implemented consumers, missing chains and validation
boundaries; enabling this feature does not certify whole-mod support.

`[General] VehicleParachute=` chooses the canopy for airdropped non-infantry, falling back to
`Parachute=` when unset. Infantry keeps `Parachute=`. The name uses the retained animation
reference and asset-loading owners, including empty-layer retention and `none` clearing.
This follows NPSE 2.0 #359's ReadGeneral (`0xC31005..0xC31017`) and ObjectClass::Paradrop
(`0x5F5ADD`, `0xC2B0E4`, `0xC2FA75`, `0xC2F939`) from the
[author-linked release drive](https://zhytat.ysepan.com/). Parachuted bombs and their `IsBomb=`
branch remain outside the current descent implementation.

`AIBasePlanningCount=` (NPAE174, `0x7138C7 → 0xC2EF3E`) and
`AIPlanningCount=` (NPSE359, `0xC2E223`) share the existing base-plan candidate
repetition owner. Both readers store one byte; the signed loop comparison repeats
counts 1–127, while zero and high-bit counts retain one entry. NPAE's constructor
starts at one (`0xC2EF82`); NPSE starts at zero, with the same one-entry result.
VERA uses NPAE's default and retained per-pass value. Compatibility policy: when
both keys occur in one physical pass, `AIBasePlanningCount` wins; a later pass
may explicitly replace the value with either spelling. Missing/failed reads keep
the current value. The NPAE consumer is `0x5056C1 → 0xC2D8C1`, and NPSE uses
`0x5056C1 → 0xC2C331`; neither repeats the finished plan or owns another RNG.

`AlternatePrerequisite=` and `Prerequisite2=` are alternative complete AND groups,
not additional requirements. NPAE174 reads them with the original 128-byte
prerequisite parser (`0xC27010`) on each physical Rules pass. Missing/empty reads
retain the list; a comma-only or all-unknown list clears it. A human's failing
CanBuild tries the first nonempty satisfied group, in that order, through the
existing prerequisite evaluator (`0x4F7F46 → 0xC27E80 → 0x4F7BF1`). This fallback
also follows early rejections such as `TechLevel=-1` or missing stolen tech;
it is deliberately not restricted to ordinary prerequisite failure.

`NegativePrerequisite=` rejects when any listed concrete BuildingType has a
positive on-map count (`0x4F7F9E → 0xC27EED`, House counter `+0x5550`). It also
applies to skip-tech, Secret Lab and prerequisite-override shortcuts before the
build limit. The ordinarily qualified nonhuman AI retains its native early Yes
and bypasses these checks; an AI admitted by the alternative fallback still
checks negatives and the limit. Negative generic groups cause native invalid
counter indexing; VERA safely ignores those entries rather than emulating heap
reads. AYMod's three negative lists contain only concrete shipyards.

In a mixed Ares/NPatch build, NPatch vectors still use the stock parser; their
ordinary group entries use the shared Ares prerequisite evaluator, while Ares
factory ownership and online checks remain required. This is an explicit playable
compatibility policy, not a claim that both binary patches coexist natively.

Per-type `AIBuildLimit=` limits AI production using the existing tracked and produced-count
owners (`Factory_AI`, `0xC335A0..0xC33726`). Positive limits include limbo factory objects;
non-positive limits compare the magnitude against both counters, and zero blocks the type.
Reaching a cap clears the standing choice, without advancing the base plan. A capped building
can therefore stall construction. NPatch builds retain independent land and naval
vehicle choices through the existing House owner; stock builds keep one vehicle
slot. The cap clears the fetched type's matching choice, not both slots. See
[Independent AI land/naval choices](#independent-ai-landnaval-choices) for the
NPAE174 consumer, native asymmetries and remaining selection-policy difference.

NPSE's naval vehicle choice uses the existing mobile-choice owner. House Update asks the
land and naval channels in order (`0xC33AB0`), sharing the harvester decision and team
chooser. Suggest_New_Object reads the producer's channel (`0xC2F130`); exit, abandonment
and build-limit rejection clear the corresponding choice (`0xC33940`, `0xC33B50`,
`0xC33570`). These paths require the `npatch` build capability.

The legacy `[General] EnableAIBuildLimitation=yes` switch applies the same comparison to
`BuildLimit=` (NPExt ext173 reader `0xC2D132`, gate `0xC2CF67`). It defaults to no. This is
compatibility with the [older NPExt release](https://ppmforums.com/topic-29537/npatch-extended-finally-released/);
NPSE #359 replaced the global switch with `AIBuildLimit=`. If a VERA ruleset explicitly sets
both, both constraints apply before production starts. Clearing a choice does not skip an
unsatisfied building-plan node or guarantee the mobile chooser selects an uncapped type.

`[General] NewPlayableCountries=` adjusts the ordinary selected-mode random-country range
from `0..9` to `0..(9 + value)`, inclusive. It defaults to zero and is read once from the
startup-selected rules root, not from LANGRULE, mode or map overrides. AYMod's `-7` selects
its first three countries without including the non-player entries later in the registry.
This follows its NP initializer `0xB9902D`, root reader `0xC275C2..0xC275DD` and shared
country-selector hook `0xB93618..0xB9363B` behind `0x69B670`. Both human and AI slots use
the existing Scenario RNG and assignment order. The value is not clamped to roster size;
unallocated or unrepresentable results are refused instead of remapped. Cooperative
selection and the stock mode-null fallback retain their separate behavior.

## AYMod facing and passenger firing ports

NPAE174 `TurretROT=` is a retained byte, initially 255 (TechnoType constructor
`0x710F93`). Each physical Rules pass reads the exact key using the current
byte as the ReadInt default, then stores the low eight bits
(`0x713879..0x71388D`). Unit construction selects this value for the turret,
with byte 255 falling back to ordinary `ROT` (`0x735584 -> 0xC2DEB6`).
Body, Aircraft and Building rotation keep their existing `ROT` behavior.
The ordinary FacingClass owner still applies its numeric limits and timing;
ongoing Unit aiming uses the same selected turret speed as construction.

NPAE174 also cycles the five `AlternateFLH0..4` ports for additional passengers:
`TechnoClass::GetFLH @ 0x6F3AFB -> 0xC2E196` maps the one-based cargo index
`k` to zero-based `(k - 1) % 5`. Thus AYMod's sixth BFRT passenger reuses port
zero. The native ART reader still stops after five entries
(`0x715FA4..0x716005`); authored `AlternateFLH5/6` do not enable extra slots.
Both ordinary negative weapon indices and open-topped passenger shots use the
existing shared FLH owner. This is the `npatch` capability's NPAE174 policy;
other NPatch generations are not certified by it. Without `npatch`, stock
overflow beyond five continues to return a zero FLH. The separate
`UseAlternateFLH` burst-index hook remains outside this passenger chain; the
inspected AYMod BFRT does not enable that key.

## AYMod naval weapon configuration

AYMod authors `AntiNaval=yes` and `AntiUnderwater=yes` on Sapper/SapperE.
The inspected actual NPAE174 Weapon ReadINI772080..7729E2, including the
772901/B937F8/B96950/B96AC8 and7729AA/B94D06..B94EB1 detours, does not read
these keys. Do not invent default values, retained fields or a new naval policy
from those names. This conclusion covers the inspected executable/reader chain,
not arbitrary external runtime injection or another patch generation.

TANY's real `NavalTargeting=4` already uses the shared stock
SelectNavalTargetingWeapon6F3820 owner:6F38FC..6F3915 picks secondary for a
non-Hover, non-Organic naval target; Hover/Organic follows primary. The ordinary
and elite AY TANY Attack commands on Return Dubai fire Sapper/SapperE at HYD
and damage it through the existing simulation. Underwater/cloak sensing and GUI
admission remain separate validation; the two unused keys do not bypass them.

## NPAE aircraft AACombat

`AACombat` belongs to AircraftType, not WeaponType. NPAE174's constructor
`41C8C3..41C8DD` clears the range containing byte `+E4C`, so it defaults false.
The Aircraft ReadINI hook `41CD68 -> B956C0`, at `B95875..B95889`, reads the
exact Rules key with `ReadBool5295F0` and the retained current value. VERA
retains it across physical Rules passes, registry saves and config hashing;
other type families and non-`npatch` reads do not enable it.

The active `AircraftAI414D36 -> C27C60` hook returns false for enabled types
instead of calling the target's `IsHighFlying`. This bypasses only the
per-frame airborne-target removal; the following cloak/alliance/sensor check
still runs. The existing `techno_ai::drop_airborne_target_step` owns this
policy. It does not change acquisition, pursuit, weapon selection,
`GetFireError6FC705..6FC73C`'s projectile AA requirement, or
`AircraftAssignDestination41AA91..41AAAE`'s airborne object-destination refusal.
Actual AY ORCA Attack commands expend ammo and damage both a still-airborne
ZEP and a ground MTNK. Long-range moving-target interception, elite air fire,
GUI interaction and sustained AI tactics remain separate acceptance coverage.

## NPAE garrison protection and penetration

NPAE174 reads `BunkerProtectionLevel` into TechnoType's byte `+0x5A3`
(`0x7138C7 -> 0xC2EF58..0xC2EF6C`) and `BunkerPenetrationLevel` into
WarheadType's byte `+0x14C` (`0x75D5DE..0x75D5F2`). Both start at zero,
retain the current byte as the per-pass ReadInt default, and truncate without
clamping. The protection constructor is `0x710F8D -> 0xC2EF82`; the warhead
constructor zeroes the byte at `0x75CF83`.

The latter **replaces**, rather than accompanies, the stock `Fire` reader.
With `npatch`, VERA follows the NPAE reader even in a mixed-capability build;
`Fire=` does not overwrite that byte. The existing destroyable-cliff action
(`0x70091A..0x700924`) still interprets the same byte as nonzero. Without
`npatch`, the stock `Fire` bool reader and ordinary cliff action are unchanged.
There is one retained byte, not independently mutable Fire and penetration flags.

`BuildingClass::ReceiveDamage` calls the new hook at
`0x442358 -> 0xC2E7C4`, after the disallowed-self-damage return and contact
snapshot, before LaserFence, Immune, Health0 and the shared damage gates.
It does not require positive damage, and it does not replace the building's
ordinary damage. Only the building's garrison Occupants vector participates:

- Nonnegative penetration strictly greater than protection removes one occupant.
- Negative penetration negates its byte and, when strictly greater than
  protection, removes all occupants. Both comparison operands are signed bytes;
  `-128` wraps back to `-128` and never penetrates.
- Equal levels do not penetrate. In AYMod, `1` cannot bypass protection `1`,
  while `-2` clears it. Protection defaults to zero, not a retail tuning value.

The helper `0xC2E751` always selects the last live Occupants entry. It plays
the victim type's first `DieSound` at the building's raw Location (`0xC2BBBD`),
then calls `Record_The_Kill @ 0x702D40` and `FootClass::UnInit @ 0x4DE5D0`,
and finally reduces the captured vector tail. It writes no HP, spawns no
InfDeath animation and does not reset the firing cursor. VERA reuses the
existing sound boundary, kill accounting, UnInit and deferred-removal owners.
There are no direct RNG draws, timer writes or new detach operations in the
hook; nested sound/RecordKill/UnInit callbacks retain their existing semantics.
Red-health ejection and empty-civilian-building ownership stay with their
ordinary later owners, not an extra penetration-specific state machine.

Bio Reactors' `InfantryAbsorb`/`UnitAbsorb` entry uses Cargo `+0x114`
(`0x51A38A` / `0x73A2E4 -> 0x4733A0`), not Occupants `+0x684/+0x694`.
Consequently AYMod's YAPOWR/YAPOWRAI protection entries do not make this hook
kill absorber Cargo. Normal building damage/destruction can still affect it.
Tank Bunkers keep their independent unit link and stock `PenetratesBunker` gate.

Safety residuals: a missing warhead keeps the existing safe-drop policy rather
than emulating a null-pointer crash. If a kill callback removes additional
occupants, tail finalization never grows an already shortened vector to revive
native stale references; arbitrary custom-trigger dangling-pointer behavior is
not claimed equivalent. The frequency of these exceptional custom-trigger or
invalid-reference inputs is not certified; divergent continuation is limited
to those error paths. Ordinary positive/negative penetration, protected
garrisons, kill credit, synchronous first-DieSound ordering, absorber separation
and actual AYMod sniper/Boris-sortie paths have production regressions.

## AYMod reward and control policy

With `npatch`, `TrainPassengers=yes` gives an open-topped shooter's kill XP to
the trainable passenger until it reaches elite; elite or untrainable passengers
retain the carrier fallback. An Aircraft with a trainable SpawnOwner and no
temporary controller keeps its ordinary award and additionally trains its parent
(`RecordKill702FF0 -> C2C49F..C2C50F`); this is not a replacement for the child's
own XP. The controller arm has a different continuation and does not add this
parent bonus. Ordinary untrainable missile-to-launcher credit remains with its
existing recipient selection.

Live managed children do not mirror each kill award immediately. The parent's
normal Techno AI rank-cache change samples the shared spawn-promotion owner
(`6FA13E -> C2D825/C2D851`): non-Infantry parents with more than one spawn slot
raise weaker trainable children, without lowering stronger ones. The literal
zero/one-slot skip at C2D85F..C2D865 remains. Regenerated trainable children inherit
the parent's veterancy through the existing spawn owner. These are distinct
reward, promotion-sampling and spawn-construction paths, not another XP store.

`[General] MindControllerGainExperienceRatio` shares the existing XP award with
a valid, non-elite, positive-cost controller of the original killer
(`RecordKill702FF0 -> C2C208`, `EDI+2C0`), not of a redirected carrier,
launcher or garrison recipient. `100%` transfers it all; unavailable or elite
controllers leave the award with the ordinary recipient. The ratio is
bounded to 0..1, with integer points conserved rather than duplicated.

The killer's `GetsBounty`, `VeteranGetsBounty` and `EliteGetsBounty` select bounty
eligibility by its rank before the kill. Payout is the victim's effective cost
times its rank's `[CombatDamage] BountyCostMult` family and its own rank's `Bounty`
family. VERA's policy defaults are 25/50/100 percent and 1 respectively; these
are not established native defaults. In NPAE174, initializer B9904D..B99065
writes global factor float bits 3EA8F5C3/3F000000/3F400000 (about 33/50/75
percent), and reader B980C9..B9811D passes each retained float as the
ReadDouble5283D0 default before storing it back to float. AYMod explicitly
authors 25/50/100 percent, so this constructor difference alone does not
establish an incorrect payout in its ordinary root configuration. Higher-tier absent
per-type values inherit the preceding tier. Multipliers are bounded to 0..16.
Only hostile terminal kills pay, once through the existing KillRecord and
Economy owners; capture bookkeeping and friendly fire cannot farm cash. The
balance cannot overflow, and registered `BountyAnim` uses the existing AnimClass
owner at the original killer's GetCoords, not at the victim. This coordinate
source follows NPAE174 ReceiveDamage702572 -> B95D16, B95D99..B95DA9 and
AnimClass construction B95DC4..B95DEA. Existing payment timing and arithmetic
remain the policies described here, not a port of the full native damage hook.

`IsPermaControlled=yes` MindControl warheads retain ordinary capture admission
but commit directly through the existing permanent-control owner, without a
temporary manager link. Controller death therefore cannot release them. The
combined MindControl/Parasite LimboLaunch boarding attack consumes its spy on
success; refused or missed boarding uses the existing parasite release path
instead of leaving an invisible tracked infantryman in limbo.

These are deliberate player-facing compatibility policies, not a claim that
unexamined NPAE #174 formulas and callback ordering have been reproduced exactly.
Without `npatch`, the added reward/control keys remain disabled.

## Manual building power

`[UISettings] UseTogglePower=yes` enables manual power with the `npatch` capability.
The input owner reads the immutable startup-selected Rules root; missing values default
to false, and mode/map overrides cannot silently enable this process UI feature.
The NPExt174 `ReadBool` site is `0xC2A362..0xC2A377` (`0x5295F0`); root-only timing
and the false fallback are explicit VERA policy where the original global lifetime is
not fully established.

The default key is V, unless already assigned to another command. `TogglePower` can be
rebound or explicitly unbound in the keyboard settings. It arms a sticky power cursor;
click an own, visible, live building with `TogglePower=yes` to change its online state.
Right-click or the key again exits; Repair, Sell, Beacon, placement and world replacement
clear the competing cursor modes. Losing the local house or its last building also exits.
Entering Power cancels an armed superweapon cursor; entering Planning clears Power,
and V does not interrupt a held Planning gesture.
Power producers without a powered/drain requirement and Firestorm walls are refused.

Hover and release share admission and queue the existing `SetBuildingOnLine` command.
There is no client-owned copy of building power state: `GoOnline/GoOffline`
(`0x452260`/`0x452360`) retain the authoritative latch, power invalidation, factory
pause/resume, Stuff and EVA callbacks. This path is not a power bar click shortcut,
and the transient targeting mode is intentionally cleared, not restored, on saved-world load.
Hover, release and the writers share the core admission query. Active EMP refuses
going online; switching off remains allowed and never clears the EMP counter.

## Independent AI land/naval choices

AYMOD's actual NPAE174 calls `C2CEE9` from House update `4F90F2/4F9252`: land then naval.
`4FEA70 -> C2BFE0` selects House+5650 or +160F8; `C2CF04` filters TaskForce need by Naval,
and `4FBDAF -> C2CE9F` selects that slot for the producing building. The existing private
HouseAiUnitChoices owns both retained indices under npatch; stock builds retain one vehicle slot.
Both passes take the original harvester arm; `C2F028` rejects a Naval mismatch and returns,
without falling through to team demand. Exit `C2BE2A` and limit `C2D039` clear the matching slot.
Abandon `4CA0B1` still clears land even for a ship, and `4FE0A3` reset and mode2's empty/buildable
checks still cover only the original three choices. These native asymmetries are intentional.
The same Scenario stream supplies the extra naval chooser's draws, in native land/naval order;
no new timer or Detach policy is introduced. Save/hash follow the existing House state owner.

Residuals: the NPAE tied-need candidate reset at C2B479..C2B481 is not yet ported, so the stock
picker's candidate set/random range can differ. `DisableParallelAIQueues`'s optional counters
are separate; the target Return Dubai Battle layers leave that key absent and native default is false.
Where NPAE174 and NPSE 2.0 disagree, VERA keeps the NPAE174 behavior required by AYMOD: in
particular, abandoning a naval product still clears the stock land choice.
The dual-slot regression is not a claim that the full AI or its attack strength is now equivalent.

Pure combat naval TaskForces deliberately bypass the two-base zone gate:
TeamType derive6F1FA0/6F1FE7 clears its relation flag for any Naval member with
Passengers=0, and AITrigger41FEEE returns true for that flag. The actual AY
FFFFFFFE-G retained metadata is covered by the Return Dubai reachability test.
Do not add a same-water-region team-creation restriction as a supposed NPatch
fix for its later action53 gather stall; map menu tags do not choose a new mode INI.
## Multiplayer dropdown colors

Startup `52D144 -> B98800 -> C2ADE9` reads root RULESMD[UISettings] UseDropDownColors with
ReadBool/defaultfalse. The processed root retains it; scenario overrides do not change this global.
When enabled, `C2AE00..C2AE52` builds lobby swatches from MPC01..16 HSV. Session conversion
`69A310 -> C2AEF0` resolves MPC(priority+1), with -2 selecting MPC17, and falls back to the
stock table if the name is missing. All match/lobby/loading/chat consumers share that resolver.
Without npatch or with the switch off, stock mapping and swatches remain unchanged.

Current scope is the existing eight Skirmish choices, not the full 16-color UI: in AYMOD,
MPC03 is white and MPC05 is blue. WOL/cooperative stock-only screens and the tech slider remain
separate UI work. Rules/launch/tint regressions do not replace the pending player GUI acceptance.

`[General] DisableParallelAIQueues=` defaults to no and requires `npatch`.
The native reader preserves the current byte (`0xC30303..0xC3031A`); the Rules
constructor zeroes it (`0x66569B`). With yes, Factory_AI reserves one of five House
production counters before compatibility, allocation and construction (`0xC2F07C`):
aircraft, land vehicles, naval vehicles, infantry and buildings. Only vehicles split
by `Naval=`. The existing House AI production owner saves and hashes those counters.

Mobile exits release at their native class admission prefix, before later placement
can fail; buildings release after successful Unlimbo (`0xC33940`, `0xC2F184`,
`0xC2F1A3`). Abandonment releases the held object's House before refund (`0xC2EFB4`).
Failed starts and PointerExpired retain their reservation. Releases run independently
of the switch and clamp signed negative counts to zero, matching the original hooks.

## Cross-extension compatibility

The three capabilities — `ares`, `phobos`, `npatch` — are orthogonal and mixable:

```sh
cargo run --release --bin vera20k --features ares,phobos,npatch
```

Every implemented feature must keep working when the other capabilities are compiled in. A chain
may depend on another **capability's own** state (for example a Phobos key that only a Phobos type
reads), but it must not assume the other features are absent, and it must not re-own state another
capability already owns. When two extensions touch the same native site, one of them wins by native
precedence and the other keeps its own owner — the Phobos radiation activation replacing the retail
one, with the retail substitution kept for builds that lack the capability, is the pattern to
follow.

The build capability is recorded per combination: a save from one combination cannot load in
another, so a mixed build never reads a save written by a single-extension build.

## Superweapon lighting reader and runtime

NPAE174 SuperWeaponType constructorB97600 zeroes the extension. ReadINI6CED28
enters B97080 after AuxBuilding. First initialization shares byte+126 across
all stock kinds: Type0/2/7 seed their seven integers from Scenario+3560/+3548/
+357C; Type1/3/5/6/8/9/10/11 initialize without a lighting seed; Type4 retains
its inverted C288D0 gate. Changing Type in a later layer must not reset the gate.
The ordered ProcessedType is the sole retained reader; runtime definitions only
receive its projection. The initialization field is mandatory on registry restore.

Normal FullInit6851F0/RulesReset6686C0 recreates the registry before map lighting
is parsed at687853. Even a map-introduced SW therefore seeds from Scenario reset,
not final map effect profiles. The six reset values have one shared constant owner
for the new reader and Scenario defaults. Storm's seventh copied dword crosses
into the following Nuke Ambient, so its default rate is200, not1.

Common B970C0 reads LightAmbient/Red/Green/Blue/Ground/Level/
LightAmbientChangeRate as retained native integers, then Lighting as a retained
bool, even when disabled. No map-double scale or clamp is applied.

Launch now binds separate last-Nuke/Storm/Dominator identities and immutable
definition snapshots before admission (6CDA5B/6CCD33/6CCDB1), without changing map
profiles. Scenario UpdateLighting53C280, LightConvert555E50 and Cell484473 retain
their different selection priorities and fall through disabled Lighting arms.
NP full relights use literal Ground/Level; status5 restores the global tint but
keeps active-Dominator cell/constructor selection until the ambient fade ends.
Actual layered AY launch, full resource save/restore and a second launch are
exercised through physical TMP GPU output in
`aymod_dominator_lighting_reaches_terrain_gpu_after_load_and_restores`.
This is an isolated terrain consumer, not full-window/native pixel equivalence.
The55B3D6 rate branch is nuclear phases or TAction127 chrono screen, not storm
and not Chronosphere SW; it reads the last nuclear profile even for that screen.
Storm alone uses the ordinary Rules rate; Dominator's rate ignores Lighting.
A screen with no previous nuclear identity has an unproven null-pointer domain;
do not silently attribute a Chronosphere profile or native crash policy to it.
