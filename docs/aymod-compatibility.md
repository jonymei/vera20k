# AYMOD 兼容性清单

更新：2026-10-25。面向“人类玩家与多个电脑对战”的分批适配工作清单。
这是当前唯一的兼容性台账；[Windows 交接](aymod-windows-handoff.md)记录此前交付历史，
不是另一个待办列表。**没有把能启动、能读配置、能创建编队当作整个 MOD 已兼容。**

## 1. 基线、状态和优先级

- 引擎基线：`feature/aymod-integration-resume`，`8675048ba`。不能用 `main` 代替；
  `main` 缺少交接中的 AI、科技树、EMP、切电、经验和永久控制适配。
- MOD 基线：配置仓库 `13f3c25`，根目录正式生效的 `rulesmd.ini` / `aimd.ini` /
  `Uimd.ini` / `mpmodesmd.ini`；原始 AIMD 快照不覆盖维护后的根文件。
- 目标补丁：**NPAE #0011 Extended #174**。NPSE #359、NPExt #173、Ares 的相似功能
  仅能提供线索，不能直接证明 #174 的读取和玩法。没有正面证据要求启用 Ares。
- 要求 `npatch` 构建。此前 Windows 在 `main` 上未带此能力的启动结果，不覆盖本分支。
  审计阶段构建了只读 `asset` 工具；后续适配已开始，见下文进度。尚未替换安装的普通游戏。
- 原始作者说明以 170507 为背景、170601 更新为后续修订；例如尤里收入奖励已被后者取消，
  不能将旧说明的全部描述当当前规则。战役不在本轮目标内，作者也提示其配置冲突。

| 状态 | 含义 |
| --- | --- |
| 缺失 | 找到当前生产路径缺口，且 MOD 有对应使用；仍须在修复时确认原生细节 |
| 待核 | 有用法或作者描述，但 #174 语义、消费者或实际可达性尚未闭环，不等于未实现 |
| 待验 | 已有生产实现，缺本轮所需的 Windows / 实际 MOD 场景验收 |
| 政策差异 | 已知为可玩性选择的兼容政策，不声称原生逐项等价 |
| 配置残留 | MOD 数据自身的异常或闲置引用；不自动归咎于引擎 |
| 已有覆盖 | 交接记载的定向回归/实际场景已通过；覆盖边界必须保留 |

P0：运行了错误基线/构建，或无法进入所需对局。
P1：常见作战机制、电脑发展和出击、单位生存/选敌、经济、确定性。
P2：较窄兵种/模式链、视觉/UI 与原生细节。P3：当前固定触发不可达的残留。
优先级不证明原生等价，低优先级也不是永久豁免。

## 2. 初次审计证据与当前适配进度

本轮读取交接分支、根配置和作者说明，并在 Windows 成功构建：

```powershell
python -m tools.cargo_run -- build --profile fast -p vera20k --bin asset --no-default-features --features npatch
```

配置查询复用生产源选择和 INI reader，不另写有效值解析器：

```powershell
# $asset 是 cargo_run --resolve asset --profile fast 返回的实际路径。
& $asset ini-get Powerups Cloak --domain rules --reader raw --map XMP03T4.MAP --mode-id 1 --ra2-dir D:/Workshop/CnC2Yuri-AYMOD
& $asset ini-get BFRT AlternateFLH6 --domain art --reader raw --ra2-dir D:/Workshop/CnC2Yuri-AYMOD
```

本轮查询选择普通 Battle（ID 1）、`XMP03T4.MAP`。RULESMD 来自维护后的 loose 文件；
LANGRULE 缺省；模式来自 `ra2md.mix -> localmd.mix / MPBattleMD.ini`；地图来自
`multimd.mix`；固定 ART 来自 `expandmd99.mix`；Sound 来自 `expandmd01.mix`。
`raw` 结果是经过生产解析/层次选择后的 accessor 文本，**不是最终类型字段或行为证明**。
表内其他未查询的配置例子是物理用法定位；修复时仍需查询对应场景并追踪 typed reader。
AI 交叉引用是 section/注册项结构清点，不冒称完整 native-exact AIMD 运行。

本轮38项生产查询全部成功（36项Rules、2项固定ART），代表结果如下；这些值不是手写INI解析结果：

| 生产 accessor 查询 | 返回文本 |
| --- | --- |
| `[Powerups] Cloak / Explosion` | `10,CLOAK,yes` / `15,<none>,yes,500`，权重不是百分比；作者所说9%/14%不能直接硬编码 |
| `[TANY]` / `[SNIPE] Cloakable` | 均为 `yes`；VIRUS的VeteranAbilities含CLOAK，IVAN的VeteranAbilities不含CLOAK、EliteAbilities含CLOAK |
| `[ORCA] AACombat`、Sapper/SapperE两项反海军键 | 均为 `yes` |
| `[MTNK] TurretROT` | `9` |
| NABNKR保护、AWPWH穿透、MIGWH穿透 | 分别为 `1`、`1`、`-2` |
| `[PsychicDominatorSpecial] Lighting / LightBlue` | `yes` / `70` |
| ROBO/DRON/CAOS的PoweredUnit及三个建筑的PowersUnit | `yes`；对应 `ROBO` / `DRON` / `CAOS` |
| 固定ART `[BFRT] AlternateFLH5 / AlternateFLH6` | `100,100,160` / `100,-100,160` |

初次审计没有运行 Windows 对战或修改玩法。后续实现阶段已通过现有 `asset extract` 从
正版 Steam 安装提取当前测试所需零售 INI 到忽略的本地 `ini/`，未用 MOD 数据替代。

### 功能适配进度

- F01：已扩展既有 cloak producer/facts 支持 Infantry，复用原有隐形、侦测和持久化状态；
  完成隐形后的 Scatter 仍严格仅限电脑 Unit，不让电脑步兵误走车辆散开分支。
- 先在旧实现复现两项失败：等级隐形步兵不创建运行态；天然隐形步兵普通帧始终停在state0。
  修复后通过 `cloak` 定向覆盖、6项 `sensor_lifecycle` 回归，以及实际AYMOD谭雅
  自主隐形后用真实 DoublePistols/Attack 开火且保持隐形的 Windows headless 场景。
  步兵完整隐形计时状态通过保存/读取检查；没有新增实例状态或第二套计时器。
- 新准入沿用已有 CloakingTick 的 RNG、计时器与 Detach_All 顺序；没有新增随机抽样政策。
  原生证据来自当前NPAE174的 Infantry InitFromType/AI → Foot AI → Techno AI/CloakingTick，
  见 `src/sim/world/techno_ai_cloak.rs` 邻近注释。
- F01独立只读critic未发现本次引入的可确认缺陷；车辆传感器回归不冒称步兵专项传感器实测。
- F05：按当前NPAE174 reader/constructor接入独立炮塔速度。逐物理层保留当前byte默认、
  读后截断到u8，255回退ROT；车体、飞机和建筑的ROT路径不变。复用FacingClass和开火对准门，
  没有新增RNG、Detach或计时器owner，只改变现有炮塔转向的时长。
  旧实现先失败的byte/layer/初始化隔离回归已通过；新增保留状态存读与配置hash回归通过。
  炮塔作战模块7项、NativeProcessing92项及不启用npatch时的隔离回归通过。
  `--profile fast`实际AYMOD地图加载/MTNK Attack场景通过：车体7、炮塔9，持续瞄准开火不重置回7。
  新增保留Rules字段会改变自动计算的存档layout；旧layout存档按既有政策拒绝，不伪造兼容。
- F05独立只读critic未发现本次引入的可确认或具体可能缺陷。
- F13：原生复读纠正了首版假设。NPAE174并非扩大ART容量，而是让第六名乘员按五端口循环，
  即回绕到AlternateFLH0。现已在唯一FLH消费owner实现该循环，不读取闲置的ART5/6，
  不新增RNG、计时器、Detach或缓存状态。
  旧实现先失败的第六端口回归已通过；真实AYMOD六人实际Enter登车后，当前cargo第六/第一名
  的生产发射坐标相同、第五/第一名不同。ArtData46项、FireCoord20项以及无npatch原版越界
  回归通过；既有实际BFRT乘员经验场景仍通过。Sidebar cameo的慢测不覆盖此链，未运行。
- F13独立只读critic未发现本次引入的具体缺陷。无NPatch的40项cloak定向覆盖也通过；
  其5项ignored是未修改机制的Mirage专项，不冒称已运行。普通Windows游戏已用
  `--profile fast --no-default-features --features npatch`构建成功，未启动/安装替换。
- F04：已从当前NPAE174指令确认并接通通用Building ReceiveDamage钩子。正/负穿透不是分摊HP，
  而是保护等级比较成功后直接对尾部驻军RecordKill/UnInit；普通建筑伤害继续执行。
  默认0、current-default逐层读取、i8截断/严格比较、NEG AL溢出、Fire槽别名均已保留。
  核心新增三项回归：读层次/存读/hash；真实Capture入驻后的等级门、零/负伤害、首DieSound同步位置/顺序、
  击杀只记一次与正常回收、Cargo隔离、穿透后红血仅弹出幸存者；实际AYMOD狙击手与鲍里斯完整攻击/空袭。
  三项均已通过；实际AYMOD场景及红血弹出回归在debug与fast优化配置均通过，普通fast+npatch游戏构建通过。
  独立只读critic未发现具体玩法缺陷。其后进一步按原生先后排列两个纯读取，并在最终候选重跑相关覆盖；
  红血回归夹具补齐既有scheduler资源绑定，未关闭正常受损火焰来绕过前置。
  新增字段保存布局自动改变，旧布局仍按既有政策拒绝。
  NP定向覆盖为NativeProcessing93、Warhead9、BuildingOccupantStorage3、GarrisonEject14、
  BunkerLink23、MoveCellInput14、Combat165；Combat的1项VoiceFeedback专项慢测不覆盖本链，未运行。
  无NPatch的读层次/Fire别名1、原有BuildingOccupantStorage2和MoveCellInput14均通过。
  这是F04功能链验收，不是全部MOD或所有异常触发回调/GUI表现认证。
- 扩大驻军相关作战覆盖时，发现一个早于本改动的旧断言：既有FireAt已经提交Report声音，
  但garrison_fire测试仍要求声音为空。已沿未改变的producer/fixture及原生6FF38F查实，
  将断言加强为在正确发射坐标恰好一条GIAttack，未压掉生产声音或跳过测试。
- 没有运行Windows GUI、多AI长局或NPAE原生对照；不将局部通过称为整个MOD兼容。

此前库存的 25,068 个物理 section/key 对、2,374 个不同键、207 个 accessor 查询，
以及相对 Steam 的 47 个字段候选，都不是已兼容机制数量。本轮没有重新认证这些计数。
下面是已定位机制及验证缺口的首版分类；全量键、各地图/模式和无键补丁的发现仍开放。

### 当前玩家测试版与下一链

- 玩家测试版：`aymod-playtest-20261025-f04`，来源干净提交`4f0f5afa6`，
  `--profile fast --no-default-features --features npatch`构建成功；固定保留，不受后续编译覆盖。
  本机启动入口：`D:/Workshop/vera20k/target/AYMOD-test-20261025.cmd`，
  使用主工作树配置和AYMOD资源；未自动启动。新保留字段改变存档布局，请新开对局。
- 明确开放的F项为F02/F03/F06/F07/F08/F09（功能）与F10/F11/F12（视觉/UI）；
  F01/F05/F13已有实现但还需画面/移动验收，F04已有定向功能验收。
  V01–V10以及AI专项仍是核对/验收项，不能计作全部缺失或全部通过。
- V01已开始原生复读：当前AYMOD的Building operational-on调用点
  `0x4549D1..0x4549F3`仍调用`HouseClass::RobotTanksBackOnline @ 0x50E010`；
  该函数先递增House共享`+0x2D8`，只有首次变1才按传入单位类型遍历。
  `LostPoweredCenter @ 0x50E0E0`及`HasPoweredCenter @ 0x50E1B0`同属共享计数链，
  `Unit Per_Cell_Process @ 0x739F21..0x739F59`仍传类型但查询共享计数。
  因此“混有多类时共享计数”本身不能直接判为VERA缺陷或擅改为按类型计数。
  这是原始指令的局部静态核对，不排除运行时补丁；EMP/恢复/换主的调用者和实际三组生产场景尚未闭环，
  V01保持开放，尚无供电玩法改动或新增回归结果。
### 玩家首轮实测反馈（2026-10-25）

- 玩家报告EMP、V手动断电、BFRT六乘员开火正常；这是实际游玩反馈，不扩大为全场景认证。
  F13已有首轮画面/游玩反馈，移动转身及重新登车未被单独报告。
- 磁暴步兵给磁暴线圈充能正常，但这是Overpower链，不是V01的PoweredUnit/PowersUnit三组关系。
- 玩家观察生产建筑也能V断电，与记忆中只限防御建筑不同。当前准入按TogglePower及耗电/Powered判定，
  不以生产/防御分类；是否偏离目标NPAE仍待原生菜单准入和对应建筑配置核对，暂不改行为。
- 狙击手不能清除战斗碉堡驻军：与已确认的AWPWH穿透1、NABNKR保护1的严格大于政策相符，
  不能作为穿透失效缺陷。普通保护0建筑和鲍里斯负2空袭仍是不同验证路径。
- 玩家确认电脑会进攻但攻势很弱。A01/A02新增实战症状，下一AI核查重点是科技/经济、编队补齐、
  生产阻塞、出击节奏和战术执行；地图、模式、难度、对手阵营及存档尚缺，不直接调整权重/平衡。
- F10电弧自定义配色仍未接通；EMP效果正常不代表其视觉配色已完成。医院及经验项需先解释
  实际行为边界，不让玩家重复验证原版全部功能。
## 3. 功能缺口与适配进度

每一行是一条可独立收口的玩法链，不要求合并成一个大改动。

| ID / 优先级 | 类别、触发与影响 | 配置和现有 owner / 证据入口 | 收口要求 |
| --- | --- | --- | --- |
| F01 / P1 / 已实现，待GUI验收 | 步兵自主隐形：已修原先Unit-only准入；天然/等级隐形进入既有更新，影响生存、AI选敌和侦测 | `[TANY]` / `[SNIPE] Cloakable`；`[GHOST]` / `[IVAN] EliteAbilities`、`[VIRUS] VeteranAbilities` 同族。`src/sim/world/techno_ai_cloak.rs` 的事实采集和更新现已允许Unit/Infantry | 已核#174步兵准入并扩展既有owner；剩余实际GUI/运动路径验收包括静止隐形、按实际武器DecloakToFire保持隐形或显形、传感器、死亡/换主和存读档；不要新增第二套隐形状态。IVAN配置为精英CLOAK，与旧说明的老兵矛盾，不擅改配置 |
| F02 / P1 | 隐形箱：启用箱子时，拾取后只有消费/动画，没有应有的实例隐形能力；不是整类单位规则变更 | `[Powerups] Cloak`；`src/sim/crates/pickup.rs:627` 明记 residual，最终落入无效果分支 | 实例能力、半径内对象准入、后续 cloak producer、持久化和清理；与 F01 共用隐形 owner，但分别验收 |
| F03 / P1 | 爆炸箱：启用箱子时不伤害拾取者/周围对象，且少消耗原生爆炸位置随机数 | `[Powerups] Explosion`；`src/sim/crates/pickup.rs:631`、`:640` 明记 actor 伤害、五次偏移爆炸及缺少 RNG draws | 核 #174 是否改了 stock 箱子；伤害/爆炸消费者、五次偏移的随机数顺序、死亡/替换箱生命周期；确定性必须一并闭环 |
| F04 / P1 / 已实现，功能验收通过 | 驻军保护/穿透：通用建筑收伤前置钩子；正等级尾序单杀、负等级尾序清空，严格大于保护值 | 两个有符号byte逐物理层读取；WH穿透复用原Fire的+14C字节，不建立竞争状态。`src/sim/passenger/garrison_eject.rs`复用声音、RecordKill/UnInit owner；`src/sim/combat/world_receiver.rs`在自伤门和联系人快照之后接入 | 已通过真实AYMOD狙击手攻击普通/保护驻军、鲍里斯召唤BPLN/Maverick3清空保护驻军；生化炉Cargo不属于此钩子，坦克掩体保留独立链接/旧PenetratesBunker门。优化场景、普通游戏构建和独立只读critic均已完成；不冒称GUI或全地图模式认证 |
| F05 / P1 / 已实现，待GUI验收 | 独立炮塔速度：已按NPAE174分离Unit炮塔与车体，影响跟踪、射击时机 | MTNK、FV、SREF、ROBO、HTK、HTNK、APOC、HORV、HARV、TTNK、TTNK2、LTNK、YTNK、TELE 共14处物理用法；`src/rules/native_processing.rs`保留byte，`ObjectType::unit_turret_rot`为共享速率入口；`src/sim/world/world_spawn/construction.rs`初始化、`src/sim/world/unit_post.rs`持续瞄准消费 | 已核#174 reader/构造消费者并通过原地Attack；行进跟踪与实际画面待验，沿用旋转owner，不拟合速度 |
| F06 / P1 | 飞机对空行为：ORCA 类型的扩展 `AACombat` 未接通，不能靠弹体 `AA` 替代 | **`[ORCA] AACombat` 是 AircraftType 键，不是 WeaponType**；`src/rules/object_type.rs`、飞机 attack/weapon-selection owner | 核 #174 飞机任务/追击/武器准入，先用 ORCA 完成对空与对地两条对照；不能以别的飞机描述推定其行为 |
| F07 / P1 | 武器海军/水下政策：Sapper/SapperE 的 `AntiNaval` / `AntiUnderwater` 无对应 NPAE 消费链 | `[Sapper]` / `[SapperE]`；`src/rules/weapon_type.rs`、`src/sim/combat/combat_weapon.rs`；既有 projectile AA/AG、海陆规则不是这些键的实现 | 先追实际使用者与 #174 语义，再验地面、船、水下目标及拒绝边界；与 F06 分开 |
| F08 / P2 | 爆炸政策：`IsFlatExplosion` 尚无匹配的 NP 输入/效果政策，具体损害仍待原生确认 | BlimpHE、KTSTLEXP、CRNUKEWH、CRTerrorBombWH、DemobombWH；`src/rules/warhead_type.rs`、combat 收伤/爆炸 owner | 必须先定原生语义，不能只凭名字假定为地形高度忽略或平面伤害 |
| F09 / P2 | 敌我过滤：`DoesntAffectEnemies` 没有对应 NPAE reader/消费者 | `[TankSnapshot]`；`warhead_type.rs:459` 读的是另一个键 `AffectsEnemies`；Ares 同类过滤也不等价 | 追发射者/受益者/目标 House 和效果类型，覆盖敌、己、盟；不直接把值取反映射到另一个扩展 |
| F10 / P2 | 电弧自定义配色：武器自定义三色没有传到最终绘制 | TankBolt、TankBoltE、TeslaFragmentT 的 `EBoltColor1/2/3`；`src/sim/combat/electric_bolt.rs:18` 只传 alternate_color 等字段；`src/render/electric_bolt.rs:25` 仍从 palette 10/5/15 取色 | reader → effect birth → renderer，全程使用既有电弧 owner；实际 TTNK 普通/精英攻击画面验证，不调截图拟合 |
| F11 / P2 | 心灵终结器独立光照：超武 section 的 NP 光照覆盖未接通，不等于 stock 全局光照缺失 | `[PsychicDominatorSpecial] Lighting / LightRed / LightGreen / LightBlue / LightAmbient / LightGround / LightLevel`；`src/rules/superweapon_type.rs`；`src/sim/superweapon/psychic_dominator.rs:8` 当前使用 map 的 Dominator profile | #174 启动、叠加/互斥、渐变和复原；实际输出验证，覆盖取消/结束/存读档，不污染下一次超武 |
| F12 / P2 | 扩展遭遇战 UI：科技等级滑条与扩展颜色未按 NP 配置进入 UI | `[UISettings] ShowTechLevelSlider / UseDropDownColors`、`[Colors] MPC01..MPC16`；`src/ui/skirmish_shell/layout.rs:159` 只有速度/金钱/单位数滑条；`src/skirmish_launch.rs:18` 颜色数8；`src/app/frontend/skirmish_shell_render/controls.rs:353` 固定8色 | 先核 #174 颜色表与选择政策，再接配置/选项/启动数据；30槽目标不等于已支持16色，验证颜色占用、同色策略和人/AI辨识 |
| F13 / P2 / 已实现，待画面验收 | NPatch超五乘员开火端口循环：已修第六名原先错误回退零偏移的问题 | NPAE174 GetFLH `0x6F3AFB → 0xC2E196`按`(k-1)%5`循环，ART reader `0x715FA4..0x716005`仍只读0..4；固定ART5/6是本链不消费的数据。`ArtEntry::alternate_fire_flh`为唯一共享owner | 六人实际Enter后的生产坐标已验；不扩大数组。下车重上/移动转身的实际画面仍待验，不把坐标测试称GPU输出认证 |

F06–F09、F11 的“缺失”指扩展读取/接线缺口；**原生效果定义尚未全部证明**，
不能直接依据键名写实现。F01–F03、F13 是首版新增的明确生产路径缺口；F13的五槽容量本身正确，缺的是NPAE循环。

## 4. 已有实现，仍需核对的功能链

| ID / 优先级 | 类别与当前边界 | 下一次验收 |
| --- | --- | --- |
| V01 / P1 | 三组供电：ROBO/GAROBO、DRON/NARADR、CAOS/NAPSIS。已有 `src/sim/powered_unit.rs`、共享 House 计数和 provider/EMP 查询；不是“PoweredUnit 未实现” | 每组失去最后供电建筑、低电、手动断电、EMP、恢复、换主；同 House 混有另外两组和同阵营另一家 AI 时不得误供电/串状态。先核 #174 的按类型与全局计数关系 |
| V02 / P2 | 医疗建筑：GAROBO、NARADR 的 `Hospital`。已有 `src/sim/docking/hospital.rs:18` 的进驻/治疗/退出owner及radio/app准入；不是Hospital未实现 | 受伤步兵 Enter、排队/满员、治疗、离开、失电/毁楼、换主；与驻军 F04 区分 |
| V03 / P1 | 连发时序：`BurstDelay0..3`、精英武器、Stop/Guard。`src/rules/object_type.rs:3108` 已读取，`src/sim/combat/rof.rs:183` 已消费；`src/sim/combat/world_fire.rs:1155` 只给Unit传值 | TTNK是已有链；NASAM建筑四项BurstDelay物理值均9，但是否为#174建筑扩展或无效配置待核。实际多发射击及S/G打断不能绕过ROF，存档不重置burst |
| V04 / P1 | 隐形/传感器的其他路径已有实现；F01 不等于整个系统缺失 | IFV/狗/英雄/空艇的传感器范围、精英潜艇/幻影、移动和开火；目标丢失/再发现以及 AI 选敌一起验，不从旧说明硬编码范围 |
| V05 / P1 | 驻军与建筑经验、子机经验。Building已支持Trainable/击杀XP，交接已覆盖BFRT和母子机部分生命周期；`src/sim/combat/record_kill.rs:33` 仍保留stock的NEXT occupant获XP顺序 | 对照作者“驻军经验计算修正”是否要求真正射手获XP；建筑/驻军经验、死亡/弹出、子机再生分别验，避免把BFRT政策复制成建筑驻军算法 |
| V06 / P1 | 作者170601：精英盖特炮从高档/盖特坦克从中档开始；已有 gattling owner | YAGGUN/YTNK 实际晋升、初始档位、停火退档、换主/存档；用当前配置与补丁消费者定语义 |
| V07 / P1 | 作者170601：辐射剥离缠船乌贼、乌贼自动选敌修复；已有 parasite 和 combat 关系 | 船被寄生后受辐射、释放/死亡/再捕获、双方归属和目标扫描；不要只用普通伤害击杀乌贼冒充剥离 |
| V08 / P1 | 作者170601：BFRT不自动冲敌坦克碾压、TTNK不再给磁暴塔充电 | 核当前 `AutoCrush`/充电资格及生产消费者；分别观察自动 Guard 和显式 Attack，不能从作者旧版说明推定当前行为 |
| V09 / P2 | 其他 stock 箱子与无键补丁 | 全量对照当前有权重/可达的效果；零权重且地图不指定的 Gas/Napalm 等不误列高频缺口 |

作者170507的无新键修复另外保留为 **V10 / P2 待核组**：精英后射程反降、光棱联击、
铁幕目标的自动攻击准入、船厂维修伤害归属、红血碉堡进驻、Alt 飞机误入维修厂/船厂、
秘密实验室单位选择概率、火箭飞行兵坠落目标层。特别是 `src/sim/secret_lab.rs:78`
明确保留stock的缩短范围位置回原数组抽签缺陷，值得优先对照作者的概率修复；
没有证据时不能直接重写RNG算法。这些项目有对应 retail owner，
本轮未逐条追 #174 补丁和实际运行，因此既不算已兼容，也不统一判缺失。
V03 和 V05 已单列作者所述 burst/驻军经验，不再在 V10 重复开链。

## 5. AI 专项清单：优先于纯画面项目

**本轮未发现足以确认为“当前固定 AIMD 活跃战术故障”的新证据。**
以下多数是生产实现已有但尚未覆盖用户多电脑场景的验收债，不是宣称 AI 系统未实现。
根 AIMD 注册脚本的物理用法共21种 opcode：
`0,3,5,6,8,9,11,14,21,43,46,47,49,53,54,55,57,58,61,62,63`。
它们均在 `src/sim/team_script_vm/actions.rs:110` 的已移植分发名单中；
这只能排除“所有这些指令没有分发”的说法，不能证明各条战术完整执行。

| ID / 优先级 / 状态 | 机制与代码入口 | 多电脑对战收口条件 |
| --- | --- | --- |
| A01 / P1 / 待验 | 注册→触发→组队→补员→脚本：`src/rules/team_ai_ini.rs:478`、`src/sim/team_script_vm/registry_install.rs:201`、`src/sim/ai_team_creation.rs:350`、`src/sim/ai_unit_choice.rs:152` | 三阵营各自完成队伍、出基地、接敌、损耗后重建；不只数“团队已创建”，不能被永久空队占满 Max |
| A02 / P1 / 待验 | 基地与独立经济：`src/sim/ai_base_building.rs:335`、`src/sim/ai_build_need.rs`；MOD `[AI] BuildPower/BuildRefinery/BuildTech/BuildWeapons` | 同阵营至少两家 AI 同场独立付款、造厂、补矿车和补兵；一家的失厂/缺钱不污染另一家选择；三阵营都可发展到实际高级科技 |
| A03 / P1 / 待验 | 多方目标、联盟、目标击败后的转移：`src/sim/house_strategy.rs:396`、`src/sim/house_diplomacy.rs:165` | 人类对多家结盟 AI、自由混战分别验；不把盟友错误选为敌对目标，范围友伤按武器政策验；目标灭亡后重选，视野/外交符合 #174，不借 retail 注释断言补丁行为 |
| A04 / P1 / 待核+待验 | 海军/陆海竞争：`src/sim/ai_team_creation.rs:577`、`:624`，`src/sim/production/factory_ai.rs:269` | Allied Naval Hunters、Soviet Navy Bombard、Yuri Naval Hunters、Boomer Bombard 在水陆图成型出海；同时有陆海需求时不长期饿死一路，无法放船厂不锁死陆军 |
| A05 / P1 / 待验 | 超武充能、选点、编队联动：`src/sim/superweapon/ai_fire.rs`、`src/sim/team_script_vm/super_actions.rs`；实际脚本用55/57 | 多家独立充能和发射，不跨House消费超武；敌对超武不错误选盟友，铁幕/超时空允许的己方或盟军目标按#174政策验，投送后继续作战；光照另见 F11 |
| A06 / P1 / 待验 | EMP、断电、捕获后队伍与工厂生命周期：既有 EMP/生产/capture owner | AI付款头暂停后恢复、其他同阵营House不受影响；被控制/死亡后原队成员/需求/计数正确更新，仅符合Reinforce/组建状态政策的队伍补员（`src/sim/team_script_vm.rs:735`），不产生跨House需求；无人单位失provider后可恢复 |
| A07 / P1 / 待验 | 长期经济恢复：`src/sim/ai_build_need.rs` 已含卖建筑、矿场/矿车重建和绝境 Hunt | 矿枯、矿车/矿场/重工损失、缺钱后的恢复或败退；无永久等待，长局各 House 资金/生产/团队计数不串 |
| A08 / P2 / 已修配置、待验 | 两个光棱 TaskForce `0CE4DCDC-G` / `0A6E3F1C-G` 已改为 SREF | 根 AIMD不再列 SERF 为现存问题；实际相关编队补齐、出击和光棱作战才关闭行为验收 |
| A09 / P1 / 已修配置、待验 | 攻城直升机 Script `07C3188C-G` / `07C48DBC-G` 的 Guard 已为 `5,0`；Team `07C460CC-G` / `07C3092C-G`；活跃触发 `07C30A5C-G` / `07C4853C-G` | 两架 SCHP 执行 `54 → 5 → 9 → 0...`，移动、落地部署、开火和再次移动；不是只加载成功。现有 TickTank DeployToFire/DeploysInto 注释残留不能直接归罪 SCHP |
| A10 / P1 / 待验 | 作者170601称电脑已修巨炮/裂缝建造并会用超级裂缝；已有基地选择、部署/Gap owner | 盟军AI实际建巨炮/裂缝，实际使用超级裂缝相关单位/能力；核当前生效配置和消费者，不把旧作者 bug 当当前引擎必然存在 |

A04 的特定残留：`src/sim/ai_unit_choice.rs:86` 只有一个车辆选择槽，
`src/sim/production/factory_ai.rs:355` 记录了独立 naval slot 未完成。
现有说明引用 **NPSE #359/ext173**，尚未证明 #174 也需要双槽；
不能直接移植另一版本或写成“AYMOD海军必然不能生产”。先原生核对和实际竞争场景复现。

### 固定 AIMD 的配置残留：单独管理，不猜修复

| ID / 优先级 | 引用链与现状 | 后续动作/风险 |
| --- | --- | --- |
| C01 / P3 | TF `07C338DC-G` 含未知成员 SHD → Team `0610813C-G` → AITrigger `0616004C-G` / `07C330CC-G`；根 AIMD 两条触发的三个难度位物理值均0 | 当前固定触发不可达；不能写成“活跃碉堡队少造兵”。地图可重新启用，届时按未知成员忽略政策运行；不猜为 SHK/SCHD |
| C02 / P3 | 已注册但缺定义的 TF `05FC0DEC-G` → Team `060F883C-G`（Yuri Armor - H3）；根 AIMD 固定 AITrigger 未引用该 Team | 保留身份，不能补抄原版编队；地图/脚本重开才会产生实际影响，若恢复战术需作明确维护设计 |
| C03 / P2 | 短行 AITrigger `0C8C51BC-G` / `FFFFFF12-G` 的对象类型字段为空；顺序token读取跳过空字段，移读后multiplayer=false，当前遭遇战不具资格（FixedAimd的enabled仍为true） | 两条原配置战术尚未恢复，但不是enabled=false或新的加载回归；reader `src/rules/team_ai_ini.rs:625`、空项读取`src/rules/ini_value.rs:804`、拒绝入口`src/sim/ai_team_creation.rs:393`。先核作者意图与原生读取，不加占位值擅自启用 |

## 6. 已交付机制和有意政策：不重复“修复”

以下是交接分支已有实现，不再列为纯缺失。macOS 的已有覆盖不冒称 Windows 重跑。

| 机制 | 实现/覆盖边界 | 尚未关闭的边界 |
| --- | --- | --- |
| 宽容 AIMD 安装、顺序 AITrigger、TeamType 查找 | 未知成员忽略、缺定义编队身份保留；已有生产 reader/VM 改动 | 不证明每条活跃战术；见 A01/C01–C03 |
| 三阵营随机范围 | `NewPlayableCountries=-7`；startup root reader，现有 RNG 顺序 | Windows 人类/AI多个随机槽；不是 mode/map 覆盖项 |
| 科技树与 AI 规划 | 七处 Prerequisite2、三处 NegativePrerequisite、八厂 AIBasePlanningCount 通过共享 owner；已有实际MOD科技/船厂生命周期覆盖 | 普通AI原生提前成功分支保留，不强制套人类前置规则；实际发展见 A02 |
| 手动切电 | UseTogglePower、V/可重绑、鼠标与事件/BuildingOnline 链；已有 GUI 兵营Off/On与地图切换/手势回归 | Windows普通输入与存档；不另建客户端电力状态 |
| EMP | 实际 TTNK → TankBolt/TankBoltE → ElectricbounceT/TE → TankBoltEMP → EMPulsWH；impact child 已接通，不能继续当作只引用未触发 | 时长/适用对象/算术为[明确可玩政策](npatch-emp.md)，不是完整 #174 等价；多AI见 A06 |
| 已付款生产头暂停/恢复 | EMP不退款/重建，手动Off和用户Hold不被到期覆盖；已有实际MOD生产场景和保存状态回归 | Windows运行与多工厂/多House |
| 乘员/母子机经验 | BFRT乘员先升、精英回退载具；子机训练母体、现存同步/再生继承；已有实际BFRT Enter场景 | 见 V05/F13；不是所有驻军链都已验 |
| 控制者经验 | 有效非精英控制者分配既有XP，精英/无效回退普通受益者；实际尤里场景 | 比例限制与整数守恒为[政策](npatch.md)，不是所有 #174 算术已证 |
| 英雄赏金 | Tanya实杀产生可支配金钱；敌对终结击杀只结一次，普通/精英击杀和去重有回归 | 默认值/钳制/比例是政策；动画外观未单独验，其他英雄未全部实际场景验 |
| 永久控制/间谍劫车 | Spy登车成功消耗自身；精英尤里夺建筑后移除控制者仍保有建筑；拒绝/落空有回归 | Windows多AI换主/补员与存档；不创建临时manager竞争owner |
| 存档地形、窗口边滚 | baseline ClearTile/LAT变体重绑与缺失radar派生面重建已修；出窗口停止边滚 | 不宣称所有存档或启动白屏已修。作者原引擎“不支持读档”不等于要求VERA禁用存档 |

经验、赏金、EMP、混合扩展、负数generic前置处理和切电root生命周期的有意差异，
继续以 [npatch.md](npatch.md) / [npatch-emp.md](npatch-emp.md) 为机制说明 owner。
不要为了清单显示“完成”而悄悄抹掉政策残留。

## 7. 建议适配顺序与验收矩阵

1. **P0基线**：使用本交接分支的 `npatch` 普通游戏构建，在 Windows 新局确认三个阵营；
   补齐正版零售 `ini/` 回归环境，不把本轮 asset 构建说成游戏已更新安装。
2. **先做A01/A02/A09最小多AI验收**：一人对三阵营电脑，至少再加同阵营重复 House；
   先找实际发展/出击阻塞，再选一条有复现的链修复，避免凭清单批量改权重。
3. **常见战斗机制**：F01/F04/F05已接入，继续V01三组供电和其他开放功能；
   每次一条普通兵种链，修必要前置和生命周期，保留其他独立机制为明确 residual。
4. **箱子**：F02与F03分别完成；关闭箱子的对局暂不受这两项影响，不能删掉要求。
5. **水陆/长局/超武**：A03–A07与A10；查到真正差异再实现，尤其不套用NPSE海军槽。
6. **较窄武器与画面/UI**：F06–F13；渲染项必须用实际输出验证，不只跑CPU测试。

| 维度 | 最小代表组合 | 观察终点 |
| --- | --- | --- |
| 三阵营/重复阵营 | 人类 + 三阵营AI + 同阵营第二家；普通难度先复现，后覆盖困难/简单 | 各自经济、科技、成队出击、损耗重建，House间无串状态 |
| 关系 | 多家AI联军；自由混战；一个主要目标被消灭 | 目标切换、盟友安全、各自超武/资金/编队归属 |
| 地图 | 陆图；明确有可用船厂出口/水路的水陆图 | 建基地/寻路/海军与陆军同时有进展；地图不适配与引擎故障分开 |
| 压力 | 失矿车、失工厂、EMP、失provider、换主、矿枯 | 恢复/败退条件到达，不永久等待；不跨House暂停生产 |
| 持久化 | Windows本分支新局保存再读；相同能力构建 | 队伍、生产头、控制/隐形/EMP/电力状态保持，不重抽奖励或RNG |
| 模式 | 先Battle ID1；其余可用AI模式单独验证 | mode/map实际层次和类型变更均生效；不声称九个模式全通过 |

不以固定6000逻辑帧或“出现一次Team”代替发展、出击和恢复终点；
需要稳定复现时使用固定seed和明确初始状态，不做大网格扫测。
每项修复补1–3个能在修复前失败的生产路径回归、必要实际MOD场景，玩法改动在PR前做只读critic。
当前授权按功能优先、UI随后逐链实施；不改MOD平衡、原始快照或AI战术权重。
没有推送/发PR/发布资源授权，不以部分机制通过宣布清单全部完成。

## 8. 已建立的原生证据

以下保留初次交接已有的原生链；本轮F04的读取、伤害顺序与清理语义见
[npatch.md](npatch.md)及对应Rust邻近注释，不提交静态反汇编包或原生运行日志。

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
