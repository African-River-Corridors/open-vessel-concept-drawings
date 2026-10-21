# OVCD input labels, Simplified Chinese. Draft: machine-translated, for native review (locales/REVIEWING.md).

## Groups

# status: draft
group-required = 必填
# status: draft
group-vessel-class = 船型等级
# status: draft
group-hull-and-containment = 船体与货物围护系统
# status: draft
group-gas-tanks = 液货舱（气体货物）
# status: draft
group-note-gas-tanks = 液货舱外径（下方的上限值和计算结果）量至绝热层外表面；间隙和图纸均采用该表面。钢壳直径为该外径减去两倍绝热层厚度。除非键入壁厚，否则壁厚按压力容器公式计算。下方的设计值取决于货物，对 LPG 还取决于装运方式；每个值都标明其来源或标为估算值。
# status: draft
group-dry-bulk-hold = 干散货货舱
# status: draft
group-aft-block = 尾部区块
# status: draft
group-waterway-limits = 航道限制
# status: draft
group-speed-and-power = 航速与功率
# status: draft
group-cost-rates = 费用单价
# status: draft
group-note-cost-rates = 示例单价——请替换为您的数值。

## Inputs

# status: draft
input-cargo = 货物
# status: draft
choice-cargo-lng = LNG
# status: draft
choice-cargo-lng-ethane = LNG / 乙烷
# status: draft
choice-cargo-lpg-propane = LPG（丙烷）
# status: draft
choice-cargo-lpg-butane = LPG（丁烷）
# status: draft
choice-cargo-lpg-mix = LPG（60/40 丙烷/丁烷）
# status: draft
choice-cargo-bauxite = 铝土矿
# status: draft
input-hull-form = 船体线型
# status: draft
choice-hull-form-raked-box = 斜首箱形
# status: draft
input-formation = 船队编组
# status: draft
choice-formation-self-propelled = 自航
# status: draft
choice-formation-pushed-barge = 顶推驳船
# status: draft
choice-formation-pushed-convoy = 顶推船队
# status: draft
input-length-overall-m = 总长
# status: draft
input-beam-m = 型宽
# status: draft
input-moulded-depth-m = 型深
# status: draft
input-vessel-scheme = 等级方案
# status: draft
input-vessel-class = 船型等级
# status: draft
input-cargo-containment = 货物围护系统
# status: draft
input-cargo-containment-default-by = cargo_phase
# status: draft
choice-cargo-containment-box-hold = 箱形货舱
# status: draft
choice-cargo-containment-sloped-hopper-hold = 斜壁斗式货舱
# status: draft
choice-cargo-containment-type-c-tanks = C型液货舱
# status: draft
input-block-coefficient = 方形系数
# status: draft
input-min-freeboard-m = 最小干舷
# status: draft
input-bow-rake-length-to-depth-ratio = 首部斜升段长度与型深之比
# status: draft
input-pushed-stern-void-m = 顶推驳船尾部空舱
# status: draft
input-lightship-factor-t-per-m3 = 空船重量系数
# status: draft
input-lightship-factor-t-per-m3-default-by = cargo_containment
# status: draft
input-ruleset = 规范集
# status: draft
choice-ruleset-igc-2g = IGC_2G
# status: draft
input-lpg-carriage-condition = LPG 装运方式
# status: draft
choice-lpg-carriage-condition-fully-pressurised = 全压式
# status: draft
choice-lpg-carriage-condition-semi-pressurised = 半压式（半冷式）
# status: draft
choice-lpg-carriage-condition-fully-refrigerated = 全冷式（低温）
# status: draft
input-fill-ratio = 充装率
# status: draft
input-fill-ratio-default-by = 货物、LPG 装运方式
# status: draft
input-max-tank-od-m = 液货舱最大外径
# status: draft
input-tank-design-pressure-mpa = 液货舱设计压力 (MARVS)
# status: draft
input-tank-design-pressure-mpa-default-by = 货物、LPG 装运方式
# status: draft
input-tank-design-temperature-c = 液货舱设计温度
# status: draft
input-tank-design-temperature-c-default-by = 货物、LPG 装运方式
# status: draft
input-tank-material = 液货舱材料
# status: draft
input-tank-material-default-by = 货物、LPG 装运方式
# status: draft
choice-tank-material-304l = 304L 不锈钢
# status: draft
choice-tank-material-316l = 316L 不锈钢
# status: draft
choice-tank-material-9ni = 9 % 镍钢
# status: draft
choice-tank-material-s690ql1 = S690QL1 调质钢
# status: draft
input-tank-allowable-stress-basis = 液货舱许用应力依据
# status: draft
choice-tank-allowable-stress-basis-igc-4-23-3-1 = IGC 4.23.3.1：Rm/A 与 Re/B 中的较小值
# status: draft
choice-tank-allowable-stress-basis-published-design = IGC 校核之前（S690QL1 取 285 MPa）
# status: draft
input-tank-allowable-stress-mpa = 液货舱许用应力（覆盖值）
# status: draft
input-tank-insulation-type = 液货舱绝热类型
# status: draft
input-tank-insulation-type-default-by = 货物、LPG 装运方式
# status: draft
choice-tank-insulation-type-unspecified = 未指定
# status: draft
choice-tank-insulation-type-none = 无
# status: draft
choice-tank-insulation-type-pu-foam = 聚氨酯泡沫
# status: draft
choice-tank-insulation-type-panel = 绝热板
# status: draft
input-tank-insulation-thickness-mm = 液货舱绝热层厚度
# status: draft
input-tank-insulation-thickness-mm-default-by = 货物、LPG 装运方式
# status: draft
input-tank-structure-factor = 液货舱结构系数
# status: draft
input-tank-structure-factor-default-by = 货物、LPG 装运方式
# status: draft
input-tanks-in-line = 纵向液货舱数
# status: draft
input-tank-support-height-m = 液货舱支座高度
# status: draft
input-tank-dome-height-m = 液货舱穹顶高度
# status: draft
input-tank-dome-diameter-m = 液货舱穹顶直径
# status: draft
input-wing-tank-gap-m = 边舱间隙
# status: draft
input-wing-tank-height-ratio = 边舱高度与液货舱直径之比

# status: draft
input-weather-shield-thickness-mm = 防风雨罩板厚

# status: draft
input-weather-shield-clearance-m = 防风雨罩高出液舱顶的间隙
# status: draft
input-tank-wall-thickness-mm = 液货舱壁厚（覆盖值）
# status: draft
input-optimise-depth = 优化型深
# status: draft
input-hold-wing-void-width-m = 边空舱宽度
# status: draft
input-hold-double-bottom-m = 货舱双层底
# status: draft
input-hopper-slope-h-per-v = 斗式货舱斜度（水平/垂直）
# status: draft
input-hatch-coaming-height-m = 舱口围板高度
# status: draft
input-n-holds = 货舱数
# status: draft
input-bulk-end-bulkhead-m = 端舱壁裕量
# status: draft
input-hold-fill-ratio = 货舱充装率
# status: draft
input-inner-bottom-plate-mm = 内底板厚度
# status: draft
input-hopper-plate-mm = 斗式货舱斜板厚度
# status: draft
input-hold-bulkhead-plate-mm = 货舱舱壁板厚度
# status: draft
input-hatch-coaming-plate-mm = 舱口围板厚度
# status: draft
input-propulsive-efficiency = 推进效率
# status: draft
input-max-speed-kn = 最大航速
# status: draft
input-thruster-count = 推进器数量（上限）
# status: draft
input-thruster-envelope-m = 推进器包络尺寸
# status: draft
input-thruster-clearance-m = 推进器间隙
# status: draft
input-thruster-room-length-m = 推进器舱长度
# status: draft
input-battery-unit-count = 电池单元数
# status: draft
input-battery-unit-count-default-by = cargo_phase
# status: draft
input-accommodation-unit-count = 居住单元数
# status: draft
input-accommodation-unit-count-default-by = cargo_phase
# status: draft
input-unit-height-m = 单元高度
# status: draft
input-unit-side-gap-m = 单元侧向间隙
# status: draft
input-unit-row-gap-m = 单元排间距
# status: draft
input-unit-foundation-m = 单元基座高度
# status: draft
input-wheelhouse-length-m = 驾驶室长度
# status: draft
input-wheelhouse-width-m = 驾驶室宽度
# status: draft
input-wheelhouse-height-m = 驾驶室高度
# status: draft
input-wheelhouse-gap-m = 驾驶室间隙
# status: draft
input-wheelhouse-eye-height-m = 驾驶室眼高（距地板）
# status: draft
input-sightline-margin-m = 视线裕量
# status: draft
input-wheelhouse-max-lift-m = 驾驶室最大升高量
# status: draft
input-air-draft-limit-m = 水上高度限值
# status: draft
input-draught-limit-m = 吃水限值
# status: draft
input-channel-depth-m = 航道水深
# status: draft
input-channel-under-keel-clearance-m = 富余水深
# status: draft
input-channel-bottom-width-m = 航道底宽
# status: draft
input-channel-bank-slope = 岸坡坡度
# status: draft
input-channel-current-kn = 水流流速
# status: draft
input-water-density-t-per-m3 = 水密度
# status: draft
input-squat-safety-margin-m = 航行下沉量安全裕量
# status: draft
input-hull-steel-rate = 船体钢材单价
# status: draft
input-tank-shell-rate = 液货舱壳体单价
# status: draft
input-tank-supports-rate = 液货舱支座单价
# status: draft
input-insulation-rate = 绝热单价
# status: draft
input-piping-rate = 管系与机械单价
# status: draft
input-outfitting-rate = 舾装单价
# status: draft
input-cargo-system-lump = 货物系统
# status: draft
input-paint-anodes-misc = 油漆、阳极及其他
# status: draft
input-battery-rate-per-kwh = 电池单价
# status: draft
input-propulsion-rate-per-kw = 推进装置单价
# status: draft
input-wheelhouse-accom-lump = 驾驶室与居住舱室
# status: draft
input-tug-outfitting-lump = 拖轮舾装
# status: draft
input-yard-margin-fraction = 船厂利润
# status: draft
input-delivery-lump = 交付

## Added after the first draft

# status: draft
input-lpg-loading-limit-basis = LPG 密度与装载限值依据

# status: draft
choice-lpg-loading-limit-basis-igc-15-1-3 = IGC 15.1.3 参考温度（默认）

# status: draft
choice-lpg-loading-limit-basis-igc-15-5-2 = IGC 15.5.2：带压时以 45 C 为参考

# status: draft
choice-lpg-loading-limit-basis-refrigerated-density = 旧法：冷冻密度，按装载时充装率

# status: draft
input-lpg-loading-temperature-c = LPG 装载温度

# status: draft
input-lpg-loading-temperature-c-default-by = 货物、LPG 装运方式
