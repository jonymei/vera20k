# AYMOD 兼容性清单

更新：2026-10-28。面向“人类玩家与多个电脑对战”的分批适配工作清单。
完整执行顺序见 [全面适配 PLAN](aymod-plan.md)。这是当前唯一的兼容性台账；[Windows 交接](aymod-windows-handoff.md)记录此前交付历史，
不是另一个待办列表。**没有把能启动、能读配置、能创建编队当作整个 MOD 已兼容。**

## 当前检查点（2026-10-28）

- 工作区D:/Workshop/vera20k-aymod-audit，分支feature/aymod-integration-resume；本轮基于2b9c9e46c补F05实际MTNK追击正常横向移动SMCV，真实位移中的车体/炮塔分向与移动开火、持续独立速率、运动中完整存读/续射实伤。最终实际1、turret组件3、FacingClass14、combat_turret_facing7，共25项通过。仅测试/台账，无生产改动/新critic/待完成Cargo；Ares旧两项gap夹具失败保留，未发布，整体goal开放。
- 玩家固定入口D:/Workshop/vera20k/target/AYMOD-test-20261026-burst.cmd、标签aymod-playtest-20261026-burst、源cfb479750不变。未启动GUI、未改MOD配置/原始存档、未替换固定构建；后续修复不会自动进入该版。
- 纯NP保存布局因私有生命周期bool改为enum而变化，当前构建存读已验；旧布局由既有校验拒绝，不提供强读/迁移或重写原档。固定构建仍使用自己的旧布局，不受本分支源码变化影响。
- Return Dubai弱攻势线继续暂停，不调AI权重/兵种数值。此前陆海双队列/Walk集结有修复，跨水区FNPC、SCHP hold、抗压和多AI长局仍开放；非盟军巨炮修复b5a046558已通过三阵营计划及实际AY三困难AI基地发展/巨炮生产落地验证。固定版不含，旧计划/巨炮不追溯删除；GUI及占领异阵营科技后的边界未验。
- V01独立供电/失联Drive政策、F11跨模块光照接线批准门禁不变；一般“继续功能”不解除。功能优先，UI随后，完整PLAN有效。
- Building+30C NaturalParticleSystem在当前Battle/XMP03T4和Return Dubai正式typed列表中均为None/位置0，通用非空配置生命周期仍未实现；不是+308/+310或RefinerySmoke，证据与适用性见下文唯一对应节。不能把NaturalSmokeLocation擅作别名。
- 下一安全动作：继续F06实际ORCA空弹返航/重装后对空对地续战及存读链，先核现有aircraft mission/dock/ammo/AACombat owners，不重写已完成Alt维修入口。F05已补普通MTNK运动组件，不扩称全部14类型/特殊大转角热载入或GPU。Return Dubai攻势暂停，V01与F11批准门禁、固定构建不变。
- V05已有cursor0的单剩GGI续火/存读、ScenarioInit内非尾移除后的顺序通过；count1/cursor1具体GetWeapon→Cant→drop与180帧存读/补员恢复本轮已核；实际普通SNIPE发射产生同一cursor1越界、两次尾序删除/保存已验；其他名单删除来源/原生循环或多AI仍未关闭，不要求正等级随机删非尾。母子机其他家族/连续晋升/旧控制者指针选择仍开放。V08解散后的重新招募/六人重新装载存读实射通过；带队卸载/竞争补员/多AI长局仍保留。Alt飞机入口修复及其实际GADEPT/GAYARD验证保留，不重写已完成链。
- V04多House/联盟重叠、V07多AI/原鱿鱼-海豚崩溃完整回放、V06完整ROF倍率/V12特殊回调、ForceShield混合保护/自主多AI边界、铁幕反击/脚本边界、V03空武器reload fallback及真人验收仍开放。此前缓存清理因Unsupported PE debug closure安全停止、移除0字节，本轮未手工清理固定构建。旧Python936项失败/错误保留，本轮无Python工具改动。

### V10坠落飞兵：NP跳过步兵Health=1复活（2026-10-28）

- 作者170507说明是“修正了原版中火箭飞行兵坠落过程中可受到地面部队攻击的BUG”。实际AY GameMD.exe的Infantry vtable7EB058、AI槽+5C指向51BAB0；InfantryClass::AI51BC57读取Health，51BC67为EB34无条件跳至51BC9D，跳过51BC96的Health=1写入，再进入FootClass::AI4DA530。该补丁覆盖所有步兵，不仅Jumpjet；不能另造坠落目标层、AG免疫或专属兵种判断。
- 现有ObjectSubstrate::infantry_health_reset在NP模式直接返回，stock保持原逻辑；唯一生产调用仍在techno_ai的Foot/Techno AI之前。原生高空虚函数+54经Foot4DE620到Object5F6B90，既有208高度判定未改。没有新增状态、reader、保存布局、RNG调用、timer写入或Detach；保持HP0会自然改变后续AI/动作抽样，不声称与错误复活路径逐帧RNG相同。
- 先失败回归：普通致死receiver击落正在非死亡动作中的飞兵，下一普通帧旧实现HP1而期望HP0；修后每个离地坠落帧均HP0，共享目标setter拒绝，覆盖低于208和最终退休。另测非Jumpjet的ENGINEER，NP保持0、stock恢复1，保护实际补丁的全步兵范围。
- 实际AY Battle场景：JUMPJET经普通Move自行起飞，使用正式致死receiver进入坠落，穿过208高度后落地退休；未手写高度、动作或坠落状态。该例不是实际AA武器命中；另有零售Dustbowl的HTK真实开火击落、整段下降/目标清理/无尸体回归在NP与stock均通过。没有新增原生执行golden；原stock crash corpus保持不变且只在stock断言，NP使用独立行为回归。
- 最终retail gate下，NP/stock各运行jumpjet_infantry16、infantry_action16、infantry_death5、chronosphere13、cell_receiver14项；另各跑零售实弹坠落1项，实际AY场景1项通过。旧零帧死亡序列测试与零售坠落测试原先硬断言HP1，在NP下先失败，现依据51BC67区分模式，其余生命周期/下降/退休断言保留。只读critic无可操作发现，之后补齐上述consumer测试；未启动GUI、未改MOD/原始保存/固定玩家版。
- 残余：未验证真实AG-only攻击者完整自主索敌/开火循环、NP原生下降/RNG逐帧对照、坠落中存读或多AI实局；不把共享setter拒绝冒称这些都通过。V10其他独立机制继续保持各自状态。

### F02完全隐形建筑受击与当前NP EMP策略的状态组合（2026-10-28）

- 将既有普通MTNK拾箱/Building与落地ORCA能力授予/正式存读/自然完全隐形前缀抽为同一aymod_cloaked_building_and_landed_aircraft夹具。原盟友DOG/红血/换主例仍用GAPOWR；新例用GAPOWR和GAROBO，对Powered的后者正常提供远处GAPOWR供电，不改规则或Cloak状态。
- 两种建筑在完全隐形后经正式Super非致死1点伤害，HP确实少1，马上进入state3；保存/完整资源恢复保留state3并自然完成过渡、重新隐形。实读Building vtable7E3EBC+FC（7E3FB8）为703850，Techno70281D通过该共享wrapper进入7036C0；不是另写建筑受击状态或把已显形时的伤害代替本例。
- EMP使用正式typed TTNK.Primary -> projectile.AirburstWeapon -> TankBoltEMP -> EMPulsWH链，确认实际子弹头EmEffect=true；初跑错误选了主武器WH，断言失败后改为正确的生产引用链，未添加假Secondary/预注册孤立武器或修改MOD。直接进入既有NP EMP落点producer，不写counter；这是影响处理的组合验证，不能冒称TTNK实际选中并打中未探测隐形建筑。
- 上游时长/目标准入仍为[npatch-emp.md](npatch-emp.md)记录的现有兼容政策，不是本轮证明的#174算术：Powered=false的GAPOWR未获counter且保持完成隐形；Powered=true的GAROBO获活EMP，普通帧后不保持state2，正式保存/恢复保留非零counter及相同state/building_stage。剩余counter>0的每一后续帧均不保持完全隐形，到期后自然重新隐形，再保存保留HP、实例能力与state2。
- 不强制EMP期间必须停在state0。实读Building455012..455053分别询问ShouldUncloak和CanAuto，不能从Foot的state-zero head照搬到建筑；本例只检查完成隐形不被保留，不扩称EMP过程的全部可见透明度或循环相位。该F02批没有生产修改；后续建筑EMP生命周期迁移见V02对应节，本例已在新路径重跑通过。既有provider/失联Drive政策批准门禁未变。
- 最终实际aymod_cloak_crate_三项（新项含两种建筑）及combat_cloak_damage_tests的已死对象负边界1项通过。仅测试，无新生产状态、RNG调用、timer/Detach实现或保存字段，无GUI/固定构建/MOD/原始存档/发布改动。受损/EMP的GPU/GUI、实际隐形目标发射准入、全部建筑家族及多AI长局仍开放。

### F02不同House盟友传感器、红血及重复隐形循环（2026-10-28）

- 扩展原aymod_cloak_crate_cloaks_a_building_and_landed_aircraft_across_load，保留普通MTNK拾真实箱子、建筑/落地ORCA保存恢复、同House DOG不显形和换主后敌方DOG显形的原前缀。未替换原敌对案例或重写Cloak owner。
- 经两个正常Command::Ally按真实House数组索引双向建立联盟，生产are_houses_mutually_allied确认为真；实际DOG仍在建筑基础的一格接触边界内，建筑持续30帧不能重新隐形。静读Building CanAuto457770/Should4578C0的House比较与Sensors分支，直接比较两House指针而不是AreAllied查询，因此不能自行豁免盟友。
- 在已显形时，通过现有direct damage receiver/Super/IgnoreDefenses非致死伤害使GAPOWR恰好HP1，生产health_category确认红血，未改Strength/ConditionRed/伤害倍率或直接设Cloak状态。正式存读保留盟约、箱子能力与显形状态；普通DOG Move离开后恢复全隐形并存读，HP仍1。
- 同一活DOG再普通Move返回原接触区域，红血建筑再次退隐到0，正式存读并保持30帧拒绝重隐；再次Move离开后回到完全隐形，最后存读保留HP1、能力、全隐形和盟约。两次离开都确认探测单位活着，且未停留在原接触位置，不能用传感器死亡替代真实运动解除；没有写sensor计数、Cloak阶段或随机状态。
- 最终实际aymod_cloak_crate_两项通过（原移动单位、扩展建筑/飞机场景）。仅测试，无新RNG调用、timer/Detach实现或保存字段，无GUI/固定构建/MOD/原始存档/发布变更。不扩称全部红血时机/RNG、EMP、隐形中受击、敌方多AI战术或受损建筑/烟火的GPU/GUI通过。

### V10实际AY秘密实验室奖励池、双授权与付款撤销存读（2026-10-28）

- 正式AY Battle/XMP03T4读取实际General的SecretInfantry/Units/Buildings，typed合计28项，CASLAB无指定覆盖奖励；不改MOD/地图/规则。测试fixture经正式spawn放29座中立CASLAB，确认场景先前无实验室，再单次调用既有assign_secret_lab_production。前28座奖励互异，第29座仍获池内奖励，覆盖#174取消labs>choices早退的实际配置边界；不手写fallback、不重置随机流。该调用在fixture布局后，不冒称原样地图Post_Map_Init启动覆盖或所有概率已统计验证；具体RNG/reset语义仍以既有原生golden为准。
- 从实际分配中找到TERROR与DESO的两座，普通ENGINEER CaptureBuilding事件步行捕获；美国玩家原先两项CanBuild=No，捕获第一座仅TERROR获准，第二座后两项均获准，House实验室列表确为两座。真实GAPOWR/GAPILE提供独立供电与工厂，不注入科技或改兵种Owner/前置条件。
- 正常QueueProduction排两次TERROR，实际花钱且付款头在1..53进度时正式保存/完整恢复；两授权及各lab原fallback保留。经既有change_owner_with_rules receiver让第一座归中立，TERROR立即失去授权，正常帧删除付款头及排队副本；DESO和第二lab列表保持。第二次正式存读保留上述差异及两lab原奖励，未重抽奖励；这是换主receiver，不称敌方工程师再捕获。
- 保留的DESO经正常生产事件付款到54步、实际人类PlaceProducedMobile事件从兵营出厂，同一生产对象活体/非limbo/玩家归属和类型正确。早期fixture误用不存在的GABRCK导致free-cell失败，改用实际GAPILE；生产完成未出厂的调查中，留出兵营通道并未解决，实际缺少无窗口测试的人类PLACE事件。补正常事件后通过，不改玩法或手工Unlimbo，也不把此前失败记作授权bug的红证据。
- 最终retail gate下新actual场景1项、secret_lab原生抽样/reader/建造准入/付款存读/战役门禁等9项通过。仅测试/台账，无新增生产状态、reader、RNG/timer/Detach调用或保存字段；不需玩法critic。未启动GUI、改MOD/原始存档/固定构建或发布。原样作者地图启动、GUI、全部奖励类别的出厂/落地、实验室毁坏/多AI长局仍开放，不关闭全V10。

### V10原样作者地图实验室：真实启动、捕获与死亡授权清理（2026-10-28）

- 直接正式load_session加载未修改的Operation Lone Guardian II(2-8).yrm、Battle/美国对苏联Easy、unit_count=0。MapFile的唯一CASLAB及实际实体一一核对出生坐标/中立主人；load返回时已具实际池内fallback。此例没有手工分配、添加/替换实验室或改地图/规则，覆盖Post_Map_Init686AFE→68C050真实入口，不靠上节fixture调用代替。
- 初始美国CanBuild=No；第一次正式保存/完整资源恢复保留同一lab/fallback。附近放实际ENGINEER后下普通CaptureBuilding事件、正常步行占领，CanBuild转Yes；第二次保存恢复仍是原奖励和授权。不是从基地走完整张地图或测试AI自主占领。
- 正式Super致死receiver使lab进入正常死亡链，普通帧等待原House实验室列表移除，授权回No；第三次正式保存恢复保持列表空/奖励撤销。没有直接清列表或UnInit代替伤害退休；此例不冒称实际投射物摧毁、整个App画面或新奖励类别实产。
- 最终retail gate下secret_lab的两个ignored真实AY场景一起通过（作者地图本例、上节29座/双授权/付款取消/实际出厂）。原有9项原生/reader/直接消费者本轮未重复执行，上轮通过保留。测试/台账-only，无新增生产状态、RNG/timer/Detach或保存字段，无新critic；无GUI/MOD/原始存档/固定构建/发布改动。原样地图单次启动与该抽中奖励不等于全部地图/全部奖励或多AI长局验收，不关闭全V10。

### V02治疗途中解盟与同帧双患者预约存读（2026-10-28）

- 从自然缺电例提取同一aymod_allied_hospital测试夹具：正式AY Battle场景、真实医院House/独立电厂、受伤步兵、结盟前拒绝Enter及双方普通ALLY事件。原低电测试继续消费该夹具，未复制医院/外交/预约玩法owner。
- GAROBO/NARADR各自让ENGINEER经普通Enter入院，等真实Repair运行后用普通ALLY事件解除联盟；正式保存/恢复保留患者inside关系，普通帧治满、活体出院、Cargo与radio自然清空、归属不变。空院对新受伤E1不再提供Enter，第二次保存保留敌对关系/原患者满血/清空预约；不把满员NoEnter混作外交拒绝，也不扩称伪造敌对Enter包被拒。
- 两名ENGINEER在事件处理前均获正常Enter cursor，同一帧下达普通Enter。第二名通过Infantry51AC6D..51ACC5自然记录pending_entry；在该等待边界完整保存/恢复后，普通帧两人先后确实入院并治满出院。逐帧检查Cargo至多一人、limbo患者与Cargo及inside双向一致、两人始终存活且归属不变；终点再次保存保留两人满血/非limbo、pending为空和radio清空。没有手工塞Cargo/清预约/重发命令或新建医院FIFO。
- 实读实际AY Infantry51AC51..51AD00、Building43CB0C..43CBBE和Hospital44B870起始指令，结合既有Hospital44B870..44BA79/RepairTick6F4CD7 owner确认：已入院治疗不逐帧重验外交；新CanLoad43C311仍核外交。两种行为不应统一成解盟强踢或永久放行。
- 最终retail gate下纯NP实际医院9项通过（含既有低电/Off/EMP/占领/死亡/出售例）。仅测试与台账，无新生产状态、RNG/timer/Detach调用或保存字段，无需新玩法critic；未启动GUI、改MOD/原始存档/固定构建或发布。不扩称步行途中解盟、超过两人的争抢/死亡/中途取消、任意外交时序或自主多AI通过；完整TTNK实战与占领旧患者其他恢复仍开放。

### V02建筑EMP：保留治疗任务、雷达顺序及付款队列恢复（2026-10-28）

- 真实差异：旧纯NP建筑复用了Ares的enable_ares_emp→Deactivate70FC90→ResetOrdersToGuard，把运行中的Hospital Repair改为Guard。正确红对照先等待Cargo入院后Repair真正commence，再临时路由到原legacy后端，实际失败MissionId5而非20；源文件在对照结束后恢复。此前“Cargo已接收但Repair还排队”或错用国家名查询动态House雷达的夹具失败不计作玩法红证据。
- 复读实际NPAE：B95185..C6的Building6分支直接到B9914C，跳过Foot目的地/伪装/Off声/EMPAnim构造和抽样；写+504后CaptureFreeAll、SpawnKillAll、Building443B90→Techno6FCDB0(null)清战斗目标，不清任务/radio/Cargo或写provider+1C8。Building43FE56→Techno6FA655/MissionAI5B3060仍在倒计时之前调度，Hospital44B870..44BA79无EMP停疗门禁。接通既有共用EMP owner的建筑分支，不加医院专属复活/任务恢复器。
- 到期6FAF2A..81调用既有4547C0槽电力丢失owner，Radar标House+5779后退出，不走Foot激活/全局EMPAnim结束循环；正常4549B0 operational边沿仍由原owner负责。时长/目标准入、90默认/900钳制及混合后端优先级保持[明确政策](npatch-emp.md)，不宣称所有#174接收算术/免疫已等价。
- Radar命中时B951AB在新计数写入之前调用House508DF0，保留“首击看旧0、即时等值重击看已有计数”的顺序。一次只读critic纠正了初稿的终止扫描判断：C2BCB8对非Radar/非零EMP均跳508F08继续下一座，不得让瘫痪首座遮住健康第二座；已有warp结果语义不改。双提供者回归先失败、修后通过。原House更新的雷达计算/客户端发布提取为同一refresh_house_radar动作供两个入口调用，未复制扫描或另建可用性状态。
- 既有NP付款策略保留：EMP时保持原付款头/FIFO、玩家Hold/手动Off不被覆盖；到期仍缺电/Off的工厂，用私有EmpLifecycle::NativeFactoryAwaitingPower记录原恢复等待，共用find_factory消费只读查询，operational后通知原FactoryRegistry恢复。普通未被EMP搁置的低电工厂仍按原规则生产，不通过假provider latch或ResetOrders实现等待；新增该等待状态的完整存读回归。生命周期enum由原EmpState单一持有，初始legacy正计数不在中途强改策略；纯NP新布局不强读旧档。
- 实际GAROBO/NARADR测试以真实TankBolt→ElectricbounceT→TankBoltEMP/EMPulsWH配置进入已有impact producer，观察两院原有患者在持续真实50帧配置脉冲中治疗、到期正常出院；空院再次EMP时新患者正常入院、第二次正式存读后治满出院。无直接写EMP计数/任务/经验/时间；不是完整TTNK对医院发射回放。建筑每次命中不再抽Foot sparkle的[0,25]随机数，既有manager/槽owner保留其正常timer/Detach副作用，不保证与旧错误路径逐帧RNG相同。
- 最终retail gate下NP57项（EMP15、医院7、实际付款工厂1/隐形建筑EMP1、power7/radar1/工程师消费者5/发现服务2/建造12/槽电力5/实际科技分支1）；stock30、Ares42、混合29项通过。critic仅一次，确认的扫描问题已修并自行重验；未重跑全workspace/Clippy或启动GUI，未改MOD/原始存档/固定玩家版、未发布。全部linked-manager重入、特殊高度/类型转换、完整建筑家族实战、多AI长局及建筑EMP像素仍未验，不关闭全EMP/V02。

### V02盟友医院、自然缺电连续接诊与恢复存读（2026-10-28）

- 新实际AY场景分别使用GAROBO/NARADR：本地人类的受伤E1送往另一真实human House的医院；第三House经正式国家构造器创建并提供实际医院/电厂，不改玩家控制标志、外交图或医院Cargo。结盟前鼠标不提供Enter；双方普通ALLY事件实际建立关系后，普通Enter走真实移动/radio/Cargo入院。本轮是多House治疗组件，不是自主多AI证明。
- 患者入院后，从正式致死receiver摧毁医院House唯一电厂，不改电力计数/online位。正常帧确认House低电、医院operational=false而+660 online=true；本地盟友另有实际正发电量，且医院House仍低电，防止错误共用电网造成空通过。自然缺电时已入院E1正式存读后继续治满出院；等原radio预约真实清理后，再送入HP1的盟友BRUTE，仍在低电下准入、第二次存读、治满出院，两个患者都保持本地House，不变成医院主人的单位。
- 原生分支复读：Building CanLoad43C311..43C323先核外交、43C422只核+660，未在此调用IsPowered4555D0；Hospital44B870..44BA79的治疗与RepairTick6F4CD7沿原owner，不按全House缺电停止。因此“手动Off拒新、已有病人继续”和“自然缺电仍准入、继续治疗”不同，不能新增统一operational拒绝来改成想象中的逻辑。
- 初次测试在首名患者已离开Cargo的同一帧马上要求下一人可入，观察到Repair当前任务/Guard排队与旧contact仍在，暂时NoEnter。补齐普通帧的预约清理终点后连续接诊通过；未手动清空radio、复位任务或补一个新的清理owner。恢复阶段补入同House真实GAPOWR作为供电恢复输入（不是付款建造验证），正常帧恢复operational，第三次正式存读保留联盟、医院主人、两患者归属与已清运输关系。
- 最终实际医院6项（含本例的两医院类型）、NP radio::receive18、docking::building_dock28、world_orders::passenger_entry4通过；该批仅测试/台账，无玩法实现/新critic，无新增RNG、timer/Detach实现或保存字段。未启动GUI、未动MOD/原始存档/固定构建、未发布。EMP由上节接续；满员争抢/患者途中解盟、占领后旧患者其他恢复及自主多AI长局仍开放，不关闭全V02。

### V02被夺医院的新服务、旧载员保留及正常出售（2026-10-28）

- 新actual aymod_captured_hospital_serves_new_patient_and_sale_cleans_old_cargo_after_load与原占领/毁坏例共用aymod_hospital_captured_with_patient，仍是同一实际GAROBO/BRUTE正常入院和SENGINEER步行占领，不复制换主/入院实现。保存/完整资源恢复后正常推进120帧，旧病人的House、HP、limbo及inside关系保持、radio contact0仍空；这是有限窗口观察，不是全局永久停滞证明。
- 继续为新主人提供实际E1伤员，经普通Enter进入，真实Cargo头序为[new, old]。第二次正式保存/恢复后，E1自然治满并活体离开，新House及无inside关系正确；原病人仍在旧Cargo槽，归属/HP/limbo/inside元组不变。没有为了恢复服务重置Guard/Repair、radio、Cargo或治疗时钟。
- 已核原生Guard449817..449833的自动Repair门只读UnitRepair/UnitReload/Bunker，不含Hospital；医院44B91E只向contact0发RepairTick，44B9B7经473430取Cargo head后调用原ExitObject。这个实际场景没有把失联老成员错误当成新病人出院，也未把新一轮治疗无差别施给旧病人，不添加自动收编/弹出政策。
- 新主人随后发普通SellBuilding，卖楼正常结束，医院和剩余旧载员均退休；第三次正式存读没有二者引用，已经出院的E1仍活、HP>0、非limbo且保持新House/无inside。静读ObjectUnInit5F65F0/5F6603..5F660D确认所有Techno的剩余Cargo走KillPassengers(NULL)，未把原生出售清理改成额外救出旧病人。
- 最终新项1与原医院4项通过；无新玩法实现/critic，无新增RNG、timer/Detach或保存字段，无GUI/固定构建/MOD/原始存档/发布改动。本占领场景不覆盖原病人的其他恢复操作、不同贴靠位置/夺取方式、满员/EMP或自主多AI；联盟/自然缺电由上节另验，V02不整体关闭。

### V02真实工程师占领时的病人归属、断联及毁坏存读（2026-10-28）

- 新actual aymod_hospital_engineer_capture_disconnects_patient_before_saved_destruction沿实际GAROBO Hospital普通Enter接入BRUTE。采用真实高生命步兵留出自然治疗窗口，只把初始伤口设为HP1，不改兵种Strength/速度、InfantryRepairRate或任何MOD配置。接收方是经实际国家注册表构造的第三human House，提供真实GAPOWR避免short-game败北及电脑捕获后自动出售，非自主多AI证明。
- 发普通CaptureBuilding命令给新House的SENGINEER；观察到真实世界坐标位移、医院所有权变化和工程师被消耗，不再直接调用change_owner_with_rules或手工断radio。占领时病人仍受伤、活着且在limbo/Cargo中，保留原House；医院换给接收方，contact0为空。
- 该实际患者落点距共用GetDockCoord大于0x3F，GAROBO不是WeaponsFactory；静读Building448644/448649..44864C比较0x40后跳4485AA，那里先发0x17 RUN_AWAY、再发3 OVER_OUT。不能因建筑换主就自动把所有Cargo改House；别的实际贴靠坐标/类型分支仍另验。另读Techno701524..70156E的非Selling/非simple-deployer特例会QueueGuard，但未据此穷尽后续Update或宣布恢复治疗永远不可能。
- 将a44f2c427原有“剩余Cargo保存/毁坏”断言提为同一测试owner：供应Radio Break的旧边界和本次真实工程师边界共用完整保存/恢复、毁坏声音前活体Cargo、最终UnInit/帧尾退休及二次存读无幽灵成员。普通占领现在可实际产生该断联Cargo窗口，补齐上一轮只供应radio断联的触发缺口；毁坏仍在正式致死receiver边界提供，不冒称炮弹实战或改变联系人C4清理规则。
- 最终AY医院4项通过，旧两类型Off/On治疗及旧断联死亡回归一并通过；无新增生产状态、RNG、timer/Detach或保存字段，无GUI/固定构建/MOD/原始存档/发布变更。新主人后续服务/卖楼收尾已由上节接续；旧病人其他恢复方式、联盟/满员、其他夺取方式及自主多AI仍开放，不关闭整个换主或V02。

### V02已入院病人的手动Off与治疗存读（2026-10-28）

- 共享aymod_hospital_patient_inside复用原实际GAROBO病例的正常生成/受伤E1/Enter路径，只在limbo且真正Cargo包含patient ID后返回；死亡回归与本次两类型共用该夹具，不复制第二套入院/治疗状态。沿正式规则提供GAPOWR供电，初始HP1是夹具输入，不改InfantryRepairStep/Rate或MOD资源。
- 新actual aymod_hospital_keeps_treating_its_admitted_patient_offline_after_load分别对GAROBO/NARADR在已经入院、尚未满血时发普通SetBuildingOnLine(false)，正常帧后Building+660为false且operational=false；正式保存/完整资源恢复保留当时HP、inside指针与Cargo名单。保持Off推进，确认HP确实增长并治满后活体出院、原owner不变、反向关系/Cargo清理；普通On恢复operational，第二次存读后仍为online、满血且非limbo/无inside。
- 这是原生政策而非新设计：43FE54无条件委托Techno6F9E50的Mission AI；Hospital44B870..44BA79及RepairTick6F4CD7对既有患者不读Building+660，CanLoad43C422..43C438才用该位拒绝新准入。不得把“Off拒绝新病人”错误推广成“已入院也必须停疗”。两种实际类型的行为均通过，没有为测试写入rearm/治疗计时或伪造离开事件。
- 最终实际医院3项通过（原Off拒绝/On治疗、原断联Cargo死亡回归、新项含两个医院类型）；仅测试/台账，无RNG/timer/Detach或保存字段修改，无GUI、MOD/原始存档/固定构建或发布变更。未扩称自然缺电、EMP、治疗中换主、多House/满员或电脑自动送医通过；没有整体医院验收完成的结论。

### V02/V05医院懒分配Cargo及建筑死亡清理顺序（2026-10-28）

- 纠正旧库存判断：Passengers>0只找到YAPOWR/YAPOWRAI，并不能排除非吸收Cargo。真实GAROBO的Hospital=true、Passengers=0、非Absorb/Explodes，但普通受伤E1的Enter经Infantry51A38A仍进入懒分配Cargo；必须看到实体limbo和Cargo中真实ID才算入院，早期Transporter指针尚可能只是接入阶段。正式保存/恢复保留该状态。
- 先核清独立的Building442511..4425EE联系人死亡臂：相近的治疗联系人会在DestructionEffects前收到真实C4致死调用，并经PointerExpired从Cargo移除。这不是本轮要取消的行为；临时owner诊断确认该调用链后已全部移除。回归通过既有Radio Break生产边界提供断联，保留实际Cargo且清空contact，隔离最终剩余Cargo路径；不冒称普通玩家点击可直接制造这个窗口。
- 原代码在BeforeBuildingDestruction提前KillPassengers，使这种剩余Cargo在毁坏效果之前被清掉。原生DestructionEffects441F12把C4AppliedBy传给SpawnSurvivors441F1B，随后441F20/441F27用原毁坏调用的attacker执行KillPassengers。现复用原kill_passengers owner，移到既有SpawnSurvivors调用后；保留Techno702603..702667的Explodes早杀分支、吸收建筑Phase A逃生和garrison442625弹出顺序，未添加Hospital特判。
- BeforeBuildingDestruction不再需要attacker字段，完整迁移core及客户端灯光测试消费者，不留无用参数/竞争清理owner。原KillPassengers的Team移除、递归Cargo、RecordKill/UnInit继续原路径；没有新增RNG、timer写入、Detach或存档字段。既有嵌套后果现在在原生规定的毁坏/幸存者之后发生，不声称保持旧错误事件顺序。
- 新actual aymod_hospital_disconnected_cargo_waits_for_destruction_effects在正式毁坏声音回调观察剩余病人仍活、HP>0、limbo且Cargo保留；毁坏返回后真实UnInit/帧尾退场，第二次正式存读没有幽灵成员。最终夹具在旧生产代码明确失败“Cargo was killed before ... DestructionEffects”，修复后通过；没有用绕过有效性校验或手工写Cargo/死亡状态来消除失败。
- 最终AY医院2（原项涵盖GAROBO/NARADR Off/On正常治疗）、生化炉1、驻军6通过；NP/stock存储4/3、DestructionEffects各20+ignored实际零售1、crew各33、客户端灯光7+移除灯1通过。最初把源码文件名误当模块路径的Death/Crew命令执行0项，已用真实模块路径重跑上述实际项，0项不计通过。crew两个ignored运输/长场景与灯光诊断地图GPU未另跑，无GUI/固定构建/MOD/原始存档修改。
- 一次只读critic无可操作发现，独立核清native前后缀和原attacker/C4AppliedBy区别。新例无攻击者，未独立证明该窗口的真实敌方击杀计分；供应断联本身不是整条用户操作验收。本次实际工程师触发已由上节补充，完整恢复治疗/联盟/多AI仍开放。

### V05不同帧双弹、第二射手死亡与击杀时活名单（2026-10-28）

- 既有aymod_garrison_missile_in_flight只增加目标类型参数，旧四场景仍用APOC；新两例用实际E2预置1HP/Sleep，使两次独立经验增长可观察，不改MOD成本/ROF/弹速/武器或重置等级。真实CAHSE01内两GGI先后普通Attack，两次fire_event在不同帧，第一枚仍活时生成第二枚，保存前同时持有两个不同原Projectile ID与不同实体目标，游标已自然回绕0。
- 名单不变例：完整保存/恢复两原弹后，建筑暂Sleep且全程无替代fire_event；两个不同命中帧均增加GGI[1] XP，GGI[0]/建筑/未驻军E1仍为0，两原弹最终退役。不是历史射手保存在子弹里：原生702FB5/702FBB/702FC0/702FCB/702FD1读取击杀时当前cursor-1并用当前count回绕。
- 第二例在第二枚刚发射时用实际AWPWH的正式damage receiver零普通伤害触发既有NP穿透前缀，确实杀死末位GGI[1]（第二原弹射手），不手写列表/游标/生命状态。普通帧完成退场后，正式存读保留两原弹、幸存GGI[0]与cursor0；两个独立原弹击杀都改付当前唯一GGI[0]。这验证穿透receiver到退休的完整边界，不冒称该短窗口内真实狙击手开火/步行准入。
- 调查中拒绝两个错误前提：普通伤害不能杀limbo驻军；裸UnInit在ScenarioInit之外不会触发Building44EAB8..44EB02名单移除，强行保存会留下不可解析引用，不能当作正常穿透删除来要求引擎改政策。本例改走实际穿透producer，其C2E751列表删除owner已存在。原高价目标的精英奖励封顶也不能按“第二次XP必须增加”误报；最终低价目标两次增长都实际通过。
- 最终真实AY garrison6项（含新增两项）、RecordKill NP奖励6项通过；读取/列表/奖励仍为既有owners，未增加RNG、timer/Detach调用或保存字段，无新玩法修改/critic，无GUI、固定构建/MOD/原始存档或发布变更。未扩称反序命中、任意非尾删除、完整自动续火、多House、全部子机家族或全V05完成。

### V07寄生源换主：第三House/宿主House、存读解缠及GPU（2026-10-28）

- 唯一visit_aymod_dolphin_rescue_after_load测试owner将原bool换成RescueTransfer枚举，原无换主/宿主换主两例保持，增加寄生源SQD换第三个人类House及换宿主House两种输入；不复制Return Dubai场景、寄生/换主/波绘制实现。新第三House仍有真实GAPOWR避免short-game败北冒充解缠；调用正式change_owner_with_rules receiver，不声称缠绕中的SQD可被任意心控武器正常选中。
- 实读实际Unit7463A0..7463F0→Foot4DBED0..4DBFC2→Techno7014A0..701893及Parasite629FD0→6297F0的Naval/Organic分支：已附着状态不因House变动逐帧重验外交。新增两种源换主后均普通推进45帧，源保持limbo/新owner，源victim与宿主eater双向一致、原缠绕动画仍活；没有因源转为同House自动释放或重建链接。
- 正式存读保留新源House及双向原链接；宿主House的DLPH经普通Attack真实Sonic开火解缠，要求fire_event与原Wave，三者均活体非limbo、宿主瘫痪解除、源victim清空。通过既有mission owner令三者暂Sleep后第二次存读保留源归属/原波，120帧无替代射击，原波自然退役、原缠绕动画回收、双方链接清空。此处Sleep隔离原波；本节下项继续消费同一结束场景验证正常恢复。
- 同一visitor现在返回结束场景与源/宿主/海豚ID供core继续验证，不复制救船setup；两种源换主均在原波退役后读取仍活的原瘫痪timer，实际SQD typed武器0 ROF的三倍与duration一致（62A7AE..62A7DD实读）。再正式保存/完整恢复保持原timer，不写时长/locomotor/任务；普通帧等待自然到期，逐帧源存活/非limbo/owner与已清双向链接不变。新主人普通Move使同一鱿鱼确实进入相邻自由同层水格，最后存读保留归属/目标格/无寄生关系。测试没有在活瘫痪期间发Move，不扩大为全部早发指令或路径阻塞/长期移动验收。
- 客户端同一既有无窗口GPU测试扩为四种Transfer；每种BGRA/RGBA都走生产emission/扫描/绘制与颜色/Z/A读回，保存前后同一原波像素一致、退役无输出，sim hash/全部RNG不被绘制改变。仅增加实际输入，不手写声波几何或修改绘制；不是gamemd全场像素golden/整个App/原始崩溃局回放。
- 最终retail gate下core实际海豚3项（新项内部两种源换主）及客户端GPU1项通过。一次客户端编译发现bool改enum后的两处诊断格式仍用Display，改为Debug后实际测试通过；不算玩法失败。无新增生产状态、RNG/timer/Detach或保存字段，无新玩法critic；未启动GUI、动MOD/原始存档/固定构建或发布。源换主后自然到期/普通移动及再存读已由上项补齐，最终core3项与GPU1项重跑通过；真实心控准入、自主多AI/长期抗压与原局崩溃仍开放，不关闭全V07。

### V07同一实际海豚声波的生产扫描与GPU输出（2026-10-28）

- 把原有Return Dubai/SQD自主缠船、普通DLPH Attack与原波保存/恢复夹具移入唯一battle_test_evidence owner。core原两例和客户端新aymod_dolphin_rescue_wave_reaches_gpu_across_save_and_retirement消费同一只读visitor，不复制地图设置、寄生状态或攻击实现；原通用spawn/cell/frame辅助一起迁移，Infantry walkability既有hook及其mask仅扩展到test-support编译，语义未变。
- 新客户端从实际sim的build_weapon_wave_draws取得原波，无手写端点/多边形；每次真实live回调扫描全部polygon，采样经实际Air Display身份、lowering、SurfaceLine批次和draw_native_object_pass到GPU。平面使用显式纹理背景/Z全开/A127以检验声波本身，不声称整个地图背景绘制。
- 普通/宿主换主两例、BGRA/RGBA两格式均有139个变化像素；shot与loaded完整颜色缓冲逐字节一致，retired变化为0。每次颜色读回同时检查Z及已绑定A缓冲不变，emission/raster/GPU前后sim hash与全部RNG一致。
- 这两例实际救船在Foot ReceiveDamage4D734F..4D7371清海豚目标，Wave762AF0因目标丢失使短波下一次更新结束。保留真实行为并断言目标已清，不写假目标/timer延长波。shot/loaded是同一相位86的保存边界，不冒称长距离完整扩展/收缩或多个不同相位；已有独立原生组件覆盖仍保留。
- 最终新GPU1（内部两个真实场景/两格式）、迁移后core海豚2、core辐射3、原生Wave扫描/颜色/遍历4通过。test-support编译初期暴露既有hook依赖仅cfg(test)，已把原方法/辅助/mask一起扩展，未以另一套walkability实现绕开。没有新玩法状态、RNG调用、timer/Detach或保存字段，无GUI/固定构建/MOD/原始存档改动。
- 残余：未恢复不可用的玩家原局，未验整个App场景/插值/雾门/跨物体遮挡；该139像素只是实际输出与保存一致性，不是gamemd全场像素对照。寄生源换主、多AI和长期场衰减仍开放。

### V07实际海豚救驱逐舰、宿主换主与原声波存读（2026-10-28）

- 两项actual aymod_dolphin_rescues_共用真实Return Dubai/Battle、1v1困难加载及AI SQD自主SquidGrab前置。先正式保存/恢复缠绕，再生成同House DLPH，经普通Attack实射Sonic救船；要求真实fire_event、海豚owner的type0 WaveClass、驱逐舰/鱿鱼/海豚均存活且非limbo、宿主瘫痪消除和双向寄生指针清除，不以直接伤害或鱿鱼死亡代替解缠。
- 第二例在缠绕时经共享change_owner_with_rules换宿主给第三个human House，实际国家注册表构造而非stock索引；正式存读后新owner、双向链接和原缠绕动画保持，再由新owner海豚解救。原生静读FootChangeOwner4DBED0..4DBFC2确认到Techno7014A0的委托，不修改/复制换主owner；此组合不等于实际心控武器准入或自主多AI。
- 两例均在首发后通过已有mission owner将三者暂Sleep以防替代火力，正式保存/恢复原Wave身份，继续120帧无替代fire_events：原波仍有生产sonic_draw_geometry，随后自然退役，旧缠绕动画回收且无残留链接。没有改ROF/伤害/MOD、写timer/链接或绕过保存身份门禁；无新增RNG/Detach/保存字段。
- 换主首跑的新House只有船，触发正常short-game败北清理（Organic海豚/鱿鱼消失、船health1），不是Sonic修复退化；夹具补真实GAPOWR基地后两例通过，未关闭败北规则。最终实际core两项通过；渲染原生表/扫描行/颜色/遍历四项、既有Wave0/3实际GPU读回一项（Z/A不变）、高度差声波防越界一项也通过。
- 本例Wave到实际GPU的缺口已由上节接续补齐；仍未恢复玩家原局/无GUI并排，寄生源换主/多AI/长期抗压仍开放。仅新增回归，无新生产玩法修复，不报告“原闪退全面排除”。

### F02建筑换主后的近邻传感器与过渡存读（2026-10-28）

- 扩展既有实际AY移动拾箱/建筑与落地飞机存读回归：真实DOG放在GAPOWR基础矩形一格外，同House连续30帧不显形；经共享change_owner_with_rules生产receiver换给另一人类House，同一个DOG变为foreign，触发Building4578C0退隐，显形中正式保存/恢复后到stage0，并保持30帧不能重新隐形。
- 原属方给DOG普通Move，离开接触范围后建筑自然重新隐形；再次正式存读仍保留新owner、crate能力与完全隐形状态。没有直接写cloak阶段/传感器计数、复制换主逻辑、改规则或原始存档；此组合不声称已验证工程师点击/步行捕获。
- 初次使用电脑接收者时建筑正常走Building4492D7出售并消失；这是已有原生规则，不是cloak丢失。改为通过实际国家注册表构造新human House隔离本链，未关闭电脑出售或硬编码stock俄罗斯索引。损坏/联盟重叠、完整多AI、启用自然粒子及GUI仍开放；无新RNG调用、timer/Detach写入或保存字段。
- 最终带零售INI强制门禁的core实际aymod_cloak_crate_两项通过（移动单位原回归、建筑/落地飞机扩展回归）；本增量仅测试与台账，未更改生产玩法或固定构建。

### F02建筑隐形箱实际无窗口GPU输出（2026-10-28）

- 新aymod_cloak_crate_building_visibility_reaches_actual_gpu_output正式加载AYMOD/XMP03T4，普通MTNK移动拾箱，GAPOWR完成隐形；不写cloak阶段或伪造SpriteInstance。生产本体与附属槽emission、lowering、BuildingBase/Display绘制后分别读回，避免一组遮住另一组缺失。
- 两种sRGB目标格式均通过：未隐形control本体4811/槽2438像素变化；隐形owner为4292/2438，unsensed enemy为0/0；真实出生DOG覆盖中心后detected enemy恢复4292/2438。检查的是相对固定背景的有无输出，不是gamemd像素golden；每次绘制前后sim hash与全部RNG不变。
- 设备使用同一BUILDNGZ.SHA加载/解码/上传owner，新增无surface入口仅委托Device/Queue；真实模板与BuildingBase阶段是测试必要输入，未关闭ZShape或修改引擎绕过。箱子摆放test-support入口仅委托既有place_crate owner。
- 新实际GPU1、原retail_building_slot_matches_original_temporal_translucency的原生色/Z/A比较1、retail_cleg_attack_freezes_and_releases_power_plant_slot生命周期1、ZShape解码1均通过。无GUI/游戏启动、原始存档/MOD或固定构建改动；最终战场遮挡/原生cloak像素与完整生命周期仍待验。

### F02自然粒子残余的身份与当前适用性（2026-10-28）

- 原生+30C是NaturalParticleSystem（Type+764，713B9A），相对位置是NaturalParticleLocation（Type+768，713BF1..713C34）；原有字段reader已存在，不是火花/受损烟雾。构造时+30C为空，Birth440E99要求类型非空，完全隐形454E7F销毁，退隐454F3D先要求非空位置才构造62DC50。坐标空值依据43B0F0初始化指令与Type_ctor零写入确认，未读BSS零填充作证。
- 新actual aymod_building_natural_particle_slot_is_dormant_in_battle_scenarios遍历普通Battle/XMP03T4及Return Dubai正式加载后的全部Building：NaturalParticleSystem=None且NaturalParticleLocation=0，两条创建入口均不工作。因此原生自然粒子缺口在这两个fresh配置没有实际使用者，不再以未核身份阻挡此范围箱子链；没有“实现了自然粒子系统”的结论。
- NAWEAP的NaturalSmokeLocation是另一个物理键名，不把它自动映射到这里。原生名称字节、constructor和reader已核；未据字节检索扩大为所有其他读者/模块的全局不可达证明。
- 实际适用性1、空引用reader1、实际隐形箱2通过；只测试/注释/台账，无新游戏状态、RNG、timer或Detach。未来模式/地图或旧外来存档带非空系统/位置时仍需实现+30C完整生命周期；当前完整GPU/真人视觉验收仍开放。

### F02建筑箱子独立隐形过渡修复，粒子/画面仍开放（2026-10-28）

- 实际普通MTNK拾Cloak箱，GAPOWR与落地ORCA在正式半径内获实例能力；健康建筑原先卡state1/depth51，正确红回归保留了两类均需完成隐形与unsensed视觉5的要求，没有按一次错误的“建筑原生可见例外”判断关项。建筑首次选址过严的问题仅修测试坐标，不改半径/MOD。
- 实读TechnoAI6FA946虚调用Building454DB0，而非6FB740。私有Building+6ED阶段加入既有CloakRuntime，454E11..454E85每次AI推进到15/state2，454EB2..454F35退到0/state0；CanAuto457770/Should4578C0及其前置base/foundation/Sensors门一起接入。BuildingVisual4544A0的非零阶段走原owner，stage0仍委托703860。
- 附属Anim+178写入由building_art共用：454E55/454EFE过渡刷新、4519B9新槽出生前刷新旧21槽；AnimClass保留写入接口。Native本地hidden=16不进入确定性状态，现有parent-slot DrawDecision在presentation隐藏；typed舞台值15的序列化/hash与不同viewer一致性回归通过。
- 只读critic确认两项：本地screen查询污染Anim hash、建筑查询anchor而非中心。已消除本地写入并用两个规范化load控制比较hash；原screen owner复用sensor_visible_to_player，presentation使用共享GetCoords，新增中心/anchor不同覆盖回归。未做第二轮critic，不把原有意见说成无发现。
- 最终实际箱子2、NP/stock producer各17/runtime各5/crates各84、建筑ART29、客户端DrawState7/helpers7/动画透明4通过。新回归两类的能力/自然隐形/两次正式存读/视觉查询均过；未改MOD、原始存档或固定构建，无GUI/GPU输出认证。
- 新状态改变保存布局；无新增直接RNG或Detach API，建筑不再错误走普通红血随机隐形概率/Techno完成Detach，沿原StartCloaking/Uncloaking/sound owner。原生+30C粒子创建/销毁尚无Rust状态/producer，与+308/+310不同；上节已确认当前两Battle全Building配置门禁关闭，其他启用配置的FX/对象ID/抽样影响仍待核，不宣称通用原生完整。CloakGenerator既有场owner及损坏/换主/多House/最终像素边界保留。

### F09实际武器分配闭包：两个Battle配置无消费者（2026-10-28）

- 两套正式加载：普通Battle/XMP03T4和Return Dubai。typed Warhead族中DoesntAffectEnemies只有TankSnapshot；遍历正式RuleSet的完整已分配WeaponType族，没有武器引用任一带此标志弹头。TankMakeupKit与CRMakeupKit两者weapon(name)均为None，不是仅当前单位没装备而仍保留可索引武器。
- RuleSet/native_processing现有分配owner从所有到达reader的对象普通/精英槽、DeathWeapon/OccupyWeapon、超武WeaponType、General DeathWeapon、Projectile Airburst/ShrapnelWeapon等建族，早层孤立分配亦保留，Trigger Action42消费同一索引族。新回归消费此生产结果，不另写配置图解析器或复制闭包实现。
- 这是“当前两配置无已分配武器消费者”的边界，不是全MOD所有模式/地图或所有直接Warhead路径的全局不可达证明。已有reader、FireError/ReceiveDamage支持仍保留，不改MGTK/FV武器去人为触发，不解释原局弱攻势。
- 实际闭包1项、NP精确键/层次/保存/hash1项、FireSubject1项、真实攻击者House伤害门1项、逐层武器读1项通过。测试/文档-only，无RNG/计时/Detach/保存字段变化；未来其他配置添加引用会由此测试或对应场景暴露，需要重新验收。

### F08普通基洛夫完整投弹、在途存读与延迟引爆（2026-10-28）

- 新实际测试使用正式AY ZEP和8格外CAMISC02，经普通Attack飞近、确认高飞、真实fire_events释放配置炸弹。不手工设置飞行高度、不调用直接receiver、不改目标血量或武器数值。放弹帧仍为初始目标HP，保证有真实在途窗口。
- 正式保存/完整资源恢复下落中的原炸弹；下一帧普通Stop阻止新放弹，后续观察同时断言无替代fire_events，因此不能以另一发命中掩盖在途弹丢失。原弹真实伤害触发IsFlatExplosion/CausesDelayKill别名的既有PostMortem，目标alive/health1及引信挂起，C4AppliedBy仍为None。
- 第二次在引信阶段正式存读，health1/引信保持；普通建筑AI到期销毁。覆盖Techno701E92..701F67的延迟恢复与既有C4 owner，未新增绘制或第二套爆炸逻辑。
- 新完整飞行1、flat_explosion3、postmortem4通过。仅测试/文档，无新直接RNG/计时/Detach/保存字段，游戏自然落弹和延迟引信沿既有owner运行。未验证精英KTSTLEXP链、长连投/多目标或GUI/GPU；本例不是原版飞行/爆炸像素对照。

### V04探测器卸载立即恢复：共用Unlimbo缺口修复（2026-10-28）

- 实际TANY自然隐形、电脑GAPILL及FV/LCRF场景，正常Enter真实载入，不手工写Cargo；运输中完整资源存读，普通Unload出车。修复前计数0/sensor_deposit=None，FV存活/在地图/已脱离carrier，旧位置与新位置均在真实SensorsSight内，排除场景误差。
- FootClass::Unlimbo4D72EA..4D7318在邻居/威胁系数尾后运行SensorsSight add；FootLimbo4DB376登车时已有配对remove。Rust原先仅在出生路径末尾add，漏掉普通重新Unlimbo。共享try_reveal_entity_with_context现调用既有add_unit_sensor_after_unlimbo，移除world_spawn的5个外部add，保持一个owner。地图出生/动态出生/部署等原调用均走共享Reveal；部署为建筑的旧Foot add本来是category过滤的no-op。
- 新实际回归断言登船1→0、运输存读0、卸载立即1、出车存读1，期间电脑GreatestThreat随己方sensor准入/拒绝完全隐形TANY。紧凑单元例保护首次出生不重复、AlreadyRevealed/RejectedEarly不改计数、成功重新Reveal加一次。没有靠走到下一格的PerCell补偿来通过卸载断言。
- 实际运输1、实际控制死亡1、NP/stock传感器各7、卸载35、构造36、重打包失败1通过；一次只读critic无可操作发现。没有新字段、直接随机调用、timer写入或Detach；已有sensor callbacks移到正确Unlimbo阶段，其他Foot重新Reveal亦共享修复。固定玩家构建未包含此修复。
- 保留建筑原生非对称remove半径、联盟/多House复杂重叠、载具跨水运输全战术、其他伪装族/空艇/GUI等边界；不宣称每个间接Reveal调用都已验收，也不为旧存档中已丢的footprint另加追溯修复策略。

### V04 IFV探测器实际控制、归还、死亡及存读（2026-10-28）

- 采用实际AY TANY自然隐形后生成FV/YURI；不手写sensor计数、不raw change_owner、不手写控制链接。普通YURI Attack实际建立CaptureManager控制；Foot4DBEFB/4DBF81沿既有owner移除旧House计数、在新House重加。正式存读验证控制者存活时双方计数保持。
- 从生产致死receiver杀控制者，经Techno死亡FreeAllCaptures归还FV；校验Owner/控制指针/实际deposit owner与双方计数。YURI自身也有sensor，按它真实circle是否覆盖目标格扣除其贡献，不能把控制者死亡的减计数误认为FV已撤销。保留同格其他来源（TANY自身）的贡献。
- 从生产receiver毁FV，旧House最后覆盖1→0；普通帧尾退休后正式存读，无实体或旧sensor复活。每个关键阶段查询电脑GAPILL的共享GreatestThreat：覆盖有/无控制其是否准入完全隐形TANY，炮塔保持Sleep以免射击干扰，不把此例当实射或完整AI交战验收。
- 新复合1项、原移动DOG/实射/存读1项、sensor_lifecycle6项、capture_manager29项通过，无生产修改、RNG/计时/Detach或存档变化。下一步是实际登离运输与载员存读；建筑原生非对称范围、联盟/多House重叠、换主同时移动及其他伪装族仍开放。

### F07潜艇感知/存读与实际海军例外（2026-10-28）

- 复用新aymod_tanya_water_pair，实际Return Dubai连续清空水道，保证从未探测距离游近的合法水路。TANY/SUB使用正式规则，SUB在Sleep先验下正常隐形，不手动设置cloak/sensor；TANY的自身Sensors真实发布随Move变动。
- 检查未探测时FireSubject拒绝攻击（排除射程，只测其他实际门），正式存读/完整资源恢复后仍无sensor且返回Cant。游近进入探测范围、第二次存读保留sensor；普通Attack真正发射Sapper并伤害SUB。并非验证玩家能在GUI点到未探测潜艇；Sleep仅隔离SUB反击。
- 实读GetFireError6FC24D..6FC29D的未感知目标拒绝是Cant6（6FCD29），不是Cloaked9。NavalTargeting4分支6F38FC..6F3915保留Hover/Organic返回0，但实际AY TANY主武器HollowPoint2对LCRF heavy、DLPH light的typed interaction均为零；SelectWeapon6F3716..6F3751先行选择副武器。新增实际例同时断言海军leaf返回0、完整选择返回1、真实Sapper命中，避免按键名/单leaf误改规则。
- 实际Tanya5项、选武器3项、GetFireError3项通过。新增两项测试，不新增玩法状态/随机抽样/计时/Detach或保存布局；未改MOD。原HYD普通/精英与其他TANY行为继续通过，无需玩法critic。
- 保留精英潜艇完整链、探测离开后再潜与重获、换主/运输、多AI长局、GUI等边界；本例只证明玩家普通Attack在真实感知门后的实际效果。不是AntiNaval/AntiUnderwater新增键实现，它们原先的残留分类不变。

### F06精英ORCA远距移动目标追击与热存读（2026-10-28）

- 新实际AY测试从正式Battle资源创建ORCA与远距ZEP；精英等级/初始高空为明示先验，正常Move/Attack命令产生双方运动，不修改MOD数值或模拟追踪路径。生产typed武器确认精英槽不同于普通槽且弹道AA=true，起始距离超过精英射程4格以上。
- 双方均水平移动且尚未第一次开火时正式save_validated/load_validated及完整资源恢复，目标指针保留；恢复后真实fire_events必须使用配置精英武器并指向ZEP，随后真实命中掉血。命中时ZEP仍高飞、Move未结束、位置相比存读边界继续变化，ORCA亦已实际位移。
- 复读414D36/C27C60..C27C78：AACombat只令IsHighFlying清目标判断返回false，继续414D4D隐形准入；不新增第二套目标追踪/绕过AA。原普通对空及对地实际场景仍通过。
- 最终实际ORCA2项、AACombat2项、Aircraft attack_mission7项通过。本条仅测试/文档，无生产修改，无新增RNG、计时、Detach或存档字段。首跑缓存保留器Unsupported PE debug closure安全停止、删除0字节，最终测试无此阻塞；未手动清理固定构建。
- 尚未覆盖：电脑自主战术选择、GUI、往返机场重装/长时间多目标战斗、玩家晋升过程（本例给定精英先验）。不把单次追击存读通过扩张为全部飞机功能完成。

### V05实际生化炉Cargo毁坏与NoSurvivor（2026-10-28）

- 通过实际AY Battle/XMP03T4生产规则all_objects检查全部Building类型，Passengers>0仅YAPOWR/YAPOWRAI且均为吸收建筑；这只是显式容量库存。旧结论“非吸收Cargo残余无此闭包使用者”已被上节Hospital/Passengers=0的实际懒分配链否定，不再用容量字段排除运行时Cargo。
- 新实际复合测试中，YAPOWR的INIT从正常Enter进入Cargo，保存/完整资源恢复后给予普通致死伤害，SpawnSurvivors441F1B/442D90沿吸收分支释放活体，原House/无运输关系在二次存读后保持。YAPOWRAI用相同实际装载/存读链，在致死receiver的IgnoreDefenses=true传递NoSurvivor，INIT被UnInit、Cargo清空，帧尾退休后再次正式存读无幽灵成员。
- 两个实例都不是garrison vector，未调用手工add或改MOD规则。YAPOWRAI只代表真实类型配置，不声称普通玩家House测试覆盖电脑自主装载/战术。C4AppliedBy源的特殊击杀归属、换主、满炉多成员与完整AI长局仍待验。
- 复合测试1项、absorbed相关4项、存储/死亡边界4项通过。无生产实现/保存布局变化；沿已有RNG/计时/Detach owners，无需玩法critic。后续优先回到实际使用的功能项，不围绕不存在的自定义类型扩张范围。

### V05建筑死亡武器与驻军弹出顺序修复（2026-10-28）

- 原生产hook在Techno死亡臂前运行，提前揭露驻军，使本楼死亡武器可以伤害本应仍在limbo的成员。新destroyed_garrison_stays_hidden_until_its_death_weapon_returns经正常E2步行入驻和生产致死receiver复现；楼外同House E2死亡证明blast真实执行。修复前楼内E2也死亡，修复后活着、满血弹出且运输关系清除。
- 实读Building442425→Techno701900；Techno70266D FireDeathWeapon、70267F BombDetonate后返回。Building Alive门44242C、saved-contact链之后，442625..44263B才SellBuilding(0,0)，44264C关灯，442665 DestructionEffects。复用现有apply_fatal_lifecycle_stage，命名BeforeBuildingDestruction并移入finish_concrete_death的Building分支；完整迁移旧枚举消费者，移除过期提前释放注释，无第二份死亡/弹出owner。
- 新回归为retail规则加内存诊断覆盖，不是未改规则AY实际自爆建筑报告；原实际AY四项驻军场景继续验证。NP/stock存储4/3、弹出14、死亡效果20+ignored1、crew33、ROF2、客户端灯光7、stock原生致死2通过。只读critic无新增可操作发现；GUI/GPU及复杂连锁炸弹未另验。
- 无新增随机调用、timer写入、Detach或存档字段；既有Scatter抽样现晚于死亡碎片/武器、早于DestructionEffects，死亡相关随机流分配按原生纠正，不声称复现旧错误分配。灯光关闭也移到原生同一边界。
- 当时保留的非吸收Cargo提前KillPassengers残余已由上节441F27顺序修复接续处理；医院懒分配Cargo不能仅以Passengers=0排除。普通Unit链未改，仍不把garrison与Cargo混淆，也不因此关闭整个V05。

### V05建筑毁坏的活体弹出与存读后重新入驻（2026-10-28）

- 新aymod_garrison_destruction_releases_survivors_for_saved_reentry复用实际GGI导弹在途场景，保留正式MOD配置；从生产direct_receiver给CAHSE01致死伤害，不直接调用弹出helper或手工清Cargo/occupants。即时校验驻军楼名单已清、两名GGI活体离开foundation、原House不变、inside/open关系已清。
- 正常tick处理延迟生命周期后，正式save_validated/load_validated及完整地图/资源恢复；存活者经普通Capture(8)命令实际步行入另一栋CAHSE01，不从入驻leaf直接添加。再次正式存读保持新楼名单、limbo与inside指针。原在途导弹允许沿其owner继续，不据此宣称原射手死亡后的XP政策已验。
- 实读Building442625..44263B调用SellBuilding(0,0)，InfantryPerCell519675..51972D检查Capture(8)并调用522910。首轮误用Enter，改正输入后通过；不为错误测试修改游戏任务语义。
- 实际驻军4项、弹出owner14项、建筑入口43项通过（5个ignored未运行）。无生产逻辑修改，无新增RNG、timer写入、Detach或保存状态；既有弹出Scatter/目标清除/Unlimbo/驻军重入的正常抽样与计时保持。
- 本例原先保留的驻军提前弹出残余已由上节生产修复处理；本例本身仍不认证无出口死亡、多House、复杂连锁死亡武器及GUI，也不替代真实Cargo顺序复核。

### V08实际电脑BFRT装载、六人开火及解散后卸载（2026-10-28）

- 采用实际AY资源、困难美国电脑自然展开MCV后，提供原TaskForce并调用正式create_team；不直接写Cargo、乘员关系或脚本游标。一个对照保留真实队伍竞争，另一个仅在测试场景关闭新AI触发器以隔离装载组件；不改MOD配置/权重/兵种数量。
- Team Load6ED200串行Enter，WaitLoaded6E953E按实际人数通过；正式资源恢复后校验Cargo顺序、inside/open关系及团队进度，5GGI对合法装甲目标、GHOST对合法步兵目标实际发射。
- 持续AreaGuard覆盖外部卸载符合DoMission6EDA25..6EDA71，不改脚本去掩盖它。经过既有TAction5 DestroyAll6F0A70/Team析构6E8DE0解散后再发普通卸载，6名活体离车，第二次正式存读后运输关系仍清除。并非验证带队卸载或自主战术卸载。
- 续验重新装载：同一台BFRT和六名活体下车并正式存读后，沿Create_Team6F09C0重建同一07C32EFC-G原配置编队；不新增/替换兵，不写Cargo/脚本游标/登车计时，也不移动单位去凑路径。新team ID与旧队分离，旧team_to_rejoin引用已清；脚本14/43正常重新招募并步行装载。部分装载时正式存读，校验Cargo顺序、脚本cursor、已登车limbo/open关系及真实rejoin值保留，之后六人装满并再存读，团队成员只余载具，六人又分别实际对合法装甲/步兵目标发射。
- 首跑错误预期登车就写team_to_rejoin，被None反例推翻；复读实际AY Techno::Fire6FF7A3..6FF7E9，+434写入受type+ D3D、Foot、现团队及目标类15门禁限制，不属于Enter固有副作用。纠正为保存前后真实值一致且不指回已销毁旧队，未为错误测试增加生产状态写入。最终扩展组件1项、保留真实高优先级竞争对照1项、actions::transport_tests9项通过；初次过滤器漏actions/模块名得到0项，不计通过，已纠正并实际运行9项。
- 无限制组件对照出现两名GGI被较高Priority基地防御队招走，符合CanAdd6EA7C7..6EA7E5；未验证后续生产补齐，不标永久卡死。组件测试不是Return Dubai经济/攻势诊断，也不是多AI长局验收。
- 本条无生产逻辑修改，无新增RNG、timer写入、Detach或持久化状态；沿现有装载/卸载owners执行其既有抽样、计时、team离入和radio清理。本轮仅测试/台账、无新critic；未启动GUI/改MOD或原始存档/替换固定玩家版/发布。带队卸载、换主、多AI竞争补员与长期对战仍开放；不把明确提供新建队意图的重装载冒称AI会自主决定重建该队。

### V05部分驻军死亡、游标存读与补员恢复（2026-10-28）

- 续验单剩GGI：在既有双弹/第二射手被AWPWH穿透杀死场景中，两枚原导弹落地后，不补员、不改游标/武器/冷却；仅给另一台原血量APOC一次普通Attack。幸存者真实发射三次OCMissileLauncher，第一发在途时正式存读，校验原弹和原rearm timer保留，再自然发出后两发并造成实伤；名单始终count1/cursor0。这验证该合法游标起点的自动续火，不声称所有单人起点都可用。
- 续验非尾移除：生产retail reader的E1/E2/E1混合驻军正常入驻、开火得到cursor1；明确提供ScenarioInit作用域，通过既有UnInit/PointerExpired移除首名。实读实际AY BuildingPointerExpired44EAB8..44EAC0门禁及44EAE8..44EB00移位，列表成为E2/E1而游标仍1。普通帧物理退场、正式存读后，无第二条Attack，接下来的真实武器依次为剩余E1、E2，游标自然1→0→1。未直接写名单/游标；这不是普通战斗杀死非尾成员的producer证明。
- 本轮最终实际AY驻军6项、NP/stock building_occupant_storage各5/4项通过；无生产逻辑修改/新critic，无新增RNG、timer写入、Detach实现或保存字段。两条场景只沿既有owner及其原有副作用；固定构建、MOD配置、原始存档均未动。以下保留较早cursor1/count1边界的证据与补员路径。
- 提取既有GGI在途场景为aymod_garrison_missile_in_flight，两个已验全员出驻/替换例继续使用同一生产规则/生成/开火路径。新aymod_garrison_tail_death_preserves_the_pending_shot_and_cursor_after_load先确认count2/cursor1，未通过测试setter伪造游标。
- 实际AWPWH的穿透等级1大于CAHSE01保护等级；直接从正式伤害receiver边界给零普通伤害，触发真实NP穿透前缀，删除末位GGI但不伤楼、不清cursor。此例隔离弹道窗口中的列表变更，不冒称完整狙击手发射；狙击手实际武器路径已有单独场景。最初错误要求UnInit后立刻实体不存在，已按既有生命周期改为先断言dead、正常advance一帧后确认物理删除，未修改生产队列或UnInit语义。
- 实读C2E751..C2E7B3及BuildingPointerExpired44EAC0..44EB02：名单缩短而不复位+69C。保存/完整资源恢复后count1/cursor1仍保持，原导弹真正杀死原目标，NPAE702FB5..702FD1仍从当前cursor-1=0支付幸存GGI，楼/未入驻E1无XP。
- 实读GetWeapon452742..452750有signed cursor>=count的回退，4527B4进入本楼基础武器槽，不是绕回第一名。新回归通过既有firing_occupant_id确认此时无可用驻军索引；不擅自clamp或reset来替代原补丁。字面字段扫描仅作为写入线索，未据它宣称已穷尽派生指针写/整个原生AI循环，因此不扩大成单人永久停火结论。
- 随后从既有入驻leaf加入新E1，保持cursor1、当前合法成员改为E1；普通Attack产生真实UCPara事件并击杀另一个预置1HP的E2，新E1获XP、旧GGI经验不变，cursor自然回绕0。没有直接设rank/XP/索引、清角色列表或修改MOD武器/投射物；没有伪造完整步行补员时间。
- 最终实际garrison3项、building_occupant_storage3、RecordKill6通过，前轮出驻/替换存读一并防回归。仅测试/文档，无玩法critic，无新RNG/timer/Detach实现或保存字段；所有伤害、穿透、记录、退场、补员、开火均调用既有owners。未启动GUI、未改MOD/原存档/固定构建、未发布。
- 边界：多弹交错、楼毁活体弹出、第二射手死亡已有各自独立覆盖；本轮补充的非尾移除仅限ScenarioInit作用域，单剩续火仅限自然得到cursor0的起点。cursor>=count的整个原生自动循环、普通战斗非尾移除producer、完整步行补员与GPU/GUI仍不在结论内，不关闭全V05。

### V05在途导弹、全员出驻/替换与击杀归属（2026-10-28）

- 新aymod_garrison_missile_uses_the_live_occupants_after_unload_and_save正式加载AY Battle。CAHSE01内两名GGI使用实际OccupyWeapon=OCMissileLauncher，typed AAHeatSeeker2.inviso=false；只预置睡眠APOC为1HP，真实开火事件后先断言它仍存活，不用瞬时枪冒充在途武器，也不改投射物Speed/武器Range。
- 正常队列发MegaMission Unload、target=building自身，匹配BuildingClass::ClickedAction_Object443545..443562（GetOccupantCount后，push0/0/self/16），而不是借用target-less UnloadPassengers快捷载体。真实Mission_Unload完成全员出驻，SellBuilding457DEB的原索引重置生效。
- 分别保持空楼，或通过现有AddGarrisonOccupant准入leaf加一个新E1。完整步行进楼另属既有准入场景，不把此leaf边界称为玩家在该短窗口内走入；楼随后经任务owner置Sleep，只排除第二次开火。正式GameSnapshot保存/完整资源恢复后，已发射导弹仍可杀死目标，全程没有后续建筑fire_event。
- 两个旧GGI都仍活、已退出limbo且XP为0；空楼不付任何旧驻军，换人例仅当前新E1得XP，建筑自身仍无XP。实读实际NPAE702FB5读当前+69C，702FBB减一，负数加当前+694，702FCB/FD1从当前+688列表取人；因此该结果是击杀时当前名单语义，不是保留真正历史射手。没有新增指针/来源缓存来“纠正”原补丁的语义。
- 最终实际aymod_garrison_2项（新项含两个名单边界）、building_occupant_storage3、RecordKill6通过。仅测试/文档，无玩法critic，无新状态/RNG/timer/Detach逻辑；正常发射、出驻、所有权重整、入驻、死亡仍走原owners的既有计时、抽样与通知。无GUI、MOD/原存档/固定构建写入，无发布。
- 未扩称通过：仅移除部分驻军但保留非零索引、穿透/建筑死亡弹出、原射手死亡、多个不同帧发射弹的交叉命中、满血目标精确伤害/全部导弹弹道数值、完整步行重新入驻和玩家GUI。上述仍属于V05开放边界。

### V05飞机子机自身奖励、母体同步与再生（2026-10-28）

- 实读实际NPAE RecordKill702FF0→C2C208：无+2C0控制者才落C2C49F..C2C50F，检查Aircraft RTTI2、SpawnOwner与母体Trainable；C2C4D9先给原接收者Add，C2C50A另给母体Add。不是把可训练飞机的奖励转走，也不要求类型Spawned字节作为额外门。旧GI步兵伪子机测试不能证明飞机分支，已换真实Aircraft类别回归；新测试修复前子机XP为0而失败。
- 删除RecordKill末尾竞争性的全子机覆盖；唯一同步在Techno AI现有等级采样处。原6FA054..6FA078比较rank cache，初次采样也走6FA13E→C2D825；非Infantry且有manager才调用C2D851。C2D85F..865按count-1及JLE保留零/单槽跳过，随后倒序检查可训练子机，parent raw严格大于child raw才复制并置child cache=-1。不降级经验更高的子机、不每击杀/每帧强制复制。
- raw和rank cache仍由Veterancy持有，SpawnManager仅沿正式slot访问子机，父对象沿现有Techno AI阶段触发。stored f32的纯比较跨平台精确，无新算术内核。再生C2D7F3只在新子机Trainable时复制parent+150，不复制cache；补齐原adopt缺少的门，复用现有inherit。
- 计价按实际分支而非统一猜测：原transport/self/missile臂702ED1/702F13/702F77读EDI击杀House；驻军702FD4独读ESI受害House；NP parent/controller额外奖励用各自House。受害者cost仍在Tag前702D61获取，用于points；不是平衡调整。一轮critic发现本次错误推广到驻军的回归，已恢复该例外并用Americans .25步兵倍率、Russians受害方的等级边界回归保护；修正后NP/stock都通过。
- 新真实aymod_carrier_child_keeps_experience_and_regenerates_after_real_load：正式Return Dubai/Battle、玩家CARRIER及自带四架HORNET，普通Attack与真实子机fire_event击杀仅预置1HP的睡眠APOC，母体有XP、至少射手保留更高自身XP。随后在正式直接致死receiver边界制造一架子机损失（不是AA武器实战），观察slot释放；正常推进后在再生等待中正式GameSnapshot保存/恢复，manager/slot/timer完整保持，按原SpawnRegenRate生成不同ID替补，SpawnOwner/raw XP正确，再次普通Attack让替补实际开火。
- 初始夹具用健康子机UnInit模拟死亡，但PointerExpired6B7C60会保留活且未跟踪的可复用飞机；现改为真实致死receiver及实际死亡收尾，不为夹具改变该原生门。合成Trainable=no/yes用分层merge覆盖现有段，避免同文件重复段导致覆盖未生效；typed值另有断言。
- 最终critic后NP/stock各RecordKill6、Veterancy12、SpawnManager37通过；实际AY驻军首杀1和航母复合链1通过。独立critic一次，确认的一项计价回归已修；不重复critic，慢速全翼strafe未改飞行/路径/每轮弹量而保留忽略。无新保存字段/独立XP状态，无新增RNG抽样、timer/Detach逻辑；实际击杀/升级、生成/死亡仍正常执行原owners的既有通知和计时。
- 边界：实际母体首杀后仍为rookie、仅积累部分raw经验，不宣称真实连续晋升/精英全链；rank变化、强子机不降级和缓存失效另由受控生产tick回归覆盖。单槽同步与未训练现存子机暂无专项回归（分支已按指令保留），未把再生Trainable边界当作现存同步全覆盖。其他子机家族/换主/母体死亡/AA拦截、多AI与GUI仍开放；既有控制者指针选择等未在本条扩称修复。

### V04移动探测、隐形步兵与电脑选敌（2026-10-28）

- 新aymod_mobile_sensors_update_cloaked_infantry_ai_acquisition_after_load以正式AY Battle资源加载，TANY为真人House、DOG与GAPILL属同一电脑House。先证明未隐形目标可被扫描，再让TANY自然完成Cloakable隐形，探测外不被选中；DOG真实移动带入范围后，TANY仍保持全隐形而电脑查询可选中。
- DOG运动意图通过现有apply_command真实payload边界提供，不伪造会被忽略的AI玩家事件，不冒称团队/战略规划覆盖。覆盖内保存/恢复→撤离→不可新获取→返回→可新获取，随后电脑碉堡普通Guard实际向TANY开火，不直接AssignTarget或调用伤害。
- 实际中弹后的显形与慢速重新隐形按已有原生伤害/隐形owner发生。为隔离该边界，正常Stop清目标、mission authority暂置碉堡和DOG为Sleep，DOG停在探测范围外，防止它正常完成Move后继续追咬可见步兵。无探测时再次正式存读、自然重新隐形，普通Guard连续60帧不向她开火；DOG返回、普通Stop清理碉堡可能已合法获取的其他目标后，再由Guard选敌/开火。未强行清cloak、改HP或取消正常伤害显形。
- 最初的移动MTNK观察者在等待期间移至目标12格外，不能据其空扫描认定侦测坏了；使用固定防御建筑隔离这一变量。重发现也不意味着必须放弃已持有的其他目标，早期尾段实际持有起始MCV。以上均为夹具/错误期待修正，未据此更改AI政策、射程、兵种数值或弱攻势链。
- 实读NPAE Evaluate_Candidate6F7DA9..6F7DEB、GetFireError6FC24D..6FC29D的攻击者House传感器查询，以及ReceiveDamage70281D→703850的StartUncloaking(0)。本轮沿现有沉积/移除、保存重建、CloakRuntime、GreatestThreat/GetFireError和任务/命令owners，无新状态、RNG/timer/Detach实现；正常调用仍消费原owner应有的抽样/延迟和隐形Detach。
- 最终复合实际场景1项和techno_ai_cloak16项通过；测试-only，无玩法critic。单一探测器/固定电脑防御的受控链，不关闭目标自身移动、传感器死亡/换主/运输、多House重叠、潜艇/其他类型或GPU/GUI边界。

### V05 count1/cursor1的普通任务退回与补员恢复（2026-10-28）

- 实读实际AY BuildingGetWeapon452734..4527B7：occupied且signed cursor>=count时转TechnoGetWeapon70E140读取本楼基础槽，不绕回驻军0。Mission_Attack44AD07..44AD09已被NP改为直接进44AF86；活动目标从44AFF2选槽、44B00F问GetFireError。GetFireError6FC31C..6FC333读vt3F8的WeaponType为空时跳6FCD29，明确EAX=6(Cant)，不是Illegal5。Jump table44B728[6]的原始DWORD为44B0DE；该臂调用vt3C8清目标、清+664，随后回Guard/idle，不写+69C。把跳表当数据读取，不将其线性反汇编误作指令。
- 扩既有aymod_garrison_tail_death_preserves_the_pending_shot_and_cursor_after_load：保留真实GGI发射、AWPWH正式receiver穿透删末位、原导弹命中与幸存者经验，以及count1/cursor1存读起点；先不补员，新增真实活E2/1HP/Sleep并下普通Attack。180普通帧内额外第60帧完整存读，名单/游标保持，生产selected_weapon_fire_error始终Cant，无楼fire_event，活目标未受伤，最终旧Attack target清除。没有直接写名单/游标或从FireAt空槽fallback凑出一发。
- 随后才沿既有add_garrison_occupant入驻leaf加入E1，cursor仍1但现在有效；对同一个未伤活目标再下普通Attack，实际UCPara击杀且新E1经验增加。此补员不是完整步行准入验收。验证支持该具体分支的原生停火/补员恢复行为，不声称已穷尽BuildingAI/所有writer、所有游标值、普通战斗非尾删除或永久无法恢复；不为更好玩擅自加cursor clamp/reset。
- 最终retail gate下实际aymod_garrison7、building_occupant_storage5共12项通过；先单跑同一例1项不叠加。仅测试/台账，无生产行为、新状态/RNG/timer/Detach/保存布局或玩法critic，无GUI/MOD/原始存档/固定构建/发布变更。既有任务/扫描仍走原owner，正常帧自然抽样不靠重置随机流；余下正常战斗删除producer与其他循环分支继续保留。

### V05真实狙击两次尾删、经验与游标存读（2026-10-28）

- 原生C2E7C4..C2E820明确正等级通过signed比较后只调用一次C2E751，负等级NEG/比较后按原count重复调用；C2E751..C2E7B3每次读取count、DEC成尾索引，经+688数组取该成员，播首个死亡音、RecordKill/+F8 UnInit，清尾槽并将count写为尾索引，无+69C写入、随机抽样或“按当前射手游标选受害者”。因此不能把“普通狙击杀非末位”当成此NP补丁尚缺机制；其他删除来源仍另查。
- 新aymod_sniper_fire_removes_tail_occupants_in_order_across_saves复用正式GGI在途场景：楼/两GGI由既有入驻leaf提供，真实OCMissileLauncher已将cursor推进1且原弹仍在途。另建实际Russian human House、真实GAPOWR/SNIPE；楼用既有mission owner暂Sleep隔离第二发和反击，不写cursor/名单/伤害/武器。初跑fixture误用未在typed Countries分配的British，虽物理INI有同名section，生产country_index仍None；已改用实际已分配的Russians，不改MOD。
- SNIPE两次普通Attack必须各出现指向同一楼的真实fire_event，实际武器typed warhead穿透等级1。每次恰删当前尾GGI，count2→1→0、名单为原顺序前缀，cursor始终1，楼仍活；狙击手每次获得真实RecordKill经验。第一发后沿原mission authority暂Sleep狙击手，保存完整partial状态、rearm/经验，恢复后原GGI导弹仍实际击杀原APOC并把XP付给唯一幸存GGI；再正常Attack等待真实重装发第二枪，未重置rearm或伪造伤害。
- 两次尾删后各完整保存恢复，保留原名单、cursor1及狙击手rearm/经验。该例把此前直接AWPWH receiver边界推进到真实SNIPE发射/命中/退场，证明正常战斗确可产生count1/cursor1；不声称完整步行进驻、AI自主狙击或其他名单损伤来源均已验证，亦不擅自修原补丁未做的索引归零。
- 最终retail gate下aymod_sniper2（含原保护楼/MiG）、aymod_garrison7、RecordKill NP奖励6共15项通过，新单跑1不叠加。仅测试/台账，无新生产行为/状态/RNG/timer/Detach/保存布局或玩法critic，无GUI/MOD/原始存档/固定构建/发布变更。

### V07部署普通/精英与独立残留场救船（2026-10-28）

- 共用aymod_grappled_ship_at_shore，复用既有正式Return Dubai加载、岸边合法位置和AI SQD自主缠玩家DEST，不复制寄生或损伤实现。既有辐射束存读测试仍执行原链。
- 新aymod_radiation_deploy_releases_a_live_squid通过普通ToggleInfantryDeploy，让真实DESO部署，分别验证RadEruptionWeapon/RadEruptionWeaponE的真实Secondary事件、自身Cell目标、实际场创建且宿主格有场、活体解缠及动画回收。普通/精英测试使用实际typed武器/弹头，不改RadLevel/ROF/伤害/CellSpread；精英由夹具设rank，仅验证对应部署槽。
- 新aymod_radiation_field_releases_a_live_squid_after_load从实际普通部署武器读取RadLevel、经现有cell_spread::whole_cells换算范围，在既有RadiationState::apply_detonation生产边界注入场；无DESO、子弹或直接ReceiveDamage。这是独立周期消费者验证，不冒称完整发射路径。实际RadSite typed warhead确认Radiation/Sonic，保存前和恢复后均保持场及缠绕，再正常advance_tick至application_delay内解除、双方存活、瘫痪清理与旧动画回收。
- 该默认类型的原生最终调用可达4DA629具体ReceiveDamage，Sonic沿既有4D7345/4D735F→ExitUnit62A4A0；NPAE站点/绑定类型分支另见检查点，未把stock连续体直接当NP证据。现有状态/加载/伤害/寄生owners无需新实现，无新增RNG、timer或Detach逻辑。
- 最终NP实际三项（含部署两个rank）及radiation owner4项通过。仅测试/文档；无GUI、MOD/原保存/固定构建写入，无发布。换主、多AI、完整抗压、部署场的完整保存后长期衰减和原鱿鱼-海豚崩溃回放仍待验，不关闭全部V07。

### V07缠船、再次捕获与正式存读（2026-10-28）

- 扩展现有aymod_radiation_beam_releases_a_live_squid_from_its_ship，不另写寄生状态或损伤路径：实际Return Dubai/Battle加载、敌方AI SQD自主选中玩家DEST，真实SquidGrab开火、生成缠绕动画后，经GameSnapshot验证保存/完整资源恢复，双方指针和动画对象保持。
- 实际DESO普通Attack触发RadBeamWeapon/Sonic解除；双方存活、鱿鱼退出limbo、寄生指针清除、宿主瘫痪结束。随后同一AI鱿鱼无需伪造玩家命令，自主再次捕获同一艘船；第二动画不同且第一动画回收。再次普通DESO Attack解除后，正式保存/恢复继续保持释放状态，船仍能执行普通Move，两次动画均回收。
- 第二次捕获是受控生命周期边界，不是完整交战强度测试：解除后DESO会通过正常被动选敌向敌方释放鱿鱼开火，早期夹具因此无法保留它。已观察真实Infantry fire_event后，经现有mission authority暂置DESO和DEST为Sleep，排除持续反击；不改武器/兵种数值，不设寄生链接/伤害timer。鱿鱼本身仍由正常AI任务选敌。
- 实读实际NPAE FootClass::ReceiveDamage4D7345读取Warhead+14B、4D734F读取Foot+694，并于4D735F调用ExitUnit62A4A0；使用现有Sonic伤害前缀owner，无新增RNG、timer写入或Detach调用。普通任务和保存恢复仍调用原有owner，原寄生timer的Load重启语义不改。
- 最终实际复合回归1项、sim::combat::parasite::tests::25项和巨炮三阵营计划回归1项通过。测试-only，无新玩法实现/critic；未启动GUI、未改MOD/原始存档/固定构建，未发布。初次测试过滤器误写导致0项，已改用真实测试名执行，不计为通过。
- 辐射场RadSite、部署普通/精英武器、换主、完整抗压及原生/GUI并排仍开放；不能用本条关闭全部V07或报告原鱿鱼/海豚闪退已全面排除。

### V12建筑换主额外旧主AI、驻军重入门与新主恢复（2026-10-28）

- 实际AY BuildingChangeOwner448BD9→C2ECB9先调用ReReveal+48C，写Building+660=false，再调用实例vt+5C；零售Building vtable7E3EBC的+5C指向43FB20。回448BE8委托完整TechnoChangeOwner7014A0，448BED→C2ECD6再恢复+660=true、清+6C8。+6C8已确定为既有gap_generator::LastOperational，不是未知字段。此为无新键的NPAE补丁，不改GoOnline/GoOffline准入或复用Ares EMP deactivation。
- 在既有change_owner_impl、旧House列表/视野移除后、Techno委托之前，以building_power owner的原始写入接口临时关在线，调用唯一object_ai_visit_one_with_effects完整AI；不推进全局帧或复制任务/EMP/盖特逻辑。完整Techno尾后、Building的factory/GrandOpening尾前恢复在线并经LastOperational owner清采样，让新主下一次正常AI产生正确边沿。同House拒绝仍早于这次调用。结构退休输出判断提取为同一ObjectAiOutcome方法，普通对象轮与本次同步调用复用；bridge/结构销毁/换主通知进原有瞬时输出缓存，无新增保存字段或第二套玩法状态。
- 必要依赖：实际458307→C2ED1D在occupied-neutral归属改变前检查+660；4582DD→C2ED3B在empty-owned的遗弃声音/radar之后检查+660。现有allegiance owner两处都缺，额外AI在旧House尚未交换时会再次要求同一换主并无限递归。唯一只读critic确认此P1；在原owner的这两个原生位置补NP门，不添加递归锁/影子归属。实际CAHSE01普通E1 Capture首次进驻夺楼→存读→普通Unload最后一人离楼归中立→再存读通过；新offline双分支回归保护NP拒换主且空楼仍通知，stock/Ares-only仍按原行为改变归属。
- 红回归：实际YAGGUN使用正式TTNK.Primary→AirburstWeapon子弹EMP配置进入生产impact；不改counter/MOD，原换主后计数仍50而期望49，修后额外Techno AI真实消费1次。另从正常手动Off换主，验证无条件恢复online、LastOperational=false、同House调用不消费EMP/不改在线位；两种状态正式保存/恢复、正常恢复operational并由新主普通Attack实射通过。不是完整EMP时长/目标政策认证或真实心控准入。
- 直接消费者发现的预期差异已解释而非修改golden：原工程师joined fixture用生产换主安装“已发生owner race”的输入，新增旧主AI会先付费修复/更新damage mode；setup现在恢复其原witness输入HP/estimated/sample/damage state，再比较arrival。实际repair_order_enemy_owner_race这一路NP输出也确实多旧主付费修复，不能再套stock输出；在该既有测试内单列NP行为断言（HP提高、旧主付费、新主归属、停付修、工程师正常延迟清理），其余stock比较/RNG与golden文件不变。stock/Ares-only完整原九路十到达仍通过。
- PowersUnit既有捕获回归先因“60帧后新主仍无计数”的stock预期在NP失败。stock确无+6C8换主写入；NP现在按原生清采样，新主下一正常边沿即获得支持，无需人为断电再来电。测试按backend分别断言，旧主两个ROBO立即offline与一次EVA/两次单位声、新主恢复、后续自然失电/恢复仍覆盖；NP/stock/Ares/混合的powered_unit各8项通过。未修改原生House聚合计数或用户尚未批准的独立供电/失联Drive移动政策。
- 最终覆盖：纯NP定向93项（实际新建筑1/驻军1、医院9、地空盖特6、实际实验室2、电脑捕获出售/重夺2；allegiance6、工程师3/power消费者5、gap13/supercharge5、槽电力5、EMP15、建造12、powered_unit8）。stock35项；Ares40项通过，另两项gap_generator_tests的defeated_viewer_fogs_hostile_gap_through_power_cycle_without_shroud、defeat_does_not_reclassify_an_active_hostile_gap失败。两者在defeat_local_viewer先仅写stock计数、没有alive House的Foot实体，Ares HouseTracking::game_alive10024990要求registry实体，因而在新NP hook执行前就失败；Ares生存owner另7项通过，未改本链外夹具/规则，不冒称Ares全绿或运行了基线全套。混合仅定向归属/工程师/换主gap/槽/供电23项通过，不把未跑两夹具算通过。早期错误gap过滤器0项及未带ignored的捕获2项不计成功。
- 71e67bf1b后续实际AY验收：同一aymod_captured_center_transfers_support夹具按typed PowersUnit逐对运行GAROBO→ROBO、NARADR→DRON、NAPSIS→CAOS，正常Battle加载后供给原主真实电厂/中心和另一个同俄罗斯Country但独立human House的电厂/对应单位。新主原先无中心，通过普通Move真实到格触发739F21离线，不手写provider/counter/latch；原主仍获支持。首次完整保存恢复后，由新主SENGINEER普通Capture实际步行夺取，旧主House计数归零/旧单位offline，工程师正常消耗；捕获边界第二次保存保留已有counted值，普通更新让新主获得支持并Reactivate。没有对中心发Off/On或补另一座中心/伪造House标志，新主普通Move使对应单位实际到达新格，第三次保存保留中心/双方单位归属、旧offline/新online、House支持与位置。
- 本次复合1项内部三组配置均通过，另重跑实际建筑AI1/驻军归属1/powered_unit8，共11项通过；与上轮93项有重叠，不相加为新总数。测试不对失去支持的旧Drive单位继续下Move，因此不能把offline旗标冒称“新命令必被拒绝/绝对不能动”；保留V01原生聚合及失联移动政策门禁。新旧House均为受控组件，不是多AI自主夺取/长局；无新玩法实现/RNG/timer/Detach调用/保存字段，无需新critic，无GUI/MOD/原始存档/固定构建改动。
- 无新直接RNG draw/保存字段；额外原生AI通过既有Mission/EMP/repair/动画/manager计时、Scenario/Main RNG及Detach/生命周期owners产生其正常副作用，不保持旧错误路径的抽样次数。稀有同步退休/多manager回调、特殊Temporal/高度、多AI长局与整个App画面尚未验证；不关闭全V06/V01或整MOD。无GUI、MOD/原始存档/固定玩家构建/发布变更，无全workspace/Clippy；一次critic后的修正由owner定向复验，未追加critic轮次。

### F05实际MTNK行进跟踪、移动开火与完整存读（2026-10-28）

- 新aymod_moving_grizzly_tracks_with_independent_turret_after_load从正式AY Battle加载MTNK/SMCV，目标经共享Unit换主receiver交给独立Russian human House并建真实GAPOWR，排除原电脑自动部署/改派。目标只收普通横向Move，射手只收普通Attack，自行追击/瞄准/发射；双方均要求实际位移，目标须被实际武器损伤，无手写朝向、位置、rank、运动/旋转timer或MOD参数。
- 实读NP UnitCtor735584→C2DEB6..C2DEC9：读Type+5A5的无符号TurretROT byte，FF才回退Type+71C普通ROT，再返回73558A；保持既有ObjectType::unit_turret_rot构造及unit_post单owner。实际typed MTNK两个速率必须不同，整段普通帧车体/炮塔FacingClass各持自己的速率；观察到真实位移时两者朝向不同，也要求至少一次真实位移帧出现射手fire_event。不是靠只观察渲染角度或改速度来凑差异。
- 在真实运动且两控制器朝向分离时完整保存/资源恢复，核世界坐标、车体/炮塔完整FacingClass、rearm、攻击目标和移动目标坐标不变；随后继续普通帧，必须再次实际发射并使原目标HP下降。初样例额外要求运动时仍有多帧is_rotating弧，但本例短角差没有这种窗口（运动分向和移动开火已观察到）；因此保存运动中的真实分离控制器，不强造大角度插值，并保留大转角特定热载入窗口为未实测。活动插值本身仍由既有FacingClass原生组件golden覆盖，不把组件等价扩大为该实际窗口。
- 最终retail gate下新实际1、movement::turret3、FacingClass14、combat_turret_facing7，共25项通过；无生产修改/新玩法critic、新状态/RNG/timer/Detach/保存布局，无GUI/MOD/原始存档/固定构建/发布变更。只验证常见MTNK这条运动追击链；其他13类型、异常运动/旋转组合和GPU/GUI仍保留。

### V03早返短间隔：实际配置影响边界（2026-10-28）

- 原生GetROF6FCFA0的Building Ammo>1分支6FCFA9..6FCFBE与空WeaponType分支6FCFD2..6FCFDD均直接EAX=1/RET4，绕过普通NP出口C2C13D（POP EDX，-1时MOV EDX,EAX）；两个早返未建立统一的EDX短间隔结果。FireAt6FF289调用返回后6FF29E仍存EDX到+2F8，berserk则先经6FF299 CDQ覆盖。不能因普通出口约定便把早返硬补成同一短间隔，当前fireat_rearm_frames的unwrap_or(reload)继续明确保留为确定性fallback，不宣称等价或把机器指针当游戏时长搬入。
- 新aymod_rearm_early_return_inventory_tracks_buildings_and_occupiers通过正式load_session/Battle加载XMP03T4和Return Dubai的全部typed对象；检查Ammo与InitialAmmo>1，不只grep段落或现有实体。实际仅CAAIRP/CAPOWR，Ammo5/InitialAmmo-1；两者全部普通/精英槽均None，CanBeOccupied/CanOccupyFire均false，所以这两个类型本身没有正常FireAt/驻军射击入口。原先空列表假设被真实typed数据推翻，已保留实际两项而非删去候选。
- FireAt6FF031..6FF085先推进驻军再GetROF；库存同时使用生产occupant_weapon核所有typed Infantry Occupier的普通/精英结果，非空且候选数>0。正常有效游标循环下没有发现下一位无武器；这不证明游标>=count/中途成员退休/升级回调等运行时替换不可达，尤其既有V05完整原生游标循环仍开放，不能据库存关闭它。普通未装武器的单位本就过不了首次发射，不因存在大量空备用槽就当成此次早返的消费者。
- 最终新复合库存1项（两地图）通过；只有测试与台账，无生产状态、RNG/timer/Detach/保存布局变更，无新玩法critic/原生执行golden。不把这一配置核查说成全部GetROF调用或所有地图/模式已等价；未启动GUI、改MOD/原始存档/固定构建或发布。现阶段不为未找到的常规射击消费者加入猜测性修复。

### V06盖特坦克/机炮地面与对空生命周期（2026-10-28）

- 原地面aymod_gattling_promotes_through_fire_and_resumes_after_hot_save与新增对空aymod_gattling_air_kills_promote_and_resume_elite_aa_fire_after_load共用aymod_gattling_promotion_lifecycle，正式AY Battle配置分别跑YTNK/YAGGUN；需要电力的机炮使用真实GAPOWR，不伪造operational/档位/等级或改MOD。
- 地面仍用原APOC链。对空用实际JUMPJET，先由普通Move自行起飞并确认生产IsHighFlying，再经同一mission owner暂Sleep隔离反击；没有手写高度/飞行flag。最终敌对目标属独立新human House并有真实GAPOWR基地，隔离电脑任务改派/败北；不声称电脑自行派出空军或自主AA作战。首次用电脑ZEP时第二目标在tick59、前帧HP500/地面高度/AreaGuard下消失；改长目的地也未解决。未追出退休原因/证明gamemd差异，不能当已修AI bug或推断所有ZEP起飞失败；AI自主边界继续开放，不恢复已暂停的Return Dubai攻势调查。
- 经验阶段唯一伤害准备是敌方残血1HP，每次普通Attack都要求真实fire_event/死亡与Record_The_Kill702D40增加经验，直至精英；对空每个攻击前帧仍高飞，发射武器必须是实际普通/精英奇数槽且Projectile.AA=true。实读SelectWeapon6F3436..6F3474：stage*2+1的选择仍受首对AA projectile和目标+54高飞门，不能替换为NP档位跳升。没有直接伤害补刀、set_elite/set_rank或改经验/Cost。
- 蓄速用友军目标隔离反击：地面NACNST；对空为真实Move起飞后暂Sleep的ZEP。正常射击让stage提高，正式保存/完整资源恢复保留精英经验/cache及stage/value；对空恢复后目标仍高飞。Native Load70BF61/70C20E清+4B8 report latch，测试继续分别核保存前true/恢复后false，不为凑结果改声音owner/RNG。
- 读档后普通Stop/Guard退至stage0/value0，经验不丢；再次普通Attack时真实发射武器匹配发射前当前stage的EliteWeapon，地面偶数槽/对空奇数槽。对空续射时目标仍高飞，允许原生瞄准时蓄速，不硬设高档或假定首发永远stage0。自然起飞辅助先让射手Sleep仅是测试隔离，不把该输入说成自主索敌/完整强度交战。
- 最终retail gate下gattling ignored5项（新地空两复合场景、原精英首发、零售机炮蓄退档、坦克碉堡准入）、Gattling组件7、Veterancy12通过。只有测试/台账改动，无新增生产状态、RNG/timer/Detach调用或保存字段，无需新玩法critic；未启动GUI、改固定构建/MOD/原始存档或发布。
- 坦克换主续验：同一生命周期helper新增transfer_unit，仅YTNK分别跑地面/对空实杀晋升和真实hot stage。热状态建立后创建带真实GAPOWR的另一个YuriCountry human House，经共享Unit7463A0→Foot4DBED0→Techno7014A0 receiver换主；即时经验/cache/stage/value不变、旧攻击目标清空。正式存读保留新House与热状态，随后新主人普通Stop/Guard退档、普通Attack按实际精英地面/对空槽重新开火。不是实际心控武器或占领准入；YAGGUN不进入此坦克保持测试；V12已接额外建筑AI，热机炮的具体档位变化仍需独立验收。最终gattling ignored6项一起通过，原组件7/Veterancy12上轮通过本轮不重复，无新玩法实现/critic。
- 机炮换主续验：aymod_hot_gattling_building_changes_owner_and_resumes_elite_fire_after_load复用同一helper，分别跑YAGGUN地面/对空实杀晋升、普通射击蓄速、新human House换主、完整保存恢复、普通Stop/Guard退档和新主人精英对应槽真实续射。新主人有真实GAPOWR；不直接设置经验、档位或供电。原生Building448BD9→C2ECB9的旧主AI43FB20已由V12实现，本测试保存额外AI后的实际stage/value，不照搬Unit的即时不变断言；仍要求换主不丢经验、旧目标清除、读档状态完全保留及report latch清除。换主调用共享receiver，不声称工程师或心控武器能实际占领该机炮。
- 本轮最终retail gate下gattling ignored7项、Gattling组件7项、Veterancy12项通过；新复合测试含地面/对空两路，早先单跑1项不重复计数。仅测试/台账，无新生产行为、RNG/timer/Detach或保存字段；不需要新玩法critic。未启动GUI、修改MOD/原始存档/固定构建或发布。缓存清理再次因Unsupported PE debug closure安全停止、移除0字节，未手工处理固定版。
- ROF/rearm续验：复用同一生命周期，实杀晋升每发、精英持续蓄速及读档退档后的新主续射均核实际发射武器与开火前rank对应的完整reload；此受控human House的typed rof_bias断言为1，复用既有ftol_scale/ROF ability判定，不复制GetROF或重新计算RNG。实读AY6FD09E..6FD14C确认RandomRanged(0,2)仍调用但6FD0CB..CE已NOP，随后仅相应经验能力应用VeteranROF。没有修改数值或NP随机流。
- 热状态现在要求至少两发实际蓄速射击，连续两发的间隔不得小于前发保留的+2F8 burst/reload时长；在真实开火后保存，确认+2EC仍在运行，完整保存恢复保留RearmTimer和+2F8，之后正常Stop/Guard退档与精英续射。该下界和有界续射不是全部档位的逐帧节奏等价，也不覆盖Bunker/占据、非1的House倍率或berserk。
- 首跑新增计时断言误用了已提交的session.tick：实际所有玩法使用native frame N，尾部才提交N+1；已按既有runtime/session契约改断言为binary_frame-1，不改生产时钟。最终gattling ignored7和ROF组件2（既有stock/NP原生输出）通过；只改测试/台账，无新玩法critic、状态、RNG/timer/Detach调用或保存布局。此前Gattling7/Veterancy12未在本轮重跑，不叠加计数；未启动GUI/修改MOD/原档/固定构建或发布。
- 碉堡进出续验：新增aymod_hot_gattling_tank_enters_fires_and_deploys_out_of_bunker，复用实杀晋升/热状态helper并用明确Continuation枚举区分普通、换主、碉堡，地面和自然起飞对空各跑一遍。实际YTNK由普通Enter接触/移动/安装进入新建NATBNK，不调用install/release或写链接/经验/档位；确认双向链接与精英经验，实际typed BunkerROFMultiplier非1。实读GetROF6FD1B1..1EF：非建筑+2E4链接时，在经验倍率后除以Rules+F50。每发实际完整reload核该顺序，原始MOD不变。
- 安装中真实射击升档后完整存读，链接/经验/stage/value/+2EC/+2F8保持，正常帧读档续射仍应用碉堡倍率。普通DeployMcv命令走Unit Mission_Unload73D63B..66D→EmptyBunker4595C0；双向链接清除时再保存，恢复后沿正常退出Move实际离开，随后Attack恢复未安装倍率。之后复用原热保存、Stop/Guard退档和精英对应地空槽续射；不硬断言经过真实行走/待机后stage/value应恒定，不增加影子状态。
- 首跑退出断言把FNPC合法的相邻出口误判为必须离建筑原点两格，已依据既有零售出口测试改为比较安装时单位自身world-XY到释放后位置的实际移动超过256 leptons；普通链通过，无生产修改。最终retail gate下gattling ignored8项和ROF组件2项通过，新单跑1项不重复计数；无新玩法critic/RNG/timer/Detach/保存布局变更，未启动GUI/改MOD/原档/固定构建/发布。当时碉堡毁坏/出售释放待验，后续四路覆盖见下；争抢/多House、完整节奏与画面仍开放。
- 毁坏/出售续验：aymod_retired_bunker_releases_hot_gattling_for_saved_move_and_fire复用同一Continuation/BunkerRelease测试分支，分别以Destroyed/Sold各跑地面/对空，共四路。均先实际杀敌晋升/蓄速、普通Enter安装NATBNK、热驻留完整存读与真实续火。Destroyed新建独立human攻击者House和真实GAPOWR/MTNK，只将目标碉堡HP备为1；普通Attack必须出现对碉堡真实发射事件并由伤害/死亡receiver击杀，不直接调用fatal/release或改MOD。击杀后测试以既有mission owner暂Sleep攻击者，隔离后续追杀；不是强度交战或AI自主进攻。Sold只发普通SellBuilding，按实际Unsellable=false准入，不直接退休建筑或索引。
- 两路共享原生release_sell_destroy4593A0，不同于EmptyBunker4595C0不会自动队列Move：链接清除时完整保存/恢复，保留建筑仍在/已退休的区别、单位链接/经验/stage/value/+2EC/+2F8；坦克存活、locomotor已通电、精英经验仍在。恢复后再发普通Move并确认实际位移，正常帧等碉堡完全退休、链接仍None，再真实Attack验证未安装倍率；之后原热保存/退档/续射断言继续执行。未硬设导航/track/电源/经验/档位或把释放移动虚构成自动出击。
- 最终retail gate下gattling ignored9项、bunker_link23项通过；新复合四路只算1项，较早仅毁坏单跑1项不叠加。只有测试/台账，无生产行为、RNG/timer/Detach/保存布局修改或新玩法critic，无GUI/MOD/原档/固定版/发布变更。现有共享release正常副作用仍走原owner；该例不覆盖Temporal/争抢/完整强度攻击或多AI，不关闭全V06。
- 建筑换主基本地空热状态链已验；V12稀有退休/多manager回调、碉堡Temporal/争抢/多House、其他载具登离、其他ROF倍率/全档精确节奏、完整强度敌军/自主多AI和GPU画面仍开放；不是全部V06或整体适配完成。

### V10精英名义射程库存与磁暴IFV：配置差异不是引擎加成缺失（2026-10-28）

- 正式load_session后遍历完整已分配ObjectType的18个普通/精英槽，逐个确认引用能解析成WeaponType，再比较成对Range；不是物理段grep或只看Primary。Battle/XMP03T4与Return Dubai各120对，均无未解析槽；所有正普通Range与非负精英Range中的较短项如下。负数特殊射程不作普通数值排序，本库存不涵盖几何/最小射程/炮口/目标选择变化。

| 类型/零基槽 | 普通武器/格数 | 精英武器/格数 | 分类 |
| --- | --- | --- | --- |
| FV / 6（Weapon7） | CRElectricBolt / 6 | TankBolt / 5.25 | 实际SHK IFVMode=6可达；原MOD明确配成更短武器，不加max(normal,elite)或修改配置 |
| UTNK / 0 | Comet / 11 | 120mmE / 6.25 | 实际typed和物理键一致，TechLevel=-1；具体普通生产/AI/地图使用未核，不能据此全局判不可达 |
| XCOMET2 / 0 | OPCoilFragment / 4 | TeslaFragment2 / 3 | 已分配Vehicle类型的明确配置，具体常规消费者未核，不因名字像弹片就猜其对象身份/可达性 |
| XCOMET2 / 1 | TeslaFragmentE / 3 | CoilFragmentEMP / 2 | 同上，保留配置而非给未知链补射程 |

- 实读实际AY GetWeapon70E15F..70E17C精英槽选择、GetWeaponRange7012CD..7012DF读取选中Weapon+B4，及InRange6F7248同一Range读取。沿既有当前gunner选择/范围owner，未发现本入口把更短精英Range夹回普通值的分支；不把全库存结果扩展成排除所有其他间接补丁。FV具体实际场景中SHK经普通Enter登车，当前槽自然为6，普通Attack真实发射CRElectricBolt；Stop后通过既有direct-receiver/RecordKill奖励取得精英，未写rank/slot，正式存读仍保留乘员和TankBolt。
- 同一实际GAPOWR的2x2 typed foundation下，给定距目标6.5格的查询源点，普通通过、精英拒绝，证明真实配置缩短会传入现有InRange；该点不是实际炮口。实际近距普通Attack在读档后仍发射TankBolt，未因更短射程失去武器/卡死。奖励用明示致死receiver输入，不冒称IFV实杀晋升；不改MOD或承诺原作者有意如此平衡。
- 最终库存1项（两正式地图）、实际IFV场景1项、NP gunner25/InRange32/GetWeapon3项通过；首轮“所有精英应不缩短”的调查假设被上述4条配置推翻，后续依据实际键/原生入口分类，不增加生产补偿。仅测试/台账，无新critic/RNG/timer/Detach实现或保存字段，未启动GUI/改原始存档/固定构建/发布。其他模式/地图、UTNK/XCOMET2消费者与最远实际炮口保留；不据此关闭全V10。

### V10磁电坦克精英射程：实际配置与既有共享owner（2026-10-28）

- 复读作者170507：第116行笼统称“部分武器精英级后射程反而降低”，第178行明确点名磁电坦克。对应TELE，不是幻影MGTK；不能将作者两句话都直接归因到Arcing补丁。正式Battle/XMP03T4及Return Dubai加载下，TELE普通/精英主槽为MagneticBeam/MagneticBeamE，Range12/13；建筑副槽MagneShake/MagneShakeE为11/12格。本轮未改这些值。
- 实读实际AY GetWeapon70E15F..70E17C选择存在的精英槽，否则70E17E..70E18B回退普通槽；沿既有get_weapon/weapon_range/InRange，不增加精英通用射程奖励或TELE专属例外。Return Dubai生产typed reader下，平地给定查询源点对车辆/建筑分别放在普通射程外半格（建筑仍经既有foundation owner），普通拒绝、精英通过，GetWeaponRange保留实际精英原Range；不是实际炮口或远距攻击回放。
- 实际Battle场景：TELE普通Attack对GAPOWR发射MagneShake，普通Stop后沿既有致死receiver/RecordKill/Add取得精英经验，普通帧采样等级，不写rank/XP。这里APOC奖励来自明示direct-receiver击杀输入，不是磁电抬升实杀。正式存读后两个武器槽仍分别13/12格；再普通Attack实际发射MagneShakeE并造成建筑伤害。这覆盖真实建筑选副槽、等级切换与保存恢复，但不是13格车辆抬升/12格最远建筑攻击验证。
- 最终实际两项、NP InRange32项及combat_weapon::tests3项通过；仅测试/台账，无生产改动/新critic，无新增RNG、timer写入、Detach或保存字段，既有owner副作用保持。未启动GUI、未改MOD/原始存档/固定玩家版或发布。该具体配置链不再列成纯缺失；作者“部分武器”其余身份、精英最远炮口/实际抬升、多AI与视觉仍保留，不据此关闭全V10。

### V10相关射程：Arcing与SubjectToCliffs联合分支（2026-10-28）

- 实读NPAE174 InRange6F73EE..7404与C2D5ED..C2D601：先检查ProjectileType+29B Arcing，再检查+296 SubjectToCliffs；两者均真才进原ballistic arm，否则落原direct arm6F751E。
  46BBCE置EBX0、46BBE1清+296；46BFE4..46BFF8以当前bool读精确SubjectToCliffs。现有typed字段及parser用于本链，没有另设开关、字段或修改MOD的Level/Arcing/Range。
- 已以RangeProfile的小型只读值选择同一个范围内核；所有生产compute_range_target调用及外部GreatestThreat调用均用active()，mapless使用同一选择器。不是两份数值实现，也不是给某个兵种单独增加范围。
  最小射程与目标坐标解析仍在分支前；direct的高差/建筑foundation/桥面检查、ballistic的物理可达性与桥面检查、最终line-of-fire保持原owner。NP新路径跳过的旧ballistic查询按原补丁分支发生，不提前模拟或复制。
- 新生产回归先真实红false而非true：现有原生控制建立的834水平/两级抬高、range900边界，在SubjectToCliffs=false下NP应走direct；真实地图实体查询与mapless均覆盖，cliffs=true仍走旧ballistic。未改任何数学内核或武器参数。
- 旧原生range/bunker控制来自Stock，现显式选择Stock运行同一生产source builder/target converter/range内核组合，保留原golden、几何及Dummy查询结果；不把这些输出冒称NP原生执行。独立NP行为通过生产默认active路径验证，未用测试开关篡改生产状态。
- 真实Return Dubai/Battle规则加载确认GTGCAN2普通/精英武器的实际投射物均Arcing=true、SubjectToCliffs=false；以真实NACNST基础数据和给定平面source点比较同一内核：NP direct计入目标foundation，Stock ballistic不计，原武器Range未改变。
  此为实际typed规则+受控范围查询，不是整局攻击、实际炮口/弹道命中或渲染验证，也不证明所有精英射程问题都来自这一条。
- 既有隔离line-domain回归增加NP新路线：Arcing=true/Cliffs=false/Walls=true，两端桥面数据保留最终射线的4次同Z采样，并确认桥下source拒绝；不是为了新路径绕开墙/桥面。
  最终NP/Stock InRange各32、GreatestThreat各43通过；NP line_of_fire1、context input39及实际AY数据1通过。一次只读critic无可操作发现，其组合桥面/射线覆盖缺口已补；未重复critic。
- 不新增随机抽样、timer/Detach或保留字段，未改MOD/原始保存/固定cfb479750构建、未启动GUI、未推送/PR。7012C0 GetWeaponRange及70E140 GetWeapon已读连续体未见静态补丁，不据此排除所有间接扩展。V10“精英射程反降”其余原因仍开放。

### V10光棱联击：断电支援塔准入（2026-10-27）

- 实读NPAE174 BuildingClass::Mission_Attack44B388→7E04BB：先读候选Building+6C8，假则跳44B4AE下一个候选，真则恢复原alive+90检查，再回44B38E。
  原producer43FBEF在Building Update的运行状态边沿处理后存储该采样；现有LastOperational由world/gap_generator唯一写入，Prism只读，不用+660手动online、+6EA stuff_enabled或现场重算替代。
- 已在唯一prism_supporter加NP采样门，stock不变。新回归先真实红：手动Off并经过真实sample producer后仍被选中；修复后拒绝。
  回归特意覆盖两个旧采样边界：Off尚未Update时仍可支援，On尚未Update时仍拒绝，下一次producer才反转。不能为了立刻生效破坏原生对象访问顺序。
- 旧stock原生shared-body比较夹具未运行Update采样，现显式给所有候选true的额外NP前置，继续比较不变的原生公共体输出；没有改golden或执行/存档harness，也不把此控制边界称为完整NP原生对照。新增边沿回归另外走实际GoOffline/GoOnline和visit_building_operational。
- 真实AYMOD：正式Battle加载、两座ATESLA和真实GAPOWR；友军APOC只供主塔显式玩家Attack，避免支援塔先自动Attack而合法失去招募资格。
  普通SetBuildingOnLine Off→采样false→正式内存保存/恢复保持false→主塔实际独立开火且支援计数0、无待发支援束；普通On→采样true→真正招募并给主塔加count→主塔再次实际开火。未改MOD数值、原保存或固定构建，未启动GUI。
- 最终NP/stock Prism各7、Building missions各16通过；NP LastOperational直接消费者gap_generator13通过；相关ignored零售Dustbowl光棱转发各1单独运行通过，剩余四个ignored是非Prism场景未扩跑。实际AY复合场景1通过；一次只读critic无可操作发现。
- 建筑任务覆盖首次发现四项旧围墙测试期待清目标重置burst，实际返回7而非0。已追到此前V03的同一Assign_Target owner及实际NP6FCF5B..60六个NOP，仅修测试profile预期：NP保留7，stock仍清0；围墙目标/命令/恢复/反击逻辑均未修改，没有靠改断言掩盖本轮玩法缺陷。
- 本检查不新增随机抽样、timer、Detach或序列化字段。已经招募/已排定的支援束仍由原ProcessDelayedFire处理，原补丁没有加中途掉电取消，不能擅加。
  实际EMP、全House缺电、抽电/出售与支援束GUI画面未单独重放；本条不关闭全部光棱、供电或V10。固定cfb479750玩家版不包含本次修复。

### V10铁幕/力场保护目标准入（2026-10-27）

- 原生Evaluate_Candidate6F826E→7E0AFC先问目标vt160，再读目标Type+233，任一为真跳6F894F拒绝；前者是IsIronCurtainActive41BF40（共享保护timer），后者已通过ReadINI5F94E7..5F9510与832B70字面值确认Immune，不是Invisible/TypeImmune。
- 扫描owner在原class/LegalTarget位置加入NP检查，保留此前GetFireError探测、距离与类别顺序，沿既有is_invulnerable读当前帧；到期帧立即不再拒绝，存档仍由原InvulnerabilityState持有。
- 对象输入What_Action_OnObject700444→7E0B35在强制Ctrl绕过早期条件之后仍问目标vt160，真则落70056C选择尾。现接既有attack_action，普通/强制对象点击都服从此门；Cell强制攻击、既有Attack命令执行与伤害免疫owner未改。
- 一次只读critic发现必要调用链缺口：普通谭雅C4/间谍渗透的点击与光标捷径绕过base Attack。已按Infantry51E9F1/51EE62让它们复用attack_action；已准入的后缀不递归，模拟PlantC4/Capture/Attack执行端不另设政策。先筛非C4类型避免无关单位重复完整查询。原NoManualFire输入绕过残余随此owner迁移修正。
- 三项聚焦回归：实际retail扫描IC/FS最后活跃帧与精确到期、Immune typed reader；普通/Ctrl点击及谭雅/间谍光标与下单、到期恢复；真实AY MTNK Guard在保护期间不自动接敌，正式内存保存恢复后等待原timer到期，再实际开火。新扫描回归修复前真实红Some(target)而非None。
- 实际场景目标MTNK通过原mission authority置Sleep，排除另一个反击准入机制；未改MOD或通过玩家事件绕过电脑命令门禁。早期barrel/NACNST不可被此Guard扫描选中、16x16输入夹具水格/炼厂位置失败均为夹具问题，已用可选敌方坦克和既有32x32零售输入arena修正，不列生产修复。
- 新检查本身不抽RNG、不写timer/Detach；更早拒绝会按原生跳过后面的伪装探测抽样。原被动扫描周期/保护到期/目标清理仍各归原owner。尚无GUI/全对局验收，不宣布所有AI、免疫或铁幕联动完成。

### V10旧Attack下实际铁幕/力场发射与存读（2026-10-28）

- 新aymod_existing_attack_survives_launched_protection_and_full_save共跑IronCurtainSpecial/ForceShieldSpecial两路。正式AY Battle的MTNK先通过普通Attack实际发射并损伤敌方NACNST；目标经正式change_owner receiver归独立新Russian human House，排除AI维修/卖房/指令改派。另放实际NANRCT/NATECH/NAIRON供应电力和provider；不改MOD、HP、保护timer或攻击target字段。
- 复用现有charge_super把“已经充好的一发Super”作为明确fixture输入，随后只发普通LaunchSuperWeapon/SPECIAL_PLACE命令；实际House Fire_SW/ClickFire/Launch按正式typed时长赋保护。不是自然充能或普通生产获得超武的经济链验收，也不是直接apply_invulnerability的45帧替代。每路要求实际kind正确、正式timer运行，保护前已有真实攻击和伤害。
- 普通攻击目标不会因保护自动清空：全保护过程保留同一Attack target且实际继续开火，目标HP保持；到自然到期后的真实武器命中重新降低HP，不重发Attack/强攻绕过输入准入。NP的新选敌/对象输入门仍仅在既有Evaluate_Candidate6F826E→7E0AFC与What_Action700444→7E0B35；这条已准入Attack和免伤消费者不新增政策。该回归是受控生产链行为，不声称已有原生整局逐帧重放或所有攻击class都等价。
- 第12个保护帧完整保存/资源恢复：原保护timer、Super fade倒计时/位置、发射House blackout剩余值、攻击者rearm计时均精确保持；后续每个保护帧均不扣血并维持旧目标，到期恢复伤害。已验证fade/blackout状态持久化，不把这等同于已走完它们各自的到期声音/House电网恢复回调；这些消费者当时待验，后续实际fade/blackout到期见下一节；多House重叠/反击/脚本以及GPU仍待验。
- 最终新复合1、实际Guard拒新/到期接敌1、ForceShield8、IronCurtain8、Invulnerability6共24项通过；无生产行为、状态、RNG/timer/Detach/保存布局变更，无新玩法critic。原保护/tint/launch/blackout的正常抽样与写入仍走共享owners；未启动GUI、改MOD/原始存档/固定玩家版或发布。

### V10实际ForceShield fade/blackout到期与恢复（2026-10-28）

- 新aymod_force_shield_saved_fade_and_blackout_restore_powered_fire用实际NATECH/YAGGUN同一human House和真实NANRCT供电，先下普通Attack对友军lab真实开火确认可用。沿既有charge_super把已充好一发作为fixture输入，再普通LaunchSuperWeapon/SPECIAL_PLACE发射正式ForceShieldSpecial；不直接写保护/blackout/fade计时或MOD配置，不声称自然充能和经济生产已测。
- 正式typed SpecialSound必须存在。从实际Super fade倒计时/位置和House blackout剩余计算边界；第12帧完整保存恢复，实际提示在原fade到期帧只发一次VocAt并将fade设-1，随即再次保存恢复，后续不重播。两次保存都核同一fade/位置、blackout剩余及cannon保护timer精确保持；这是模拟输出事件，不声称音频设备实际播放或GPU渐退画面已验。
- 从保护发射后到blackout结束前，每帧House低电、YAGGUN building_operational_state=false且无实际发射，manual Building+660 online仍true。实读AY House4F844B..4F8489确认remaining==1帧清计时并置重算位；该精确帧后实际House blackout=0/低电解除、同一机炮operational=true，普通新Attack真正发射。没有重建电厂/切V电源或手写缓存来恢复。
- 首跑仅最终resume断言失败，不是计时或供电故障：目标是友军，Building Mission_Attack的Cant分支44B0DE..44B0F3通过vt+3C8清目标，Guard不自动再打友军。已实读该指令段，并修正fixture在供电恢复后由普通事件重下Attack；不把移动坦克保持旧目标的结论误套到停电建筑，不改变native owner/策略。测试仍逐帧保护停火/供电精确边界及实际续射。
- 最终新实际1、ForceShield组件8、power_system remaining-one1，共10项通过；上轮IC/FS旧Attack复合与其他Invulnerability组件未重跑，不叠加计数。仅测试/台账，无新增生产状态/RNG/timer/Detach/保存布局或玩法critic，无GUI/MOD/原始存档/固定构建/发布变更。多House/联盟重叠当时待验，后续三House组件见下一节；自然充能、其他反击/脚本和GPU仍开放。

### V10三House单向联盟与重叠ForceShield（2026-10-28）

- 新aymod_allied_force_shields_refresh_receivers_without_sharing_blackout通过正式AY Battle建三独立human House：A为原玩家，B/C均为真实Russian Country构造但不同House ID。各有真实NANRCT/NATECH与近邻实际YAGGUN，先正常帧确认三家powered consumer均operational。受控接收机炮通过既有mission owner暂Sleep隔离敌对扫描/反击；不写电源/保护/计时。fixture联盟刻意只让B列A、C列B，未声称正常Ally事件/UI外交或AI自主结盟验收。
- 实读原生ForceShield Launch6CD217..6CD235：从Building Array取接收者，问receiver.owner+21C的IsAlliedWith4F9A50，入参为launcher House ID，而不是反向。普通已充好Super/SPECIAL_PLACE第一次由A发射：A/B同受保护，C无保护。第一次保存恢复后正常23帧，再由B发射：B保护刷新、C新保护且同一新timer，A仍保留第一次timer；没有因同国家或发射者单方面联盟错误刷新A。两次Super就绪均由既有charge_super准备输入，不覆盖自然充能经济链。
- 第二发不重置A的blackout原timer，A fade仅随本帧自然减少；B有自己新blackout/fade；C虽受保护，却始终blackout0且YAGGUN可工作。正常帧逐一检查A/B各在自己的remaining1原生边界恢复House电网和building_operational_state，二者不会同步提前/延迟清除，C全程保持正常。两家的SpecialSound各在自己的原fade到期帧发一次，最终顺序/次数精确等于两个原到期点。
- 三次完整存读分别在第一发后、第二发重叠后、A已恢复而B仍停电的边界；逐House保留保护timer、raw blackout timer及各Super fade/坐标（含实例是否存在），然后按原时间继续。结论是受控三House实际组件，不是自主多AI或所有对称/非对称联盟组合；混合IC/FS、更多重叠/解盟中途、自然充能/正常外交输入与GPU仍开放。
- 最终新实际1、ForceShield组件8、Invulnerability6共15项通过；前轮单House10项未重跑，不重复计数。只有测试/台账，无生产行为/状态/RNG/timer/Detach/保存布局变更，无新玩法critic，无GUI/MOD/原始存档/固定构建/发布变更。

### 本轮功能验证：F02实际AY隐形箱（2026-10-27）

- 正式AYMOD Battle加载后，按实际Cloak权重10、CrateRadius768，生产place_crate放置固定Cloak箱，普通Move让MTNK拾取；
  周围同House E1通过真实范围owner接收实例能力。没有改MTNK/E1类型Cloakable、计时器或MOD规则，没有直接调用授予/隐形更新冒充实际拾取。
- 新增aymod_cloak_crate_grants_mobile_instances_and_survives_real_load：拾取→GameSnapshot正式验证保存→完整生产恢复/重建→MTNK/E1均完成隐形。
  仅内存快照，不写原始存档。新实际测试1项和原有Cloak箱owner3项通过，现有F02生产实现无需再改；本批仅新增测试，无玩法critic重复轮次。
- F02因此补齐实际MOD移动单位拾取/保存恢复链，不关闭建筑/落地飞机生命周期、敌方/传感器实战或GPU/GUI验收。
  未启动游戏、不替换固定玩家版、未发PR/推送；Return Dubai弱攻势线按用户要求暂停。
- Alt飞机链已完成本轮生产修正与回归，详见下一节。固定cfb479750构建未替换；完整功能清单仍开放，不结束整体goal。

### V10飞机维修建筑点击：NP跳过stock维修行（2026-10-27）

- 实际NPAE174 AircraftClass::What_Action_OnObject417CC0的417E24为EB57无条件JMP417E7D，stock为7457/JZ。
  因而整个UnitRepair/Dock点击覆盖分支被跳过，原Select/Alt Move保留；不是给所有飞机下达Enter的总禁令。
  对照原始4D74E0..4D7CF0、41AA80..41ADC0、417CC0..417F70连续字节，在这些边界只发现417E24这一处差异。
- 已在既有entry_click_action Aircraft分支增加NP能力门，避免仍用stock NoEnter拦截普通选择。核心Select回归修复前真实失败Some(NoEnter)而非None。
  Move/目的地/Fly/Radio/机场owner未另造政策，合法机场返回不禁止。证据说明：本轮证明的旧实现差异是Select覆盖；Alt运行时测试是完整相关路径防回归，不伪称重现了此前独立Alt误入。
- 核心retail覆盖GADEPT及GAYARD/NAYARD/YAYARD，船厂用真实Water基础地形准入；stock保留NoEnter，NP保留Select，Alt Move两者均不改成维修Enter。
  客户端ORCA/BEAG实际光标查询和entry dispatch验证：NP选择不被消费，stock NoEnter消费无命令，Alt都留给Move路径。
- 真实AYMOD场景另外覆盖ORCA对GADEPT和地形合法GAYARD：正式load_session、普通object_move_click(force_move=true)解析、正常命令队列及每案180帧；
  飞机确实发生XY运动、存活、从无目标维修厂/船厂的无线电联系/候场指针或Enter目的地。提供正常GAAIRC，允许合法返机场，不把无机场AirportBound坠机当作拒绝维修成功。
  三种船厂仅GAYARD做完整帧，其余两种已覆盖输入owner；GUI人工操作、其他飞机以及自定义Dock包含维修建筑的stock残余未扩称完成。
- 最终NP/stock核心入口各4项、NP fly_orders机场绑定1项、NP/stock客户端各1项、实际AYMOD双场景1项均通过。
  一次只读critic无可操作发现；其指出的仅维修厂实战缺口已再补GAYARD完整帧，无重复critic。早期夹具未设遭遇战导致shroud拒绝飞机、船厂落陆地、缺机场导致正常坠机，均修正夹具，不当作引擎缺陷。
- 没有新增RNG抽样、timer/Detach、保留状态或保存字段；未改MOD/原始存档、未启动GUI、未更换固定构建、未推送/PR。下一个明确功能入口为铁幕目标准入。
- 后续铁幕目标准入已沿两个入口接通并验证，最新结论见当前检查点，不再保留此处过期的“仅定位”状态。

### 非盟军AI巨炮：基地规划前置被错误强制绕过（2026-10-27）

- 用户已暂停Return Dubai弱攻势调查；本项是另外明确提出的基地建造问题，不继续攻势trace，不改AI权重/兵种数值/MOD文件或固定cfb479750玩家构建。
- 实际AY GTGCAN的Owner允许全部十国、AIBuildThis=yes、TechLevel=-1、AIBasePlanningSide默认-1，但Prerequisite=GACNST,GATECH；
  GTGCAN2为人类版、AIBuildThis=no。不能只看Owner就认定苏联/尤里应建，也不能因TechLevel=-1便套用人类拒绝条件。
- 原生AI_RecalcBuildOptions505890每轮EBP=-1；5058B5..5058CF只对字面值GAPLUG（824EB8）记录延期索引，
  无进展尾50594D..5059AD仅强制这一个特殊类型。Rust错误记录所有未选中类型，导致缺盟军前置的巨炮也被强行插入，随后AI生产快捷路径不再按人类条件拦截。
- 已修既有recalc_base_plan owner，只记录GAPLUG，不加阵营名单。旧回归错误期望被替换，修复前真实失败[8,7]而非[8]；
  修复后NP/stock planner各11项通过，实际Return Dubai正式load_session+三阵营计划1项通过：盟军仍有GTGCAN，苏联/尤里无GTGCAN，三者均无GTGCAN2。
  NP/stock基地选择各15项、NP MCV部署25项通过；MCV六个ignored不涉及本次规划后备分支未重跑。一次只读critic无可操作发现。
- 没有新增RNG调用、timer/Detach或保存字段；较短计划会自然改变原有炼厂/防御插入抽样范围，不伪装成原有错误计划的逐帧重放。
  原存档中已保留的错误计划和已经造好的巨炮不自动删除；此修复作用于后续新生成的计划，新局验证。
- 2026-10-28续验实际建造：正式AY Battle/XMP03T4新局，一人闲置、三阵营困难AI，从原始MCV、正常资金和经济启动，unit_count=0；仅在测试内关闭新AITrigger以隔离进攻团队，不注入科技/基地/钱、不写计划/选择/队列/进度或地图落点。盟军tick15378由实际工厂构造GTGCAN，逐帧观察剩余应付款下降；同一对象tick16459正式退出工厂、取得地图占格，随后退出Construction。三家最终各有25/24/27个活体落地建筑，排除非盟军根本没发展造成的空通过。
- 全段普通帧检查苏联/尤里的计划、House building_choice和实际工厂，无GTGCAN；所有AI均无GTGCAN2生产。盟军第二座巨炮是合法独立生产，首跑错误要求所有生产共用首座ID被反例推翻；改为每个实际工厂对象独立跟踪付款与落地身份，保留完整施工结束门槛，而非禁止后续建造或放宽为计划出现即通过。
- 最终三AI实际场景1项、Return Dubai正式规则三阵营计划1项、NP planner11项和ai_base_building::tests12项通过；未重跑既有perimeter/stock覆盖，无生产修改/新critic。没有新增RNG、timer/Detach或保存字段，所有发展/生产/落地仍走现有owners；未启动GUI、未动固定玩家版/MOD/原始存档。结论限未占领异阵营科技的这次新局、关闭进攻触发的基地组件；不是完整多AI战斗/长局、Return Dubai发展或真人GUI验收，不关闭A10裂缝/超级裂缝等独立项。

### F09：已接生产链，当前两Battle武器闭包无消费者

- NPAE WarheadType ctor75CF25清零+129；ReadINI75D3F1→C2E853→C2E90D..927按当前bool读取精确键DoesntAffectEnemies（字面值BA4C63），
  早于IsFlatExplosion。现由既有逐层registry保留，投影到WarheadType，覆盖空值/坏值回退、注册表保存恢复及NP配置hash。
  此新增必需registry字段改变保存布局；旧玩家构建和原始存档不强读，后续累计测试版用新局。
- GetFireError6FC3FE→C2E94E/C2EA23：有Techno目标才按开火者House→目标House检查，不是反向盟友位；非盟友返回5。
  新门接既有FireError owner，目标Cell/Object不因该标志拒绝，自己/己方允许；不把Phobos AffectsEnemies取反作别名。
- TechnoReceiveDamage701CD7→C2D62A：在AffectsAllies之后、Psychedelic之前，实际攻击者非空且非盟友才清伤害并返回0。
  现接既有late-gates，保留readiness写入顺序；ignoreDefenses不跳过这道门，只有SourceHouse而无攻击者不触发。
  连带修正同一owner原有AffectsAllies701CB3误用归属House的问题：两者共用真实攻击者House；Psychedelic继续单独反向查询SourceHouse。
- 三项定向回归覆盖逐层/精确大小写/保存/hash、实际FireSubject的敌我/自己/Cell/单向盟友、实际receiver的攻击者与归属House冲突/空攻击者。
  开火测试修复前真实失败（Ok而非Illegal）；修复后全部通过。最终NP reader100、warhead9、FireError10、damage16、Unit9、Object6、wave26通过；
  stock reader97、FireError8、damage16、Unit9、Object6通过。两个错误的Object/receiver过滤曾得0项，Object已按正确模块补跑6项，receiver_health本身没有独立测试，不列覆盖。
- --profile fast真实AYMOD加载及既有TTNK Attack→Stop/Guard→内存保存恢复→继续连发1项显式ignored通过；未启动GUI、修改MOD或原存档、重建/替换固定玩家版。
  一次只读critic无可操作发现。未新增RNG抽样、timer或Detach调用；保存测试是typed registry与现有实际场景，不冒称TankSnapshot完整实战存读档。
- 实际Battle/Return Dubai生产accessor确认TankSnapshot=yes；规则根无include，LANGRULE缺省，MPBattleMD/地图无该键覆盖。
  物理TankMakeupKit/CRMakeupKit段引用此弹头；最新正式类型分配闭包已证实两者均未分配，且全武器族没有TankSnapshot消费者，详见当前F09节；并非仅由MGTK/FV当前槽推断。
  因而本项不能当作原局弱AI原因或所有模式/地图实际可达的证明。伤害单向盟友/治疗输入、完整实战和GUI边界仍未另验。

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

### 机制证据：1v1持续进攻已观察，继续区分集结与行军（2026-10-26）

- 本批无玩法/MOD/AI权重改动。正式Return Dubai/Battle，美国被动人类对苏联困难，默认选项、
  seed=0x00C0FFEE，分别观察9000与18000次调用；两报告到tick9000的House/工厂/编队/开火计数前缀完全相同。
  该启动出生点为Player(102,180)、Computer1(118,40)，不是原玩家存档的出生位置或玩家操作重放。
- 电脑资金410000→448508，tick9000时跟踪建筑计数15；不能将全部实体/已付款limbo产品当作已在地图上的建筑。
  Team3（12名E2/SHK/FLAKT）在4500/6000/7500采样为53，9000已进入action0攻击玩家MTNK2201，
  成员在(108..110,155..157)，持实际Attack与导航，不能把此前四千余帧的集结/行军认定为永久停滞。
  Team0脚本05C77DBC-G是来回基地建筑/等待的防御巡逻，不用其未向玩家推进证明出击损坏。
- 到tick10500，电脑有18次开火（17次移动单位对可解析非盟友、16次formed成员）；
  tick11518人类战败，工具以outcome_ready终止，累计211/205/202次。Team21由53推进到脚本
  0CA1FDBC-G的cursor2/action3。由此证明本1v1能成队、移动并持续接敌，不证明原局弱攻势全部解决，
  也不替代多AI长局、主动玩家抗压和失厂/矿枯恢复验收。
- 新的V01/A06关联现象：9000帧House powered_unit_centers=0、没有在场NARADR；
  Team21两辆和Team65四辆DRON为deactivated=true，仍有Move/NavCom及远离基地的位置。
  10500时中心计数为1，末次已有NARADR4392。不能靠提前建雷达或改兵种/权重掩盖原生失联Drive可移动；
  独立供电/禁移动的作者预期与原补丁行为冲突仍需既定政策批准，不能静默裁决。
- 独立只读复核实际NPAE action53的6EF943..6EF98A、54的6EFBEE..6EFC3A：
  FNPC15参数与现有共享team_assign_gather_cell一致，zone=-1、MovementZone=Normal、3x3、
  allowBridge=1，其余限制为0；原seed先复制再清零，最终targetCell指针确为(0,0)，不是seed。
  现有None/按帧候选选择不能改为“最近seed”。只排除调用参数错配，地形来源/FNPC最终点仍待闭环。
- 独立V10原生调查发现明确后续缺口：实际68C077..7E移除“lab数不能超过choices”门禁；
  68C0F1→C2E5E8把候选标记置0，68C11B..149每次按完整total抽样并调用C2E5FC，
  从所抽位置环扫未用项；用尽后清零标记并返回本次起点。当前secret_lab.rs仍为stock缩短范围算法，
  npatch尚无分支。不能简单替换为均匀洗牌；应沿现有owner补此算法与RNG/超过候选数边界回归。
  此处为后续机制调查，不把原生指令读取说成已适配或已运行验证。
### 机制证据：多AI诊断与结局边界（2026-10-26）

- 本批只完善既有telemetry-serve诊断与文档，未修改MOD配置、AI权重、兵种数值或玩法。
  新增trace-skirmish模式复用正式Battle启动/load_session，显式指定1–7家困难电脑与重复阵营；
  记录实际选项、seed、map/rules hash、完整House/工厂/编队及末次实体。用法和边界见
  [telemetry README](../tools/telemetry-dashboard/README.md#trace-a-fresh-ai-skirmish)。
- 实际Return Dubai，美国被动人类对美国/苏联/尤里/第二家苏联，seed=0x00C0FFEE，默认Battle选项：
  6000逻辑帧时四家电脑已有10/11/7/8栋建筑，3/2/7/3支formed队，资金均高于40万；
  美国电脑累计70次移动单位对可解析非盟友开火，均观察到formed成员。其余三家尚无此类开火记录。
  这排除该运行中“完全不发展/完全不能编队”，不证明整体进攻强度，也不代替原1v1玩家复盘。
- 同时刻苏联两家有队停在53/54集结动作，但尤里已有4队进入action0攻击，另有守卫/等待动作；
  不能把所有formed队、守卫队或单次光标不动都当作出击卡死。继续优先调查实际集结/出击链，
  已证海军184的跨水区集结仍待原生地形来源/FNPC闭环，不擅加可达fallback或禁止建队。
- 旧12000次调用报告实际tick在6621停止，人类已is_defeated/has_lost；7500以后停产不是新AI缺陷。
  工具现遇正常outcome_wait/ready即停止，不确认/重置结局。复跑同seed实际在第6622次调用、tick6621
  以outcome_ready结束，保留末次完整实体；没有继续推进或用假状态保活。
- fast+npatch诊断构建通过；模式互斥、无trace时AI参数拒绝、24001上限拒绝通过。
  旧存档布局门禁保持不变。开火归属在帧后观察，可能漏计/重归属，不冒称权威开火历史。
- 玩家版仍为aymod-playtest-20261026-explosion-crate；manifest源e3e05ec46已核对包含372af3d64颜色修复。
  入口D:/Workshop/vera20k/target/AYMOD-test-20261026-explosion-crate.cmd；新大厅按显示蓝色色块选择，
  AY当前蓝色为MPC05（第5色），不是旧stock第3色。尚需玩家GUI确认，不自动启动游戏。
### 机制证据：V08玩家BFRT与磁暴充能对照通过（2026-10-26）

- 当前任务树/分支不变，本批基于`9b4ca8c96`；仅新增两项真实AY场景功能测试和海军诊断断言，
  没有玩法、MOD数值、AI权重或被拒绝的F11跨模块接线改动。无运行中Cargo、PR/推送或新玩家构建。
- BFRT实际配置CanRetaliate=no；Unit Approach741603..741635在human+passive50C时跳过OmniCrusher，
  不是新增AutoCrush全局开关或彻底取消Crusher。测试通过原生payload边界提供敌MTNK的攻击意图
  （明确组件边界，不伪称自主AI），之后普通帧实际受击/被动扫描开火90帧仍无XY移动；
  显式玩家Attack同一可压目标后真实移动且passive标志清除，保留手动进攻/碾压能力。
  初始天启对照有OmniCrushResistant=yes，已换灰熊并断言其无此抗性；未改防碾压规则来凑结果。
  逐帧记录受击，避免BFRT自愈使末帧HP掩盖命中。新增实际资源测试1项通过。
- TTNK/TESLA真实同House相邻构造，使用正常GAPOWR和House电力更新；普通/精英各90普通帧
  不进charger列表、不持自动充能标志，typed TTNK确无Secondary。随后同一有电TESLA旁SHK
  自主Guard选中并实际发射AssaultBolt，列表包含SHK不含TTNK、overpowered=true且不扣线圈HP。
  新增实际资源测试1项通过；这沿用既有secondary ElectricAssault/BuildingCharge owners，
  不是新增NP供电状态，亦不是V01三类无人单位的中心供电验收。
- BFRT验证限定玩家空载场景、普通攻击与近距可压目标，不认证电脑BFRT、六载员复杂目标、全退档/
  保存/换主。TTNK验证自动充能否定及SHK正对照，不认证所有强制友军攻击、低电/三步兵重叠或精英线圈。
  V08保留这些边界，不用两个场景关闭整个MOD。
- 独立只读A04调查：loose Return Dubai Basic的GameMode=standard,navalwar是菜单标签，
  Battle仍明确加载MPBattleMD.ini；IgnoreGlobalAITriggers=no已由现有reader/触发门消费。
  没有据此猜出一个缺失的Basic Naval/NavalTargeting开关。实际NPAE TeamType derive6F1FA0/6F1FE7
  遇Naval且Passengers=0成员清base-zone约束，AITrigger gate41FEEE在false时直接返回true。
  正式地图现有舰队FFFFFFFE-G的base_zone_relation_enforced=false、combined_movement_zone=Water
  新增断言已通过（原真实地图/原生连通/Foot门禁测试1项重跑通过）。不能新增“同水区才准造海军”来冒充修复。
- 下一步仍按PLAN继续功能/AI边界：真正海军集结的原生地形来源/FNPC未闭环，F11接线批准与V01
  作者设计/原生供电政策仍开放；不将自动goal续接当批准，也不改目标完成定义。

### 已有覆盖：V07自主缠船已验，电脑输入探针误报已排除（2026-10-26）

- 任务树/分支不变，本批基于`be390424d`；收敛既有实际MOD场景测试并纠正诊断结论，
  未改玩法、MOD配置、AI权重或被拒绝的F11跨模块接线。无运行中Cargo、PR/推送或新玩家版。
- 正式加载实际AYMOD、Return Dubai、Battle/1v1困难，选择合法空水面与邻近Clear岸地；
  正常出生电脑SQD、玩家DEST，由电脑真实Guard选敌/开火和逐帧实际SquidGrab建立存活寄生及grapple动画，
  再由岸上的己方DESO普通Attack触发真实RadBeamWeapon，而非直接调用伤害/Detach或杀死乌贼。
- typed RadBeamWarhead同时Radiation/Sonic=true；实际NPAE FootReceiveDamage4D7345..4D7364
  的Sonic前缀在普通Techno伤害/免疫之前调用Parasite ExitUnit62A4A0。当前生产前缀已接；
  因此此功能由原生Sonic语义和MOD弹头配置构成，不是缺一个Radiation专用NP状态。
- 新增aymod_radiation_beam_releases_a_live_squid_from_its_ship最终候选显式实际资源1项通过：
  捕获两种真实开火事件，SQD/DEST均存活，源脱离limbo且双方寄生关联清空，grapple引用清空，
  旧动画对象在后续普通帧回收，host麻痹结束并对空闲可达水面Move产生真实XY位移。
  位置判断不使用船体摇摆/渲染偏移代替移动，不用死亡回调冒充剥离。
- 纠正parasite.rs旧残余：当前app/input/context_order.rs已通过选中武器warhead.sonic与
  FootParasite::is_infested允许己方被寄生Foot的攻击入口；本轮普通友军Attack已通过，
  但未做Windows鼠标/光标或GPU画面验收，不称GUI通过。
- 上轮电脑host探针仍无位移的根因已查明：它把玩家Move输入排入普通Computer1的DoList，
  但实际NPAE Execute_DoList64C4FB..64C50D仅派发IsHuman或PlayerControl House；
  生产WorldCommandHost::house_is_eligible→HouseState::event_dispatch_eligible保持此门禁。
  原Move根本未执行，不能据此断言AI覆盖命令、麻痹未清或海军运动卡死；该探针移出A04故障候选。
  最初一处目的格被释放乌贼占用亦是夹具问题，但不能用占格独自解释后续空格版本。
- 同一门禁意味着原测试排给电脑SQD的Attack也未执行；已移除无效输入并断言正式载入的电脑
  House确实不具玩家事件资格。最终实际AY测试重新通过，证明这一个邻近敌舰场景中SQD能自行选敌、
  使用真实SquidGrab缠船，再由己方DESO救出玩家DEST；不是手动把电脑伪装成玩家来凑通过。
  既有synchronized_tail_admits_human_or_player_controlled_houses_only定向回归亦通过，
  覆盖Human/Computer/PlayerControl/未注册House；不放宽队列门禁。
- 真正电脑编队的任务命令直接经现有Team/mission/destination owners，并非上述玩家输入队列。
  AI控制下的恢复仍需走真实Team/Hunt或明确的组件payload边界验证；本次不声称全部海军/AI恢复正常。
  先前本机诊断日志保留在忽略target目录，不将无效探针结果提交为原生golden或已知引擎缺陷。
- V07尚未关闭辐射场/部署普通精英武器、乌贼自主选敌、释放后再次捕获、保存/换主和多AI等边界。
  F11接线批准和V01作者设计/原生政策问题仍开放；下一安全动作按PLAN继续独立调查/测试，
  不把自动goal续接当作跨模块批准，不缩小整个AYMOD可玩的目标。

### 已有覆盖：V06精英盖特首发由现有配置实现（2026-10-26）

- 任务树/分支不变，本批基于`e85e073df`；仅新增真实MOD headless测试和台账，无游戏逻辑、
  MOD数值或跨模块光照接线改动。无运行中Cargo/PR/推送；已有玩家构建保留，不需要换版本。
  F11跨模块接线仍等待知情用户批准，不能将自动goal续接当作这项明确批准或交给agent绕过。
- 重读作者更新日志170601（Windows ANSI解码）：写的是精英机炮“最高射速”、坦克“中级射速”，
  不是把Techno+140初始化为高档。根配置YTNK EliteWeapon1=AGGattlingE、
  YAGGUN EliteWeapon1=AGGattCannE；两者仍有普通Stage/EliteStage阈值。
- 实读实际NPAE What_Weapon_Should_I_Use6F3428..6F3474：Gattling按当前+140 stage选择2*s/2*s+1；
  该已达分支没有额外精英跳stage，精英武器替换由既有GetWeapon/rank owner完成。
  不从作者短句推定另缺NP状态或修改平衡值。
- 新增aymod_elite_gattling_first_shots_use_authored_faster_weapons，正式加载实际AY规则/模式/地图，
  分别正常出生YTNK/YAGGUN与APOC，以真实GAPOWR和House电力更新支持机炮；
  fixture边界仅将射手rank设elite。通过普通Attack命令和逐帧fire_events捕获首次实际发射，
  武器分别为AGGattlingE/AGGattCannE，首发后stage仍0；typed ROF分别等于普通中/高档武器，
  且快于各自普通第一槽。NP显式实际资源测试1项通过（覆盖两条单位/建筑链）。
- 这证明精英首发射速不是当前纯缺失；不认证自然击杀晋升回调、完整射击间隔/所有ROF倍率、
  对空槽、Stop/Guard退档、保存换主或GPU画面。V06保留这些边界，下一功能核查按PLAN继续，
  不因一个已有功能通过而缩小“完整AYMOD可玩”的目标。
- 2026-10-28已另补两类地面实杀晋升/热存读/退档/续战，最新覆盖与未验边界见当前检查点及V06行。

### 已完成前置：F11来源状态已验证，跨模块接线未执行（2026-10-26）

- 当前任务树/分支不变，本批基于`359b83b58`。仅ScenarioLightingState新增private三来源owner，
  对应BA91E0/E4/EC，保留SWType ID与同一不可变Rules配置快照；新局None，效果结束不清来源。
  map normal/ion/dominator未改写；NP profile/rate/RGB/top-Level查询和保存/hash已由这一个owner承担。
- NP来源存在时RGB按53C3A4 wrapping*1000再signed/100；NP full relight Level跳过stock Domi的NukeLevel。
  Lighting=false不抹掉profile/rate；未绑定来源仍走既有map路径，这是direct-native夹具和未证明null域
  的安全fallback，不宣称原生会安全处理空SW指针。当前尚无生产发射调用新来源setter。
- 验证：新owner两项NP生产reader→state/save/hash通过；既有lighting默认/计时/hash两项NP和无NP均通过。
  新增临时`npatch_dominator_launch_remembers_type_even_when_not_charged`走真实未充能launch，
  在“Start准入前应记住类型”断言上按预期失败，证明尚缺6CCDB1生产接线，不是只缺GPU测试。
  测试补丁保存在忽略的`D:/Workshop/vera20k/target/aymod-save-analysis/f11-before-admission-regression.patch`；
  临时红测已移出正常源码，原文件git diff为空；后续实现前先恢复该回归，不能把它算作通过。
- 批量跨发射/事件/呈现模块的修改请求被安全审批拒绝，命令未执行；不得拆命令或改由agent绕过。
  本轮仅完成更窄的单文件状态验证与失败前回归。下一步影响超武发射、全局光照和绘制消费者，
  需知情用户明确批准该跨模块变更后再实施；仍需三条优先级/disabled回落、status5/rate/保存恢复、
  实际AY逐帧事件/GPU与完整链critic。没有新玩家构建/PR/推送，无运行中Cargo，F11和整体目标保持开放。
- 海军地形来源/FNPC、无人单位供电设计政策和其余PLAN项照旧保留；不因本次前置或授权门禁缩小目标。

### 已完成前置：F11光照规则读取，完整效果未完成（2026-10-26）

- 工作树`D:/Workshop/vera20k-aymod-audit`、`feature/aymod-integration-resume`，本批基于`62e3cd133`；
  无PR/推送，无运行中Cargo。整体PLAN不变，海军地形类别/FNPC、V01供电政策与其余F/V/A项仍开放。
- 完成NpatchSuperWeaponLighting的逐物理层七整数/末尾Lighting读取、共享首次初始化门、
  注册表保存恢复、RuleSet投影和配置hash。默认值来源与Scenario现有三组默认函数共用；
  不从最终地图取seed、不因Lighting=no跳过读取、不把Type3先初始化后改Type7误当首次Domi。
  Type4的反常门保留。新增保留字段会改变保存layout，仍按原门禁拒绝旧布局，不伪造兼容。
- 2项生产回归覆盖root/mode/map与迟注册、坏Type保留、exact-case/原生整数、disabled读取、
  Type3/4→7、保存/hash；NP NativeProcessing98、SuperWeaponType4、ScenarioLighting2通过；
  无NP新增2与SuperWeaponType4通过。无NP新字段保持constructor零且不消费扩展键。
- 原生复读纠正两个容易误写的细节：Storm第七值实际跨读核爆Ambient，默认200；
  55B3D6的rate分支没有Storm，包含两个核爆阶段与TAction127屏幕效果（不是Chronosphere SW）。
  规则实现/必要运行时边界见[npatch.md](npatch.md#superweapon-lighting-reader-runtime-integration-still-open)。
- 此批没有把profile接到launch/ambient/cell/palette，所以F11仍不兼容完成，不构建新玩家版。
  下一步在既有ScenarioLightingState/light_sources owners接三份最近SW身份及其immutable配置快照，
  不改写map profile；分别处理global/constructor/cell优先级、Lighting=false回落、status5、rate与保存恢复，
  再用真实AY普通发射/逐帧lighting_events及既有TMP GPU探针验收、做完整链只读critic。
- 本轮未修改Python工具；上一检查点记录的Windows symlink权限和hashbrown特性测试异常仍未解决，
  不将Rust局部通过称为Python全套或整个MOD通过。已有保留玩家构建和原始存档均未改。

### 已定位批次：A04原生连通复算排除算法分区差异（2026-10-26）

- 工作树/分支仍为`D:/Workshop/vera20k-aymod-audit`、`feature/aymod-integration-resume`；
  本批基于`65e821929`，只修改诊断/测试/说明，不改游戏玩法/保存布局、MOD配置或保留玩家构建。
  没有PR/推送、运行中的Cargo或待完成命令；无需玩法critic（无行为改动）。
- 复用既有bridge_test_evidence::navigation_snapshot，不复制导航状态导出器；仅将该test-support
  只读函数开放给本crate的海军测试。实际地图的Size100,110、210x210类别/高度平面和空桥记录
  通过可选VERA20K_AYMOD_NAVIGATION_EXPORT写入忽略的本机target，不提交完整平面或玩家存档。
- 扩展原有tools.spatial_oracle.bridge_connectivity，独立较大node/height内存仍调用既有native
  初始化/分配钩子。实际NPAE174 Map56C510、Flood56CB90、pair56C6CB及13行producer执行到返回；
  未供应任何连通结果。Water是原生row10，不是Infantry row7；最终复算99,149→4，108,55→2，109,55→2。
- 仅保留三点输入/输出的aymod_naval_connectivity.json及复现命令说明；真实AY测试现消费这些
  原生结果并通过，既有live Foot门禁/集结生成器诊断保留。原工具9条旧native用例--check通过。
  此结论排除“相同保留类别/高度输入下，Rust连通算法错误拆水域”；不证明原生地图加载/Cell
  Recalc生成相同平面，不证明原生FNPC最终点或完整Foot任务，不能将原战术/地图问题提前定案。
- Python新工具2项通过，真实AY Rust测试1项通过。工具全套实际运行936项：2failure/6error/49skip；
  逐项查明6error均在未改的cache/seed测试创建symlink时触发Windows1314权限错误；
  dependency一致性失败的实际差异为hashbrown0.16.1（workspace额外allocator-api2/default/equivalent/raw-entry），
  本批未改Cargo清单、runner或这些测试。另一failure是本任务旧交接中的个人home路径，已改成~，
  no_home_paths3项+新增工具2项重跑通过。剩余7项仍未通过，不声称Python全套通过，也不在本链改依赖。
- 下一步：继续核对原生Cell/Recalc地形类别来源和FNPC最终点；这是海军184剩余证据边界。
  AI整体、多House/长局及其他F/V项照PLAN继续；原生/作者设计供电政策问题仍未获答复，不能擅自定案。

### 已定位批次：A04海军184卡在跨水域集结（2026-10-26）

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
| F02 / P1 / 建筑受击/策略EMP、盟友探测与红血重隐已验，健康建筑GPU门禁已验 | 隐形箱已授予实例能力，接入共享隐形AI及持久化 | CellPickup48294F/481D3A；原生产半径/敌我/未占格/重复/保存/hash与换主/Unlimbo及critic修正保留 | 实际MTNK/E1、GAPOWR与ORCA、正常ALLY/活DOG双向接触及HP1重隐已验；新增GAPOWR/GAROBO全隐形受击state3存读重隐，以及既有策略EMP在powered门下的状态/到期恢复。上游EMP政策不宣称#174等价；受损/EMP FX绘制、启用自然粒子、多AI选敌和完整生命周期仍开放。健康建筑两格式GPU门禁已验，既有字段改变保存布局 |
| F03 / P1 / 已接入并通过实际场景，画面待验 | 爆炸箱直接C4与5次径向爆炸、动画/光照和死亡后续已接通 | 482565..482719；3项新生产回归含实际AY放箱/Move/真实500幅度和5个帧输出光照请求。同步后果通知不丢失，扩展真实桥梁回归并验证第二帧不重复；critic未发现具体缺陷 | 最终GPU/GUI画面、复杂Dummy重入及中断帧真实通知未逐项实测；不等于所有箱子或所有地图/难度通过 |
| F04 / P1 / 已实现，功能验收通过 | 驻军保护/穿透：通用建筑收伤前置钩子；正等级尾序单杀、负等级尾序清空，严格大于保护值 | 两个有符号byte逐物理层读取；WH穿透复用原Fire的+14C字节，不建立竞争状态。`src/sim/passenger/garrison_eject.rs`复用声音、RecordKill/UnInit owner；`src/sim/combat/world_receiver.rs`在自伤门和联系人快照之后接入 | 已通过真实AYMOD狙击手攻击普通/保护驻军、鲍里斯召唤BPLN/Maverick3清空保护驻军；生化炉Cargo不属于此钩子，坦克掩体保留独立链接/旧PenetratesBunker门。优化场景、普通游戏构建和独立只读critic均已完成；不冒称GUI或全地图模式认证 |
| F05 / P1 / 已实现，MTNK运动存读/实射已验，画面待验 | 独立炮塔速度：已按NPAE174分离Unit炮塔与车体，影响跟踪、射击时机 | MTNK、FV、SREF、ROBO、HTK、HTNK、APOC、HORV、HARV、TTNK、TTNK2、LTNK、YTNK、TELE 共14处物理用法；`src/rules/native_processing.rs`保留byte，`ObjectType::unit_turret_rot`为共享速率入口；`src/sim/world/world_spawn/construction.rs`初始化、`src/sim/world/unit_post.rs`持续瞄准消费 | 已核#174 reader/构造消费者及原地Attack；实际MTNK追横移SMCV、运动分向/移动开火、独立速率/运动存读续战实伤已验；其他类型/特殊转角及实际画面仍待验，沿用旋转owner，不拟合速度 |
| F06 / P1 / 已实现，部分场景待验 | ORCA的AACombat现保留空中攻击目标，不改变投射物AA准入 | AircraftType逐层读取/保存/hash；既有`techno_ai::drop_airborne_target_step`按NPAE174 414D36/C27C60跳过高空清目标，仍检查隐形感知。实际AY ORCA普通Attack已伤害仍在空中的ZEP及地面MTNK；精英远距追击移动ZEP在双方运动/开火前热存读后保留目标并实际命中 | 精英远距移动目标及热存读已验；玩家GUI、持续AI战术、返航重装/多目标和实际晋升过程仍待验；不将此链扩大为所有飞机行为通过 |
| F07 / P1 / 已有实现，部分待验 | 谭雅海军副武器由原有NavalTargeting=4选择，不是两个新Weapon flags | 实际TANY Secondary=Sapper、EliteSecondary=SapperE；`combat_weapon::select_naval_targeting_weapon`匹配6F3820/6F38FC..6F3915。Return Dubai正式加载后，普通/精英Attack均实际发射对应武器并伤害HYD | 已检查的NPAE174 Weapon reader不读AntiNaval/AntiUnderwater，见C04。已验普通SUB自然隐形/未探测拒绝、游近探测及两次存读/Sapper命中；Hover LCRF/Organic DLPH实际被主武器零倍率提前回退Sapper，海军leaf仍返回0。精英潜艇、探测丢失后再潜/续战及GUI仍待验，不扩大为所有海军行为通过 |
| F08 / P2 / 普通完整投弹/两次存读已验，画面待验 | IsFlatExplosion与CausesDelayKill顺序别名，接既有建筑PostMortem，不另造平面伤害 | C2EF94/75D443共同写WH+130，701E92消费；逐层保留/保存/hash及3项回归通过。实际AY ZEP primary→BlimpHE直接命中CAMISC02后引爆，debug/fast均通过；critic无具体缺陷 | 普通ZEP完整飞行/放弹→原下落弹存读命中→PostMortem引信再次存读→自然销毁通过。精英/其他别名弹头、长连投/多目标、GUI及全部既有引爆算术仍未逐项认证 |
| F09 / P2 / 已接入，当前两Battle武器闭包无消费者 | `DoesntAffectEnemies` 不只是伤害过滤，还会拒绝对敌Techno开火；有攻击者时才按其House拒伤 | 实际Battle/Return Dubai生产accessor确认TankSnapshot=yes。NP75D3F1→C2E853→C2E90D按当前bool读WH+129；ctor75CED0/75CF25清零。FireError6FC3FE→C2E94E/C2EA23拒非盟友Techno返回5；ReceiveDamage701CD7→C2D62A在攻击者存在且非盟友时清伤害并返回0 | 已接逐层registry/保存/hash、FireError及伤害late-gates；敌我/单向盟友/攻击者与SourceHouse冲突/无攻击者回归通过，不与Phobos AffectsEnemies混用。普通Battle/XMP03T4及Return Dubai全已分配武器族无此标志消费者，TankMakeupKit/CRMakeupKit均未分配；其他模式/地图及直接Warhead路径不据此全局排除，不能据此解释原局弱攻势 |
| F10 / P2 / 已实现、玩家待验 | 自定义电弧已从保留reader经出生参数接入GPU绘制 | 实际TankBolt/TankBoltE生产开火及RGB565 GPU读回通过；默认/禁用/逐层开启、存读/hash与无NP隔离通过。TeslaFragmentT保留颜色但不强开IsElectricBolt | 玩家新局观察普通/精英磁暴坦克电弧；不把EMP逻辑正常代替配色验收 |
| F11 / P2 / reader已接、运行时仍开放 | 心灵终结器独立光照：超武 section 的 NP 读取已保留，覆盖效果尚未接通，不等于 stock 全局光照缺失 | `[PsychicDominatorSpecial] Lighting / LightRed / LightGreen / LightBlue / LightAmbient / LightGround / LightLevel`；`src/rules/superweapon_type.rs`；`src/sim/superweapon/psychic_dominator.rs:8` 当前使用 map 的 Dominator profile | #174 启动、叠加/互斥、渐变和复原；实际输出验证，覆盖取消/结束/存读档，不污染下一次超武 |
| F12 / P2 | 扩展遭遇战 UI：8色大厅/战场错配本轮已修；科技等级滑条与完整16色界面仍开放 | `[UISettings] ShowTechLevelSlider / UseDropDownColors`、`[Colors] MPC01..MPC16`；`src/ui/skirmish_shell/layout.rs:159` 只有速度/金钱/单位数滑条；`src/skirmish_launch.rs:18` 颜色数8；`src/app/frontend/skirmish_shell_render/controls.rs:353` 固定8色 | 先核 #174 颜色表与选择政策，再接配置/选项/启动数据；30槽目标不等于已支持16色，验证颜色占用、同色策略和人/AI辨识 |
| F13 / P2 / 已实现，待画面验收 | NPatch超五乘员开火端口循环：已修第六名原先错误回退零偏移的问题 | NPAE174 GetFLH `0x6F3AFB → 0xC2E196`按`(k-1)%5`循环，ART reader `0x715FA4..0x716005`仍只读0..4；固定ART5/6是本链不消费的数据。`ArtEntry::alternate_fire_flh`为唯一共享owner | 六人实际Enter后的生产坐标已验；不扩大数组。下车重上/移动转身的实际画面仍待验，不把坐标测试称GPU输出认证 |

F06–F09已按各行接入对应扩展生产链，仍保留各自场景边界；F11运行时接线未完成。**适用原生效果尚未全部证明**，
不能直接依据键名写实现。F01–F03、F13 是首版新增的明确生产路径缺口；F13的五槽容量本身正确，缺的是NPAE循环。

## 4. 已有实现，仍需核对的功能链

| ID / 优先级 | 类别与当前边界 | 下一次验收 |
| --- | --- | --- |
| V01 / P1 / 原生与作者设计冲突仍开放 | 三组供电已有owner；普通地面Drive的provider/EMP分离与恢复已修，原生共享House计数保留 | 原生允许失联Drive被AI重新下令移动；作者说明却要求三组指定建筑失去支持即瘫痪。不能把此差异归因于漏读PoweredUnit或把EMP修复标成蜘蛛已修；独立供电/禁移动须明确兼容策略 |
| V02 / P2 / 入口/Cargo/EMP任务保留已修，联盟/缺电/Off/EMP治疗存读已验 | GAROBO/NARADR Hospital保留患者owner，普通步兵共用医院动作，Ammo=-1准入；Passengers=0仍可有Cargo；建筑EMP不再误调Deactivate重置治疗 | 手动Off拒新、已有患者继续；自然缺电/EMP下原有与新患者可治疗出院，正式存读/预约清理、独立House电网及恢复已验。EMP原生生命周期/雷达顺序与付款恢复见对应节；治疗中解盟/新申请拒绝、同帧双人预约等待及依次治满/存读已验；完整TTNK医院实战、步行途中解盟/更多争抢取消、占领旧患者其他恢复及自主多AI仍开放；不是科技医院全军被动自愈 |
| V03 / P1 / 普通连发链已验证，边界开放 | 原有BurstDelay读取不足以适配NP：GetROF返回完整ROF和短间隔，FireAt复用+2F8，GetFireError分两时钟，Assign_Target不再因空目标清burst；精确开关UseROFAsBurstDelays取代NoAutoFire | 双计时已接共享内核/FireAt/FireError/目标清理/保存/hash，原红测及原生六例+writer三路通过。用户批准后，NP乘员原生三例、实际TTNK完整开火/Stop/Guard/保存续战均通过，既有critic缺口已补；空武器/建筑ammo早返短值域仍保留明确fallback；两图typed库存仅无武器/不可驻军CAAIRP/CAPOWR具Ammo5，正常Occupier普通/精英均有武器，暂未找到常规FireAt消费者；游标/回调等运行时边界仍开放，不宣称所有武器等价。本轮未交付新玩家版。NASAM单数BurstDelay仍不消费，见C05 |
| V04 / P1 / 移动/换主死亡/运输存读与AI准入已验，其余开放 | 实际DOG移动带入/带出范围，电脑GAPILL共享传感器获取全隐形TANY；覆盖内外存读后无丢失/残留；普通Guard实际开火/自然重新隐形后的拒绝/重发现后Stop再获取通过 | 范围按实际typed规则，不硬编码旧说明；不要求强制转火。IFV实际控制/归还/死亡、登离LCRF/两次存读及计数/AI准入已验；卸载缺失sensor恢复已修共享Unlimbo。移动目标自身、多House/联盟重叠、空艇/精英潜艇/幻影等其他路径及画面仍待验 |
| V05 / P1 / 驻军双弹/射手死亡存读已验，剩余Cargo死亡顺序已修，生命周期仍开放 | 驻军按击杀时活索引减一并回绕，成本保留受害House例外；飞机保留普通奖励并额外训练母体，rank采样同步弱子机、再生按Trainable继承raw | 出驻/替换、末位死亡/补员、楼毁活体弹出、双弹与第二射手死亡后的名单XP已验。ScenarioInit内非尾删除/存读后真实交替开火、cursor0单人自然续火/热存读已验；count1/cursor1的普通GetWeapon→Cant→drop、180帧含存读不归零/不发射及补员恢复已验；实际SNIPE两次尾删/游标1/经验/两次存读及原弹归属已验，正等级不应随机删非尾；其他名单损伤来源和原生循环分支仍待核。剩余Cargo在SpawnSurvivors后清理；其他子机家族与母体连续晋升/换主仍待验，不关闭全V05 |
| V06 / P1 / 地空晋升/热存读退档已验，其余开放 | 作者170601说的是“最高/中级射速”，由现有EliteWeapon1配置实现，不要求内部stage跳档 | 实际AY YTNK/YAGGUN地面及自然起飞JUMPJET对空实杀晋升、热stage存读、Stop/Guard退档和精英对应槽续射已验；对空目标是受控human House组件，不是AI自主飞行。既有stage0首发由配置实现；YTNK/YAGGUN热状态地空换主存读/新主人续射已验；机炮保存额外旧主AI后的状态，普通human House的实际地空reload及运行中双计时存读已验；实际NATBNK正常进出/非1倍率地空射击及驻留/释放存读已验；碉堡实弹毁坏/正常出售四路释放存读、Move和续射已验；V12特殊回调、碉堡Temporal/争抢、其他倍率/全档精确节奏及GPU仍待验，不加NP跳档 |
| V07 / P1 / 辐射与实际海豚解缠、宿主换主存读/原波GPU已验，生命周期仍开放 | AY Sonic沿既有Foot伤害前缀解除寄生，无须另造NP剥离状态 | 自主缠船/缠绕中存读→辐射束救船/再次自主捕获/再次解除/释放后存读移动；普通/精英DESO部署及独立场救船通过。实际DLPH救DEST/宿主换主两例及同一原波的两格式GPU、保存像素一致/退役零输出已验。寄生源第三House/同宿主House换主后的解缠、两次存读与GPU已验；源自然瘫痪到期/新主人普通移动/再存读已验；多AI、完整抗压/长期场衰减及整个App/玩家原局仍待验 |
| V08 / P1 / 玩家BFRT/TTNK及电脑六人装载/卸载/重装存读开火已验 | BFRT的CanRetaliate=no与原生被动目标标志保留“不自动冲敌坦克”；TTNK没有充能副武器，普通/精英均不充能 | 实际被动开火/受击不移动、手动Attack可压灰熊；TTNK不入charger列表而SHK正常充能通过。电脑原脚本装满6人、存读实射、解散全卸载/存读、同一批活体重建队重装、部分与全装存读并六人再次实射通过；不代表自主重建决策。优先级招走与AreaGuard保留原生行为，带队卸载/竞争补员/换主/多AI长局仍待验；不把线圈充能当V01中心供电 |
| V09 / P2 | 其他 stock 箱子与无键补丁 | 全量对照当前有权重/可达的效果；零权重且地图不指定的 Gas/Napalm 等不误列高频缺口 |
| V12 / P1 / 已接基础链，特殊回调仍待验 | NP建筑换主的临时online/完整旧主AI/新主采样，及必要驻军重入门 | 448BD9/448BED→C2ECB9/C2ECD6与C2ED1D/C2ED3B已沿共享owners接通；实际EMP/Off换主存读续射、普通进驻/离楼归属、AY三组中心工程师夺取/三次存读/新主支持及运动恢复通过。稀有退休/多manager/Temporal与多AI仍待验，详见当前V12节 |

作者170507的无新键修复另外保留为 **V10 / P2 部分已适配，其他待核**：精英后射程反降、光棱联击、
秘密实验室单位选择概率、火箭飞行兵坠落时被地面单位攻击。后者已按实际51BC67跳过Health复活接入，
不是改变目标层；NP/stock死亡与实际AY坠落验证、未验边界见当前检查点对应节。秘密实验室已按实际#174修改接入：
68C077移除lab数上限门禁，C2E5FC以完整候选数抽样后环扫未用标记，用尽时清零但不标记返回项；
`src/sim/secret_lab.rs`沿原owner保留NP/stock分支，原生RNG三例和直接消费者NP/无NP各九项通过。
不改变既有保存fallback/夺取/授权owner；实际AY奖励池29座fixture、双捕获独立授权/付款撤销存读及剩余奖励实产已验，见当前检查点；原样作者Operation Lone Guardian II单图启动/捕获/死亡授权清理及存读已验；GUI、其他地图/全部奖励家族/多AI长局仍待验。
红血碉堡准入已按457D8D→7E0712..31接入既有CanBeOccupiedBy owner：NP仅同House跳过红血拒绝，stock与不同passive House保留原规则；
真实AY普通驻军/留驻、AI择楼、客户端点击与边界回归通过，critic无可操作发现。尚未进入当前cfb479750玩家标签，不新增序列化字段或触碰平民红血撤出。
船厂维修免伤已按737CE4→7E0495修正：仅NP伤害查询排除UnitRepair，不误用Naval；其他工厂查询保留，真实付费维修受击/修满/释放及stock保护回归通过。
不是直接把船伤害转给船厂；实际投射物对共格船厂的合法AoE不在此修复中取消，完整实战待验。
飞机维修建筑点击已按417E24的NP无条件跳转接入：Select不被stock NoEnter覆盖，Alt仍走Move；实际GADEPT/GAYARD运动与无误入回归通过，详见当前检查点。
铁幕/力场自动新选敌和对象点击准入已接6F826E→7E0AFC及700444→7E0B35，包含Immune筛除、C4/渗透的共享准入、到期恢复和实际AY Guard保存恢复验收；实际AY已有Attack在正式IC/FS发射后保持目标、保护免伤、保存恢复和自然到期恢复伤害已验；实际ForceShield fade提示一次/提示后存读不重播、blackout remaining1精确清除和用电实射恢复已验；三House单向联盟两发重叠、独立blackout/fade/接收刷新与三次存读已验；混合保护、正常外交输入、反击/直接脚本与自主多AI仍待验。
光棱支援招募已按44B388→7E04BB接LastOperational采样，真实Off/存读/独立开火/On/重新联击通过；不现场重算供电，不取消已排定束，EMP/缺电/绘制边界仍开放。
Arcing/SubjectToCliffs联合射程分支已接6F7404→C2D5ED，覆盖实体/mapless、桥面/射线及真实AY普通/精英巨炮typed查询；其他精英退射程原因、实际炮口与攻击验收仍开放。
作者明确点名的TELE已核实际主槽12→13、副槽11→12格，实际边界查询及普通实射→经验receiver晋升→存读→精英建筑实射通过，见对应节；不是通用精英加射程补丁，也不冒称最远炮口/车辆抬升已验。
其他槽的名义Range已做Battle/Return Dubai各120对完整typed库存，无未解析槽；4条配置缩短见C07及对应节，常规SHK/FV已实际登车/晋升/存读实射验证，保留原配置，不当成兼容缺失去加射程。其他几何与未点名补丁仍不据此全部关闭。
其他项目尚未逐条追#174补丁和实际运行，既不算已兼容，也不统一判缺失。
V03 和 V05 已单列作者所述 burst/驻军经验，不再在 V10 重复开链。
**V11 / P2 适用性调查**：NPAE的TechnoType `DeployFacing`（+5A4）部署转向门不同于已有BuildingType同名键。
当前Return Dubai/Battle各实际物理层均未给SCHP配置该键、TrueConversion或DeployToFire；对应默认为-1/false/false。
SMIN实际非simple、DeploysInto=YAREFN，实际转换消费YAREFN的建筑+EDC朝向，已有owner覆盖；见C06。
`TrueConversion`是另外的部署变型开关，不能凭SCHP的`DeployFire`认成它。缺少独立扩展实现不等于当前场景必需缺失；
其他模式/地图或未来重新启用仍须重审。两类扩展尚未移植，不以默认未启用声称全引擎支持。
原生读取/默认：Techno DeployFacing71385F..713873 current-default ReadInt截byte，710F93 seedFF；
TrueConversion714C02..714C1D Bool，7113A7=false；DeployToFire Unit747127=false。
消费：739B0E→C2DE77/C2DF74等待停止/转向，739B6A/739B98→7E0D3C/6B仅在TrueConversion与UnloadingClass俱有时变型。
实际层前置测试复用RetailRulesSources和NativeRulesProcessOwner，不以根文件grep取代层次验证。

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
| A09 / P1 / 已修配置、待验 | 攻城直升机 Script `07C3188C-G` / `07C48DBC-G` 的 Guard 已为 `5,0`；Team `07C460CC-G` / `07C3092C-G`；活跃触发 `07C30A5C-G` / `07C4853C-G` | 真实AY两机已复现action54精确到达仍moving/NavCom，持续观察停在cursor0；显式组件seek到9后Unload被后续Attack接续、两机未部署。三项相关NP开关已核各物理层未设置，对应默认不能解释此停滞；原生相关门同样保留。仍不能擅改悬停参数或直接归为原MOD错误；自然移动→部署→开火→再移动仍开放 |
| A10 / P1 / 巨炮实际建造已验，其余待验 | 作者170601称电脑已修巨炮/裂缝建造并会用超级裂缝；已有基地选择、部署/Gap owner | 非盟军错误强制计划已修；实际AY三困难AI正常基地发展中，仅盟军选择/付款/落地并结束巨炮施工，非盟军无巨炮计划/选择/生产。隔离进攻组件不等于完整战斗验收；占领科技/GUI、裂缝建造与超级裂缝相关单位/能力仍待验，不把旧作者bug当当前引擎必然存在 |

A04 双队列已按实际NPAE174指令适配，见本轮定位与 [npatch.md](npatch.md)；海陆编队实战验收、择兵同需求策略和可选并行限制仍开放。

### 固定配置残留：单独管理，不猜修复

| ID / 优先级 | 引用链与现状 | 后续动作/风险 |
| --- | --- | --- |
| C01 / P3 | TF `07C338DC-G` 含未知成员 SHD → Team `0610813C-G` → AITrigger `0616004C-G` / `07C330CC-G`；根 AIMD 两条触发的三个难度位物理值均0 | 当前固定触发不可达；不能写成“活跃碉堡队少造兵”。地图可重新启用，届时按未知成员忽略政策运行；不猜为 SHK/SCHD |
| C02 / P3 | 已注册但缺定义的 TF `05FC0DEC-G` → Team `060F883C-G`（Yuri Armor - H3）；根 AIMD 固定 AITrigger 未引用该 Team | 保留身份，不能补抄原版编队；地图/脚本重开才会产生实际影响，若恢复战术需作明确维护设计 |
| C03 / P2 | 短行 AITrigger `0C8C51BC-G` / `FFFFFF12-G` 的对象类型字段为空；顺序token读取跳过空字段，移读后multiplayer=false，当前遭遇战不具资格（FixedAimd的enabled仍为true） | 两条原配置战术尚未恢复，但不是enabled=false或新的加载回归；reader `src/rules/team_ai_ini.rs:625`、空项读取`src/rules/ini_value.rs:804`、拒绝入口`src/sim/ai_team_creation.rs:393`。先核作者意图与原生读取，不加占位值擅自启用 |
| C04 / P3 | Sapper/SapperE物理配置含AntiNaval=yes、AntiUnderwater=yes；实际已检查NPAE174 Weapon reader不读取两键 | 已逐分支追772080..7729E2，以及772901→B937F8/B96950/B96AC8→772907和7729AA→B94D06..B94EB1→7729B0，固定key读入无这两项或外部reader调用；字节/根DLL字面值检查只是补充证据，不单凭搜不到作结论。谭雅对舰由现有NavalTargeting4解释且已实测，不添加无原生定义的默认值/字段。换目标补丁/有运行时注入证据时重审；不宣称排除了任意外部注入 |

| C05 / P3 | NASAM的BurstDelay0..3=9不参与实际#174建筑开火ROF，不是遗漏建筑扩展 | Building构造43B71F的vtable7E3EBC，+2C=459EC0返回类号6；Mission_Attack44B6C4..D0经+3CC=6FDD50进入FireAt，6FF274..289经+318=6FCFA0进入GetROF。6FD03E取得类号后只保留Unit1，Building6成为null，6FD067..69跳过delay数组。Unit reader747B03..47才读取四个单数键。另有复数字符串BurstDelays不建立活跃消费者或别名，不能凭字面值扩展NASAM射速 |
| C06 / P3 | SMIN的TechnoType DeployFacing不消费于NP简易部署方向点；YAREFN建筑同名键已有消费者 | 生产实际规则确认SMIN非simple、DeploysInto=YAREFN，YAREFN朝向0；原生739AD4入口拒绝非simple，MCV转换7395C0..C6→465D70读取目标BuildingType+EDC而非UnitType+5A4。仅关闭这条具体误归因，不宣称所有可能间接调用或其他类型从不使用NP键 |
| C07 / P2 / 配置差异，部分适用性待核 | Battle/Return Dubai各120对普通/精英槽中，FV槽6、UTNK槽0、XCOMET2槽0/1四项精英名义Range更短；不是未知槽被跳过 | 实际键与typed一致，具体武器/格数及原生读取见V10对应节。SHK实际登FV后6→5.25格、晋升/存读/近距实射通过，不修改MOD或额外补射程；UTNK/XCOMET2常规消费者、其他模式/地图和完整最远几何仍待核，不泛称它们不可达或全部兼容 |
## 6. 已交付机制和有意政策：不重复“修复”

以下是交接分支已有实现，不再列为纯缺失。macOS 的已有覆盖不冒称 Windows 重跑。

| 机制 | 实现/覆盖边界 | 尚未关闭的边界 |
| --- | --- | --- |
| 宽容 AIMD 安装、顺序 AITrigger、TeamType 查找 | 未知成员忽略、缺定义编队身份保留；已有生产 reader/VM 改动 | 不证明每条活跃战术；见 A01/C01–C03 |
| 三阵营随机范围 | `NewPlayableCountries=-7`；startup root reader，现有 RNG 顺序 | Windows 人类/AI多个随机槽；不是 mode/map 覆盖项 |
| 科技树与 AI 规划 | 七处 Prerequisite2、三处 NegativePrerequisite、八厂 AIBasePlanningCount 通过共享 owner；已有实际MOD科技/船厂生命周期覆盖 | 普通AI原生提前成功分支保留，不强制套人类前置规则；实际发展见 A02 |
| 手动切电 | UseTogglePower、V/可重绑、鼠标与事件/BuildingOnline 链；已有 GUI 兵营Off/On与地图切换/手势回归 | Windows普通输入与存档；不另建客户端电力状态 |
| EMP | 实际TTNK→TankBolt/TankBoltE→ElectricbounceT/TE→TankBoltEMP→EMPulsWH；Drive及普通建筑分别走共用owner的原生类分支，医院任务不再被清掉，双雷达扫描/付款恢复与存读已验 | 时长/适用对象/算术仍为[明确政策](npatch-emp.md)，不是完整#174等价；特殊运动/联动manager/完整建筑家族与多AI见A06，不宣称全EMP完成 |
| 已付款生产头暂停/恢复 | EMP不退款/重建，手动Off和用户Hold不被到期覆盖；已有实际MOD生产场景和保存状态回归 | Windows运行与多工厂/多House |
| 乘员/母子机经验 | BFRT乘员先升、精英回退载具；飞机子机保留自身奖励并额外训练母体，母体等级采样时仅提高弱子机、再生按Trainable继承；已有实际BFRT Enter与航母再生存读场景 | 见 V05/F13；不是所有驻军链都已验 |
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
