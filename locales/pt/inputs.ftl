# OVCD inputs, Portuguese (Brazil). Draft: machine-translated, for native review (locales/REVIEWING.md).

## Grupos

# status: draft
group-required = Obrigatório
# status: draft
group-vessel-class = Classe de embarcação
# status: draft
group-hull-and-containment = Casco e contenção
# status: draft
group-gas-tanks = Tanques de gás
# status: draft
group-note-gas-tanks = O diâmetro externo do tanque (o limite abaixo e o resultado) é medido na face externa do isolamento; as folgas e o desenho usam essa face. O diâmetro do aço é esse valor menos duas vezes a espessura do isolamento. A espessura da parede é calculada pela fórmula de vasos de pressão, a menos que uma espessura seja digitada. Os valores de projeto abaixo dependem da carga e, para GLP, da condição de transporte; cada um mostra sua fonte ou é marcado como estimativa.
# status: draft
group-dry-bulk-hold = Porão de granel sólido
# status: draft
group-aft-block = Bloco de popa
# status: draft
group-waterway-limits = Limites da hidrovia
# status: draft
group-speed-and-power = Velocidade e potência
# status: draft
group-cost-rates = Taxas de custo
# status: draft
group-note-cost-rates = Taxas de exemplo — substitua pelas suas.

## Entradas

# status: draft
input-cargo = Carga
# status: draft
choice-cargo-lng = GNL
# status: draft
choice-cargo-lng-ethane = GNL / Etano
# status: draft
choice-cargo-lpg-propane = GLP (Propano)
# status: draft
choice-cargo-lpg-butane = GLP (Butano)
# status: draft
choice-cargo-lpg-mix = GLP (60/40 Propano/Butano)
# status: draft
choice-cargo-bauxite = Bauxita

# status: draft
input-hull-form = Forma do casco
# status: draft
choice-hull-form-raked-box = Caixa com lançamento

# status: draft
input-formation = Formação
# status: draft
choice-formation-self-propelled = Autopropulsada
# status: draft
choice-formation-pushed-barge = Barcaça empurrada
# status: draft
choice-formation-pushed-convoy = Comboio empurrado

# status: draft
input-length-overall-m = Comprimento total

# status: draft
input-beam-m = Boca

# status: draft
input-moulded-depth-m = Pontal moldado

# status: draft
input-vessel-scheme = Esquema de classes

# status: draft
input-vessel-class = Classe de embarcação

# status: draft
input-cargo-containment = Contenção da carga
# status: draft
input-cargo-containment-default-by = cargo_phase
# status: draft
choice-cargo-containment-box-hold = Porão em caixa
# status: draft
choice-cargo-containment-sloped-hopper-hold = Porão com tremonha inclinada
# status: draft
choice-cargo-containment-type-c-tanks = Tanques tipo C

# status: draft
input-block-coefficient = Coeficiente de bloco

# status: draft
input-midship-coefficient = Coeficiente da secção mestra

# status: draft
input-min-freeboard-m = Borda livre mínima

# status: draft
input-bow-rake-length-to-depth-ratio = Comprimento do lançamento de proa sobre o pontal

# status: draft
input-pushed-stern-void-m = Espaço vazio de popa da barcaça empurrada

# status: draft
input-lightship-factor-t-per-m3 = Fator de peso leve
# status: draft
input-lightship-factor-t-per-m3-default-by = cargo_containment

# status: draft
input-lpg-carriage-condition = Condição de transporte do GLP
# status: draft
choice-lpg-carriage-condition-fully-pressurised = Totalmente pressurizado
# status: draft
choice-lpg-carriage-condition-semi-pressurised = Semipressurizado (semirrefrigerado)
# status: draft
choice-lpg-carriage-condition-fully-refrigerated = Totalmente refrigerado (criogênico)

# status: draft
input-fill-ratio = Taxa de enchimento
# status: draft
input-fill-ratio-default-by = Carga, condição de transporte do GLP

# status: draft
input-max-tank-od-m = Diâmetro externo máximo do tanque

# status: draft
input-tank-design-pressure-mpa = Pressão de projeto do tanque (MARVS)
# status: draft
input-tank-design-pressure-mpa-default-by = Carga, condição de transporte do GLP

# status: draft
input-tank-design-temperature-c = Temperatura de projeto do tanque
# status: draft
input-tank-design-temperature-c-default-by = Carga, condição de transporte do GLP

# status: draft
input-tank-material = Material do tanque
# status: draft
input-tank-material-default-by = Carga, condição de transporte do GLP
# status: draft
choice-tank-material-304l = Aço inoxidável 304L
# status: draft
choice-tank-material-316l = Aço inoxidável 316L
# status: draft
choice-tank-material-9ni = Aço 9 % níquel
# status: draft
choice-tank-material-s690ql1 = Aço temperado e revenido S690QL1

# status: draft
input-tank-allowable-stress-basis = Base da tensão admissível do tanque
# status: draft
choice-tank-allowable-stress-basis-igc-4-23-3-1 = IGC 4.23.3.1: o menor entre Rm/A e Re/B
# status: draft
choice-tank-allowable-stress-basis-published-design = Antes da verificação IGC (S690QL1 a 285 MPa)

# status: draft
input-tank-allowable-stress-mpa = Tensão admissível do tanque (substituição)

# status: draft
input-tank-insulation-type = Tipo de isolamento do tanque
# status: draft
input-tank-insulation-type-default-by = Carga, condição de transporte do GLP
# status: draft
choice-tank-insulation-type-unspecified = Não especificado
# status: draft
choice-tank-insulation-type-none = Nenhum
# status: draft
choice-tank-insulation-type-pu-foam = Espuma de poliuretano
# status: draft
choice-tank-insulation-type-panel = Painel isolante

# status: draft
input-tank-insulation-thickness-mm = Espessura do isolamento do tanque
# status: draft
input-tank-insulation-thickness-mm-default-by = Carga, condição de transporte do GLP

# status: draft
input-tank-structure-factor = Fator de estrutura do tanque
# status: draft
input-tank-structure-factor-default-by = Carga, condição de transporte do GLP

# status: draft
input-tanks-in-line = Tanques em linha

# status: draft
input-tank-support-height-m = Altura do berço do tanque

# status: draft
input-tank-dome-height-m = Altura do domo do tanque

# status: draft
input-tank-dome-diameter-m = Diâmetro do domo do tanque

# status: draft
input-wing-tank-gap-m = Folga do tanque lateral

# status: draft
input-wing-tank-height-ratio = Altura do tanque lateral sobre o diâmetro do tanque

# status: draft
input-weather-shield-thickness-mm = Espessura da chapa da cobertura de proteção

# status: draft
input-weather-shield-clearance-m = Folga da cobertura de proteção sobre os tanques

# status: draft
input-tank-wall-thickness-mm = Espessura da parede do tanque (substituição)

# status: draft
input-optimise-depth = Otimizar o pontal

# status: draft
input-hold-wing-void-width-m = Largura do espaço vazio lateral

# status: draft
input-hold-double-bottom-m = Fundo duplo do porão

# status: draft
input-hopper-slope-h-per-v = Inclinação da tremonha (horizontal sobre vertical)

# status: draft
input-hatch-coaming-height-m = Altura da braçola da escotilha

# status: draft
input-n-holds = Número de porões

# status: draft
input-bulk-end-bulkhead-m = Margem da antepara de extremidade

# status: draft
input-hold-fill-ratio = Taxa de enchimento do porão

# status: draft
input-inner-bottom-plate-mm = Espessura da chapa do fundo interno

# status: draft
input-hopper-plate-mm = Espessura da chapa da tremonha

# status: draft
input-hold-bulkhead-plate-mm = Espessura da chapa da antepara do porão

# status: draft
input-hatch-coaming-plate-mm = Espessura da chapa da braçola da escotilha

# status: draft
input-propulsive-efficiency = Eficiência propulsiva

# status: draft
input-max-speed-kn = Velocidade máxima

# status: draft
input-thruster-count = Número de propulsores (máximo)

# status: draft
input-thruster-envelope-m = Envelope do propulsor

# status: draft
input-thruster-clearance-m = Folga do propulsor

# status: draft
input-thruster-room-length-m = Comprimento da praça dos propulsores

# status: draft
input-battery-unit-count = Unidades de bateria
# status: draft
input-battery-unit-count-default-by = cargo_phase

# status: draft
input-accommodation-unit-count = Unidades de acomodação
# status: draft
input-accommodation-unit-count-default-by = cargo_phase

# status: draft
input-unit-height-m = Altura da unidade

# status: draft
input-unit-side-gap-m = Folga lateral entre unidades

# status: draft
input-unit-row-gap-m = Folga entre fileiras de unidades

# status: draft
input-unit-foundation-m = Altura da base da unidade

# status: draft
input-wheelhouse-length-m = Comprimento da casa do leme

# status: draft
input-wheelhouse-width-m = Largura da casa do leme

# status: draft
input-wheelhouse-height-m = Altura da casa do leme

# status: draft
input-wheelhouse-gap-m = Folga da casa do leme

# status: draft
input-wheelhouse-eye-height-m = Altura dos olhos na casa do leme acima do piso

# status: draft
input-sightline-margin-m = Margem da linha de visada

# status: draft
input-wheelhouse-max-lift-m = Elevação máxima da casa do leme

# status: draft
input-air-draft-limit-m = Limite de calado aéreo

# status: draft
input-draught-limit-m = Limite de calado

# status: draft
input-channel-depth-m = Profundidade do canal

# status: draft
input-channel-under-keel-clearance-m = Folga abaixo da quilha

# status: draft
input-channel-bottom-width-m = Largura do fundo do canal

# status: draft
input-channel-bank-slope = Inclinação da margem

# status: draft
input-channel-current-kn = Correnteza

# status: draft
input-water-density-t-per-m3 = Densidade da água

# status: draft
input-squat-safety-margin-m = Margem de segurança para squat

# status: draft
input-hull-steel-rate = Taxa do aço do casco

# status: draft
input-tank-shell-rate = Taxa do costado do tanque

# status: draft
input-tank-supports-rate = Taxa dos berços dos tanques

# status: draft
input-insulation-rate = Taxa do isolamento

# status: draft
input-piping-rate = Taxa de tubulações e máquinas

# status: draft
input-outfitting-rate = Taxa de equipamento

# status: draft
input-cargo-system-lump = Sistema de carga

# status: draft
input-paint-anodes-misc = Pintura, ânodos e diversos

# status: draft
input-battery-rate-per-kwh = Taxa da bateria

# status: draft
input-propulsion-rate-per-kw = Taxa da propulsão

# status: draft
input-wheelhouse-accom-lump = Casa do leme e acomodações

# status: draft
input-tug-outfitting-lump = Equipamento do rebocador

# status: draft
input-yard-margin-fraction = Margem do estaleiro

# status: draft
input-delivery-lump = Entrega

## Added after the first draft

# status: draft
input-lpg-loading-limit-basis = Base da densidade do GLP e do limite de carregamento

# status: draft
choice-lpg-loading-limit-basis-igc-15-1-3 = Temperatura de referência IGC 15.1.3 (padrão)

# status: draft
choice-lpg-loading-limit-basis-igc-15-5-2 = IGC 15.5.2: referência de 45 C quando pressurizado

# status: draft
choice-lpg-loading-limit-basis-refrigerated-density = Anterior: densidade refrigerada, grau de enchimento no carregamento

# status: draft
input-lpg-loading-temperature-c = Temperatura de carregamento do GLP

# status: draft
input-lpg-loading-temperature-c-default-by = Carga, condição de transporte do GLP

# status: draft
input-currency = Moeda

# status: draft
input-price-base-date = Data-base dos preços
