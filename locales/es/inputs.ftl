# OVCD input labels, Spanish (Latin America). Draft: machine-translated, for native review (locales/REVIEWING.md).

## Grupos

# status: draft
group-required = Obligatorias
# status: draft
group-vessel-class = Clase de embarcación
# status: draft
group-hull-and-containment = Casco y contención
# status: draft
group-gas-tanks = Tanques de gas
# status: draft
group-note-gas-tanks = El diámetro exterior del tanque (el límite de abajo y el resultado) se mide hasta la cara exterior del aislamiento; las holguras y el plano usan esa cara. El diámetro del acero es ese valor menos dos veces el espesor del aislamiento. El espesor de pared se calcula con la fórmula de recipientes a presión, salvo que se ingrese un espesor. Los valores de diseño de abajo dependen de la carga y, para el GLP, de la condición de transporte; cada uno muestra su fuente o indica que es una estimación.
# status: draft
group-dry-bulk-hold = Bodega de granel seco
# status: draft
group-liquid-cargo-tanks = Tanques de carga líquida
# status: draft
group-note-liquid-cargo-tanks = Tanques de carga rectangulares dentro de un doble casco, entre los espacios de aislamiento (cofferdams) de los extremos de la zona de carga. Los anchos del doble casco y la longitud del cofferdam son los mínimos de ADN para un buque de tipo C hasta que existan los conjuntos de reglas para carga líquida; cada valor muestra su fuente o indica que es una estimación. El petróleo crudo es trabajo en desarrollo.
# status: draft
group-aft-block = Bloque de popa
# status: draft
group-waterway-limits = Límites de la vía navegable
# status: draft
group-speed-and-power = Velocidad y potencia
# status: draft
group-cost-rates = Tarifas de costo
# status: draft
group-note-cost-rates = Tarifas de ejemplo: reemplácelas por las suyas.

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
choice-cargo-crude-oil = Petróleo crudo

# status: draft
input-hull-form = Forma del casco
# status: draft
choice-hull-form-raked-box = Caja con lanzamiento

# status: draft
input-formation = Formación
# status: draft
choice-formation-self-propelled = Autopropulsada
# status: draft
choice-formation-pushed-barge = Barcaza empujada
# status: draft
choice-formation-pushed-convoy = Convoy empujado

# status: draft
input-length-overall-m = Eslora total

# status: draft
input-beam-m = Manga

# status: draft
input-moulded-depth-m = Puntal de trazado

# status: draft
input-vessel-scheme = Esquema de clases

# status: draft
input-vessel-class = Clase de embarcación

# status: draft
input-cargo-containment = Contención de la carga
# status: draft
input-cargo-containment-default-by = cargo_phase
# status: draft
choice-cargo-containment-box-hold = Bodega tipo caja
# status: draft
choice-cargo-containment-sloped-hopper-hold = Bodega con tolva inclinada
# status: draft
choice-cargo-containment-type-c-tanks = Tanques tipo C
# status: draft
choice-cargo-containment-integral-tanks = Tanques integrales

# status: draft


# status: draft
input-min-freeboard-m = Francobordo mínimo

# status: draft
input-bow-rake-length-to-depth-ratio = Longitud del lanzamiento de proa sobre puntal

# status: draft
input-bow-rake-max-length-m = Lanzamiento de proa máximo a partir del puntal

# status: draft
input-pushed-stern-void-m = Espacio vacío de popa de una barcaza empujada

# status: draft
input-lightship-factor-t-per-m3 = Factor de peso en rosca
# status: draft
input-lightship-factor-t-per-m3-default-by = cargo_containment

# status: draft
input-lpg-carriage-condition = Condición de transporte del GLP
# status: draft
choice-lpg-carriage-condition-fully-pressurised = Totalmente presurizado
# status: draft
choice-lpg-carriage-condition-semi-pressurised = Semipresurizado (semirrefrigerado)
# status: draft
choice-lpg-carriage-condition-fully-refrigerated = Totalmente refrigerado (criogénico)

# status: draft
input-fill-ratio = Grado de llenado
# status: draft
input-fill-ratio-default-by = Carga, condición de transporte del GLP

# status: draft
input-max-tank-od-m = Diámetro exterior máximo del tanque

# status: draft
input-tank-design-pressure-mpa = Presión de diseño del tanque (MARVS)
# status: draft
input-tank-design-pressure-mpa-default-by = Carga, condición de transporte del GLP

# status: draft
input-tank-design-temperature-c = Temperatura de diseño del tanque
# status: draft
input-tank-design-temperature-c-default-by = Carga, condición de transporte del GLP

# status: draft
input-tank-material = Material del tanque
# status: draft
input-tank-material-default-by = Carga, condición de transporte del GLP
# status: draft
choice-tank-material-304l = Acero inoxidable 304L
# status: draft
choice-tank-material-316l = Acero inoxidable 316L
# status: draft
choice-tank-material-9ni = Acero al 9 % de níquel
# status: draft
choice-tank-material-s690ql1 = Acero templado y revenido S690QL1

# status: draft
input-tank-allowable-stress-basis = Base de la tensión admisible del tanque
# status: draft
choice-tank-allowable-stress-basis-igc-4-23-3-1 = IGC 4.23.3.1: el menor de Rm/A y Re/B
# status: draft
choice-tank-allowable-stress-basis-published-design = Antes de la verificación IGC (S690QL1 a 285 MPa)

# status: draft
input-tank-allowable-stress-mpa = Tensión admisible del tanque (valor manual)

# status: draft
input-tank-insulation-type = Tipo de aislamiento del tanque
# status: draft
input-tank-insulation-type-default-by = Carga, condición de transporte del GLP
# status: draft
choice-tank-insulation-type-unspecified = Sin especificar
# status: draft
choice-tank-insulation-type-none = Ninguno
# status: draft
choice-tank-insulation-type-pu-foam = Espuma de poliuretano
# status: draft
choice-tank-insulation-type-panel = Panel aislante

# status: draft
input-tank-insulation-thickness-mm = Espesor del aislamiento del tanque
# status: draft
input-tank-insulation-thickness-mm-default-by = Carga, condición de transporte del GLP

# status: draft
input-tank-structure-factor = Factor de estructura del tanque
# status: draft
input-tank-structure-factor-default-by = Carga, condición de transporte del GLP

# status: draft
input-tanks-across = Tanques en manga (valor manual)

# status: draft
input-tanks-in-line = Tanques en línea

# status: draft
input-tank-support-height-m = Altura del soporte del tanque

# status: draft
input-tank-dome-height-m = Altura del domo del tanque

# status: draft
input-tank-dome-diameter-m = Diámetro del domo del tanque

# status: draft
input-wing-tank-gap-m = Holgura del tanque lateral

# status: draft
input-wing-tank-height-ratio = Altura del tanque lateral sobre diámetro del tanque

# status: draft
input-min-side-space-m = Espacio lateral mínimo

# status: draft
input-weather-shield-thickness-mm = Espesor de la chapa del techo de protección

# status: draft
input-weather-shield-clearance-m = Holgura del techo de protección sobre los tanques

# status: draft
input-tank-forming-allowance-pct = Sobreespesor de conformado de la chapa del tanque

# status: draft
input-tank-dome-wall-thickness-mm = Espesor de pared del domo del tanque

# status: draft
input-tank-wall-thickness-mm = Espesor de pared del tanque (valor manual)

# status: draft
input-optimise-depth = Optimizar el puntal

# status: draft
input-hold-wing-void-width-m = Ancho del espacio vacío lateral

# status: draft
input-hold-double-bottom-m = Doble fondo de la bodega

# status: draft
input-hopper-slope-h-per-v = Pendiente de la tolva (horizontal sobre vertical)

# status: draft
input-hatch-coaming-height-m = Altura de la brazola de escotilla

# status: draft
input-n-holds = Número de bodegas

# status: draft
input-bulk-end-bulkhead-m = Margen para mamparo de extremo

# status: draft
input-hold-fill-ratio = Grado de llenado de la bodega

# status: draft
input-inner-bottom-plate-mm = Espesor de plancha del fondo interior

# status: draft
input-hopper-plate-mm = Espesor de plancha de la tolva

# status: draft
input-hold-bulkhead-plate-mm = Espesor de plancha del mamparo de bodega

# status: draft
input-hatch-coaming-plate-mm = Espesor de plancha de la brazola de escotilla
# status: draft
input-double-side-m = Ancho del doble costado
# status: draft
input-double-bottom-m = Altura del doble fondo
# status: draft
input-liquid-tanks-in-line = Tanques de carga en línea
# status: draft
input-centreline-bulkhead = Mamparo longitudinal de crujía
# status: draft
input-cofferdam-m = Longitud del cofferdam
# status: draft
input-pump-arrangement = Disposición de las bombas
# status: draft
choice-pump-arrangement-deepwell = Bombas de pozo profundo
# status: draft
choice-pump-arrangement-pump-room = Cámara de bombas
# status: draft
input-pump-room-length-m = Longitud de la cámara de bombas
# status: draft
input-heating-kw-per-m3 = Calefacción de la carga
# status: draft
input-liquid-fill-ratio = Grado de llenado de los tanques de carga

# status: draft
input-propulsive-efficiency = Rendimiento propulsivo

# status: draft
input-max-speed-kn = Velocidad máxima

# status: draft
input-thruster-count = Número de propulsores (máximo)

# status: draft
input-thruster-envelope-m = Envolvente del propulsor

# status: draft
input-thruster-clearance-m = Holgura del propulsor

# status: draft
input-thruster-room-length-m = Longitud de la sala de propulsores

# status: draft
input-battery-unit-count = Unidades de batería
# status: draft
input-battery-unit-count-default-by = cargo_phase

# status: draft
input-accommodation-unit-count = Unidades de alojamiento
# status: draft
input-accommodation-unit-count-default-by = cargo_phase

# status: draft
input-unit-height-m = Altura de la unidad

# status: draft
input-unit-side-gap-m = Separación lateral entre unidades

# status: draft
input-unit-row-gap-m = Separación entre filas de unidades

# status: draft
input-unit-foundation-m = Altura de la base de la unidad

# status: draft
input-wheelhouse-length-m = Longitud del puente de gobierno

# status: draft
input-wheelhouse-width-m = Ancho del puente de gobierno

# status: draft
input-wheelhouse-height-m = Altura del puente de gobierno

# status: draft
input-wheelhouse-gap-m = Separación del puente de gobierno

# status: draft
input-wheelhouse-eye-height-m = Altura de los ojos sobre el piso del puente de gobierno

# status: draft
input-sightline-margin-m = Margen de la línea de visión

# status: draft
input-wheelhouse-max-lift-m = Elevación máxima del puente de gobierno

# status: draft
input-air-draft-limit-m = Límite de calado aéreo

# status: draft
input-lock-length-m = Longitud de la esclusa

# status: draft
input-lock-width-m = Anchura de la esclusa

# status: draft
input-draught-limit-m = Límite de calado

# status: draft
input-channel-depth-m = Profundidad del canal

# status: draft
input-channel-under-keel-clearance-m = Resguardo bajo la quilla (UKC)

# status: draft
input-channel-bottom-width-m = Ancho del fondo del canal

# status: draft
input-channel-bank-slope = Pendiente del talud

# status: draft
input-channel-current-kn = Corriente

# status: draft
input-water-density-t-per-m3 = Densidad del agua

# status: draft
input-squat-safety-margin-m = Margen de seguridad por asentamiento dinámico (squat)

# status: draft
input-hull-steel-rate = Tarifa del acero del casco

# status: draft
input-tank-shell-rate = Tarifa de la envolvente del tanque

# status: draft
input-tank-supports-rate = Tarifa de los soportes del tanque

# status: draft
input-insulation-rate = Tarifa del aislamiento

# status: draft
input-piping-rate = Tarifa de tuberías y maquinaria

# status: draft
input-outfitting-rate = Tarifa de armamento

# status: draft
input-cargo-system-lump = Sistema de carga

# status: draft
input-paint-anodes-misc = Pintura, ánodos y varios

# status: draft
input-battery-rate-per-kwh = Tarifa de baterías

# status: draft
input-propulsion-rate-per-kw = Tarifa de propulsión

# status: draft
input-wheelhouse-accom-lump = Puente de gobierno y alojamiento

# status: draft
input-tug-outfitting-lump = Armamento del remolcador

# status: draft
input-yard-margin-fraction = Margen del astillero

# status: draft
input-delivery-lump = Entrega

## Added after the first draft

# status: draft
input-lpg-loading-limit-basis = Base de la densidad del GLP y del límite de carga

# status: draft
choice-lpg-loading-limit-basis-igc-15-1-3 = Temperatura de referencia IGC 15.1.3 (predeterminada)

# status: draft
choice-lpg-loading-limit-basis-igc-15-5-2 = IGC 15.5.2: referencia de 45 C con presión

# status: draft
choice-lpg-loading-limit-basis-refrigerated-density = Anterior: densidad refrigerada, grado de llenado al cargar

# status: draft
input-lpg-loading-temperature-c = Temperatura de carga del GLP

# status: draft
input-lpg-loading-temperature-c-default-by = Carga, condición de transporte del GLP

# status: draft
input-currency = Moneda

# status: draft
input-price-base-date = Fecha base de precios

# status: draft
input-tank-internals-fraction = Parte de los internos del tanque en el excedente de estructura
