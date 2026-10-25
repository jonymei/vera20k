# AYMOD 兼容性清单

更新：2026-10-26。面向“人类玩家与多个电脑对战”的分批适配工作清单。
完整执行顺序见 [全面适配 PLAN](aymod-plan.md)。这是当前唯一的兼容性台账；[Windows 交接](aymod-windows-handoff.md)记录此前交付历史，
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

### 首份玩家测试版与下一链

- 玩家测试版：`aymod-playtest-20261025-f04`，来源干净提交`4f0f5afa6`，
  `--profile fast --no-default-features --features npatch`构建成功；固定保留，不受后续编译覆盖。
  本机启动入口：`D:/Workshop/vera20k/target/AYMOD-test-20261025.cmd`，
  使用主工作树配置和AYMOD资源；未自动启动。新保留字段改变存档布局，请新开对局。
- 本组明确功能缺口仍有F09；F02/F03/F08已接入但保留场景/画面边界；F06已接通ORCA对空链，远距追击/GUI待验。
  F07的Sapper普通/精英对机械水面舰船已有实际场景覆盖，两个疑似键改列C04，不虚构新开关；水下/GUI仍待验。
  F11/F12（视觉/UI）开放；F10已接通并通过GPU定向验收，玩家画面待验；
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
### Return Dubai 首次闪退：声波扫描越界已修复

- 玩家报告Return Dubai、1v1、困难，鱿鱼缠绕驱逐舰后海豚攻击时闪退，无存档。
  本机追加日志确认地图名；首个panic为`render/wave_geometry.rs:137`的len6/index6。
  随后还有图形swapchain析构panic；优先修复前面的实际越界，不据此宣称所有图形退出问题已修。
- 修复提交`e18a67f21`：投影声波六边形不一定Y单调，边链可能产生额外扫描行。
  原生Polygon63F080在`0x63F1DF`分配逻辑行数外的scratch，DrawSonic75FA90在
  `0x75FB48..0x75FB54`仅绘制逻辑前缀。现只保存此前缀，不扩大绘制范围，不跳过声波，
  不改伤害、缠绕、RNG、计时器、Detach或模拟状态。
- 临时生产Wave角度/高度诊断先在旧实现复现同类越界，已移除扫描，仅保留一个高度差声波
  生命周期→rasterize回归（通过，并断言仍输出像素）。既有Wave原生表/扫描行/颜色/遍历4项
  和真实GPU读回1项均通过，GPU测试包含深度与A缓冲不变检查。没有恢复玩家整局或逐帧原生对战比较。
- 视觉残差：原生首次分配与复用缓冲的初始skip不同（63F1EF），当前未模拟该缓冲生命周期细节；
  不因本次防崩修复引入新的模拟状态。此项不阻塞当前越界修复。
- 修正版已保留：`aymod-playtest-20261025-sonic-fix`，干净`e18a67f21`，fast+npatch普通游戏构建成功。
  本机新入口`D:/Workshop/vera20k/target/AYMOD-test-20261025-sonic-fix.cmd`；旧测试版仍保留。
  AI攻势弱的复现条件补充为上述地图/1v1/困难，尚无足够证据定位具体AI链。
### Return Dubai 困难 1v1：生产瓶颈与颜色错配

- 按用户要求先分段定位，不修改AIMD权重、兵种数值或MOD配置。旧实战日志已显示雷达、作战实验室和工业工厂；
  不是完全卡在发展。使用真实AYMOD、Return Dubai、美国对苏联困难、固定0/3出生点、默认选项和固定seed，
  通过生产load_session诊断至9000帧：AI资金410000→446621，两重工两船厂，多个团队补齐并推进到出击动作。
  人类在该诊断中不操作，因此不是恢复玩家原局，也不能据此认证攻势强度。
- 7500帧可见陆军工厂生产而海军队伍缺HYD/SQD/SUB；9000帧仅船厂生产SUB，陆军工厂等待。
  实际NPAE174指令确认独立陆/海选择槽，原实现却共享；这是可达的生产/补员瓶颈，不是AI完全不出击。
  `8e944009f`已在既有HouseAiUnitChoices与工厂owner分离槽位，并覆盖需求过滤、出厂、限额、取消和保存/hash。
  保留原生矿车优先及取消/重启的不对称清空，不另建编队或生产计数。
- 新增3项生产回归通过；NP FactoryAI13、Chooser4、BaseBuilding及直接生产生命周期/队列覆盖通过。
  无NP FactoryAI8、Chooser4、BaseBuilding12及颜色解析隔离通过。独立只读critic未发现本链具体缺陷。
  双调用增加的Scenario RNG抽样沿原owner按陆→海顺序；未新增计时器、Detach或线程相关状态。
  存档布局随新增保留字段改变，旧布局照常拒绝。
- 仍开放：实际多AI/长局攻势验收；NPAE同需求数量时的候选清空与stock picker差异；
  可选DisableAIParallelProduction机制。生产查询确认实际根/模式/地图均未设置该键，原生默认false，
  本轮不声称支持开启后的计数政策。没有把一处双队列修复标成整个AI完成。
- `372af3d64`已修“美国选蓝色进图暗红”：AY根UseDropDownColors=yes，但大厅和开局仍用stock序号表。
  现共用按MPC名称解析的NPAE映射，大厅、House配色、加载标记和聊天一起迁移；不是地图强制改色。
  AY当前第3色是白色MPC03，第5色是蓝色MPC05；旧蓝色选择应在新大厅重新按显示色块选择。
  根开关层次/保存回归与真实AY蓝色映射、白/蓝色块两项回归通过，相关前端/加载/消息覆盖通过。
  现有真实AY地图加载与MTNK攻击场景也通过；尚未做本轮GUI画面验收。
  科技等级滑条与完整16色界面仍开放，其他WOL/合作界面不冒称全部支持NP扩展色。
最新玩家版为`aymod-playtest-20261025-ai-colors`：干净`dc330e87b`的普通fast+npatch构建，
已读取manifest并通过固定标签解析校验；本机入口`D:/Workshop/vera20k/target/AYMOD-test-20261025-ai-colors.cmd`。
包含此前声波防崩、本轮陆海双队列与颜色一致性；旧版保留，未自动启动或替换用户正在玩的进程。
请新开局，旧布局存档不兼容；完整GUI与弱攻势是否充分改善仍待玩家验收。
### 全面适配 goal：两份玩家存档与下一批次

执行范围与完成定义见 [全面适配 PLAN](aymod-plan.md)，不能把本轮静态分析或单点修复当作目标完成。
用户确认修复后的海豚正常；AI强度与新颜色构建的玩家验证留待次日，不据此关闭对应验收项。

- 两份中/末盘存档已通过历史`e18a67f21`的正式GameSnapshot载入门禁和完整反序列化，均为旧布局
  `0xd4be79fe`（tick84941/161562）；当前代码布局`0x91e21ca0`正确拒绝它们。
  因此这些数据用于旧版基线，不能当作新双队列/颜色修复后的实测结果。
- 批次0已具备只读工具：`telemetry-serve --inspect-save`复用保存owner输出完整House/工厂/团队/实体，
  不推进、不重绑资源、不写回；不使用仪表盘截断列表。原始存档与完整报告只保存在本机忽略目录。
- V01强证据：末盘玩家没有在场GAROBO（仅有暂停的limbo产品），但有已计数NARADR，House共享数为1；
  五辆ROBO仍未deactivated且Hover有电。中盘玩家两种中心均已计数。串供电是明确状态证据，
  不是简单的V按钮未生效；目标NPAE原生也保留共享计数，需明确区分原生行为与独立供电的设计预期。
- 蜘蛛报告保持开放：末盘仅余DRON确实offline且Drive无电、无导航；静态快照不证明也不否定它先前移动。
  原生Drive不直接消费locomotor power字节，火花也可涉及EMP；不为掩盖问题冻结所有离线对象AI。
- V01依赖另发现NPAE与Ares策略混用：Reactivate70FBE0/C2BC8B与EMP到期C2BC5D/C2BC6E的门禁，
  不等于当前复用的Ares全函数早退/恢复链。尚未修改；需在完整停机/EMP链中处理，不先做不一致的局部替换。
- A01/A02新证据：中盘已有高科技、两重工两船厂与大量资金，15队中14队formed；
  应优先查集结→出击而非先改权重。团队200在53动作，鲍里斯接近集结点却保留blocked一格导航；
  早期2/3队仍在54动作，海军184的目标与成员位置相距很远。下一步查6EBAD0的组移动完成、拥挤占格、
  CloseEnough/RelaxedStray和实际路径；一次快照不足以宣布永久卡死。
- F10原生研究已补齐：还必须读IsCustomColor（默认false）；RGB三色默认83,83,255 / 255,255,255 /
  83,83,255，逐物理层current-default读取，即使gate关闭也保留颜色；出生复制后用RGB565绘制，custom覆盖alternate。
  TankBolt/TankBoltE的生产配置已确认使用；TeslaFragmentT没有IsElectricBolt，不为显色擅改其准入。
  F10现已按此链接通，并通过实际AYMOD普通/精英开火和GPU读回；详见下方当前检查点。
### 存档驱动 A01：集结堵塞已修一条真实链

- `6c2b97629`修复Walk重试续接：原生75B6D1/75BBBF在同一对象访问中进入Process(false)，
  批量寻路却把该续接延期，下一普通访问重新允许重试，近距离占格拒绝就永远到不了75B7E1的停步尾。
  现读取既有私有visit标记，让这一次有界续接同步完成；不改Team Coordinate_Move6EBAD0、CloseEnough、
  RelaxedStray、权重、兵种或MOD配置，不新增持久状态/计数器。
- 旧实现先在零售BORIS/APOC拥挤场景失败：串行停步，batch开启后120帧仍moving/NavCom不清。
  修复后同一普通命令/帧回归与停步→Coordinate_Move完成→下一脚本行回归通过。
  既有明确要求错误延期的测试已改为同步安装路径、无排队注册，未删除/忽略该测试。
- NP PathBatch24、TeamActions25及Walk准入/失败/完成相关覆盖通过；无NP PathBatch24、TeamActions25通过。
  包括既有线程数确定性、批量上限、排队/保存、Drive重试不变覆盖。独立只读critic未发现本链具体缺陷。
- 正式旧存档动态对照也通过：历史e18源码按正常资源加载/布局/规则/地图/缓存门禁载入中盘，
  BatchON，消费原到期命令，无新增玩家输入，各推进300普通帧。只复制两份生产移动修复，未改保存布局。
  旧版鲍里斯与团队200始终停留集结；修正版第1帧清其Nav/moving，第10帧14人队进入实际Attack，
  第300帧已从原处继续向目标行进。基地2/3队也越过原集结动作；3队后继63动作移除成员，不说成被消灭。
- 对照为相同冷加载MainRng0/Scenario恢复，不是原进程逐tick回放；观察对象有限，后续瞬时拥挤仍存在。
  原始存档未写回，报告仅在忽略目录；不能据此关闭海军184、所有堵塞或整个AI强度验收。
- 更新顺序变化：仅Walk本次递归Find_Path改回同步，最多增加一次该访问内的搜索；普通batch和Drive/Ship不变。
  搜索kernel不加随机抽样，既有Find_Path回调/RNG、movement timer写入与清目标仍由原owner执行；
  不新增计时器或Detach_All。大军广泛拥堵时该例外的总体性能与画面对照仍是残余验收项。

### A01/A04补充排除：同需求择兵不是本局弱攻势原因（2026-10-26）

- 复读实际NPAE174的Unit chooser：C2B479..C2B481在need相等时也重置候选；
  stock 4FEDF6..4FEDFE仅在need严格增加时重置。Infantry4FF120/Aircraft4FF450仍为stock比较，
  不能把Unit特例套给全部类别。
- 但该列表只供第二次随机选择：C2B4D0先抽Scenario[0,0x7FFFFFFE]，
  C2B4FE..C2B50B通过FillEarliestTeamProbability门后直接选最早队伍，不访问候选列表。
- 新增`aymod_return_dubai_hard_uses_earliest_team_without_a_tie_draw`，通过正式load_session加载
  实际AYMOD重返迪拜Battle/美国对困难苏联；根、模式、地图处理后的概率为100,100,100。
  复用生产pick，两个相同need且不同创建时间的候选，即便首draw为最大值仍选最早者，
  只调用一次Scenario draw、不产生第二次候选draw。NP显式实际资源测试1项通过；择兵模块既有4项回归通过。
- 因此这个未移植差异在本次具体配置下不影响被选兵种，不能当作已定位的弱攻势原因或据此调整权重。
  其他地图/模式覆盖概率时仍是residual；未修改chooser、MOD数值、随机数owner或存档布局。
- 上一轮重跑Walk集结→下一脚本行1项、真实AY大厅白/蓝色块1项均通过；保留玩家测试版
  `aymod-playtest-20261026-explosion-crate`不变，本次仅增加诊断保护，不需要换游戏构建。
  下一AI动作仍为海军184的目标/交战/水路链及新版多AI长局，不再优先追此默认不可选分支。

### 当前检查点：A04海军184卡在跨水域集结（2026-10-26）

- 任务仍在`D:/Workshop/vera20k-aymod-audit`、`feature/aymod-integration-resume`；
  本批基于`4187596e2`，仅新增诊断测试/台账，无玩法改动、无PR/推送，无待完成Cargo。
  总体范围与[PLAN](aymod-plan.md)不变；V01作者设计/原生冲突及F11等未完成项仍开放。
- 重新读原始完整mid报告：队184成员SQD30732、HYD30542/30541、SUB11968/11967
  均属于Computer1，Health非0、非limbo、非deactivated，attack_target均为空。
  五者mission.current=2是Move，不是AreaGuard(11)，也不是SUB在攻击己方HYD；
  SUB11968的NavCom108,55恰好落在同队HYD脚下，导航位置不能当作攻击对象。
  脚本FFFFFFF1-G实际为53→11,6，队伍仍在第0行动，尚未执行后继任务。
- 已有正式旧存档300帧追踪中，五人成型团队仍停在53，focus99,149不变；
  新增实际AYMOD/Return Dubai/Battle/困难加载诊断，按旧SUB109,55出生，
  直接调用Foot4D3810→Map56D100生产门禁：108,55可达，99,149不可达。
  这比仅比较raw Water2/4更强；初始地图即重现，不依赖存档中派生缓存损坏。
- 同一测试使用存档记录的Computer1基地117,44与玩家基地93,154，经现有gather_seed_cell、
  team_assign_gather_cell/FNPC生产路径重新选点，结果也被同一SUB运动门禁拒绝。
  是新地图+记录位置的定向诊断，不是恢复原局、原生地图执行或全队长局回放。
- 实读NPAE174 Gather_At_Enemy_Base6EF947..6EF98A，FNPC传leader SpeedType、3x3，
  不约束领队的水域编号；Coordinate_Move6EBE42..6EBEAC没有给普通船只增设跨水域绕过。
  现有Rust沿用这些参数。尚未证明原生在这张地图上生成相同连通分区/最终集结点，
  所以不能据此宣布“原版也必卡死”，也不能直接合并水域或发明可达点回退来冒充兼容修复。
- 验证：新增显式实际资源测试1项通过；TeamActions既有25项通过，新增ignored项已单独运行。
  无新随机draw、计时器、Detach或持久状态；原始存档和保留玩家构建未改，不需重构游戏。
- 下一步：核原生地图水域拓扑/FNPC最终点与Foot准入，区分地形/区域生成差异与原战术限制；
  不再把海军184解释为发展不足、缺员或近敌交战。其他海军、多AI/长局和整体攻势仍未验收。

### 已完成批次：F03爆炸箱生产链与即时伤害通知（2026-10-26）

- 工作区 `D:/Workshop/vera20k-aymod-audit`，分支 `feature/aymod-integration-resume`；玩法提交
  `27050a89f`，另补连续两帧通知断言已通过。一次独立只读critic未发现具体新增缺陷；未推送/PR/合并/启动游戏。
  原始保存、历史分析工作区、主工作区用户`.gitignore`与已保存构建均保留。
- F03已接通CellPickup482565..482719：ftol幅度的独立副本直接C4命中拾取者(ignoreDefenses1,arg6=0)，
  然后5轮半径[0,512]→重读retained Cell坐标/地板→49F420 raw方向/snap0→同步区域C4伤害→
  SelectAnim literal land0→真实Anim(loop1/2600/-15)→CombatLight(extra0/forcefalse)。均无攻击者或来源House。
  已用共享数值/伤害/动画/光照owners；不复制散射公式、不预抽五组随机数。较多raw消耗可能来自ranged拒绝或下游回调，
  不是固定10个raw词。没有新增producer计时器/直接Detach；现有死亡、桥梁、地形与动画owner保持自身副作用。
- 拾取者死亡不会提前返回，5轮和通用Powerups动画尾继续。移除与MP替换箱仍在本轮效果前，由原有owner完成。
  修复所需的另一个前置：即时伤害的bridge/structure receipt可能被只返回AL的移动调用者丢弃，导致显示刷新遗漏。
  现由既有DamageConsequences记录两个serde-skip/不hash的派生通知，在现有frame边界取走，不重放任何玩法后果。
- 3项新箱子回归：直接packet被晚期免疫清零后仍5次足额区域伤害/RNG/动画/光照；正常Drive中拾取者死亡后
  5爆/动画尾/替换箱；实际AY正式加载、生产放箱+普通Move、实际weight15/magnitude500和5个光照请求进入帧输出。
  原有真实IFV/桥梁回归扩展到后果提交和普通帧，再验证第二帧不重复通知；原native ID/动画/RNG对照保留。
  两项合成箱子和桥梁通知均先失败后通过。夹具重复物理段落、ignoreDefenses绕过IC、水木箱同图像造成ghost，
  以及误读已drain的内部光照队列均已纠正，不计引擎bug或修改MOD配置。
- 最终NP箱子84 passed、伤害后果3 passed、network wait5 passed、真实AY箱子1 passed；
  无NP箱子84 passed、伤害后果3 passed；桥梁/两帧通知NP与无NP各1 passed；Ares箱子83 passed。
  未跑全仓库。实际AY测试不是Return Dubai困难/多AI长局，未作最终GPU/GUI爆炸、光照画面对照。
  共享Dummy被复杂嵌套回调restamp的所有组合、中断帧携带真实bridge通知还未独立场景穷举，代码经静态审查。
- 本轮不改AI权重、兵种数值、MOD文件或旧保存。已从干净e3e05ec46保存普通fast+npatch构建
  `aymod-playtest-20261026-explosion-crate`，manifest已读且固定标签解析通过。最新入口
  `D:/Workshop/vera20k/target/AYMOD-test-20261026-explosion-crate.cmd`，含此轮F03及此前全部已交付修复。
  旧入口/构建保留，未启动游戏；测试本项需开带箱子的新局，保存仍走正常门禁、不强读旧布局。
  当前无待完成Cargo或未提交代码；继续下一条链。
- V01/A06保持既有Drive EMP修复和原生/作者冲突结论，详见npatch-emp：原补丁可让失联Drive被AI重发Move，
  作者却要求指定三组建筑失去支持则瘫痪。独立供电与禁失联移动仍待明确兼容策略确认，未静默加全局冻结。
- 全面PLAN继续有效。下一安全动作：保存F03可玩版，然后推进F11 Psychic Dominator独立光照的原生读取/生命周期调查；
  F02实际箱子/画面、F09、F12、三阵营/重复House与多AI长局等仍开放，不以本轮箱子通过结束全面目标。

### 已完成批次：F02隐形箱接入，场景/画面待验（2026-10-26）

- 工作区 `D:/Workshop/vera20k-aymod-audit`，分支 `feature/aymod-integration-resume`；
  本地玩法提交 `958963e4f`、审查修正 `f288fe425`。无推送/PR/合并/启动另一份游戏授权。
  主工作区用户 `.gitignore`、旧存档、历史分析工作区与已有游戏构建均保留。
- F02：CellPickup48294F的实例能力由既有CloakRuntime持有，半径内任意House的marked地面Techno接收；
  重复MP拾取转Money、现有隐形AI、hash及保存已接通。原类型不变，不以rank冒充实例能力。
  只授予能力，不直接执行隐形、不额外抽RNG、不改计时器、不Detach；后续转换沿既有owner。
- 同实例Unlimbo现保留隐形状态/计时器/实例能力；Unit737BEB场外例外仅写state2，Infantry不套此特例。
  独立只读critic发现未占格对象误收能力，已先复现失败再修正。共享范围收集器显式区分准入：
  Cloak482887/Veteran4829B9要求Object+74，Armor482D9A/Firepower48316B没有该前置，未一刀切改变其他箱子。
- 三项新增回归覆盖半径/敌我/未标记对象、正常移动单位隐形、重复/存读/hash、换主及同实例入场。
  最终NP箱子82 passed、隐形44 passed/6 ignored；无NP箱子82 passed、此前无NP隐形43 passed/5 ignored；
  正确的object_entry::unit_entry_tests为6 passed/2 ignored。早期错误筛选的0项不计覆盖。
  未跑全仓库；忽略项未因本次改动触及其数值内核，不额外执行。
- 仍缺实际AYMOD正式加载的箱子场景与GUI/原生并排验收。建筑和落地飞机目前测试仅证明授予能力；
  不把它说成四类完整视觉/生命周期通过，现有建筑visual-character不自动等于车辆全透明。
  保存布局随CloakRuntime新字段改变，旧布局仍拒绝，不放宽门禁。此批不能据此关闭F02整体验收。
- 已从干净 `9f198cee9` 保存普通fast+npatch构建 `aymod-playtest-20261026-cloak-crate`，
  manifest已读且固定标签解析通过。新入口 `D:/Workshop/vera20k/target/AYMOD-test-20261026-cloak-crate.cmd`；
  含F02及此前陆海生产槽、Walk集结、颜色映射、海豚防崩、电弧等。旧入口/构建保留，未启动游戏。
  本版须新局；旧布局存档未迁移，不能强读。正式AY箱子场景和玩家画面仍待验。
- 下一功能链F08已用实际指令再核：C2EF94..C2EFA8读取IsFlatExplosion到WH+130，
  75D42F..75D443随后读取CausesDelayKill到同一字节，701E92参与已有建筑PostMortem门禁。
  必须在有序规则处理器逐物理层保留，而不是合并INI后让旧别名覆盖新层；不另造平面伤害或死亡计时器。
  F03/F08/F09/F11/F12、V01/A06及多House/长局AI仍开放，原生聚合供电不擅改为独立类型设计。
### 已完成批次：谭雅海军验证与F07纠正（2026-10-26）

- 工作区`D:/Workshop/vera20k-aymod-audit`，分支`feature/aymod-integration-resume`；本轮仅测试/文档，无玩法改动。
  主工作区用户`.gitignore`、隔离历史分析工作区、旧构建与原存档保留；无推送/PR/合并/启动另一份游戏授权。
  全面PLAN的F/V/AI/视觉/UI与多电脑长局目标不缩小为本轮核查项，仍未达到整体完成条件。
- F07已纠正：独立只读调查完整追实际Weapon ReadINI772080..7729E2及两个扩展detour，固定键读取无
  AntiNaval/AntiUnderwater，未见动态拼键或外部插件reader分支。ASCII/UTF16/根DLL无字面值仅补充，不代替控制流。
  现不虚构两键的构造默认、保留字段或消费者；物理配置列C04。任意运行时注入不在已证范围。
- 实际消费者是现有SelectNavalTargetingWeapon6F3820的mode4，6F38FC..6F3915对非Hover、非Organic选slot1。
  实际AY TANY普通/精英副武器Sapper/SapperE通过生产reader确认，未改MOD、权重、伤害或新建选武器算法。
- 新增真实Return Dubai、美国对苏联困难、零额外初始部队的定向场景：正式load_session后，在实际水格放置
  已注册玩家House的谭雅和敌方HYD，普通/精英各自SelectWeapon与实际fire事件武器/发射者/目标吻合，
  普通Attack确实造成舰船掉血；此test1项（含两个rank边界）通过。潜艇传感器/悬浮与有机例外/GUI仍待验。
  初版夹具误用国家名作为House名，造成fear缺House；已改用既有house_of解析实际House，装配错误不算引擎缺陷。
- Hover本轮读了Move_To514D90、Process514310、SpeedUpdate515ED0及其已选速度路径：Unit vt538→4DB1A0，
  vt38C→70EFE0只读Type+678，70D0D0是等级能力；514442/514459是坐标转Cell/比较，不是House或EMP门禁。
  IsPowered516C70是基础powered byte或高度>0。已读路径没有给出应照抄Drive的EMP早退；不基于有限扫描
  声称不存在任何间接影响。当前paid Drive/Ship入口4B0F20/6A05F0按descriptor/turret/tube准入，
  不能擅加fresh-selection的EMP全冻结。悬浮first-hit/恢复、下游回调及持续运行仍未整体认证。
- V01/A06仍开放：NPAE实际EMEffect施加不写provider offline+1C8；Reactivate/expiry需完整分离。
  当前时长/目标/队列及Ares式offline/Hunt政策保留；三组独立计数待用户政策确认，维持原生共享计数。
  已补普通运动消费者406c9f314及Move_To eb35eca1e；对应定向/真实AY场景与独立critic结果见npatch-emp及提交。
- AI已修陆海双队列/Walk集结；海军184在同版本同map/rules的初始图即跨Water raw2/4，非读档新隔断。
  F06 AACombat、F10 GPU电弧及其他已交付链保持，不重复实现；多House持续发展/出击与全量作者补丁仍开放。
- 最新玩家版仍`aymod-playtest-20261026-emp-motion`，来自干净3026d4e16，入口
  `D:/Workshop/vera20k/target/AYMOD-test-20261026-emp-motion.cmd`。本轮不产新游戏版、不改变保存布局。
  下一安全动作：继续完整NP施加/恢复链，并按PLAN推进箱子、F08/F09/F11/F12及实际AI未验项；
  不因F07重分类或一个场景通过宣称整个海军/NPatch/MOD完成。

## 3. 功能缺口与适配进度

每一行是一条可独立收口的玩法链，不要求合并成一个大改动。

| ID / 优先级 | 类别、触发与影响 | 配置和现有 owner / 证据入口 | 收口要求 |
| --- | --- | --- | --- |
| F01 / P1 / 已实现，待GUI验收 | 步兵自主隐形：已修原先Unit-only准入；天然/等级隐形进入既有更新，影响生存、AI选敌和侦测 | `[TANY]` / `[SNIPE] Cloakable`；`[GHOST]` / `[IVAN] EliteAbilities`、`[VIRUS] VeteranAbilities` 同族。`src/sim/world/techno_ai_cloak.rs` 的事实采集和更新现已允许Unit/Infantry | 已核#174步兵准入并扩展既有owner；剩余实际GUI/运动路径验收包括静止隐形、按实际武器DecloakToFire保持隐形或显形、传感器、死亡/换主和存读档；不要新增第二套隐形状态。IVAN配置为精英CLOAK，与旧说明的老兵矛盾，不擅改配置 |
| F02 / P1 / 已接入，场景与画面待验 | 隐形箱已授予实例能力，接入共享隐形AI及持久化 | CellPickup48294F/481D3A；3项生产回归涵盖严格半径、敌我、未占格对象、重复/存读/hash与换主/Unlimbo保留。NP/无NP箱子各82项通过；独立critic修正已回归 | 实际AY正式加载箱子与GUI待验；建筑/落地飞机仅能力接收已测，不冒称完整生命周期/画面等价；新字段改变保存布局 |
| F03 / P1 / 已接入并通过实际场景，画面待验 | 爆炸箱直接C4与5次径向爆炸、动画/光照和死亡后续已接通 | 482565..482719；3项新生产回归含实际AY放箱/Move/真实500幅度和5个帧输出光照请求。同步后果通知不丢失，扩展真实桥梁回归并验证第二帧不重复；critic未发现具体缺陷 | 最终GPU/GUI画面、复杂Dummy重入及中断帧真实通知未逐项实测；不等于所有箱子或所有地图/难度通过 |
| F04 / P1 / 已实现，功能验收通过 | 驻军保护/穿透：通用建筑收伤前置钩子；正等级尾序单杀、负等级尾序清空，严格大于保护值 | 两个有符号byte逐物理层读取；WH穿透复用原Fire的+14C字节，不建立竞争状态。`src/sim/passenger/garrison_eject.rs`复用声音、RecordKill/UnInit owner；`src/sim/combat/world_receiver.rs`在自伤门和联系人快照之后接入 | 已通过真实AYMOD狙击手攻击普通/保护驻军、鲍里斯召唤BPLN/Maverick3清空保护驻军；生化炉Cargo不属于此钩子，坦克掩体保留独立链接/旧PenetratesBunker门。优化场景、普通游戏构建和独立只读critic均已完成；不冒称GUI或全地图模式认证 |
| F05 / P1 / 已实现，待GUI验收 | 独立炮塔速度：已按NPAE174分离Unit炮塔与车体，影响跟踪、射击时机 | MTNK、FV、SREF、ROBO、HTK、HTNK、APOC、HORV、HARV、TTNK、TTNK2、LTNK、YTNK、TELE 共14处物理用法；`src/rules/native_processing.rs`保留byte，`ObjectType::unit_turret_rot`为共享速率入口；`src/sim/world/world_spawn/construction.rs`初始化、`src/sim/world/unit_post.rs`持续瞄准消费 | 已核#174 reader/构造消费者并通过原地Attack；行进跟踪与实际画面待验，沿用旋转owner，不拟合速度 |
| F06 / P1 / 已实现，部分场景待验 | ORCA的AACombat现保留空中攻击目标，不改变投射物AA准入 | AircraftType逐层读取/保存/hash；既有`techno_ai::drop_airborne_target_step`按NPAE174 414D36/C27C60跳过高空清目标，仍检查隐形感知。实际AY ORCA普通Attack已伤害仍在空中的ZEP及地面MTNK | 远距移动目标截击、精英武器对空、玩家GUI与持续AI战术尚未单独验；不将此链扩大为所有飞机行为通过 |
| F07 / P1 / 已有实现，部分待验 | 谭雅海军副武器由原有NavalTargeting=4选择，不是两个新Weapon flags | 实际TANY Secondary=Sapper、EliteSecondary=SapperE；`combat_weapon::select_naval_targeting_weapon`匹配6F3820/6F38FC..6F3915。Return Dubai正式加载后，普通/精英Attack均实际发射对应武器并伤害HYD | 已检查的NPAE174 Weapon reader不读AntiNaval/AntiUnderwater，见C04。仍需单验感知/未感知潜艇、Hover/Organic主武器例外与GUI，不扩大为所有海军行为通过 |
| F08 / P2 / 已接入，完整投弹/画面待验 | IsFlatExplosion与CausesDelayKill顺序别名，接既有建筑PostMortem，不另造平面伤害 | C2EF94/75D443共同写WH+130，701E92消费；逐层保留/保存/hash及3项回归通过。实际AY ZEP primary→BlimpHE直接命中CAMISC02后引爆，debug/fast均通过；critic无具体缺陷 | 未认证基洛夫完整飞行投弹、GUI或所有既有引爆算术；其他使用此别名的弹头共享reader，未逐个实战验证 |
| F09 / P2 | 敌我过滤：`DoesntAffectEnemies` 没有对应 NPAE reader/消费者 | `[TankSnapshot]`；`warhead_type.rs:459` 读的是另一个键 `AffectsEnemies`；Ares 同类过滤也不等价 | 追发射者/受益者/目标 House 和效果类型，覆盖敌、己、盟；不直接把值取反映射到另一个扩展 |
| F10 / P2 / 已实现、玩家待验 | 自定义电弧已从保留reader经出生参数接入GPU绘制 | 实际TankBolt/TankBoltE生产开火及RGB565 GPU读回通过；默认/禁用/逐层开启、存读/hash与无NP隔离通过。TeslaFragmentT保留颜色但不强开IsElectricBolt | 玩家新局观察普通/精英磁暴坦克电弧；不把EMP逻辑正常代替配色验收 |
| F11 / P2 | 心灵终结器独立光照：超武 section 的 NP 光照覆盖未接通，不等于 stock 全局光照缺失 | `[PsychicDominatorSpecial] Lighting / LightRed / LightGreen / LightBlue / LightAmbient / LightGround / LightLevel`；`src/rules/superweapon_type.rs`；`src/sim/superweapon/psychic_dominator.rs:8` 当前使用 map 的 Dominator profile | #174 启动、叠加/互斥、渐变和复原；实际输出验证，覆盖取消/结束/存读档，不污染下一次超武 |
| F12 / P2 | 扩展遭遇战 UI：8色大厅/战场错配本轮已修；科技等级滑条与完整16色界面仍开放 | `[UISettings] ShowTechLevelSlider / UseDropDownColors`、`[Colors] MPC01..MPC16`；`src/ui/skirmish_shell/layout.rs:159` 只有速度/金钱/单位数滑条；`src/skirmish_launch.rs:18` 颜色数8；`src/app/frontend/skirmish_shell_render/controls.rs:353` 固定8色 | 先核 #174 颜色表与选择政策，再接配置/选项/启动数据；30槽目标不等于已支持16色，验证颜色占用、同色策略和人/AI辨识 |
| F13 / P2 / 已实现，待画面验收 | NPatch超五乘员开火端口循环：已修第六名原先错误回退零偏移的问题 | NPAE174 GetFLH `0x6F3AFB → 0xC2E196`按`(k-1)%5`循环，ART reader `0x715FA4..0x716005`仍只读0..4；固定ART5/6是本链不消费的数据。`ArtEntry::alternate_fire_flh`为唯一共享owner | 六人实际Enter后的生产坐标已验；不扩大数组。下车重上/移动转身的实际画面仍待验，不把坐标测试称GPU输出认证 |

F06–F09、F11 的“缺失”指扩展读取/接线缺口；**原生效果定义尚未全部证明**，
不能直接依据键名写实现。F01–F03、F13 是首版新增的明确生产路径缺口；F13的五槽容量本身正确，缺的是NPAE循环。

## 4. 已有实现，仍需核对的功能链

| ID / 优先级 | 类别与当前边界 | 下一次验收 |
| --- | --- | --- |
| V01 / P1 / 原生与作者设计冲突仍开放 | 三组供电已有owner；普通地面Drive的provider/EMP分离与恢复已修，原生共享House计数保留 | 原生允许失联Drive被AI重新下令移动；作者说明却要求三组指定建筑失去支持即瘫痪。不能把此差异归因于漏读PoweredUnit或把EMP修复标成蜘蛛已修；独立供电/禁移动须明确兼容策略 |
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

初次审计未确认固定 AIMD 活跃战术故障；后续已定位并修复陆海生产选择共享问题，尚不等于完整 AI 验收。
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
| A06 / P1 / 普通Drive重叠已修，整体待验 | 供电/EMP/捕获/队伍/工厂既有owners；NP Drive重叠保存、恢复/不恢复、无Hunt通过，已付款队列政策保留 | 多House/长局、特殊运动器/建筑/管理器及作者设计供电策略仍开放；队员归属与重新组建不能仅由一次到期通过推断 |
| A07 / P1 / 待验 | 长期经济恢复：`src/sim/ai_build_need.rs` 已含卖建筑、矿场/矿车重建和绝境 Hunt | 矿枯、矿车/矿场/重工损失、缺钱后的恢复或败退；无永久等待，长局各 House 资金/生产/团队计数不串 |
| A08 / P2 / 已修配置、待验 | 两个光棱 TaskForce `0CE4DCDC-G` / `0A6E3F1C-G` 已改为 SREF | 根 AIMD不再列 SERF 为现存问题；实际相关编队补齐、出击和光棱作战才关闭行为验收 |
| A09 / P1 / 已修配置、待验 | 攻城直升机 Script `07C3188C-G` / `07C48DBC-G` 的 Guard 已为 `5,0`；Team `07C460CC-G` / `07C3092C-G`；活跃触发 `07C30A5C-G` / `07C4853C-G` | 两架 SCHP 执行 `54 → 5 → 9 → 0...`，移动、落地部署、开火和再次移动；不是只加载成功。现有 TickTank DeployToFire/DeploysInto 注释残留不能直接归罪 SCHP |
| A10 / P1 / 待验 | 作者170601称电脑已修巨炮/裂缝建造并会用超级裂缝；已有基地选择、部署/Gap owner | 盟军AI实际建巨炮/裂缝，实际使用超级裂缝相关单位/能力；核当前生效配置和消费者，不把旧作者 bug 当当前引擎必然存在 |

A04 双队列已按实际NPAE174指令适配，见本轮定位与 [npatch.md](npatch.md)；海陆编队实战验收、择兵同需求策略和可选并行限制仍开放。

### 固定配置残留：单独管理，不猜修复

| ID / 优先级 | 引用链与现状 | 后续动作/风险 |
| --- | --- | --- |
| C01 / P3 | TF `07C338DC-G` 含未知成员 SHD → Team `0610813C-G` → AITrigger `0616004C-G` / `07C330CC-G`；根 AIMD 两条触发的三个难度位物理值均0 | 当前固定触发不可达；不能写成“活跃碉堡队少造兵”。地图可重新启用，届时按未知成员忽略政策运行；不猜为 SHK/SCHD |
| C02 / P3 | 已注册但缺定义的 TF `05FC0DEC-G` → Team `060F883C-G`（Yuri Armor - H3）；根 AIMD 固定 AITrigger 未引用该 Team | 保留身份，不能补抄原版编队；地图/脚本重开才会产生实际影响，若恢复战术需作明确维护设计 |
| C03 / P2 | 短行 AITrigger `0C8C51BC-G` / `FFFFFF12-G` 的对象类型字段为空；顺序token读取跳过空字段，移读后multiplayer=false，当前遭遇战不具资格（FixedAimd的enabled仍为true） | 两条原配置战术尚未恢复，但不是enabled=false或新的加载回归；reader `src/rules/team_ai_ini.rs:625`、空项读取`src/rules/ini_value.rs:804`、拒绝入口`src/sim/ai_team_creation.rs:393`。先核作者意图与原生读取，不加占位值擅自启用 |
| C04 / P3 | Sapper/SapperE物理配置含AntiNaval=yes、AntiUnderwater=yes；实际已检查NPAE174 Weapon reader不读取两键 | 已逐分支追772080..7729E2，以及772901→B937F8/B96950/B96AC8→772907和7729AA→B94D06..B94EB1→7729B0，固定key读入无这两项或外部reader调用；字节/根DLL字面值检查只是补充证据，不单凭搜不到作结论。谭雅对舰由现有NavalTargeting4解释且已实测，不添加无原生定义的默认值/字段。换目标补丁/有运行时注入证据时重审；不宣称排除了任意外部注入 |

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
