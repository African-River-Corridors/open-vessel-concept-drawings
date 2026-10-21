# OVCD engine messages, Portuguese (Brazil). Draft: machine-translated, for native review (locales/REVIEWING.md).


## crates/ovcd-core/src/aft_arrangement.rs

# status: draft
check-thrusters-fit-beam = { $count } x propulsores de { $envelope } m precisam de { $needed } m em uma boca de { $beam } m
# status: draft
check-requested-thruster-count-fits = { $fit } de { $requested } propulsores pedidos cabem; { $requested } precisariam de { $needed } m de { $beam } m de boca
# status: draft
check-containers-fit-beam = { $count } x unidades de 40 ft lado a lado precisam de { $needed } m em uma boca de { $beam } m
# status: draft
check-units-fit-in-one-row = { $units } unidades em { $rows } fileira(s) de { $per_row }; uma segunda fileira acrescenta { $added } m ao bloco de popa e sai diretamente da zona de carga
# status: draft
check-wheelhouse-fits-beam = casa do leme de { $width } m de largura em uma boca de { $beam } m
# status: draft
check-accom-beside-wheelhouse = { $units } x unidades de 40 ft ao lado de uma casa do leme de { $wheelhouse } m precisam de { $needed } m de { $beam } m de boca
# status: draft
check-accom-beside-wheelhouse-rows = { $units } x unidades de 40 ft ao lado de uma casa do leme de { $wheelhouse } m precisam de { $needed } m de { $beam } m de boca; elas ficam em uma fileira a ré da cabine, o que alonga o bloco
# status: draft
check-gensets-fit-beam = { $count } x geradores de { $width } m lado a lado precisam de { $needed } m em uma boca de { $beam } m
# status: draft
check-gensets-fit-under-deck = gerador { $height } m + 1.00 m de fundo duplo e estrutura do convés (ESTIMATIVA) <= pontal moldado { $depth } m
# status: draft
check-gensets-carry-load-with-one-in-reserve = { $running } de { $count } x geradores de { $set_kwe } kWe suportam { $load } kWe
# status: draft
warn-aft-gas-stern-estimates = Bloco de popa movido a gás: os envelopes dos geradores além do motor publicado (alternador, base) e as folgas da praça de máquinas e da sala da GVU são ESTIMATIVAS (sem arranjo geral do fornecedor). A descarga de gases e o mastro de suspiro de combustível não entram no calado aéreo (supostos rebatíveis). O combustível vem do boil-off da carga (Código IGC, capítulo 16); não há tanque de combustível separado no bloco.
# status: draft
check-wheelhouse-sees-over-cargo = olho elevado { $eye } m >= obstrução { $obstruction } m + margem de { $margin } m (elevação usada { $lift } m de { $max_lift } m)
# status: draft
warn-aft-sightline-lift = A linha de visada precisa de { $lift } m de elevação; a coluna dá { $max_lift } m. Eleve a coluna ou baixe o topo da carga.

## crates/ovcd-core/src/barge_capacity.rs

# status: draft
check-feasible-geometry = o volume de carga deve ser positivo (o tanque/porão cabe)
# status: draft
check-not-overloaded = a carga limitada por volume deve flutuar dentro do pontal moldado
# status: draft
check-freeboard-ok = borda livre { $freeboard } m >= exigida { $required } m
# status: draft
check-cargo-zone-positive = zona de carga { $zone } m = LOA { $loa } - espaço vazio de proa { $bow_void } m - espaço vazio de popa { $stern_void } m
# status: draft
check-air-draft-condition-ballasted-light = leve com lastro
# status: draft
check-air-draft-condition-light = leve
# status: draft
check-air-draft-within-limit = calado aéreo { $condition } { $governing } m <= limite { $limit } m (carregado é { $laden } m e NÃO é o caso governante)
# status: draft
warn-barge-pushed = Barcaça EMPURRADA: tripulação, bateria e propulsão estão no rebocador e NÃO entram no peso leve, na capacidade nem no calado aéreo deste casco. O rebocador é uma embarcação separada e deve ser orçado e verificado quanto ao calado aéreo separadamente.
# status: draft
check-gm-positive = GM intacto da tremonha { $gm } m >= 0.15 m (primeira ordem; KG_light @0.5D)
# status: draft
check-intact-gm-reported-cargo = GM intacto { $gm } m >= 0.15 m com a carga indicada { $cargo } t, calado { $draught } m (caixa de primeira ordem; KG da carga com o porão cheio; KG_light @0.5D)
# status: draft
warn-barge-bulk-gm = O GM de granel é de primeira ordem (KG de peso leve suposto); deslocamento da carga e ângulo de repouso não modelados — confirme com um caderno completo de estabilidade intacta.

## crates/ovcd-core/src/barge_cost.rs

# status: draft
check-masses-positive = entradas do modelo de massas presentes
# status: draft
check-no-reliquefaction-in-scope = o sistema de carga exclui a planta de reliquefação e de nitrogênio, conforme o conceito operacional de acúmulo de pressão
# status: draft
check-fabrication-rates-are-estimates = as taxas de fabricação/fornecimento são estimativas do usuário; substitua-as por cotações do estaleiro antes de confiar no total

## crates/ovcd-core/src/electric_energy.rs

# status: draft
check-service-below-schijf = serviço { $service } kn fica abaixo do limite de Schijf { $limit } kn
# status: draft
check-service-below-schijf-open-water = sem limite confinado (águas abertas)
# status: draft
check-energy-positive = energia { $energy } kWh/km na velocidade de serviço
# status: draft
check-resistance-coeffs-first-order = a resistência em águas confinadas (residual/Lackenby/bloqueio) é uma estimativa de primeira ordem (RS1) — o atrito ITTC-1957 é padrão; calibre com ensaio de modelo/CFD antes do uso para classificação.

## crates/ovcd-core/src/gas_arrangement.rs

# status: draft
check-tank-length-over-inner-diameter = comprimento do tanque { $length } m / diâmetro interno { $diameter } m = { $ratio } <= { $max }
# status: draft
check-freeboard-at-payload = borda livre { $freeboard } m no calado com carga { $draught } m >= exigida { $required } m
# status: draft
check-stability = GM { $gm } m (fluido), GZ máx. { $max_gz } m a { $angle }°
# status: draft
check-stability-fails = GM { $gm } m (fluido), GZ máx. { $max_gz } m a { $angle }° — reprova: { $fails }
# status: draft
check-stability-not-assessed = não foi possível avaliar a estabilidade: { $reason }

## crates/ovcd-core/src/gas_separation.rs

# status: draft
check-cofferdam-min-width = coferdã { $cofferdam } m >= { $min } m (cl. 3.6.13)
# status: draft
check-opening-setback = separação a ré { $aft } m >= { $min } m da área de carga até qualquer entrada ou janela que abra (cl. 3.3.7 / 3.3.11)
# status: draft
check-fwd-end-bulkhead-space = espaço da antepara de extremidade de vante { $fwd } m >= { $min } m ({ $tank_ends }, cl. 3.5.3)
# status: draft
check-fwd-end-bulkhead-space-domed-ends = extremidades de tanque abauladas
# status: draft
check-fwd-end-bulkhead-space-plane-ends = extremidades de tanque planas
# status: draft
warn-separation-clauses = As cláusulas de separação são citadas das regras de classificação para embarcações de vias navegáveis interiores (Pt 4 Ch 4 Sec 3, Jul 2022). Confirme com o texto do ADN antes da liberação.

## crates/ovcd-core/src/gas_stability.rs

# status: draft
stab-limit-concept-stage = Fase conceitual: seção prismática em caixa, alturas de pesos de primeira ordem, sem aberturas de alagamento progressivo.
# status: draft
stab-limit-no-wind-or-damage = Sem adernamento por vento e sem estabilidade em avaria. Ambos são exigidos antes da aprovação: a estabilidade em avaria pelo ADN 9.3.1.15 e pelo IGC 2.7, o adernamento por vento pelo critério meteorológico do Código IS 2008 (parte A 2.3).
# status: draft
stab-limit-stern-block-mass = A massa do bloco de popa autopropulsado não está no peso leve do gás, então o KG e o deslocamento a excluem; o GM real pode ser menor.
# status: draft
stab-limit-weight-split = A divisão de pesos soma { $split } t contra peso leve { $lightship } t.

## crates/ovcd-core/src/journey_derating.rs

# status: draft
check-capacity-is-volume-bound = a capacidade de entrada deve ser um projeto viável limitado por volume (não sobrecarregado)
# status: draft
check-floats = o canal deve permitir flutuar alguma carga (T_max { $t_max } m contra calado leve)
# status: draft
check-ukc-consistent = o pontal de projeto deve ser >= UKC (informe profundidade + UKC, nunca um T_max já reduzido)
# status: draft
check-utilisation-reasonable = utilização do volume { $utilisation } >= mínimo { $floor }
# status: draft
warn-journey-draft-bound = Limitado pelo calado: retire { $shed } t de { $capacity } t de capacidade em volume para flutuar com T_max = { $t_max } m (calado a plena carga { $full_load_draft } m).

## crates/ovcd-core/src/resistance.rs

# status: draft
check-service-speed-below-limiting = { $environment }: serviço { $service } kn <= 0.95 x limite de Schijf { $limit } kn
# status: draft
check-power-within-installed = { $environment }: potência { $power } kW <= instalada { $installed } kW
# status: draft
check-blockage-in-range = { $environment }: fator de bloqueio n = { $blockage } deve estar em (0, 1)
# status: draft
check-coefficients-are-first-order = O ajuste residual Cr/Cf, Lackenby em águas rasas, o fator de bloqueio e o fator de forma 1.02 são estimativas de primeira ordem não calibradas (RS1) — o atrito ITTC-1957 é padrão.

## crates/ovcd-core/src/squat.rs

# status: draft
check-keel-clears-bed = UKC dinâmica { $ukc } m = estática { $static_ukc } m - squat { $squat } m ({ $method }) a { $speed } kn
# status: draft
check-keel-clears-bed-with-margin = UKC dinâmica { $ukc } m contra uma margem de trabalho de { $margin } m
# status: draft
check-froude-depth-subcritical = Froude de profundidade { $froude }; acima de 0.7 o escoamento se aproxima do crítico e as duas formas de squat perdem validade
# status: draft
check-blockage-within-barrass-range = fator de bloqueio { $blockage }; Barrass é ajustado abaixo de cerca de 0.35 e extrapola acima disso
# status: draft
check-methods-agree-within-50pct = Barrass { $barrass } m contra ICORELS { $icorels } m, { $spread }% de diferença
# status: draft
warn-squat-icorels-not-evaluated = ICORELS não avaliado: Froude de profundidade { $froude } igual ou acima de 0.99
# status: draft
warn-squat-open-water = nenhuma área de canal informada, então o fator de bloqueio é 0 e Barrass retorna 0 — este é o caso de ÁGUAS ABERTAS e o squat é então apenas ICORELS

## crates/ovcd-core/src/vessel_standards.rs

# status: draft
check-unique-ids = cada chave corresponde ao seu standard_id
# status: draft
check-unique-ids-mismatch = chave/standard_id não correspondem: { $keys }
# status: draft
check-ranges-ordered = cada faixa (mín., máx.) está em ordem
# status: draft
check-ranges-ordered-inverted = faixa invertida: { $keys }
# status: draft
check-positive-dimensions = todas as bocas e comprimentos são positivos
# status: draft
check-positive-dimensions-bad = dimensão não positiva: { $keys }
# status: draft
check-convoy-beam-consistent = boca do comboio = boca da unidade x n_abreast
# status: draft
check-convoy-beam-consistent-bad = a boca não concorda com n_abreast: { $keys }
# status: draft
check-source-cited = cada classe cita sua tabela
# status: draft
check-source-cited-missing = sem fonte: { $keys }

## crates/ovcd-core/src/waves.rs

# status: draft
check-stawave1-short-wave-validity = lambda/Lpp = { $ratio } (lambda { $wavelength } m em Lpp { $lpp } m). O STAwave-1 supõe ondas curtas, lambda/Lpp < 0.5; fora disso ele SUBESTIMA, e perto de lambda/Lpp = 1 pode subestimar em mais de 2x
# status: draft
check-wave-height-within-screening-range = H_s { $hs } m; acima de cerca de 3 m uma fórmula de triagem não é a ferramenta certa, e a redução voluntária de velocidade em geral governa de qualquer forma
# status: draft
check-bow-length-supplied = L_BWL não informado, suposto { $bow_length } m = 0.2 x Lpp (proa rombuda de barcaça). Uma proa mais fina reduz a resistência adicional, então esta é a suposição conservadora
# status: draft
warn-waves-period-estimated = nenhum período de onda informado; T_p estimado em { $tp } s a partir de H_s, o que define apenas a verificação de validade, não a resistência
# status: draft
warn-waves-beam-seas-floor = mar de través: este módulo retorna um PISO, porque a perda por deriva, leme e manutenção de rumo não é modelada e domina fora da proa

## crates/ovcd-format/src/class_input.rs

# status: draft
msg-class-scheme = { $input } { $class } pertence ao esquema { $family }, não a { $scheme }
# status: draft
msg-class-none = o esquema { $scheme } não tem classe para uma formação { $kind }, então nenhuma classe se aplica
# status: draft
msg-class-moved = { $input } { $typed } não comporta este comprimento e esta boca; a embarcação se enquadra em { $class }
# status: draft
msg-class-beyond = este comprimento e esta boca excedem a maior classe { $scheme } ({ $class })
# status: draft
msg-class-draught-above = { $input } = { $value } está acima do calado da classe { $class } ({ $limit } m, o topo da sua faixa)
# status: draft
msg-class-clearance-above = { $input } = { $value } está acima da menor altura livre sob ponte da classe { $class } ({ $limit } m)

## crates/ovcd-format/src/run.rs

# status: draft
msg-convoy-not-available = resultados do comboio ainda não disponíveis

## crates/ovcd-format/src/simulate.rs

# status: draft
warn-channel-no-bottom-width = O canal tem profundidade mas não tem largura de fundo, por isso o afundamento dinâmico (squat), a velocidade limite de Schijf e o fator de bloqueio não são verificados, e as curvas de velocidade são só para águas profundas. O limite de calado usa a profundidade. Indique a largura de fundo do canal para os verificar.
# status: draft
name-governs-draught = calado — { $limit }
# status: draft
check-feasible-arrangement = Nenhum arranjo passa em todas as verificações neste casco. O mais próximo é mostrado. Ele reprova em: { $fails }.

## crates/ovcd-format/src/text_ids.rs

# status: draft
name-condition-laden = Carregado
# status: draft
name-condition-half-laden = Meia carga
# status: draft
name-condition-light = Leve
# status: draft
name-criterion-area-0-30 = Área 0–30°
# status: draft
name-criterion-area-0-40 = Área 0–40°
# status: draft
name-criterion-area-30-40 = Área 30–40°
# status: draft
name-criterion-gz-at-30 = GZ a ≥ 30°
# status: draft
name-criterion-angle-of-max-gz = Ângulo de GZ máx.
# status: draft
name-criterion-initial-gm = GM inicial (fluido)
# status: draft
name-criterion-floats = Flutua com borda livre
# status: draft
name-check-stability-laden = Estabilidade — carregado
# status: draft
name-check-stability-half-laden = Estabilidade — meia carga
# status: draft
name-check-stability-light = Estabilidade — leve
# status: draft
name-check-stability-not-assessed = Estabilidade — não avaliada
# status: draft
name-check-cargo = Carga > 0
# status: draft
name-check-feasible-arrangement = Arranjo viável
# status: draft
name-check-tank-length = comprimento do tanque sobre o diâmetro
# status: draft
name-check-ok = OK
# status: draft
name-curve-deep-water = Águas profundas
# status: draft
name-governs-hold-volume = volume do porão
# status: draft
name-governs-tank-volume = volume do tanque
# status: draft
name-governs-no-cargo-area = sem área de carga (casco mais curto que seus espaços fixos de extremidade)
# status: draft
name-vessel-box = Barcaça de granel sólido com porão em caixa
# status: draft
name-vessel-hopper = Barcaça de granel sólido com tremonha inclinada
# status: draft
name-vessel-gas = Barcaça de gás, tanques IMO tipo C
# status: draft
name-cargo-lightship-alone = O peso leve sozinho excede o calado permitido.
# status: draft
name-cargo-nothing-fits = Nada cabe: alongue ou alargue o casco.

## crates/ovcd-format/src/vdesign.rs

# status: draft
msg-file-not-json = o arquivo não é JSON: { $error }
# status: draft
msg-file-the-file = o arquivo
# status: draft
msg-file-duplicate-key = `{ $key }` aparece duas vezes em { $place }; uma chave só pode aparecer uma vez
# status: draft
msg-file-not-object = o arquivo deve conter um objeto JSON
# status: draft
msg-file-format = o arquivo precisa de `"format": "vdesign"`
# status: draft
msg-file-version-unsupported = format_version { $version } não é suportada; esta ferramenta lê as versões { $oldest } a { $newest }
# status: draft
msg-file-version-missing = o arquivo precisa de um `format_version` inteiro
# status: draft
msg-file-engine-version = o arquivo precisa de uma string não vazia `engine_version`
# status: draft
msg-file-units = `units` deve ser "SI", não { $units }
# status: draft
msg-file-unknown-key = a chave de nível superior desconhecida `{ $key }` é ignorada
# status: draft
msg-file-no-options = o arquivo precisa de uma lista `options`
# status: draft
msg-file-empty = um projeto precisa de pelo menos uma Opção
# status: draft
msg-option-no-name = a Opção { $number } precisa de uma string não vazia `name`
# status: draft
msg-option-names-differ = duas Opções se chamam '{ $option }'; os nomes devem ser diferentes
# status: draft
msg-option-no-inputs = precisa de um objeto `inputs`
# status: draft
msg-option-unknown-key = a chave desconhecida `{ $key }` é ignorada
# status: draft
msg-results-hash-shape = `results_hash` deve ter 64 dígitos hexadecimais
# status: draft
msg-results-hash-text = `results_hash` deve ser texto
# status: draft
msg-ruleset-exact-renamed = IGC_2G_EXACT agora é IGC_2G: a mesma regra por tanque do IGC 2.4.1.1
# status: draft
msg-ruleset-per-tank = IGC_2G agora usa a regra por tanque do IGC 2.4.1.1 (d(Vc) a partir do volume bruto de cada tanque, com o domo, medido de chapa a chapa conforme 2.4.2), então os resultados podem mudar
# status: draft
msg-input-note = { $input }: { $note }
# status: draft
msg-v1-adn-g = { $input } ADN_G não é mais oferecido (o motor aplica apenas as regras IGC aos tanques de gás); este arquivo da versão 1 foi aberto como IGC_2G
# status: draft
msg-v1-tank-cap = { $input } não é mais uma entrada (não há limite de volume do tanque); foi removida
# status: draft
msg-v1-max-tank-od = este arquivo da versão 1 não tem { $input }; agora usa o padrão { $value } m (a versão 1 usava um limite fixo de 12 m, então os resultados podem mudar)
# status: draft
msg-marker-source = `source` deve ser "typed", "default" ou "class"
# status: draft
msg-marker-value = uma entrada marcada precisa de um `value`
# status: draft
msg-marker-only = uma entrada marcada contém apenas `value` e `source`
# status: draft
msg-unknown-input = entrada desconhecida `{ $key }`
# status: draft
msg-input-unused-cargo = { $input } não é usada para a carga { $cargo }; ignorada
# status: draft
msg-input-unused-containment = { $input } não é usada para uma contenção da carga { $containment }; ignorada
# status: draft
msg-input-unused-formation = { $input } não é usada para uma formação { $formation }; ignorada
# status: draft
msg-input-required = falta a entrada obrigatória { $input }
# status: draft
msg-input-filled = { $input } não estava gravada no arquivo; foi preenchida com o valor { $value } do perfil de valores padrão, marcada como padrão
# status: draft
msg-legacy-dome = este arquivo não tem { $input }; agora ela contém { $value } m (marcada como padrão). O domo agora conta em Vc para d(Vc) (IGC 2.4.1.1), então os resultados podem mudar
# status: draft
msg-legacy-carriage = este arquivo não tem { $input } para a carga de GLP { $cargo }; é lida como { $condition } (marcada como padrão). Os valores do tanque que faltam no arquivo mantêm o que o motor usava antes (pressão de projeto, material, isolamento, fator de estrutura), então os resultados não mudam; defina a condição de transporte e redefina essas entradas para adotar os padrões dessa condição
# status: draft
msg-legacy-fixed-one = este arquivo não tem { $input }; agora ela contém o valor fixo que o motor usava antes de ser uma entrada (marcada como padrão), então os resultados não mudam
# status: draft
msg-legacy-fixed-many = este arquivo não tem { $inputs }; cada uma agora contém o valor fixo que o motor usava antes de ser uma entrada (marcada como padrão), então os resultados não mudam
# status: draft
msg-inputs-not-stored = entradas não gravadas no arquivo: { $inputs }. Um projeto salvo contém cada entrada como usada, então uma edição do perfil de valores padrão não pode alterá-lo; execute `ovcd fill` para gravá-las a partir do perfil de valores padrão, marcadas como padrão
# status: draft
msg-profile-case-keyed = `default_by_{ $by }` para { $key }
# status: draft
msg-profile-case-default = um `default`
# status: draft
msg-profile-no-default = o perfil de valores padrão '{ $profile }' não dá padrão para { $input } ({ $case } está faltando)
# status: draft
msg-profile-no-table-default-carriage = o perfil de valores padrão '{ $profile }' não dá padrão para { $input } quando a contenção da carga é { $containment }, a carga { $cargo } e a condição de transporte { $condition } (nenhuma linha de `{ $key }.default_by_table` se aplica)
# status: draft
msg-profile-no-table-default = o perfil de valores padrão '{ $profile }' não dá padrão para { $input } quando a contenção da carga é { $containment }, a carga { $cargo } (nenhuma linha de `{ $key }.default_by_table` se aplica)
# status: draft
msg-profile-no-depth-ratio = o perfil de valores padrão '{ $profile }' não dá razão pontal/boca para a carga { $group } (`depth_to_beam_ratio.default_by_cargo_group.{ $group }` está faltando)
# status: draft
msg-number-or-null = { $input } deve ser um número ou null (nenhum)
# status: draft
msg-bool = { $input } deve ser true ou false
# status: draft
msg-choice = { $input } = { $value } deve ser um de: { $choices }
# status: draft
msg-string = { $input } deve ser uma string
# status: draft
msg-currency = { $input } deve ser um código de moeda de três letras maiúsculas, como USD
# status: draft
msg-price-base-date = { $input } deve ser uma data como AAAA, AAAA-MM ou AAAA-MM-DD, ou null (nenhuma)
# status: draft
msg-scheme = { $input } = { $value } não é um esquema com uma classe de embarcação que possa definir uma embarcação (veja `ovcd schema`)
# status: draft
msg-class-cannot-define = { $input } = { $value } não pode definir uma embarcação: a classe { $class } não dá comprimento nem boca, ou sua formação ({ $formation }) não é uma que um projeto tenha
# status: draft
msg-class-unknown = { $input } = { $value } não é uma classe de embarcação (veja `ovcd schema`)
# status: draft
msg-class-id = { $input } deve ser um id de classe ou null
# status: draft
msg-formation-object = { $input } deve ser um objeto com `kind`
# status: draft
msg-formation-kind = { $input } deve ser um de: { $choices }
# status: draft
msg-formation-only-kind = { $input }: um { $kind } contém apenas `kind`; `abreast` e `in_line` são para um pushed_convoy
# status: draft
msg-formation-count = { $input } deve ser um número inteiro para um pushed_convoy
# status: draft
msg-formation-convoy-only = { $input }: um pushed_convoy contém apenas `kind`, `abreast` e `in_line`
# status: draft
msg-hard-range = { $input } = { $value } está fora da faixa: deve ser { $bound } (limite rígido)
# status: draft
msg-soft-range = { $input } = { $value } está fora da faixa usual de { $min } a { $max } (limite flexível); aceito
# status: draft
msg-slender-hull = { $input } = { $length } é { $ratio } vezes { $depth_input } = { $depth }, acima de cerca de { $limit }: um casco muito esbelto para o seu pontal (limite flexível); aceito
# status: draft
msg-containment-cargo = { $input } = { $containment } não pode transportar { $cargo } ({ $phase }); use { $fits }
# status: draft
msg-freeboard-depth = { $input } = { $freeboard } deve ser menor que { $depth_input } = { $depth } (limite rígido)
# status: draft
msg-whole-number = { $input } = { $value } deve ser um número inteiro (limite rígido)
# status: draft
msg-insulation-disagree = { $input } é { $kind }, mas { $thickness_input } é { $thickness }: o tipo e a espessura não concordam (verificação flexível); aceito
# status: draft
msg-stress-from-input = { $input }
# status: draft
msg-stress-from-basis = a base de tensão admissível `published_design`
# status: draft
msg-stress-from-material = o material
# status: draft
msg-stress-above-igc = { $from } dá uma tensão admissível de { $stress } MPa, acima do limite do IGC 4.23.3.1 para { $material } de { $limit } MPa, o menor entre Rm/A = { $rm } / { $a } e Re/B = { $re } / { $b } (verificação flexível); aceito
# status: draft
msg-freeboard-search = { $input } = { $freeboard } deve ser no máximo { $limit }, o casco mais profundo que { $search_input } pesquisa (limite rígido)
# status: draft
msg-channel-ukc = { $input } = { $depth } deve ser pelo menos { $ukc_input } = { $ukc } (limite rígido)
# status: draft
msg-hold-width = { $input } = { $beam } deixa uma largura de porão de { $width } m após os dois espaços vazios laterais de { $void } m; { $reason }
# status: draft
msg-negative-hold = um porão negativo dá massa de carga e calado negativos (limite rígido)
# status: draft
msg-hold-height = { $input } = { $depth } deixa uma altura de porão de { $height } m (pontal menos o fundo duplo de { $double_bottom } m, mais a braçola de { $coaming } m); { $reason }
# status: draft
msg-number = { $input } deve ser um número
# status: draft
msg-input-error = { $input }: { $error }
# status: draft
msg-option = Opção '{ $option }': { $message }

## Check names
# status: draft
check-name-thrusters-fit-beam = os propulsores cabem na boca
# status: draft
check-name-requested-thruster-count-fits = o número de propulsores pedido cabe
# status: draft
check-name-containers-fit-beam = os contêineres cabem na boca
# status: draft
check-name-units-fit-in-one-row = as unidades cabem em uma fileira
# status: draft
check-name-wheelhouse-fits-beam = a casa do leme cabe na boca
# status: draft
check-name-accom-beside-wheelhouse = acomodações ao lado da casa do leme
# status: draft
check-name-gensets-fit-beam = os geradores cabem na boca
# status: draft
check-name-gensets-fit-under-deck = os geradores cabem sob o convés
# status: draft
check-name-gensets-carry-load-with-one-in-reserve = os geradores suportam a carga com um de reserva
# status: draft
check-name-wheelhouse-sees-over-cargo = a casa do leme vê por cima da carga
# status: draft
check-name-feasible-geometry = geometria viável
# status: draft
check-name-not-overloaded = não sobrecarregado
# status: draft
check-name-freeboard-ok = borda livre ok
# status: draft
check-name-cargo-zone-positive = zona de carga positiva
# status: draft
check-name-air-draft-within-limit = calado aéreo dentro do limite
# status: draft
check-name-gm-positive = GM positivo
# status: draft
check-name-intact-gm-at-reported-cargo = GM intacto com a carga indicada
# status: draft
check-name-masses-positive = massas positivas
# status: draft
check-name-no-reliquefaction-in-scope = sem reliquefação no escopo
# status: draft
check-name-fabrication-rates-are-estimates = as taxas de fabricação são estimativas
# status: draft
check-name-service-below-schijf = serviço abaixo de Schijf
# status: draft
check-name-energy-positive = energia positiva
# status: draft
check-name-resistance-coeffs-first-order = coeficientes de resistência de primeira ordem
# status: draft
check-name-tank-length-over-diameter = comprimento do tanque sobre diâmetro
# status: draft
check-name-freeboard-at-payload = borda livre com carga útil
# status: draft
check-name-stability-not-assessed = Estabilidade — não avaliada
# status: draft
check-name-cofferdam-min-width = largura mínima do coferdã
# status: draft
check-name-opening-setback = afastamento das aberturas
# status: draft
check-name-fwd-end-bulkhead-space = espaço da antepara de extremidade de vante
# status: draft
check-name-capacity-is-volume-bound = capacidade limitada por volume
# status: draft
check-name-floats = flutua
# status: draft
check-name-ukc-consistent = UKC consistente
# status: draft
check-name-utilisation-reasonable = utilização razoável
# status: draft
check-name-coefficients-are-first-order = os coeficientes são de primeira ordem
# status: draft
check-name-keel-clears-bed = a quilha passa livre do fundo
# status: draft
check-name-keel-clears-bed-with-margin = a quilha passa livre do fundo com margem
# status: draft
check-name-froude-depth-subcritical = Froude de profundidade subcrítico
# status: draft
check-name-blockage-within-barrass-range = fator de bloqueio dentro da faixa de Barrass
# status: draft
check-name-methods-agree-within-50pct = os métodos concordam dentro de 50 %
# status: draft
check-name-unique-ids = ids únicos
# status: draft
check-name-ranges-ordered = faixas em ordem
# status: draft
check-name-positive-dimensions = dimensões positivas
# status: draft
check-name-convoy-beam-consistent = boca do comboio consistente
# status: draft
check-name-source-cited = fonte citada
# status: draft
check-name-stawave1-short-wave-validity = validade do stawave1 para ondas curtas
# status: draft
check-name-wave-height-within-screening-range = altura de onda dentro da faixa de triagem
# status: draft
check-name-bow-length-supplied = comprimento da proa informado

## Added after the first draft

# status: draft
msg-loading-above-marvs = { $input } = { $value }: a pressão de vapor de { $cargo } nessa temperatura é { $pressure } bar, acima do MARVS de { $marvs } bar absolutos; o tanque é carregado até o limite de enchimento, não acima (verificação flexível); aceito
