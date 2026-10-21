# OVCD engine messages, Simplified Chinese. Draft: machine-translated, for native review (locales/REVIEWING.md).

## crates/ovcd-core/src/aft_arrangement.rs

# status: draft
check-thrusters-fit-beam = { $count } x { $envelope } m 推进器横向需要{ $needed } m，型宽为{ $beam } m
# status: draft
check-requested-thruster-count-fits = 请求的{ $requested }台推进器中可布置{ $fit }台；{ $requested }台需要型宽{ $beam } m 中的{ $needed } m
# status: draft
check-containers-fit-beam = { $count } x 40 ft 单元并列，横向需要{ $needed } m，型宽为{ $beam } m
# status: draft
check-units-fit-in-one-row = { $units }个单元分{ $rows }排，每排{ $per_row }个；第二排使尾部区块增加{ $added } m，并直接占用货物区长度
# status: draft
check-wheelhouse-fits-beam = 驾驶室宽{ $width } m，型宽为{ $beam } m
# status: draft
check-accom-beside-wheelhouse = { $units } x 40 ft 单元布置在{ $wheelhouse } m 驾驶室旁，需要型宽{ $beam } m 中的{ $needed } m
# status: draft
check-accom-beside-wheelhouse-rows = { $units } x 40 ft 单元布置在{ $wheelhouse } m 驾驶室旁，需要型宽{ $beam } m 中的{ $needed } m；因此改为在驾驶室后方布置一排，这会加长区块
# status: draft
check-gensets-fit-beam = { $count } x { $width } m 发电机组并列，横向需要{ $needed } m，型宽为{ $beam } m
# status: draft
check-gensets-fit-under-deck = 发电机组{ $height } m + 1.00 m 双层底及甲板结构（估算）<= 型深{ $depth } m
# status: draft
check-gensets-carry-load-with-one-in-reserve = { $count } x { $set_kwe } kWe 机组中{ $running }台运行，承担{ $load } kWe
# status: draft
warn-aft-gas-stern-estimates = 燃气动力尾部区块：已公布发动机以外部分（发电机、底座）的发电机组包络尺寸，以及机舱和 GVU 舱的间隙，均为估算值（无供应商总布置图）。排气管和燃料透气桅不计入水上高度（假定可放倒）。燃料取自货物蒸发气（IGC 规则第 16 章）；区块内不设单独的燃料舱。
# status: draft
check-wheelhouse-sees-over-cargo = 升高后眼高{ $eye } m >= 障碍物{ $obstruction } m + { $margin } m 裕量（已用升高量{ $lift } m，最大{ $max_lift } m）
# status: draft
warn-aft-sightline-lift = 视线需要升高{ $lift } m；升降柱可提供{ $max_lift } m。请升高升降柱或降低货物顶面。

## crates/ovcd-core/src/barge_capacity.rs

# status: draft
check-feasible-geometry = 货物容积必须为正（液货舱/货舱可布置）
# status: draft
check-not-overloaded = 按容积确定的载荷必须在型深范围内浮起
# status: draft
check-freeboard-ok = 干舷{ $freeboard } m >= 要求值{ $required } m
# status: draft
check-cargo-zone-positive = 货物区{ $zone } m = LOA { $loa } - 首部空舱{ $bow_void } m - 尾部空舱{ $stern_void } m
# status: draft
check-air-draft-condition-ballasted-light = 压载空载
# status: draft
check-air-draft-condition-light = 空载
# status: draft
check-air-draft-within-limit = { $condition }水上高度{ $governing } m <= 限值{ $limit } m（满载时为{ $laden } m，不是控制工况）
# status: draft
warn-barge-pushed = 顶推驳船：船员、电池和推进装置位于推轮上，不计入本船体的空船重量、载货能力或水上高度。推轮是另一艘船舶，必须单独计算费用并单独校核水上高度。
# status: draft
check-gm-positive = 斗式货舱驳船完整稳性 GM { $gm } m >= 0.15 m（一阶估算；KG_light @0.5D）
# status: draft
warn-barge-bulk-gm = 散货 GM 为一阶估算（假定空船 KG）；未模拟货物移动/休止角——请用完整的完整稳性计算书确认。

## crates/ovcd-core/src/barge_cost.rs

# status: draft
check-masses-positive = 质量模型输入齐全
# status: draft
check-no-reliquefaction-in-scope = 按压力累积运行方案，货物系统不包括再液化装置和制氮装置
# status: draft
check-fabrication-rates-are-estimates = 制造/供货单价为用户提供的估算值；在依赖总价之前，请用船厂报价替换

## crates/ovcd-core/src/electric_energy.rs

# status: draft
check-service-below-schijf = 营运航速{ $service } kn 低于 Schijf 限速{ $limit } kn
# status: draft
check-service-below-schijf-open-water = 无受限水域限速（开阔水域）
# status: draft
check-energy-positive = 营运航速下能耗{ $energy } kWh/km
# status: draft
check-resistance-coeffs-first-order = 受限水域阻力（剩余阻力/Lackenby/阻塞比）为一阶估算（RS1）——ITTC-1957 摩擦阻力为标准方法；用于船级之前，请用模型试验/CFD 校准。

## crates/ovcd-core/src/gas_arrangement.rs

# status: draft
check-tank-length-over-inner-diameter = 液货舱长度{ $length } m / 内径{ $diameter } m = { $ratio } <= { $max }
# status: draft
check-freeboard-at-payload = 载货吃水{ $draught } m 时干舷{ $freeboard } m >= 要求值{ $required } m
# status: draft
check-stability = GM { $gm } m（液体），最大 GZ { $max_gz } m，位于{ $angle }°
# status: draft
check-stability-fails = GM { $gm } m（液体），最大 GZ { $max_gz } m，位于{ $angle }°——不通过：{ $fails }
# status: draft
check-stability-not-assessed = 无法评估稳性：{ $reason }

## crates/ovcd-core/src/gas_separation.rs

# status: draft
check-cofferdam-min-width = 隔离空舱{ $cofferdam } m >= { $min } m（cl. 3.6.13）
# status: draft
check-opening-setback = 尾部间隔{ $aft } m >= { $min } m，自货物区量至任何入口或可开启窗（cl. 3.3.7 / 3.3.11）
# status: draft
check-fwd-end-bulkhead-space = 首端舱壁处所{ $fwd } m >= { $min } m（{ $tank_ends }，cl. 3.5.3）
# status: draft
check-fwd-end-bulkhead-space-domed-ends = 液货舱端部为凸形
# status: draft
check-fwd-end-bulkhead-space-plane-ends = 液货舱端部为平面
# status: draft
warn-separation-clauses = 间隔条款引自内河船舶入级规范（Pt 4 Ch 4 Sec 3，Jul 2022）。发布前请对照 ADN 文本确认。

## crates/ovcd-core/src/gas_stability.rs

# status: draft
stab-limit-concept-stage = 概念阶段：棱柱形箱形剖面，一阶重心高度估算，未计进水开口。
# status: draft
stab-limit-no-wind-or-damage = 未计风压横倾，也未计破舱稳性（批准前 ADN 9.3.1.15 / IGC 2.7 要求两者都有）。
# status: draft
stab-limit-stern-block-mass = 自航尾部区块的质量不在气体空船重量中，因此 KG 和排水量未计入该质量；实际 GM 可能更低。
# status: draft
stab-limit-weight-split = 重量分项合计为{ $split } t，而空船重量为{ $lightship } t。

## crates/ovcd-core/src/journey_derating.rs

# status: draft
check-capacity-is-volume-bound = 输入的载货能力必须是可行的、受容积限制的设计（未超载）
# status: draft
check-floats = 航道必须能浮起部分货物（T_max { $t_max } m 对比空船吃水）
# status: draft
check-ukc-consistent = 设计水深必须 >= UKC（传入水深 + UKC，切勿传入预先折减的 T_max）
# status: draft
check-utilisation-reasonable = 容积利用率{ $utilisation } >= 下限{ $floor }
# status: draft
warn-journey-draft-bound = 受吃水限制：为在 T_max = { $t_max } m 时浮起，从{ $capacity } t 容积载货能力中减去{ $shed } t（满载吃水{ $full_load_draft } m）。

## crates/ovcd-core/src/resistance.rs

# status: draft
check-service-speed-below-limiting = { $environment }：营运航速{ $service } kn <= 0.95 x Schijf 限速{ $limit } kn
# status: draft
check-power-within-installed = { $environment }：功率{ $power } kW <= 装机功率{ $installed } kW
# status: draft
check-blockage-in-range = { $environment }：阻塞比 n = { $blockage } 必须在 (0, 1) 之内
# status: draft
check-coefficients-are-first-order = 剩余阻力 Cr/Cf 拟合、Lackenby 浅水修正、阻塞比和 1.02 形状因子均为未经校准的一阶估算（RS1）——ITTC-1957 摩擦阻力为标准方法。

## crates/ovcd-core/src/squat.rs

# status: draft
check-keel-clears-bed = 动态 UKC { $ukc } m = 静态{ $static_ukc } m - 航行下沉量{ $squat } m（{ $method }），航速{ $speed } kn
# status: draft
check-keel-clears-bed-with-margin = 动态 UKC { $ukc } m，对比{ $margin } m 工作裕量
# status: draft
check-froude-depth-subcritical = 水深弗劳德数{ $froude }；超过 0.7 时水流接近临界，两种航行下沉量公式均失效
# status: draft
check-blockage-within-barrass-range = 阻塞比{ $blockage }；Barrass 公式的拟合范围约在 0.35 以下，超过时为外推
# status: draft
check-methods-agree-within-50pct = Barrass { $barrass } m 对比 ICORELS { $icorels } m，相差{ $spread }%
# status: draft
warn-squat-icorels-not-evaluated = 未计算 ICORELS：水深弗劳德数{ $froude }达到或超过 0.99
# status: draft
warn-squat-open-water = 未给出航道断面积，因此阻塞比为 0，Barrass 结果为 0——这是开阔水域工况，此时航行下沉量仅按 ICORELS 计算

## crates/ovcd-core/src/vessel_standards.rs

# status: draft
check-unique-ids = 每个键都与其 standard_id 一致
# status: draft
check-unique-ids-mismatch = 键/standard_id 不一致：{ $keys }
# status: draft
check-ranges-ordered = 每个 (min, max) 范围的顺序正确
# status: draft
check-ranges-ordered-inverted = 范围颠倒：{ $keys }
# status: draft
check-positive-dimensions = 所有型宽和长度均为正
# status: draft
check-positive-dimensions-bad = 非正尺度：{ $keys }
# status: draft
check-convoy-beam-consistent = 船队型宽 = 单船型宽 x n_abreast
# status: draft
check-convoy-beam-consistent-bad = 型宽与 n_abreast 不符：{ $keys }
# status: draft
check-source-cited = 每个等级都引用其来源表
# status: draft
check-source-cited-missing = 无来源：{ $keys }

## crates/ovcd-core/src/waves.rs

# status: draft
check-stawave1-short-wave-validity = lambda/Lpp = { $ratio }（波长 lambda { $wavelength } m，Lpp { $lpp } m）。STAwave-1 假定为短波，lambda/Lpp < 0.5；超出此范围会低估，在 lambda/Lpp = 1 附近可能低估 2 倍以上
# status: draft
check-wave-height-within-screening-range = H_s { $hs } m；超过约 3 m 时，筛选公式不再适用，而且通常由主动减速控制
# status: draft
check-bow-length-supplied = 未给出 L_BWL，假定为{ $bow_length } m = 0.2 x Lpp（钝形驳船首部）。较尖的首部会减小波浪增阻，因此此假定偏保守
# status: draft
warn-waves-period-estimated = 未给出波浪周期；由 H_s 估算 T_p 为{ $tp } s，它只用于适用性校核，不影响阻力
# status: draft
warn-waves-beam-seas-floor = 近横浪：本模块给出的是下限，因为未模拟漂移、舵和航向保持损失，而偏离船首方向时它们起主导作用

## crates/ovcd-format/src/class_input.rs

# status: draft
msg-class-scheme = “{ $input }”{ $class }属于等级方案{ $family }，而不是{ $scheme }
# status: draft
msg-class-none = 等级方案{ $scheme }中没有适用于{ $kind }船队编组的等级，因此不适用任何等级
# status: draft
msg-class-moved = “{ $input }”{ $typed }不满足此船长和型宽；该船舶属于{ $class }
# status: draft
msg-class-beyond = 此船长和型宽超出最大的{ $scheme }等级（{ $class }）
# status: draft
msg-class-draught-above = “{ $input }” = { $value }高于等级{ $class }的吃水（{ $limit } m，其范围上限）
# status: draft
msg-class-clearance-above = “{ $input }” = { $value }高于等级{ $class }的最低桥梁净空（{ $limit } m）

## crates/ovcd-format/src/run.rs

# status: draft
msg-convoy-not-available = 船队结果尚不可用

## crates/ovcd-format/src/simulate.rs

# status: draft
check-air-draft-limit = { $air_draft } m，限值{ $limit } m
# status: draft
name-governs-draught = 吃水——{ $limit }
# status: draft
check-feasible-arrangement = 此船体上没有通过全部校核的布置。显示的是最接近的布置。它不通过：{ $fails }。

## crates/ovcd-format/src/text_ids.rs

# status: draft
name-condition-laden = 满载
# status: draft
name-condition-half-laden = 半载
# status: draft
name-condition-light = 空载
# status: draft
name-criterion-area-0-30 = 面积 0–30°
# status: draft
name-criterion-area-0-40 = 面积 0–40°
# status: draft
name-criterion-area-30-40 = 面积 30–40°
# status: draft
name-criterion-gz-at-30 = ≥ 30° 时的 GZ
# status: draft
name-criterion-angle-of-max-gz = 最大 GZ 角
# status: draft
name-criterion-initial-gm = 初稳性高度 GM（液体）
# status: draft
name-criterion-floats = 浮起且有干舷
# status: draft
name-check-stability-laden = 稳性——满载
# status: draft
name-check-stability-half-laden = 稳性——半载
# status: draft
name-check-stability-light = 稳性——空载
# status: draft
name-check-stability-not-assessed = 稳性——未评估
# status: draft
name-check-cargo = 货物 > 0
# status: draft
name-check-feasible-arrangement = 可行布置
# status: draft
name-check-air-draft = 水上高度（空载，控制工况）
# status: draft
name-check-tank-length = 液货舱长度与直径之比
# status: draft
name-check-ok = 正常
# status: draft
name-curve-deep-water = 深水
# status: draft
name-governs-hold-volume = 货舱容积
# status: draft
name-governs-tank-volume = 液货舱容积
# status: draft
name-governs-no-cargo-area = 无货物区（船体短于其固定端部处所）
# status: draft
name-vessel-box = 箱形货舱干散货驳船
# status: draft
name-vessel-hopper = 斜壁斗式货舱干散货驳船
# status: draft
name-vessel-gas = 液化气驳船，IMO C型液货舱
# status: draft
name-cargo-lightship-alone = 仅空船重量就超过允许吃水。
# status: draft
name-cargo-nothing-fits = 无法布置：请加长或加宽船体。

## crates/ovcd-format/src/vdesign.rs

# status: draft
msg-file-not-json = 该文件不是 JSON：{ $error }
# status: draft
msg-file-the-file = 该文件
# status: draft
msg-file-duplicate-key = `{ $key }` 在{ $place }中出现两次；一个键只能出现一次
# status: draft
msg-file-not-object = 该文件必须包含一个 JSON 对象
# status: draft
msg-file-format = 该文件需要 `"format": "vdesign"`
# status: draft
msg-file-version-unsupported = 不支持 format_version { $version }；本工具读取版本{ $oldest }至{ $newest }
# status: draft
msg-file-version-missing = 该文件需要整数 `format_version`
# status: draft
msg-file-engine-version = 该文件需要非空字符串 `engine_version`
# status: draft
msg-file-units = `units` 必须为 "SI"，而不是{ $units }
# status: draft
msg-file-unknown-key = 未知的顶层键 `{ $key }` 已忽略
# status: draft
msg-file-no-options = 该文件需要 `options` 列表
# status: draft
msg-file-empty = 设计至少需要一个方案
# status: draft
msg-option-no-name = 方案{ $number }需要非空字符串 `name`
# status: draft
msg-option-names-differ = 有两个方案名为“{ $option }”；名称必须不同
# status: draft
msg-option-no-inputs = 需要 `inputs` 对象
# status: draft
msg-option-unknown-key = 未知的键 `{ $key }` 已忽略
# status: draft
msg-results-hash-shape = `results_hash` 必须为 64 位十六进制数字
# status: draft
msg-results-hash-text = `results_hash` 必须为文本
# status: draft
msg-ruleset-exact-renamed = IGC_2G_EXACT 现为 IGC_2G：采用 IGC 2.4.1.1 的同一逐舱规则
# status: draft
msg-ruleset-per-tank = IGC_2G 现采用 IGC 2.4.1.1 的逐舱规则（d(Vc) 取自每个液货舱的总容积，含穹顶，按 2.4.2 从板到板量取），因此结果可能变化
# status: draft
msg-input-note = “{ $input }”：{ $note }
# status: draft
msg-v1-adn-g = “{ $input }”ADN_G 已不再提供（引擎对液货舱只采用 IGC 规则）；此版本 1 文件已按 IGC_2G 打开
# status: draft
msg-v1-tank-cap = “{ $input }”已不再是输入项（没有液货舱容积上限）；已删除
# status: draft
msg-v1-max-tank-od = 此版本 1 文件没有“{ $input }”；现采用默认值{ $value } m（版本 1 使用固定的 12 m 上限，因此结果可能变化）
# status: draft
msg-marker-source = `source` 必须为 "typed"、"default" 或 "class"
# status: draft
msg-marker-value = 带标记的输入需要 `value`
# status: draft
msg-marker-only = 带标记的输入只包含 `value` 和 `source`
# status: draft
msg-unknown-input = 未知输入 `{ $key }`
# status: draft
msg-input-unused-cargo = “{ $input }”不用于货物{ $cargo }；已忽略
# status: draft
msg-input-unused-containment = “{ $input }”不用于{ $containment }货物围护系统；已忽略
# status: draft
msg-input-unused-formation = “{ $input }”不用于{ $formation }船队编组；已忽略
# status: draft
msg-input-required = 缺少必填输入“{ $input }”
# status: draft
msg-input-filled = 文件中未存储“{ $input }”；已写入默认值配置中的值{ $value }，标记为默认
# status: draft
msg-legacy-dome = 此文件没有“{ $input }”；现取{ $value } m（标记为默认）。穹顶现计入 d(Vc) 的 Vc（IGC 2.4.1.1），因此结果可能变化
# status: draft
msg-legacy-carriage = 此文件没有 LPG 货物{ $cargo }的“{ $input }”；按{ $condition }读取（标记为默认）。文件中缺少的液货舱数值保持引擎以前使用的值（设计压力、材料、绝热、结构系数），因此结果不变；如需采用该装运方式的默认值，请设置装运方式并重置这些输入
# status: draft
msg-legacy-fixed-one = 此文件没有“{ $input }”；现取引擎在其成为输入项之前使用的固定值（标记为默认），因此结果不变
# status: draft
msg-legacy-fixed-many = 此文件没有{ $inputs }；每项现取引擎在其成为输入项之前使用的固定值（标记为默认），因此结果不变
# status: draft
msg-inputs-not-stored = 文件中未存储的输入：{ $inputs }。已保存的设计按实际使用值保存每个输入，因此修改默认值配置不会改变它；运行 `ovcd fill` 可从默认值配置写入这些输入，标记为默认
# status: draft
msg-profile-case-keyed = { $key }的 `default_by_{ $by }`
# status: draft
msg-profile-case-default = 一个 `default`
# status: draft
msg-profile-no-default = 默认值配置“{ $profile }”未给出“{ $input }”的默认值（缺少{ $case }）
# status: draft
msg-profile-no-table-default-carriage = 当货物围护系统为{ $containment }、货物为{ $cargo }且装运方式为{ $condition }时，默认值配置“{ $profile }”未给出“{ $input }”的默认值（`{ $key }.default_by_table` 中没有匹配的行）
# status: draft
msg-profile-no-table-default = 当货物围护系统为{ $containment }、货物为{ $cargo }时，默认值配置“{ $profile }”未给出“{ $input }”的默认值（`{ $key }.default_by_table` 中没有匹配的行）
# status: draft
msg-profile-no-depth-ratio = 默认值配置“{ $profile }”未给出{ $group }货物的型深型宽比（缺少 `depth_to_beam_ratio.default_by_cargo_group.{ $group }`）
# status: draft
msg-number-or-null = “{ $input }”必须为数字或 null（无）
# status: draft
msg-bool = “{ $input }”必须为 true 或 false
# status: draft
msg-choice = “{ $input }” = { $value }必须为以下之一：{ $choices }
# status: draft
msg-string = “{ $input }”必须为字符串
# status: draft
msg-scheme = “{ $input }” = { $value }不是含有可定义船舶的船型等级的等级方案（见 `ovcd schema`）
# status: draft
msg-class-cannot-define = “{ $input }” = { $value }无法定义船舶：等级{ $class }未给出船长或型宽，或其船队编组（{ $formation }）不是设计所具有的船队编组
# status: draft
msg-class-unknown = “{ $input }” = { $value }不是船型等级（见 `ovcd schema`）
# status: draft
msg-class-id = “{ $input }”必须为等级 id 或 null
# status: draft
msg-formation-object = “{ $input }”必须为含有 `kind` 的对象
# status: draft
msg-formation-kind = “{ $input }”必须为以下之一：{ $choices }
# status: draft
msg-formation-only-kind = “{ $input }”：{ $kind }只包含 `kind`；`abreast` 和 `in_line` 用于 pushed_convoy
# status: draft
msg-formation-count = “{ $input }”对于 pushed_convoy 必须为整数
# status: draft
msg-formation-convoy-only = “{ $input }”：pushed_convoy 只包含 `kind`、`abreast` 和 `in_line`
# status: draft
msg-hard-range = “{ $input }” = { $value }超出范围：必须{ $bound }（硬性限值）
# status: draft
msg-soft-range = “{ $input }” = { $value }超出常用范围{ $min }至{ $max }（建议限值）；已接受
# status: draft
msg-slender-hull = “{ $input }” = { $length }是“{ $depth_input }” = { $depth }的{ $ratio }倍，超过约{ $limit }：相对其型深而言船体非常细长（建议限值）；已接受
# status: draft
msg-containment-cargo = “{ $input }” = { $containment }不能装载{ $cargo }（{ $phase }）；请使用{ $fits }
# status: draft
msg-freeboard-depth = “{ $input }” = { $freeboard }必须小于“{ $depth_input }” = { $depth }（硬性限值）
# status: draft
msg-whole-number = “{ $input }” = { $value }必须为整数（硬性限值）
# status: draft
msg-insulation-disagree = “{ $input }”为{ $kind }，但“{ $thickness_input }”为{ $thickness }：类型与厚度不一致（建议性校核）；已接受
# status: draft
msg-stress-from-input = “{ $input }”
# status: draft
msg-stress-from-basis = `published_design` 许用应力依据
# status: draft
msg-stress-from-material = 材料
# status: draft
msg-stress-above-igc = { $from }给出的许用应力为{ $stress } MPa，高于 IGC 4.23.3.1 对{ $material }规定的限值{ $limit } MPa，即 Rm/A = { $rm } / { $a } 与 Re/B = { $re } / { $b } 中的较小值（建议性校核）；已接受
# status: draft
msg-freeboard-search = “{ $input }” = { $freeboard }必须不大于{ $limit }，即“{ $search_input }”搜索的最深船体（硬性限值）
# status: draft
msg-channel-ukc = “{ $input }” = { $depth }必须不小于“{ $ukc_input }” = { $ukc }（硬性限值）
# status: draft
msg-hold-width = “{ $input }” = { $beam }在两侧各{ $void } m 边空舱之后，留下的货舱宽度为{ $width } m；{ $reason }
# status: draft
msg-negative-hold = 负的货舱会得出负的货物质量和吃水（硬性限值）
# status: draft
msg-hold-height = “{ $input }” = { $depth }留下的货舱高度为{ $height } m（型深减去{ $double_bottom } m 双层底，加上{ $coaming } m 舱口围板）；{ $reason }
# status: draft
msg-number = “{ $input }”必须为数字
# status: draft
msg-input-error = “{ $input }”：{ $error }
# status: draft
msg-option = 方案“{ $option }”：{ $message }

## Check names
# status: draft
check-name-thrusters-fit-beam = 推进器横向可布置
# status: draft
check-name-requested-thruster-count-fits = 请求的推进器数量可布置
# status: draft
check-name-containers-fit-beam = 集装箱横向可布置
# status: draft
check-name-units-fit-in-one-row = 单元可布置成一排
# status: draft
check-name-wheelhouse-fits-beam = 驾驶室横向可布置
# status: draft
check-name-accom-beside-wheelhouse = 居住舱室位于驾驶室旁
# status: draft
check-name-gensets-fit-beam = 发电机组横向可布置
# status: draft
check-name-gensets-fit-under-deck = 发电机组可布置在甲板下
# status: draft
check-name-gensets-carry-load-with-one-in-reserve = 发电机组留一台备用时可承担负荷
# status: draft
check-name-wheelhouse-sees-over-cargo = 驾驶室视线越过货物
# status: draft
check-name-feasible-geometry = 几何可行
# status: draft
check-name-not-overloaded = 未超载
# status: draft
check-name-freeboard-ok = 干舷满足
# status: draft
check-name-cargo-zone-positive = 货物区为正
# status: draft
check-name-air-draft-within-limit = 水上高度在限值内
# status: draft
check-name-gm-positive = GM 为正
# status: draft
check-name-masses-positive = 质量为正
# status: draft
check-name-no-reliquefaction-in-scope = 范围内不含再液化
# status: draft
check-name-fabrication-rates-are-estimates = 制造单价为估算值
# status: draft
check-name-service-below-schijf = 营运航速低于 Schijf 限速
# status: draft
check-name-energy-positive = 能耗为正
# status: draft
check-name-resistance-coeffs-first-order = 阻力系数为一阶估算
# status: draft
check-name-tank-length-over-diameter = 液货舱长径比
# status: draft
check-name-freeboard-at-payload = 载货时干舷
# status: draft
check-name-stability-not-assessed = 稳性——未评估
# status: draft
check-name-cofferdam-min-width = 隔离空舱最小宽度
# status: draft
check-name-opening-setback = 开口间隔
# status: draft
check-name-fwd-end-bulkhead-space = 首端舱壁处所
# status: draft
check-name-capacity-is-volume-bound = 载货能力受容积限制
# status: draft
check-name-floats = 可浮起
# status: draft
check-name-ukc-consistent = UKC 一致
# status: draft
check-name-utilisation-reasonable = 利用率合理
# status: draft
check-name-coefficients-are-first-order = 系数为一阶估算
# status: draft
check-name-keel-clears-bed = 龙骨不触底
# status: draft
check-name-keel-clears-bed-with-margin = 龙骨留有裕量不触底
# status: draft
check-name-froude-depth-subcritical = 水深弗劳德数亚临界
# status: draft
check-name-blockage-within-barrass-range = 阻塞比在 Barrass 适用范围内
# status: draft
check-name-methods-agree-within-50pct = 方法结果相差在 50% 以内
# status: draft
check-name-unique-ids = id 唯一
# status: draft
check-name-ranges-ordered = 范围顺序正确
# status: draft
check-name-positive-dimensions = 尺度为正
# status: draft
check-name-convoy-beam-consistent = 船队型宽一致
# status: draft
check-name-source-cited = 已引用来源
# status: draft
check-name-stawave1-short-wave-validity = stawave1 短波适用性
# status: draft
check-name-wave-height-within-screening-range = 波高在筛选范围内
# status: draft
check-name-bow-length-supplied = 已给出首部长度

## Added after the first draft

# status: draft
msg-loading-above-marvs = “{ $input }” = { $value }：{ $cargo } 在该温度下的蒸气压为 { $pressure } bar，高于 { $marvs } bar（绝对压力）的 MARVS；液货舱装至充装限值，不超过该限值（建议性校核）；已接受
