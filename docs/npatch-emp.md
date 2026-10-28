# NPatch EMP compatibility

This is VERA's player-facing NPatch compatibility design, not an instruction-for-
instruction port of NPAE #0011 Ext #174. It requires the `npatch` capability;
retail builds keep their existing behavior. It does not translate MOD files to
Ares syntax or enable unrelated Ares gameplay.

## Delivery and duration

`EMEffect=yes` uses the existing retained Warhead field. Each actual detonation
delivers EMP before ordinary damage. For zero-spread hits only the actual object
target is affected. Positive `CellSpread` uses the existing deterministic ground
collector and unique-object ordering. `AffectsAllies`, invulnerability and zero
armor interaction are honored.

Positive weapon Damage supplies the duration in simulation frames, clamped to
1..900. Zero Damage defaults to 90 frames; negative Damage gives a one-frame
pulse, not an undocumented repair operation. A repeated hit refreshes to the
greater of the remaining duration and the new pulse, rather than adding time.
These bounded defaults are explicit engineering choices, not asserted native
NPAE arithmetic. In a combined Ares/NPatch build an explicitly configured
nonzero `EMP.Duration` owns delivery instead, so no hit delivers twice.

Mechanical ground vehicles and powered buildings are susceptible. Ordinary
organic infantry/vehicles are immune; cyborg infantry may be disabled. Aircraft,
jumpjets and rockets are not forced out of flight. Falling objects, construction,
selling/unloading transitions and war-factory contacts are not interrupted.
Team membership, harvesters, mind control and transport managers do not silently
make ordinary mechanical targets immune.

The private `EmpState` remains the single counter/lifecycle owner. In pure
NPatch builds, a first accepted Building or ordinary ground Drive hit selects
the native lifecycle below; that policy is retained for the whole positive
counter lifetime. Positive-height/special locomotors retain their separate
boundary. Reclassifying an already-live legacy EMP on
landing would mistake its synthetic offline flag for permanent provider loss.
A boundary regression reproduced that failure before this policy fix.
The private lifecycle enum also retains a factory-only power-recovery wait;
it does not replace the provider latch. This enum changes pure-NPatch snapshot
layout: old layouts are rejected by the existing guard, not force-loaded or
rewritten. Same-build snapshots retain the lifecycle and pending paid head.

The selected Drive path does not acquire the provider offline latch, deselect,
save or replace the mission, or PowerOff a ground vehicle. It clears the base
Foot destination, clears non-permanent disguise, stores the counter, invokes
existing Capture/Spawn callbacks and clears the class target in native order.
Control/fire/movement gates consume EMP independently. At expiry, the provider
latch gates sound and PowerOn; no Reactivate/Guard/Hunt recovery is inserted.
The Anim store owns every attached sparkle: each admitted hit creates one,
and expiry ends all matching owner/type loops. No NP retained-ID list is added.

The selected Building path stores the counter, invokes Capture/Spawn callbacks
and clears the combat target without Deactivate, ResetOrdersToGuard, radio/Cargo
reset, provider-latch or Stuff writes. It skips the Foot sparkle constructor and
its RNG draw. Hospital Repair consequently continues under EMP, including new
admissions while the independent online latch remains set. Expiry visits the
existing Building animation power-loss owner and dirties radar when applicable;
it does not use Foot activation or the global Foot EMP-animation loop.

Other locomotors and Ares/mixed builds retain their explicitly bounded legacy
lifecycle, including their existing player-friendly Stuff restoration policy.
This does not certify those remaining paths as NPAE equivalents.

EMP transitions also notify the existing House factory-queue owner. An affected
factory remains physically present, so its paid product and FIFO survive; only
online searches reject its live EMP/deactivated state. With no other eligible
online producer, production takes an automatic system hold. Expiry or later
power recovery resumes that same head, but never overrides a player hold or a
manual offline latch. A native factory whose counter expires while low-power
or manually Off retains `NativeFactoryAwaitingPower` in the EMP owner. Shared
factory admission consumes that cause; the next operational head clears it
and notifies the same queue owner, without a fake provider latch or mission
reset. Ordinary low-power factories that were not EMP-held keep their existing
production rules. Construction yards update both Building and Defense slots.
This is an explicit playable-compatibility policy shared with Ares, not a claim
about NPAE's queue callbacks.

## Movement admission and native lifecycle residuals

Drive Move_To4AFD4B and Ship Move_To69F45B now consume the shared positive
Techno70EFD0 EMP counter before paralysis, warp tests and destination stores,
including null coordinates. Earlier Foot NavCom and timer writes still stand;
Stop_Moving is a separate entry and remains callable. A production EMP hit,
reissued Unit destination and ordinary object-AI expiry regression covers Drive.
The Ship branch is supported by its original instructions and existing normal
movement coverage, not a new positive-EMP Ship scenario.

Fresh-track admission now consumes the same positive counter at
Drive4B2746/Ship6A1D96, after warp/missile checks and before path/cell work.
An explicitly retained-route prestate plus a real NPatch hit proves these
consumers reject both locomotors; this is not first-hit route survival evidence.
Walk Move_To75ACBD also refuses the counter. Its ordinary Process keeps the
existing <17-lepton completion first, then75BF87..75BFA6 clears motion animation
and the scold latch without clearing the head/destination or IsMoving, before
any paid displacement. A real cyborg hit covers the16/17 boundary and expiry.

This does not finish NPAE174 EMP movement or recovery. Hover consumers and
existing active Drive/Ship track tails still need their own evidence, rather
than a blanket freeze. Unit746C90's death-frame override remains unrepresented.

## Native Building and ground Drive lifecycle: implemented boundary

Actual NPAE174 instructions establish independent provider latch `Techno+1C8`
and EMP countdown `+504`. Pure NP admitted Buildings and ordinary ground Drive
share one counter transaction, with their native class branches:

- `B9517D`: reject only an existing stronger timer. Equal admitted hits repeat
  side effects; the upstream 90/default/clamp/target policy above is unchanged
  and is NOT native receiver arithmetic. Weaker hits do not allocate or draw.
- `B95185..B951C6`: Building RTTI6 bypasses the Foot prelude. Radar calls
  House508DF0 BEFORE the incoming counter store; a fresh refresh therefore
  sees the old zero, while an immediate equal rehit sees the live counter.
  C2BCB8 skips non-Radar or EMP-disabled providers through508F08 and continues
  in House acquisition order; it must not hide a later healthy radar. The
  subsequent eligible provider's warp result retains its existing semantics.
- `C2EFB7`: a new attached EMP Anim per admitted Foot hit, one Scenario `[0,25]`
  constructor-delay draw, loop1/flags600, existing GetCoords/Anim constructor
  and SetOwnerObject owners. Constructor-owned draws stay with those owners.
- `B990DC..B991DF`: direct base Foot4D94B0 null destination (not Unit override
  or ResetOrders), non-permanent Unit746720 disguise clear, combined-disabled
  first sound, incoming counter store, Capture FreeAll472140 then Spawn
  KillAll6B7100, class target clear. Ground Unit height<=0 bypasses PowerOff.
  No direct selection/mission/Archive/provider-latch write or detach is added;
  existing setters/managers own their timer changes and transitive detach.
- Building enters directly at `B9914C`: incoming counter, Capture then Spawn
  callbacks, then Building443B90's null-target call into Techno6FCDB0. It does
  not erase an admitted Hospital Repair task. BuildingAI43FE56 reaches
  Techno6FA655/MissionAI5B3060 before countdown; Hospital44B870..44BA79 has no
  EMP gate. Pausing all service dispatch would differ from this native path.
- Provider loss still uses Deactivate70FC90: reset/deselect, Foot PowerOff and
  provider latch set. NP Reactivate70FBE8 always resets orders; C2BC8B skips only
  PowerOn/sound during nonzero EMP, while70FC5A still clears the provider latch.
  Ares/mixed admission is isolated and unchanged.
- TechnoAI6FAF0D/C2BB3E decrements a positive counter, storing only for HP!=0.
  Expiry tests the computed result even when HP0 skipped the store. C2BC5D/6E
  checks provider latch for Foot activation sound/PowerOn, without Hunt/order reset.
  6FAFB9..6FAFFB sets loop remaining0 on ALL matching attached EMP anims; normal
  animation boundary lifecycle, not immediate UnInit, ends them.
- Building expiry `6FAF2A..6FAF81` instead invokes4547C0 through the existing
  `apply_building_anim_power(false)` owner, then sets Radar House+5779 for a
  later assessment and returns before the Foot suffix. Building4549B0 retains
  ownership of operational slot edges; no unconditional Reactivate/Guard or
  generic EnableStuff is substituted at this boundary.
- UnitAI7360C9 calls Techno70FBD0, whose NPAE7E0F9D disjunct is provider-offline
  OR nonzero EMP. The shared query now feeds both first-hit sound and disabled
  Unit sparks. Keeping only the provider flag here would drop the existing
  two Scenario `[-100,100]` draws on spark frames after separating the states.
  The regression reuses an existing executed robot_offline_oracle spark vector.

Three new production tests cover accepted/equal/weaker hits, one sparkle-delay
RNG draw, multi-Anim expiry, preserved selection/mission/power, provider return
during EMP, overlap save/load, no-provider expiry and legacy-to-ground rehit
policy stability. Two primary tests and both later boundary controls failed
before their respective fixes. The actual AY TTNK→APOC battle/ordinary expiry
scenario also passes; this is not the actual DRON provider-overlap scenario.
Actual AY GAROBO/NARADR coverage additionally starts a running Repair, delivers
the configured TankBoltEMP child through the impact-producer seam, saves during
EMP, sustains the50-frame pulses long enough to observe healing, then covers
expiry, another admission during a pulse and its saved completion. This is not
an actual TTNK hospital sortie. Synthetic boundaries cover two radar providers
and a saved factory-only power wait without provider deactivation. Re-routing
the ordinary Building to the old legacy backend makes the running Repair become
Guard; the production regression rejects that loss.

Residuals: duration/target/receiver preprocessing remains compatibility policy;
positive-height/lift/locomotor changes during a live timer keep its initial
policy, not complete airborne semantics. Walk/Ship/Hover, special Building
height/type transitions, all linked-manager reentrancy combinations and
parasite/tether cases need their own closure. Building sparkle removal changes
the old incorrect RNG stream; animation-power callbacks keep their own draws
and detach semantics. Building EMP rendered output and multi-AI long matches
are not established by the hospital logic tests.
Nonempty Rules ActivateSound/DeactivateSound fallbacks remain unresolved; AY's
root globals are empty and the three drone types supply their type sounds.
Do not mark V01/A06 or the entire NPatch EMP complete from these bounded paths.

## Provider-only Drive motion: native/design conflict

The original NPAE also admits a fresh AI Move for an ordinary ground, unattached
DRON with provider latch1 and EMP0: Team6EBE0D..56 dispatches Unit741970,
its ordinary path reaches Foot4D94B0 then Drive4AFD4B. Unit746C90/70EFD0 asks
EMP/death-frame, not provider. Unit7360C9's spark arm rejoins7361A9; Foot4DA86E
still processes locomotion. Fresh-track4B2746 and speed70EFE0 likewise do not
add a provider veto. This is a bounded positive call-chain trace, not a native
playback or proof of legal offline attack/parasite admission.

AY's `完整说明.txt` says each unmanned unit needs its designated building and
power, otherwise it is paralyzed; it specifically says DRON needs Radar to act.
That intended design differs from both the original shared House center count
and its provider-only Drive motion admission. The reported symptom is not closed
by the EMP fix. Independent three-pair supply and no offline drone movement
would be explicit compatibility corrections, not claimed native behavior;
retain the pending policy decision instead of silently applying a global freeze.

## AYMod's Tesla projectile

AYMod authors `TankBolt -> ElectricbounceT` (elite `TankBoltE -> ElectricbounceTE`)
with `Airburst=no`, `Cluster=1` and `AirburstWeapon=TankBoltEMP`. The child weapon
uses `EMPulsWH` with `EMEffect=yes`. Stock Airburst requires `Airburst=yes`, so it
previously discarded this producer before EMP delivery was possible.

Stock Rules processing also discovers that child only in the Bullet sweep,
after the Weapon sweep: a child referenced nowhere else retains constructor
fields until a later pass. NPatch compatibility eagerly discovers this impact
child while its parent's Weapon is being read. The existing Weapon factory and
same ordered sweep then read the new child, including its own Projectile and
Warhead. No second body reader or duplicate registry is introduced. This loader
ordering is also an explicit compatibility choice, not asserted NPAE chronology.
Discovery follows the retained Projectile pointer on later passes too: changing
only the projectile's AirburstWeapon cannot strand a new child as a stub when
the parent Weapon/Projectile entry is omitted.

The NPatch compatibility policy emits one real child projectile after the
parent's shared impact receivers, preserving the live source and original target.
It uses the existing projectile allocation, launch, admission and collision
owners. It is not the stock nine-neighbor fan. Normal `Airburst=yes` behavior and
the existing Ares unsupported-fan boundary are unchanged; mixed builds accept
only this explicitly selected NPatch impact-child path.

## Validation boundary

Focused regressions cover the actual parent-impact/child-projectile/EMP chain,
refresh/expiry and ownership change, and organic/allied immunity plus powered-
tower disable/recovery/destruction. A whole AYMod battle, airborne EMP, precise
NPAE duration/mask semantics, all linked-building manager effects and combat
balance are not implied by these checks. This deliberately improves playable
coverage without claiming native parity for those mechanisms.
