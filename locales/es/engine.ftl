# OVCD engine messages, Spanish (Latin America). Draft: machine-translated, for native review (locales/REVIEWING.md).

## crates/ovcd-core/src/aft_arrangement.rs

# status: draft
check-thrusters-fit-beam = { $count } propulsores de { $envelope } m necesitan { $needed } m a lo ancho de una manga de { $beam } m
# status: draft
check-requested-thruster-count-fits = Caben { $fit } de { $requested } propulsores pedidos; { $requested } necesitarían { $needed } m de { $beam } m de manga
# status: draft
check-containers-fit-beam = { $count } unidades de 40 ft de costado necesitan { $needed } m a lo ancho de una manga de { $beam } m
# status: draft
check-units-fit-in-one-row = { $units } unidades en { $rows } fila(s) de { $per_row }; una segunda fila agrega { $added } m al bloque de popa y los quita directamente de la zona de carga
# status: draft
check-wheelhouse-fits-beam = puente de gobierno de { $width } m de ancho en una manga de { $beam } m
# status: draft
check-accom-beside-wheelhouse = { $units } unidades de 40 ft junto a un puente de gobierno de { $wheelhouse } m necesitan { $needed } m de { $beam } m de manga
# status: draft
check-accom-beside-wheelhouse-rows = { $units } unidades de 40 ft junto a un puente de gobierno de { $wheelhouse } m necesitan { $needed } m de { $beam } m de manga; en su lugar van en una fila a popa de la cabina, lo que alarga el bloque
# status: draft
check-gensets-fit-beam = { $count } grupos electrógenos de { $width } m de costado necesitan { $needed } m a lo ancho de una manga de { $beam } m
# status: draft
check-gensets-fit-under-deck = grupo electrógeno { $height } m + 1.00 m de doble fondo y estructura de cubierta (ESTIMACIÓN) <= puntal de trazado { $depth } m
# status: draft
check-gensets-carry-load-with-one-in-reserve = { $running } de { $count } grupos de { $set_kwe } kWe soportan { $load } kWe
# status: draft
warn-aft-gas-stern-estimates = Bloque de popa con combustible gas: las envolventes de los grupos electrógenos más allá del motor publicado (alternador, bastidor) y las holguras de la sala de máquinas y de la sala GVU son ESTIMACIONES (no hay plano general del fabricante). La toma de escape y el mástil de venteo de combustible no están en el calado aéreo (se suponen abatibles). El combustible se toma del boil-off de la carga (Código IGC, capítulo 16); no hay un tanque de combustible separado en el bloque.
# status: draft
check-wheelhouse-sees-over-cargo = ojo elevado { $eye } m >= obstrucción { $obstruction } m + { $margin } m de margen (elevación usada { $lift } m de { $max_lift } m)
# status: draft
warn-aft-sightline-lift = La línea de visión necesita { $lift } m de elevación; la columna da { $max_lift } m. Eleve la columna o baje el tope de la carga.

## crates/ovcd-core/src/barge_capacity.rs

# status: draft
check-feasible-geometry = el volumen de carga debe ser positivo (caben tanques/bodega)
# status: draft
check-not-overloaded = la carga limitada por volumen debe flotar dentro del puntal de trazado
# status: draft
check-freeboard-ok = francobordo { $freeboard } m >= requerido { $required } m
# status: draft
check-cargo-zone-positive = zona de carga { $zone } m = LOA { $loa } - vacío de proa { $bow_void } m - vacío de popa { $stern_void } m
# status: draft
check-air-draft-condition-ballasted-light = en rosca con lastre
# status: draft
check-air-draft-condition-light = en rosca
# status: draft
check-air-draft-within-limit = calado aéreo { $condition } { $governing } m <= límite { $limit } m (cargada es { $laden } m y NO es el caso determinante)
# status: draft
warn-barge-pushed = Barcaza EMPUJADA: la tripulación, la batería y la propulsión están en el remolcador y NO están en el peso en rosca, la capacidad ni el calado aéreo de este casco. El remolcador es una embarcación separada y debe costearse y verificarse por calado aéreo por separado.
# status: draft
check-gm-positive = GM intacto de la tolva { $gm } m >= 0.15 m (primer orden; KG_light @0.5D)
# status: draft
warn-barge-bulk-gm = El GM a granel es de primer orden (KG de peso en rosca supuesto); no se modelan el corrimiento de la carga ni el ángulo de reposo. Confírmelo con un cuadernillo completo de estabilidad intacta.

## crates/ovcd-core/src/barge_cost.rs

# status: draft
check-masses-positive = hay entradas del modelo de masas
# status: draft
check-no-reliquefaction-in-scope = el sistema de carga excluye la planta de relicuefacción y de nitrógeno, según el concepto operativo de acumulación de presión
# status: draft
check-fabrication-rates-are-estimates = las tarifas de fabricación y suministro son estimaciones del usuario; reemplácelas por cotizaciones del astillero antes de confiar en el total

## crates/ovcd-core/src/electric_energy.rs

# status: draft
check-service-below-schijf = servicio { $service } kn queda por debajo del límite de Schijf { $limit } kn
# status: draft
check-service-below-schijf-open-water = sin límite por aguas restringidas (aguas abiertas)
# status: draft
check-energy-positive = energía { $energy } kWh/km a velocidad de servicio
# status: draft
check-resistance-coeffs-first-order = la resistencia en aguas restringidas (residual/Lackenby/factor de bloqueo) es una estimación de primer orden (RS1); la fricción ITTC-1957 es estándar. Calíbrela con un ensayo de modelo o CFD antes de usarla para clasificación.

## crates/ovcd-core/src/gas_arrangement.rs

# status: draft
check-tank-length-over-diameter = longitud del tanque { $length } m / diámetro exterior { $diameter } m = { $ratio } <= { $max }
# status: draft
check-freeboard-at-payload = francobordo { $freeboard } m al calado de carga { $draught } m >= requerido { $required } m
# status: draft
check-stability = GM { $gm } m (fluido), GZ máx. { $max_gz } m a { $angle }°
# status: draft
check-stability-fails = GM { $gm } m (fluido), GZ máx. { $max_gz } m a { $angle }°: no cumple: { $fails }
# status: draft
check-stability-not-assessed = no se pudo evaluar la estabilidad: { $reason }

## crates/ovcd-core/src/gas_separation.rs

# status: draft
check-cofferdam-min-width = coferdán { $cofferdam } m >= { $min } m (cl. 3.6.13)
# status: draft
check-opening-setback = separación a popa { $aft } m >= { $min } m desde la zona de carga hasta cualquier entrada o ventana que se pueda abrir (cl. 3.3.7 / 3.3.11)
# status: draft
check-fwd-end-bulkhead-space = espacio del mamparo de extremo de proa { $fwd } m >= { $min } m ({ $tank_kind }, cl. 3.5.3 / 3.6.3)
# status: draft
check-fwd-end-bulkhead-space-pressure-tank = tanque a presión
# status: draft
check-fwd-end-bulkhead-space-standard = estándar
# status: draft
warn-separation-clauses = Las cláusulas de separación se citan de las reglas de clasificación para buques de vías navegables interiores (Pt 4 Ch 4 Sec 3, Jul 2022). Confírmelas con el texto del ADN antes de publicar.

## crates/ovcd-core/src/gas_stability.rs

# status: draft
stab-limit-concept-stage = Etapa conceptual: sección prismática de caja, alturas de pesos de primer orden, sin aberturas de inundación progresiva.
# status: draft
stab-limit-no-wind-or-damage = Sin escora por viento y sin estabilidad en averías (ADN 9.3.1.15 / IGC 2.7 exigen ambas antes de la aprobación).
# status: draft
stab-limit-stern-block-mass = La masa del bloque de popa autopropulsado no está en el peso en rosca de gas, así que KG y el desplazamiento no la incluyen; el GM real podría ser menor.
# status: draft
stab-limit-weight-split = El desglose de pesos suma { $split } t frente a un peso en rosca de { $lightship } t.

## crates/ovcd-core/src/journey_derating.rs

# status: draft
check-capacity-is-volume-bound = la capacidad de entrada debe ser un diseño factible limitado por volumen (no sobrecargado)
# status: draft
check-floats = el canal debe permitir flotar algo de carga (T_max { $t_max } m frente al calado en rosca)
# status: draft
check-ukc-consistent = la profundidad de diseño debe ser >= UKC (pase profundidad + UKC, nunca un T_max ya reducido)
# status: draft
check-utilisation-reasonable = aprovechamiento del volumen { $utilisation } >= mínimo { $floor }
# status: draft
warn-journey-draft-bound = Limitado por calado: se descargan { $shed } t de { $capacity } t de capacidad por volumen para flotar a T_max = { $t_max } m (calado a plena carga { $full_load_draft } m).

## crates/ovcd-core/src/resistance.rs

# status: draft
check-service-speed-below-limiting = { $environment }: servicio { $service } kn <= 0.95 x límite de Schijf { $limit } kn
# status: draft
check-power-within-installed = { $environment }: potencia { $power } kW <= instalada { $installed } kW
# status: draft
check-blockage-in-range = { $environment }: el factor de bloqueo n = { $blockage } debe estar en (0, 1)
# status: draft
check-coefficients-are-first-order = El ajuste residual Cr/Cf, Lackenby en aguas poco profundas, el factor de bloqueo y el factor de forma 1.02 son estimaciones de primer orden sin calibrar (RS1); la fricción ITTC-1957 es estándar.

## crates/ovcd-core/src/squat.rs

# status: draft
check-keel-clears-bed = UKC dinámico { $ukc } m = UKC estático { $static_ukc } m - asentamiento dinámico (squat) { $squat } m ({ $method }) a { $speed } kn
# status: draft
check-keel-clears-bed-with-margin = UKC dinámico { $ukc } m frente a un margen de trabajo de { $margin } m
# status: draft
check-froude-depth-subcritical = Froude de profundidad { $froude }; por encima de 0.7 el flujo se acerca al crítico y ambas fórmulas de squat pierden validez
# status: draft
check-blockage-within-barrass-range = factor de bloqueo { $blockage }; Barrass está ajustado por debajo de aproximadamente 0.35 y extrapola por encima
# status: draft
check-methods-agree-within-50pct = Barrass { $barrass } m frente a ICORELS { $icorels } m, { $spread }% de diferencia
# status: draft
warn-squat-icorels-not-evaluated = ICORELS no evaluado: Froude de profundidad { $froude } igual o mayor que 0.99
# status: draft
warn-squat-open-water = no se dio el área del canal, así que el factor de bloqueo es 0 y Barrass da 0. Este es el caso de AGUAS ABIERTAS y el asentamiento dinámico (squat) es entonces solo ICORELS

## crates/ovcd-core/src/vessel_standards.rs

# status: draft
check-unique-ids = cada clave coincide con su standard_id
# status: draft
check-unique-ids-mismatch = clave y standard_id no coinciden: { $keys }
# status: draft
check-ranges-ordered = cada rango (mín., máx.) está ordenado
# status: draft
check-ranges-ordered-inverted = rango invertido: { $keys }
# status: draft
check-positive-dimensions = todas las mangas y esloras son positivas
# status: draft
check-positive-dimensions-bad = dimensión no positiva: { $keys }
# status: draft
check-convoy-beam-consistent = manga del convoy = manga de la unidad x n_abreast
# status: draft
check-convoy-beam-consistent-bad = la manga no concuerda con n_abreast: { $keys }
# status: draft
check-source-cited = cada clase cita su tabla
# status: draft
check-source-cited-missing = sin fuente: { $keys }

## crates/ovcd-core/src/waves.rs

# status: draft
check-stawave1-short-wave-validity = lambda/Lpp = { $ratio } (lambda { $wavelength } m sobre Lpp { $lpp } m). STAwave-1 supone olas cortas, lambda/Lpp < 0.5; fuera de ese rango SUBESTIMA, y cerca de lambda/Lpp = 1 puede hacerlo por más de 2x
# status: draft
check-wave-height-within-screening-range = H_s { $hs } m; por encima de unos 3 m una fórmula de evaluación preliminar no es la herramienta adecuada y, de todos modos, suele determinar la reducción voluntaria de velocidad
# status: draft
check-bow-length-supplied = no se dio L_BWL; se supone { $bow_length } m = 0.2 x Lpp (proa roma de barcaza). Una proa más fina reduce la resistencia añadida, así que este supuesto es conservador
# status: draft
warn-waves-period-estimated = no se dio el período de ola; T_p estimado en { $tp } s a partir de H_s, que solo fija la verificación de validez, no la resistencia
# status: draft
warn-waves-beam-seas-floor = mar de través o casi: este módulo da un LÍMITE INFERIOR, porque no se modela la pérdida por deriva, timón y mantenimiento del rumbo, que domina fuera de la proa

## crates/ovcd-format/src/class_input.rs

# status: draft
msg-class-scheme = { $input } { $class } pertenece al esquema { $family }, no a { $scheme }
# status: draft
msg-class-none = el esquema { $scheme } no tiene clase para una formación { $kind }, así que no aplica ninguna clase
# status: draft
msg-class-moved = { $input } { $typed } no admite esta eslora y esta manga; la embarcación pertenece a { $class }
# status: draft
msg-class-beyond = esta eslora y esta manga superan la mayor clase de { $scheme } ({ $class })
# status: draft
msg-class-draught-above = { $input } = { $value } supera el calado de la clase { $class } ({ $limit } m, el tope de su rango)
# status: draft
msg-class-clearance-above = { $input } = { $value } supera el gálibo de puente más bajo de la clase { $class } ({ $limit } m)

## crates/ovcd-format/src/run.rs

# status: draft
msg-convoy-not-available = los resultados del convoy aún no están disponibles

## crates/ovcd-format/src/simulate.rs

# status: draft
check-air-draft-limit = { $air_draft } m frente al límite { $limit } m
# status: draft
name-governs-draught = calado: { $limit }
# status: draft
check-feasible-arrangement = Ninguna disposición cumple todas las verificaciones en este casco. Se muestra la más cercana. No cumple: { $fails }.

## crates/ovcd-format/src/text_ids.rs

# status: draft
name-condition-laden = Cargada
# status: draft
name-condition-half-laden = Media carga
# status: draft
name-condition-light = En rosca
# status: draft
name-criterion-area-0-30 = Área 0–30°
# status: draft
name-criterion-area-0-40 = Área 0–40°
# status: draft
name-criterion-area-30-40 = Área 30–40°
# status: draft
name-criterion-gz-at-30 = GZ a ≥ 30°
# status: draft
name-criterion-angle-of-max-gz = Ángulo de GZ máx.
# status: draft
name-criterion-initial-gm = GM inicial (fluido)
# status: draft
name-criterion-floats = Flota con francobordo
# status: draft
name-check-stability-laden = Estabilidad: cargada
# status: draft
name-check-stability-half-laden = Estabilidad: media carga
# status: draft
name-check-stability-light = Estabilidad: en rosca
# status: draft
name-check-stability-not-assessed = Estabilidad: no evaluada
# status: draft
name-check-cargo = Carga > 0
# status: draft
name-check-feasible-arrangement = Disposición factible
# status: draft
name-check-air-draft = Calado aéreo (vacía, determinante)
# status: draft
name-check-tank-length = longitud del tanque sobre diámetro
# status: draft
name-check-ok = OK
# status: draft
name-curve-deep-water = Aguas profundas
# status: draft
name-governs-hold-volume = volumen de la bodega
# status: draft
name-governs-tank-volume = volumen de los tanques
# status: draft
name-governs-no-cargo-area = sin zona de carga (casco más corto que sus espacios fijos de extremo)
# status: draft
name-vessel-box = Barcaza de granel seco con bodega tipo caja
# status: draft
name-vessel-hopper = Barcaza de granel seco con tolva inclinada
# status: draft
name-vessel-gas = Barcaza de gas, tanques IMO tipo C
# status: draft
name-cargo-lightship-alone = Solo el peso en rosca ya supera el calado permitido.
# status: draft
name-cargo-nothing-fits = No cabe nada: alargue o ensanche el casco.

## crates/ovcd-format/src/vdesign.rs

# status: draft
msg-file-not-json = el archivo no es JSON: { $error }
# status: draft
msg-file-the-file = el archivo
# status: draft
msg-file-duplicate-key = `{ $key }` aparece dos veces en { $place }; una clave puede aparecer una sola vez
# status: draft
msg-file-not-object = el archivo debe contener un objeto JSON
# status: draft
msg-file-format = el archivo necesita `"format": "vdesign"`
# status: draft
msg-file-version-unsupported = format_version { $version } no es compatible; esta herramienta lee las versiones { $oldest } a { $newest }
# status: draft
msg-file-version-missing = el archivo necesita un entero `format_version`
# status: draft
msg-file-engine-version = el archivo necesita una cadena no vacía `engine_version`
# status: draft
msg-file-units = `units` debe ser "SI", no { $units }
# status: draft
msg-file-unknown-key = la clave de nivel superior desconocida `{ $key }` se ignora
# status: draft
msg-file-no-options = el archivo necesita una lista `options`
# status: draft
msg-file-empty = un diseño necesita al menos una opción
# status: draft
msg-option-no-name = la opción { $number } necesita una cadena no vacía `name`
# status: draft
msg-option-names-differ = dos opciones se llaman '{ $option }'; los nombres deben ser distintos
# status: draft
msg-option-no-inputs = necesita un objeto `inputs`
# status: draft
msg-option-unknown-key = la clave desconocida `{ $key }` se ignora
# status: draft
msg-results-hash-shape = `results_hash` debe tener 64 dígitos hexadecimales
# status: draft
msg-results-hash-text = `results_hash` debe ser texto
# status: draft
msg-ruleset-exact-renamed = IGC_2G_EXACT ahora es IGC_2G: la misma regla por tanque de IGC 2.4.1.1
# status: draft
msg-ruleset-per-tank = IGC_2G ahora usa la regla por tanque de IGC 2.4.1.1 (d(Vc) a partir del volumen bruto de cada tanque, con el domo incluido, medido de plancha a plancha según 2.4.2), así que los resultados pueden cambiar
# status: draft
msg-input-note = { $input }: { $note }
# status: draft
msg-v1-adn-g = { $input } ADN_G ya no se ofrece (el motor aplica solo las reglas IGC a los tanques de gas); este archivo de versión 1 se abrió como IGC_2G
# status: draft
msg-v1-tank-cap = { $input } ya no es una entrada (no hay límite de volumen del tanque); se eliminó
# status: draft
msg-v1-max-tank-od = este archivo de versión 1 no tiene { $input }; ahora usa el valor por defecto { $value } m (la versión 1 usaba un límite fijo de 12 m, así que los resultados pueden cambiar)
# status: draft
msg-marker-source = `source` debe ser "typed", "default" o "class"
# status: draft
msg-marker-value = una entrada marcada necesita un `value`
# status: draft
msg-marker-only = una entrada marcada contiene solo `value` y `source`
# status: draft
msg-unknown-input = entrada desconocida `{ $key }`
# status: draft
msg-input-unused-cargo = { $input } no se usa para la carga { $cargo }; se ignora
# status: draft
msg-input-unused-containment = { $input } no se usa para una contención de la carga { $containment }; se ignora
# status: draft
msg-input-unused-formation = { $input } no se usa para una formación { $formation }; se ignora
# status: draft
msg-input-required = falta la entrada obligatoria { $input }
# status: draft
msg-input-filled = { $input } no estaba guardada en el archivo; se escribió el valor del perfil de valores por defecto { $value }, marcado como por defecto
# status: draft
msg-legacy-dome = este archivo no tiene { $input }; ahora contiene { $value } m (marcado como por defecto). El domo ahora cuenta en Vc para d(Vc) (IGC 2.4.1.1), así que los resultados pueden cambiar
# status: draft
msg-legacy-carriage = este archivo no tiene { $input } para la carga de GLP { $cargo }; se lee como { $condition } (marcado como por defecto). Los valores del tanque que faltan en el archivo conservan lo que el motor usaba antes (presión de diseño, material, aislamiento, factor de estructura), así que los resultados no cambian; defina la condición de transporte y restablezca esas entradas para tomar los valores por defecto de esa condición
# status: draft
msg-legacy-fixed-one = este archivo no tiene { $input }; ahora contiene el valor fijo que el motor usaba antes de que fuera una entrada (marcado como por defecto), así que los resultados no cambian
# status: draft
msg-legacy-fixed-many = este archivo no tiene { $inputs }; cada una ahora contiene el valor fijo que el motor usaba antes de que fuera una entrada (marcado como por defecto), así que los resultados no cambian
# status: draft
msg-inputs-not-stored = entradas no guardadas en el archivo: { $inputs }. Un diseño guardado contiene cada entrada tal como se usó, así que un cambio en el perfil de valores por defecto no puede modificarlo; ejecute `ovcd fill` para escribirlas desde el perfil de valores por defecto, marcadas como por defecto
# status: draft
msg-profile-case-keyed = `default_by_{ $by }` para { $key }
# status: draft
msg-profile-case-default = un `default`
# status: draft
msg-profile-no-default = el perfil de valores por defecto '{ $profile }' no da un valor por defecto para { $input } (falta { $case })
# status: draft
msg-profile-no-table-default-carriage = el perfil de valores por defecto '{ $profile }' no da un valor por defecto para { $input } cuando la contención de la carga es { $containment }, la carga { $cargo } y la condición de transporte { $condition } (ninguna fila de `{ $key }.default_by_table` corresponde)
# status: draft
msg-profile-no-table-default = el perfil de valores por defecto '{ $profile }' no da un valor por defecto para { $input } cuando la contención de la carga es { $containment } y la carga { $cargo } (ninguna fila de `{ $key }.default_by_table` corresponde)
# status: draft
msg-profile-no-depth-ratio = el perfil de valores por defecto '{ $profile }' no da una relación puntal/manga para la carga { $group } (falta `depth_to_beam_ratio.default_by_cargo_group.{ $group }`)
# status: draft
msg-number-or-null = { $input } debe ser un número o null (ninguno)
# status: draft
msg-bool = { $input } debe ser true o false
# status: draft
msg-hull-form = { $input } = { $value } no es compatible; por ahora solo existe raked_box
# status: draft
msg-choice = { $input } = { $value } debe ser uno de: { $choices }
# status: draft
msg-string = { $input } debe ser una cadena
# status: draft
msg-scheme = { $input } = { $value } no es un esquema con una clase de embarcación que pueda definir una embarcación (vea `ovcd schema`)
# status: draft
msg-class-cannot-define = { $input } = { $value } no puede definir una embarcación: la clase { $class } no da eslora ni manga, o su formación ({ $formation }) no es una que tenga un diseño
# status: draft
msg-class-unknown = { $input } = { $value } no es una clase de embarcación (vea `ovcd schema`)
# status: draft
msg-class-id = { $input } debe ser un id de clase o null
# status: draft
msg-formation-object = { $input } debe ser un objeto con un `kind`
# status: draft
msg-formation-kind = { $input } debe ser uno de: { $choices }
# status: draft
msg-formation-only-kind = { $input }: un { $kind } contiene solo `kind`; `abreast` e `in_line` son para un pushed_convoy
# status: draft
msg-formation-count = { $input } debe ser un número entero para un pushed_convoy
# status: draft
msg-formation-convoy-only = { $input }: un pushed_convoy contiene solo `kind`, `abreast` e `in_line`
# status: draft
msg-hard-range = { $input } = { $value } está fuera de rango: debe ser { $bound } (límite estricto)
# status: draft
msg-soft-range = { $input } = { $value } está fuera del rango habitual { $min } a { $max } (límite recomendado); se acepta
# status: draft
msg-slender-hull = { $input } = { $length } es { $ratio } veces { $depth_input } = { $depth }, más de aproximadamente { $limit }: un casco muy esbelto para su puntal (límite recomendado); se acepta
# status: draft
msg-containment-cargo = { $input } = { $containment } no puede transportar { $cargo } ({ $phase }); use { $fits }
# status: draft
msg-freeboard-depth = { $input } = { $freeboard } debe ser menor que { $depth_input } = { $depth } (límite estricto)
# status: draft
msg-whole-number = { $input } = { $value } debe ser un número entero (límite estricto)
# status: draft
msg-insulation-disagree = { $input } es { $kind } pero { $thickness_input } es { $thickness }: el tipo y el espesor no concuerdan (verificación recomendada); se acepta
# status: draft
msg-stress-from-input = { $input }
# status: draft
msg-stress-from-basis = la base de tensión admisible `published_design`
# status: draft
msg-stress-from-material = el material
# status: draft
msg-stress-above-igc = { $from } da una tensión admisible de { $stress } MPa, por encima del límite de IGC 4.23.3.1 para { $material } de { $limit } MPa, el menor de Rm/A = { $rm } / { $a } y Re/B = { $re } / { $b } (verificación recomendada); se acepta
# status: draft
msg-freeboard-search = { $input } = { $freeboard } debe ser como máximo { $limit }, el casco más profundo que explora { $search_input } (límite estricto)
# status: draft
msg-channel-ukc = { $input } = { $depth } debe ser al menos { $ukc_input } = { $ukc } (límite estricto)
# status: draft
msg-hold-width = { $input } = { $beam } deja un ancho de bodega de { $width } m después de los dos espacios vacíos laterales de { $void } m; { $reason }
# status: draft
msg-negative-hold = una bodega negativa da una masa de carga y un calado negativos (límite estricto)
# status: draft
msg-hold-height = { $input } = { $depth } deja una altura de bodega de { $height } m (el puntal menos el doble fondo de { $double_bottom } m, más la brazola de { $coaming } m); { $reason }
# status: draft
msg-number = { $input } debe ser un número
# status: draft
msg-input-error = { $input }: { $error }
# status: draft
msg-option = Opción '{ $option }': { $message }

## Check names
# status: draft
check-name-thrusters-fit-beam = los propulsores caben en la manga
# status: draft
check-name-requested-thruster-count-fits = cabe el número de propulsores pedido
# status: draft
check-name-containers-fit-beam = los contenedores caben en la manga
# status: draft
check-name-units-fit-in-one-row = las unidades caben en una fila
# status: draft
check-name-wheelhouse-fits-beam = el puente de gobierno cabe en la manga
# status: draft
check-name-accom-beside-wheelhouse = alojamiento junto al puente de gobierno
# status: draft
check-name-gensets-fit-beam = los grupos electrógenos caben en la manga
# status: draft
check-name-gensets-fit-under-deck = los grupos electrógenos caben bajo cubierta
# status: draft
check-name-gensets-carry-load-with-one-in-reserve = los grupos electrógenos soportan la carga con uno en reserva
# status: draft
check-name-wheelhouse-sees-over-cargo = el puente de gobierno ve por encima de la carga
# status: draft
check-name-feasible-geometry = geometría factible
# status: draft
check-name-not-overloaded = no sobrecargado
# status: draft
check-name-freeboard-ok = francobordo correcto
# status: draft
check-name-cargo-zone-positive = zona de carga positiva
# status: draft
check-name-air-draft-within-limit = calado aéreo dentro del límite
# status: draft
check-name-gm-positive = GM positivo
# status: draft
check-name-masses-positive = masas positivas
# status: draft
check-name-no-reliquefaction-in-scope = sin relicuefacción en el alcance
# status: draft
check-name-fabrication-rates-are-estimates = las tarifas de fabricación son estimaciones
# status: draft
check-name-service-below-schijf = servicio por debajo de Schijf
# status: draft
check-name-energy-positive = energía positiva
# status: draft
check-name-resistance-coeffs-first-order = coeficientes de resistencia de primer orden
# status: draft
check-name-tank-length-over-diameter = longitud del tanque sobre diámetro
# status: draft
check-name-freeboard-at-payload = francobordo con carga útil
# status: draft
check-name-stability-not-assessed = Estabilidad — no evaluada
# status: draft
check-name-cofferdam-min-width = ancho mínimo del coferdán
# status: draft
check-name-opening-setback = separación de aberturas
# status: draft
check-name-fwd-end-bulkhead-space = espacio del mamparo de extremo de proa
# status: draft
check-name-capacity-is-volume-bound = capacidad limitada por volumen
# status: draft
check-name-floats = flota
# status: draft
check-name-ukc-consistent = UKC coherente
# status: draft
check-name-utilisation-reasonable = aprovechamiento razonable
# status: draft
check-name-coefficients-are-first-order = los coeficientes son de primer orden
# status: draft
check-name-keel-clears-bed = la quilla libra el fondo
# status: draft
check-name-keel-clears-bed-with-margin = la quilla libra el fondo con margen
# status: draft
check-name-froude-depth-subcritical = Froude de profundidad subcrítico
# status: draft
check-name-blockage-within-barrass-range = factor de bloqueo dentro del rango de Barrass
# status: draft
check-name-methods-agree-within-50pct = los métodos coinciden dentro del 50 %
# status: draft
check-name-unique-ids = ids únicos
# status: draft
check-name-ranges-ordered = rangos ordenados
# status: draft
check-name-positive-dimensions = dimensiones positivas
# status: draft
check-name-convoy-beam-consistent = manga del convoy coherente
# status: draft
check-name-source-cited = fuente citada
# status: draft
check-name-stawave1-short-wave-validity = validez de stawave1 para olas cortas
# status: draft
check-name-wave-height-within-screening-range = altura de ola dentro del rango de evaluación preliminar
# status: draft
check-name-bow-length-supplied = longitud de proa indicada

## Added after the first draft

# status: draft
msg-loading-above-marvs = { $input } = { $value }: la presión de vapor de { $cargo } a esa temperatura es { $pressure } bar, por encima del MARVS de { $marvs } bar absolutos; el tanque se carga hasta el límite de llenado, no más (verificación recomendada); aceptado
