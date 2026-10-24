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
can therefore stall construction. VERA still has one vehicle choice slot; NPSE's separate
naval AI choice remains a separate unimplemented mechanism.

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

## AYMod reward and control policy

With `npatch`, `TrainPassengers=yes` gives an open-topped shooter's kill XP to
the trainable passenger until it reaches elite; elite or untrainable passengers
retain the carrier fallback. A `Spawned` child's kills instead train its living,
trainable mother unit, even when the child is itself trainable. Live managed
children mirror that award, and regenerated children inherit the parent's
veterancy through the existing spawn owner.

`[General] MindControllerGainExperienceRatio` shares the existing XP award with
a valid, non-elite, positive-cost controller. `100%` transfers it all; unavailable
or elite controllers leave the award with the ordinary recipient. The ratio is
bounded to 0..1, with integer points conserved rather than duplicated.

The killer's `GetsBounty`, `VeteranGetsBounty` and `EliteGetsBounty` select bounty
eligibility by its rank before the kill. Payout is the victim's effective cost
times its rank's `[CombatDamage] BountyCostMult` family and its own rank's `Bounty`
family. The defaults are 25/50/100 percent and 1 respectively. Higher-tier absent
per-type values inherit the preceding tier. Multipliers are bounded to 0..16.
Only hostile terminal kills pay, once through the existing KillRecord and
Economy owners; capture bookkeeping and friendly fire cannot farm cash. The
balance cannot overflow, and registered `BountyAnim` uses the existing AnimClass
owner at the victim's position.

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
