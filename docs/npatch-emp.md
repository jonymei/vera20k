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
NPatch builds, a first accepted ordinary ground Drive hit selects the native
lifecycle below; that policy is retained for the whole positive counter lifetime.
This is a compatibility boundary while positive-height/special locomotor and
building paths remain separate. Reclassifying an already-live legacy EMP on
landing would mistake its synthetic offline flag for permanent provider loss.
A boundary regression reproduced that failure before this policy fix.

The selected Drive path does not acquire the provider offline latch, deselect,
save or replace the mission, or PowerOff a ground vehicle. It clears the base
Foot destination, clears non-permanent disguise, stores the counter, invokes
existing Capture/Spawn callbacks and clears the class target in native order.
Control/fire/movement gates consume EMP independently. At expiry, the provider
latch gates sound and PowerOn; no Reactivate/Guard/Hunt recovery is inserted.
The Anim store owns every attached sparkle: each admitted hit creates one,
and expiry ends all matching owner/type loops. No NP retained-ID list is added.

Other locomotors, buildings and Ares/mixed builds retain their explicitly bounded
legacy lifecycle. Buildings keep paid-head pause/resume, manual online intent
and later operational recovery, including the existing player-friendly Stuff
restoration policy. These mechanisms are not newly certified as NPAE equivalents.

EMP transitions also notify the existing House factory-queue owner. An affected
factory remains physically present, so its paid product and FIFO survive; only
online searches reject its live EMP/deactivated state. With no other eligible
online producer, production takes an automatic system hold. Expiry or later
power recovery resumes that same head, but never overrides a player hold or a
manual offline latch. Construction yards update both Building and Defense slots.
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

## Ground Drive provider/EMP lifecycle: implemented boundary

Actual NPAE174 instructions establish independent provider latch `Techno+1C8`
and EMP countdown `+504`. Pure NP ordinary ground Drive now uses:

- `B9517D`: reject only an existing stronger timer. Equal admitted hits repeat
  side effects; the upstream 90/default/clamp/target policy above is unchanged
  and is NOT native receiver arithmetic. Weaker hits do not allocate or draw.
- `C2EFB7`: a new attached EMP Anim per admitted hit, one Scenario `[0,25]`
  constructor-delay draw, loop1/flags600, existing GetCoords/Anim constructor
  and SetOwnerObject owners. Constructor-owned draws stay with those owners.
- `B990DC..B991DF`: direct base Foot4D94B0 null destination (not Unit override
  or ResetOrders), non-permanent Unit746720 disguise clear, combined-disabled
  first sound, incoming counter store, Capture FreeAll472140 then Spawn
  KillAll6B7100, class target clear. Ground Unit height<=0 bypasses PowerOff.
  No direct selection/mission/Archive/provider-latch write or detach is added;
  existing setters/managers own their timer changes and transitive detach.
- Provider loss still uses Deactivate70FC90: reset/deselect, Foot PowerOff and
  provider latch set. NP Reactivate70FBE8 always resets orders; C2BC8B skips only
  PowerOn/sound during nonzero EMP, while70FC5A still clears the provider latch.
  Ares/mixed admission is isolated and unchanged.
- TechnoAI6FAF0D/C2BB3E decrements a positive counter, storing only for HP!=0.
  Expiry tests the computed result even when HP0 skipped the store. C2BC5D/6E
  checks provider latch for activation sound/PowerOn, without Hunt/order reset.
  6FAFB9..6FAFFB sets loop remaining0 on ALL matching attached EMP anims; normal
  animation boundary lifecycle, not immediate UnInit, ends them.
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

Residuals: duration/target/receiver preprocessing remains compatibility policy;
positive-height/lift/locomotor changes during a live timer keep its initial
policy, not complete airborne semantics. Walk/Ship/Hover/buildings, all manager
reentrancy combinations and parasite/tether cases need their own closure.
Nonempty Rules ActivateSound/DeactivateSound fallbacks remain unresolved; AY's
root globals are empty and the three drone types supply their type sounds.
Do not mark V01/A06 or the entire NPatch EMP complete from this Drive chain.

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
