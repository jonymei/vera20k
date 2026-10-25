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

The private `EmpState` remains the single counter/held-animation owner shared
with Ares. Existing Techno AI tails decrement it, the existing Deactivate and
Reactivate owners stop/resume locomotion, control, firing and deploy admission,
and the existing building operational edges handle powered defenses/providers.
Ownership changes and power-center recovery cannot bypass a live counter. On
expiry the owner reactivates only where power allows, retires the held sparkle
and returns computer-controlled ground victims to the existing Hunt mission.
If EMP expires during low power, a later operational recovery also enables the
existing building Stuff owner, restoring lights and powered animations. This is
an intentional player-friendly correction to Ares's delayed recovery, which
reactivates but leaves Stuff disabled. It does not change the manual online latch.
Lifecycle pointer-expiry/destruction already retires the same held Anim; no new
timer, dead-object sweep or duplicate power state is introduced.

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

This does not finish NPAE174 EMP movement or recovery. Fresh-track admission
(4B2746/6A1D96), Walk destination/paid-step gates and Hover consumers remain
required work; existing heads can require their native movement tail rather than
a blanket freeze. Unit746C90's independent death-frame override is unrepresented.

The current shared Ares-style Deactivate/Reactivate and Hunt recovery remains an
explicit compatibility policy, not the inspected NPAE lifecycle. NPAE's active
Object damage hook5F5505/B95100 reads WH+154 and reaches C2EFB7/B990DC for mobile
victims; that chain clears navigation/target and writes+504 without setting the
provider offline byte+1C8. Reactivate70FBE0/C2BC8B resets orders first, skips
Power_On while EMP is nonzero, and clears+1C8; expiryC2BC5D/C2BC6E separately
preserves the offline latch. Migrating only one end would leave the current
Ares-style application inconsistent. Keep the existing duration/target/queue
policies until the application, movement consumers and expiry can move together.

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
