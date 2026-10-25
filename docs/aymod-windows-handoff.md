# AYMod Windows 开发交接

截至 2026-10-25。本文件交接已经做过的工作，不是新一轮兼容性审计。
未完成项沿用现有记录，没有为本次交接重新探索。后续以 Windows 实机结果为准。

## 1. 从哪里继续

- 引擎 fork：<https://github.com/jonymei/vera20k>。
- 开发分支：`feature/aymod-integration-resume`。
- 交接前分支提交：`a19c10a23`；本文件之后的文档提交不改变引擎行为。
- 已安装、已验收的干净 macOS 构建来源：`f1d141c7a`；与上述提交仅有文档差异。
- MOD 配置仓库：<https://github.com/jonymei/CnC2Yuri-AYMOD>，默认分支 `main`，当前本地提交 `10156cb`。
- 引擎远程约定：`origin` 指自己的 fork，`upstream` 指 `YuriPlanet/vera20k`。
- 没有向上游提 PR。不要从上游 `main` 直接开始而遗漏这条开发分支的改动。

实际目标补丁被识别为 **NPAE #0011 Extended #174**，不是 NPSE #359。
没有正面证据证明当前 AYMod 使用 Ares。当前引擎构建启用 `npatch`，不是靠启用
`ares` 替代 NPAE 适配。相近补丁的源码只能作参考，不能默认语义完全相同。

## 2. AYMod 本身改过什么

这轮 AI/科技树/切电/EMP/奖励控制适配没有新增 MOD 配置改动；实现位于 VERA20k。
当前 MOD 工作区干净。以下是此前已经应用的修改。

### 生效的四行 AIMD 修正

根目录 `aimd.ini` 是当前正式生效的配置，覆盖资源包中的旧 AIMD，**不是临时覆盖**。
原始快照保留在配置仓库 `upstream/aimd.ini`；两文件比较为四行替换。

| 位置 | 原值 | 当前值 | 判断性质 |
| --- | --- | --- | --- |
| TaskForce `0CE4DCDC-G`，键 `2` | `2,SERF` | `2,SREF` | 修正光棱坦克标识拼写 |
| TaskForce `0A6E3F1C-G`，键 `0` | `6,SERF` | `6,SREF` | 同上 |
| Script `07C3188C-G`，键 `1` | `5,` | `5,0` | 明确选择的修复设计，不声称考证出作者原值 |
| Script `07C48DBC-G`，键 `1` | `5,` | `5,0` | 同上 |

没有借此改动三个玩家阵营、科技树、兵种数值或战术权重。
`SHD`、缺定义的 TaskForce `05FC0DEC-G` 和两条异常 AITrigger 没有靠猜名称、
补原版编队或加占位参数消除报错；相关兼容行为交给引擎读取链处理。
`rulesmd.ini`、`Uimd.ini`、`mpmodesmd.ini` 未随这轮适配调整。

### 资源与仓库维护

- 从用户提供的 Steam 正版目录补入 `movmd03.mix`，替换原零字节文件。
  当前文件为 394,116,336 字节，原空文件保留为 `movmd03.mix.empty-backup`。
- 本次交接重新读取文件的 SHA-256 与维护记录一致：
  `26959d790cf1f3fa98b078a6b58a6d6cb3167be4495145b65ad2f853a4a5ef71`。
- 资源包、游戏二进制和个人运行文件不在配置 Git 仓库中。只克隆配置仓库不能运行游戏。
- 建立了配置维护记录、作者资料留存及原始 AIMD 快照；`10156cb` 仅补资源依赖与远程分支说明。
- 后续修改生效配置时保留编码、CRLF、作者原文和 `upstream/` 快照，不做整文件转码。

详细历史见 MOD 配置仓库的 `README.md`、`docs/维护记录.md`。

## 3. 引擎已经做过的工作

### AI 与科技树

- 修复前期 AIMD 加载、未知成员/缺失编队身份及原生查找、顺序读取方面的兼容问题。
- `NewPlayableCountries=-7` 使用前三个玩家国家，避免随机到非玩家条目。
- 接入 NPAE 的 `AIBasePlanningCount`，保留默认值、物理读取层次和原有候选重复消费者。
- `AlternatePrerequisite` / `Prerequisite2` 为完整 AND 组之间的 OR 回退；不只接在普通前置失败处。
- `NegativePrerequisite` 使用既有 House 在场建筑计数，覆盖共享成功路径；保留普通 AI 的原生提前成功分支。
- 没有通过改 MOD 键名去伪装适配另一个 NPatch 版本。

### 手动断电与 EMP

- `[UISettings] UseTogglePower` 接入普通输入、默认可重绑定的 `V`、目标判定和通断电事件链。
- 清理旧拖选手势、切换地图时的光标模式，避免松开旧鼠标手势误触切电。
- `EMEffect` 接入既有 EMP、停用/恢复、计时、建筑效果和供电状态 owner。
- 打通实际 `TTNK -> TankBolt -> ElectricbounceT -> TankBoltEMP -> EMPulsWH` 攻击链，
  包括 `Airburst=no` 的 impact child；没有给特斯拉坦克捏造 Secondary 来预注册武器。
- EMP 工厂只暂停现有已付款生产头，不销毁、退款或另建一个暂停状态。
  到期不会覆盖玩家手动断电或用户暂停；真正供电恢复后才恢复生产。

### 经验、赏金与永久控制

- `TrainPassengers`：可训练乘员先得经验，精英乘员回退给载具。
- `MindControllerGainExperienceRatio`：共享已有经验点，控制者无效或已精英时不吞掉原单位经验。
- Spawned 子机/导弹击杀训练母体，现有子机同步、再生子机继承母体经验。
- 赏金复用击杀记账和经济 owner：敌对终结击杀只结算一次，不用换主人或友军击杀刷钱。
- `IsPermaControlled` 接入既有永久控制 owner；间谍成功登车消耗自身，失败/落空恢复已有生命周期。
- 经验比例、赏金公式、EMP 时长/适用范围等存在明确的玩家友好政策；
  **这些政策没有全部被证明与 NPAE #174 原生算术一致**。细节见 `npatch.md`、`npatch-emp.md`。

### 验收过程中顺带修复

- 存档 baseline 重绑地形时，不再把 source/ClearTile 的已保存变体误用于转换后的 LAT tile。
  只重建缺失的 damaged-radar 派生数据，不改保存字段、不绕过布局校验、不 clamp 变体。
- 鼠标离开窗口客户区后停止边滚和残留惯性；保留窗口内自然惯性及捕获右拖的既有规则。
- 当时的“全黑战场”已用完整输入证明是相机滚出探索区，而非 Shroud 帧覆盖错误。
  没有修改正常战争迷雾来掩盖现象。
- 早期启动白屏后来没有稳定复现；没有提交基于猜测的 startup splash 修复。

## 4. 已验证到什么程度

以下均为 macOS 本地结果，**尚未在 Windows 重新验证**。

- 使用实际 AYMod rules/assets 的七个 headless 生产路径场景通过：特斯拉 EMP 与到期恢复、
  EMP/手动断电暂停并恢复已付款生产、谭雅赏金、BFRT 乘员经验与精英回退、
  尤里控制者经验与精英回退、间谍永久劫持、精英尤里永久控制建筑并移除控制者。
- 已验证实际 AYMod 科技树与船厂 Unlimbo/Limbo 前置计数，以及 AI 规划候选相关读取/消费者。
- 定向模块结果包括 NativeProcessing 91 项、EMP 8 项、切电输入 5 项、相机 26 项、
  地形存读档 5 项；另有奖励、捕获、寄生/登车和子机生命周期回归。
- Computer Use 走过普通菜单、遭遇战、基地部署、发电厂/兵营建造、旧 QA 存档恢复、兵营通断电。
  临时诊断确认第二次点击读到第一次点击已经提交的在线状态，不只检查输入 receipt。
- 干净优化构建已覆盖 macOS `/Applications/VERA20K.app`，签名/哈希检查通过，旧版备份保留。
  临时日志、个人存档、截图、完整视野 dump 和一次性诊断程序均没有提交到引擎仓库。
- 没有跑全仓库测试或逐项检查所有地图/模式；赏金动画外观及完整平衡性未单独认证。

## 5. 剩余工作：仅转交已有记录

下表不是新的实现结论。`缺失` 来自已有审计；`待验证` 不等于引擎完全没有实现。
到 Windows 后先复现玩家可见行为，再确定修复方案，不照表批量猜实现。

| 已有条目 | 已知边界与接手入口 |
| --- | --- |
| 驻军/掩体保护与穿透 | 尚未完成 NPAE 准入、受害者效果链；从现有 docking/bunker、combat/garrison owner 接入 |
| `TurretROT` | 记录了 14 处 MOD 用法；独立炮塔速度入口未适配，现有旋转链使用 `ROT` |
| `EBoltColor1/2/3` | 自定义颜色读取和最终绘制未接通；已有 `sim/combat/electric_bolt.rs` 与 `render/electric_bolt.rs` |
| `AACombat` / `AntiNaval` / `AntiUnderwater` | 尚未建立匹配 NPAE 的武器级目标政策；已有 projectile AA/AG 不能直接算作支持 |
| `IsFlatExplosion` / `DoesntAffectEnemies` | 原生语义与伤害消费者仍待核对；不能把 Ares 同类字段直接当等价 |
| Psychic Dominator 独立光照 | `Lighting`、`LightRed/Green/Ambient/Ground/Level` 的启用、恢复和最终画面尚未闭环 |
| 扩展 UI | `ShowTechLevelSlider`、`UseDropDownColors`、`MPC01..MPC16` 未完成配置到 UI 的验证 |
| PoweredUnit/PowersUnit 三组关系 | ROBO/GAROBO、DRON/NARADR、CAOS/NAPSIS 需专门检查原生链、供电、失效和恢复；已有 `sim/powered_unit.rs` |
| BFRT 第 5/6 座开火位置 | 固定 ART 中的 `AlternateFLH5/6` 已识别，索引与最终渲染未认证 |
| 其他已有字段/无新键补丁 | Hospital、BurstDelay、cloak/sensor、建筑经验、箱子与作者所述继承修复仍需分类核对；不要仅因 MOD 使用就判缺失 |

库存不是完整覆盖率：初次记录为 25,068 个物理 section/key 对、2,374 个不同键字符串，
生产 accessor 查询 207 项。它们都不是已验证的玩法机制数量，不能用来宣布“100%”。
完整已有清单及证据入口见 [aymod-compatibility.md](aymod-compatibility.md)。

## 6. Windows 接续的最小路径

先准备自己的合法原版/MOD 完整资源，以及 Rust、Git、Visual Studio C++ Build Tools、uv。
仓库 README 最低 Rust 要求为 1.97；这次 macOS 构建使用 1.99.1，不代表已验证 Windows 工具链。
下面路径只是 PowerShell 示例，请替换为 Windows 实际目录。

```powershell
# 新机器上的全新克隆；已有工程不要覆盖或 reset。
git clone --branch feature/aymod-integration-resume https://github.com/jonymei/vera20k.git D:\Work\vera20k
Set-Location D:\Work\vera20k
git remote add upstream https://github.com/YuriPlanet/vera20k.git
git config remote.pushDefault origin

$env:RA2_DIR = 'D:\Games\Red Alert II'
$env:VERA20K_AYMOD_DIR = 'D:\Games\CnC2Yuri-AYMOD'
$env:VERA20K_AYMOD_RULES = "$env:VERA20K_AYMOD_DIR\rulesmd.ini"
uv run --no-project python -m tools.prepare_checkout --ra2-dir $env:VERA20K_AYMOD_DIR

# 游戏使用本地 config.toml 指向 AYMod；RA2_DIR 留给正版数据测试。
uv run --no-project python -m tools.cargo_run -- build --profile fast -p vera20k --bin vera20k --no-default-features --features npatch
```

从 `cargo_run --resolve vera20k --profile fast` 返回的真实路径启动普通菜单，不猜 target 路径。
Windows 不用复制 macOS `.app`、target 缓存、个人 `VERA.INI` 或整个用户配置目录。
MOD 配置另从配置仓库 `main` 取回，先备份资源目录中的原配置，再应用当前生效文件；
不要把 `upstream/aimd.ini` 放到根目录覆盖已修正的 `aimd.ini`。

接手冒烟可复用已有场景，不用从头重写：

```powershell
$env:VERA20K_REQUIRE_RETAIL_INI = '1'
uv run --no-project python -m tools.cargo_run -- test -p vera20k-core --lib --no-default-features --features npatch headless_scenario::retail_construction_tests::aymod_ -- --ignored --test-threads=1
uv run --no-project python -m tools.cargo_run -- test -p vera20k-core --lib --no-default-features --features npatch maintained_aymod_tech_branches_and_shipyard_lifecycle -- --ignored
```

若工具报告 `ini/` 缺失，按 `tools.prepare_checkout` 的现有提示，从正版目录运行
`extract-ini` 生成本地零售测试数据；不要用 MOD 数据替代零售基线。不要照搬 macOS
个人 QA 存档作为必须通过的跨平台格式基准。后续每项缺口优先用 Windows 新局复现，
复用既有状态 owner，补少量生产路径回归；需要原生比对时再运行实际 NPAE MOD。

## 7. 已有参考与作者资料

- [完整兼容性记录](aymod-compatibility.md)
- [NPatch 现有实现与有意政策](npatch.md)
- [EMP 的实现边界](npatch-emp.md)
- [Ghidra 工作流](ghidra-workflow.md)
- 作者帖子：<https://www.zhihu.com/question/24050222/answer/34256484>
- MOD 配置仓库内的 `使用说明 170507版.txt`、`内容介绍 170507版.txt`、`更新日志 170601.txt`。

作者意图、NPAE 原生事实和为了可玩性作出的兼容政策应继续分开记录。
本次交接不要求在 macOS 继续探查未知项，也不授权额外发布完整 MOD 资源或向上游提 PR。
