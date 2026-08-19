# Roadmap grio

> Document maintenu au fil du développement. Chaque fonctionnalité livrée
> coche sa case, met à jour le code (`///`), la doc (`README.md`) et cette
> roadmap. Sources d'inspiration : guide Gradio *Blocks and Event Listeners*
> et les fonctions *Queuing, Streaming Outputs, Streaming Inputs, Alerts,
> Progress Bars*.

## Où on en est (base)

Déjà en place :
- [x] Moteur : serveur `axum` + rendu UI (CSS3 + JS vanilla, registre `MG.register`)
- [x] Composants : `Text`, `Slider`, `Output`, `Markdown`, `Button`
- [x] Conteneurs : `Row`, `Column`, `Panel` + racine
- [x] Événements : `on_submit`, `on_change(id)`, `on_click(id)`, `on_event(nom)` + bus `ctx.emit`
- [x] Rôle déclaratif `Role::Input/Output` → API REST auto (`/api/predict`, `/api/schema`)
- [x] Console bavarde (`[http]`, `[ws]`, `[api]`) désactivable (`.quiet()`)
- [x] Docs : `#![warn(missing_docs)]` + `README.md`

Légende : **[P0]** prioritaire · **[P1]** souhaitable · **[P2]** à l'étude

---

## Phase 1 — Interaction avancée (inspiration : Blocks) · [P0]

> **Livrée.** Écart assumé : le **flux** n'est pas déclaré à la Gradio
> (`inputs=`/`outputs=` à l'appel) mais par chaînage **`.flow(inputs,
> outputs)`** appliqué au dernier handler — même effet (scoping strict des
> lectures/écritures). Le **chaînage** `.then/.success/.failure` s'attache lui
> aussi au dernier handler. Tous les critères ont été joués à la main sur les
> tests WebSocket (`examples/blocks.rs`) : voir le détail de chaque item.

### 1.1 Événements par composant avec flux déclarés · ✅
`.flow(["a","b"], ["cmp_out"])` déclare les entrées lisibles et les sorties
écrivables d'un handler ; tout accès hors liste est **rejeté en lecture**
(`get` → erreur) et **ignoré en écriture** (`set`/`set_prop`/`append`/
`progress`). Plusieurs flux indépendants coexistent sans `on_submit`.
- **Fichiers** : `app.rs` (`HandlerDef.inputs/outputs`, `App::flow`), `context.rs` (`set_flow`, gardes)
- **Accepté quand** : démo « a > b / b > a » de Gradio répliquée dans
  `examples/blocks.rs` — deux boutons, deux handlers scopés (testé).

### 1.2 Chaînage d'événements `.then()` / `.success()` / `.failure()` · ✅
Chaque handler principal accepte une **chaîne** de maillons : `then` (toujours),
`success` (si succès), `failure` (si échec — un `failure` qui réussit
**récupère** et réactive les `success` suivants).
- **Fichiers** : `app.rs` (`Sibling`, `RunCond`, `handler.chain`, `run_handler`)
- **Accepté quand** : chatbot dans `examples/blocks.rs` —
  `on_submit(user_fn).then(bot_fn)` affiche la réponse **en streaming** après le
  message (testé : fragments `Vous : …` puis `Bot : ****** (réponse simulée)`).

### 1.3 Données d'événement exposées au handler · ✅
`ctx.event() -> Option<&WireEvent>` expose la cible (`c`), l'action (`e`), les
données (`d`) et l'instantané (`v`). Le `WireEvent` est enregistré dans le
`Context` par `server::run_event`.
- **Fichiers** : `context.rs` (`Context.event`, accesseur), `server.rs` (clone du wire)
- **Accepté quand** : le handler `cmp_lt` affiche `(vu depuis `cmp_lt`)` — testé.

### 1.4 `gr.skip()` / patch de toute prop (`gr.update`) · ✅
- `ctx.skip("id")` / `ctx.unskip` : les écritures suivantes sur `id` sont
  ignorées — testé (out_b figé alors que out_a est mis à jour).
- `ctx.set_prop("id", "visible"|"label", v)` était déjà là ; le front `apply`
  des sorties/boutons gère désormais `visible`/`disabled`/`label` — testé
  (masquage d'un `Output` + changement de libellé d'un bouton).
- **Fichiers** : `context.rs` (`skip`/`unskip`/`skipped`), `app.js`, `styles.css`

### 1.5 Interactivité explicite · ✅
`.interactive(bool)` sur `Text` et `Slider` : prop transmise en `props`,
champ **grisé et non éditable** (`disabled`) mais toujours inclus dans le
snapshot d'entrées.
- **Fichiers** : `components.rs` (Text/Slider), `app.js` (disabled), `styles.css` (.mg-disabled)
- **Accepté quand** : `Text::new("ro").interactive(false)` figé dans
  `examples/blocks.rs` (rendu vérifié au HTML).

### 1.6 Événement `load` (montage de la page) · ✅
`App::on_load(fn)` : émis par le client à l'**ouverture du WebSocket**
(`{t:'event', c:'', e:'load'}`), distribué comme un événement classique,
`server` loggue `load (page)`.
- **Fichiers** : `events.rs` (`EventName::Load`), `app.rs` (`on_load`),
  `app.js` (émission), `server.rs` (bannière + log)
- **Accepté quand** : testé — à la connexion WS, `load_note` reçoit
  « Page montée — événement `load` reçu par le serveur ».

### 1.7 Multi-déclencheurs `gr.on` · ✅
`App::on("click"|"change", [ids…], fn)` lie la **même** fonction (partagée via
`Arc<HandlerFn>`) à plusieurs composants ; `ctx.event()` différencie la cible.
- **Fichiers** : `app.rs` (`App::on`, `HandlerFn = Arc<…>`)
- **Accepté quand** : deux boutons « Option A/B » → le dernier cliqué est
  affiché (`Dernier clic : opt_a`) — testé.

### 1.8 Layout avancé : onglets et repli · ✅
- `Tabs::new(id).tab(label, builder)` : barre + panneaux, **pilotés client**
  (JS bascule `mg-active`, ouvrable au rendu, indexés `data-i`).
- `Accordion::new(id).section(label, builder)` : `<details>/<summary>` natifs —
  repli intégré navigateur, prop `open` pour déplier la première section.
- **Fichiers** : `components.rs` (SectionBuilder, Tabs, Accordion),
  `server.rs` (rendu), `app.js` (câblage onglets), `styles.css` (.mg-tabs-*,
  .mg-accordion-*)
- **Accepté quand** : `examples/blocks.rs` — 2 onglets + 2 sections,
  structure vérifiée dans le HTML servi.

---

## Phase 2 — Temps réel (Queuing, Streaming Outputs, Alerts, Progress) · [P0]

> **Livré.** Écart d'implémentation assumé : les handlers restent **synchrones**
> — le moteur les exécute sur `spawn_blocking` (payload `tokio`) dans une file
> sérialisée, donc `sleep`/boucles longues ne gèlent plus rien. L'annulation se
> fait à l'**enfilement** (un re-déclenchement de la même cible+action pose le
> flag immédiatement), et non à la consommation — nécessaire car la file est
> séquentielle.
> Vérifié : build zéro warning, streaming dédupliqué (pas de doublon entre le
> push temps réel et la réponse finale), annulation + alertes + progress testés
> via WebSocket sur `examples/greet.rs`.

### 2.1 Handlers async + émission poussée · ✅
`ctx.set` / `ctx.append` / `ctx.progress` / `ctx.alert` poussent immédiatement
sur le broadcast WS pendant qu'un handler long tourne (sur `spawn_blocking`).
- **Fichiers** : `server.rs` (`dispatcher`, `run_event`), `context.rs` (envoyeur `push`)
- **Accepté quand** : boucle 10×`append`+`progress` + `sleep` → l'UI se met à
  jour en direct (testé : 10 fragments, 10 points de progress, 1 alerte).

### 2.2 Queuing & annulation (à la Gradio) · ✅
File `tokio::mpsc` sérialisée : ordre stable, pas de chevauchement de handlers.
L'annulation sur re-déclenchement (composant + événement) est posée à
l'**enqueue** (`AppServer::enqueue`) via un `Arc<AtomicBool>` consultable par
`ctx.cancelled()`.
- **Fichiers** : `server.rs` (`Job`, `dispatcher`, `enqueue`), `context.rs` (`cancelled`)
- **Accepté quand** : handler lent + re-clic → le job 1 s'arrête au pas suivant
  (alerte `Warn`) et le job 2 part (testé : 4 fragments coupés, puis 10
  complets ; alertes `warn` puis `success`).

### 2.3 Streaming outputs (LLM, génération en continu) · ✅
Composant `Output` streamable via **`ctx.append(id, fragment)`** : fragments
**poussés uniquement** en temps réel (absents de la réponse finale → pas de
doublon côté client). Le client (`output.apply`) concatène `patch.append`.
- **Fichiers** : `context.rs` (`append`), `assets/app.js` (`apply`), `examples/greet.rs`
- **Accepté quand** : démo « token par token » (10 fragments + sleep 350 ms)
  affichée progressivement sans freeze — testé.

### 2.4 Alerts / toasts utilisateur · ✅
`ctx.alert(AlertLevel, msg)` envoie `{t:"alert", level, msg}` → toast stylé par
niveau (`info`/`success`/`warn`/`error`). Les alertes ne sont jamais dans la
réponse finale.
- **Fichiers** : `context.rs` (`AlertLevel`, `alert`), `app.js` (toast coloré),
  `styles.css` (.mg-toast-*)
- **Accepté quand** : warn + success affichés distinctement (testé).

### 2.5 Progress bars · ✅
Composant `Progress` + **`ctx.progress(id, f, label)`** : barre animée + label
à droite, état « terminé » (vert) à 100 %.
- **Fichiers** : `components.rs` (`Progress`), `context.rs`, `app.js`
  (registre `progress`), `styles.css` (.mg-progress-*)
- **Accepté quand** : tâche longue → barre 0→100 % + message d'étape (testé).

---

## Phase 3 — Média & Streaming Inputs · [P1]

> **Livrée.** Choix assumés : les médias voyagent en **data URLs**
> (`data:<mime>;base64,…`), pas en multipart — lecture par
> `ctx.get::<String>/get_str`, analyse serveur via `media::inspect` (type,
> taille, dimensions PNG/JPEG/GIF) et `media::decode`. Le **streaming live**
> passe par un message WS dédié `{t:"stream", c, p:{mime, b64}}` ; le serveur
> cumule des **statistiques** `StreamInfo { mime, bytes, chunks }` dé-sérialisables
> par le handler (`ctx.get::<StreamInfo>`), et l'UI met à jour le total en temps
> réel (patch `stream`). Dépendance ajoutée : `base64 = "0.22"`.
> Vérifié : build + doc zéro warning, 11 scénarios WebSocket verts sur
> `examples/media.rs` (stream cumulé, alertes, événements transports,
> analyse d'image).

### 3.1 Composants média (upload/affichage) · ✅
`Image`, `Audio`, `Video` : kinds `image`/`audio`/`video`, rôles `Input`
(upload) / `Output` (lecteur/affichage) via `.input()`/`.output()`,
`.interactive(bool)`, `.live(bool)`, `.value(data_url)`, `.label(...)`.
Upload client : FileReader → data URL → événement `change`.
- **Fichiers** : `components.rs` (3 kinds + docs), `app.js` (registres + upload
  + drag & drop), `styles.css` (.mg-media-*), `server.rs` (rendu)
- **Accepté quand** : une image uploadée est traitée côté serveur —
  `media::inspect` renvoie type, taille et **dimensions** (PNG 1×1 testé :
  `image · image/png · 1x1 · 70 octets`).

### 3.2 Streaming Inputs (micro/caméra) · ✅
`getUserMedia` + `MediaRecorder` → fragments (Blob) → `FileReader` → WS
`{t:"stream", c, p:{mime, b64}}`. `server::handle_stream` met à jour les stats
cumulées du composant, pousse un patch temps réel `{stream: stats}`, et enfile
l'événement `"stream"` → `App::on_stream(id, fn)` lit le total via
`ctx.get::<StreamInfo>`.
- **Fichiers** : `media.rs` (`StreamInfo`, `decode`), `server.rs`
  (`WireStream`, `handle_stream`), `app.js` (`MG.stream(id).send(blob)`,
  `sendStream`), `examples/media.rs`
- **Accepté quand** : 5 fragments envoyés → `StreamInfo {chunks:5}` lu par le
  handler, alerte à `chunks % 5 == 0`, `stats_out` mise à jour en direct
  (testé).

### 3.3 Événements propres à un composant · ✅
`EventName::Play|Pause|Stop|Stream` (+ parse), `.on_play/.on_pause/.on_stop/
.on_stream` (et `.on("play"|"pause"|"stop"|"stream", …)`), bannière serveur,
boutons play/pause/stop côté client émettant `{t:"event", e:"play"|…}`.
- **Fichiers** : `events.rs`, `app.rs` (builders + routage), `server.rs`
  (armes de bannière), `app.js` (playerButtons)
- **Accepté quand** : play/pause/stop déclenchés par WS → alertes serveur
  (testé : `lecture démarrée (play)`, `flux arrêté (stop)`).

---

## Phase 4 — Dix widgets paramétrables (à l'image de Gradio) · [P0]

> **Livrée.** Choix assumés : **Plot** = SVG dessiné à la main en JS vanilla
> (zéro dépendance — `drawPlot` : axes, grille, légende, barres, polyline,
> scatter). **Code** = tokenizer maison (`LANGS` : rust/python/javascript/
> json/markdown) rendu dans un `<pre>` surmonté d'un `<textarea>` transparent
> (texte invisible, `caret-color` visible, scroll synchronisé). **Explorer** =
> vrai navigateur de fichiers **serveur** via `GET /api/explore` (racine
> bornée par `canonicalize` + `strip_prefix`, filtre de fichiers par **globe**
> sans regex — `glob_match` itératif sur `Vec<char>`). Tous les widgets sont
> entièrement configurables par builder et leurs valeurs se lisent
> classiquement via `ctx.get` (bool, String, Vec, Value/tableau…).
> Vérifié : build + doc zéro warning, 16 scénarios verts (WS + HTTP) sur
> `examples/forms.rs` — dont le garde-fou anti-sortie-de-racine de
> `/api/explore` (`path=../../..` → `error`).

### 4.1 Composants paramétrables · ✅
`Checkbox` (bool), `Dropdown` (`choices`/`choices_str`, `multiple`,
`allow_custom`), `DatePicker` (`min`/`max`), `TimePicker`, `Dataframe`
(`headers`, `data`, éditable en place : cellules + ajout/suppression de
lignes), `Plot` (`line`/`bar`/`scatter`, titre/axes/légende/couleurs),
`Gallery` (`columns`, upload multiple + drag-drop, clic → index), `SortableList`
(drag & drop HTML5 natif, `change` = nouvel ordre), `Code`
(`language`/`theme`/numéros de ligne, `input`=éditeur / `output`=lecture seule),
`Explorer` (racine + `pattern` globe).
- **Fichiers** : `components.rs` (10 types + kinds + docs), `app.js`
  (registres + moteurs `highlight`/`drawPlot`), `styles.css` (10 blocs
  `.mg-*`), `server.rs` (rendu), `lib.rs` (ré-exports)
- **Accepté quand** : `examples/forms.rs` — submit avec toutes les valeurs
  lues (`ctx.get`), re-tri de la liste, édition du code, clic galerie, chemin
  explorateur (testés).

### 4.2 Explorer de fichiers serveur (`/api/explore`) · ✅
`GET /api/explore?root=&path=&pattern=` → `{t:"ok", root, path, dirs, files}`.
`std::fs::canonicalize(base.join(rel))` + `starts_with(&base)` borne la racine
(une tentative de sortie → `{t:"error", msg}`) ; `pattern` (globe `*`/`?`)
filtre les fichiers ; entrées triées, dossiers en premier. Le front affiche
une liste + breadcrumbs et émet `change` avec le chemin relatif cliqué.
- **Fichiers** : `server.rs` (`ExploreQuery`, `glob_match`, `explore`),
  `app.js` (registre `explorer`)
- **Accepté quand** : `root=.`, `path=crates/grio/src`, `pattern=*.rs` →
  `lib.rs` présent et aucun fichier non-`.rs` ; `path=../../..` → erreur
  (testés).

### 4.3 Édition de code colorisée + dessin SVG maison · ✅
`highlight(src, lang)` tokenise et émet des `<span class="tok-k|s|n|c">`
(5 langages) ; `drawPlot(spec, p)` génère le SVG (grille, axes gradués —
`fmtV` en k/m/M —, légende, barres `rect`, `polyline`, `circle` scatter).
Les `set`/`set_prop` serveur redessinent (`apply`).
- **Fichiers** : `assets/app.js` (`LANGS`, `highlight`, `drawPlot`, `fmtV`),
  `styles.css` (.tok-*, .mg-plot-*)
- **Accepté quand** : clic « Barres »/« Lignes » → `ctx.set("chart", …)`
  pousse un spec rendu en SVG (séries `{name, data}` vérifiées côté test).

### 4.4 Mise en page modulaire (`Layout` / `WithLayout`) · ✅
Un **réglage commun à tout composant** : `width`, `height`, `scale`
(`flex-grow`), `min_width` (px). Une seule mécanique — champ `layout` fusionné
dans `props` par `server::merge_props` au rendu **et** dans `/api/schema`.
Deux entrées : l'enveloppe générique `WithLayout::new(comp)` (brique **ou**
conteneur, sans toucher au type) et les builders de groupe
(`RowBuilder.width/height/scale/min_width` → `App::row/column/panel`). Le
front applique le tout en un point (`app.js` `applyLayout` dans `mount`).
- **Fichiers** : `components.rs` (`Layout`, `WithLayout`, `Component::layout`),
  `app.rs` (`RowBuilder` + wrap conditionnel), `server.rs` (`merge_props`),
  `app.js` (`applyLayout`), `examples/forms.rs` (démo)
- **Accepté quand** : `WithLayout::new(Code…).height(220)`,
  `WithLayout::new(Button…).scale(1)`, `r.min_width(360)`, `p.min_width(400)`,
  dataframe `.width(520)` — toutes les clés `layout` présentes dans le HTML
  servi, et **16 scénarios WS verts** (les enveloppes n'altèrent ni ids, ni
  events, ni lecture `ctx.get`).

### 4.5 Éditeur d'image (`ImageEditor` → masque d'inpainting) · ✅
Retouche **côté client** sur canvas : pinceau + gomme (étroits degrés),
formes (rectangle/ligne/flèche), rognage (crop), rotation 90°, filtres
(gris/inverse/clair/sombre/flou), annuler/rétablir (pile 20), zoom/pan, et
**1–4 calques RGBA** (`.layers(n)`, visibilité + opacité par calque).
La valeur de `change` est `{image, layers[], mask}` — `mask` = calques rendus
**blanc sur noir**, donc les zones à repeindre : sortie idéale pour de
l'**inpainting** côté serveur. Fond par défaut en dégradé si pas d'image.
- **Fichiers** : `components.rs` (`ImageEditor`), `app.js` (registre
  `imageeditor` : outils, pointeur, calques, historique, masque),
  `styles.css` (.mg-ie-*), `lib.rs`, `examples/forms.rs` (panneau retouche +
  `on_change("photo")` qui lit `{layers, mask}`)
- **Accepté quand** : widget rendu (`data-kind="imageeditor"`, `layers:2`) ;
  un `change photo` synthétique avec `{image, layers:[2], mask}` → alerte
  serveur « retouche : 2 calque(s), masque … — prêt pour de l'inpainting »
  (testé) + 16 scénarios WS toujours verts.

---

## Phase 5 — Production · ✅

- [x] **Sessions isolées** : valeurs d'état et flux de messages temps réel routés par session client (`sess_id`).
- [x] **OpenAPI complet** : spécification OpenAPI 3.0.3 auto-générée sur `GET /api/openapi.json` + documentation Swagger UI sur `GET /docs`.
- [x] **Auth simple** : clé d'API paramétrable via `App::api_key(...)` (vérification `X-API-Key` ou `Bearer <token>`).
- [x] **CORS optionnel** + configuration fluide (`.cors(bool)`, `.docs(bool)`, `.isolate_sessions(bool)`).
- [x] **CLI `grio`** : outil en ligne de commande et générateur de projets `crates/grio-cli` (`grio new <nom> --template <chatbot|vision|greet>`).
- [x] **Tests d'intégration** : suite de tests automatisés `crates/grio/tests/api_predict.rs` validant le pipeline complet, l'OpenAPI, la doc et l'authentification.

---

## Phase 6 — Écosystème IA & Test de Modèles · [P0]

> **En cours.** Fournir un outil clé en main, ultra-rapide et performant pour
> prototyper et tester des modèles en Rust (LLMs, vision, embeddings, audio).

### 6.1 Grille responsive & Conteneurs imbriqués (`Grid`) · ✅
- `Grid::new(id).columns(n).gap(g).gap_x(gx).gap_y(gy)` : CSS grid responsive
  native (auto 1 col sur mobile).
- `RowBuilder` fluide : `b.row(...)`, `b.column(...)`, `b.grid(...)`, `b.panel(...)`.
- Alignements fins `align` / `justify` / `wrap` sur `Row` & `Column`.
- Moteur de graphiques SVG `drawPlot` corrigé pour le mode barres.
- **Fichiers** : `components.rs` (`Grid`, `Row`, `Column`), `app.rs` (`App::grid`, `RowBuilder`),
  `server.rs`, `app.js`, `styles.css`, `examples/grid.rs`.

### 6.2 Composant natif d'interaction LLM : `Chatbot` · ✅
- Rendu d'historique de conversation : messages utilisateur et bot (`user`/`assistant`).
- Support natif du streaming de tokens (mise à jour incrémentale fluide de la dernière bulle via `ctx.append`).
- Formatage Markdown enrichi + blocs de code intégrés dans les bulles.
- **Fichiers** : `components.rs` (`Chatbot`, `ChatMessage`), `app.js` (`register('chatbot', ...)`),
  `styles.css` (.mg-chatbot-*, .mg-chat-*), `lib.rs`, `examples/chatbot.rs`.

### 6.3 Documentation Complète & Référence des Composants en Anglais · ✅
- Rédaction d'un guide exhaustif des composants en anglais ([COMPONENTS.md](COMPONENTS.md)).
- Pour chaque composant : description, capture/concept, signature du builder Rust, format des données I/O, exemple minimaliste prêt à copier-coller.
- Référencé en tête de [README.md](README.md) pour les nouveaux utilisateurs.
- **Fichiers** : `COMPONENTS.md`, `README.md`, `ROADMAP.md`.

### 6.4 Système de Thèmes & Dark Mode Natif · ✅
- Mode sombre / clair / auto avec bascule rapide (`Theme::dark()`, `Theme::light()`, `Theme::system()`).
- Personnalisation fluide de palette (couleur d'accentuation `primary`, arrondis `radius`, police `font`) via `App::theme`.
- Bouton interactif toggle Dark/Light dans l'en-tête client avec persistance locale (`localStorage`).
- **Fichiers** : `app.rs`, `server.rs`, `styles.css`, `app.js`.

### 6.5 Système d'Onglets Fluide (`Tabs`) & Métriques IA (`Metric`) · ✅
- `App::tabs` et `RowBuilder::tabs` avec panneaux et barre d'onglets réactive.
- `Metric::new("id").label("Throughput").value("54.2").unit("tok/s").delta("+14.8%")` pour benchmarks et observabilité IA.
- Exemple complet : `examples/theme_and_tabs.rs`.
- **Fichiers** : `components.rs` (`Metric`, `Tabs`, `SectionBuilder`), `app.rs`, `lib.rs`, `server.rs`, `styles.css`, `app.js`.

### 6.6 Exemples & Templates d'inférence IA (Candle / ONNX / LLM) · [P1]
- Exemple d'inférence locale LLM / embeddings avec streaming réel.
- Exemple vision / classification d'images via `Image` et `ImageEditor`.
- **Fichiers à toucher** : `examples/`.

---

## Conventions pour tenir la roadmap

1. Une tâche = une case cochée plus loin + une ligne dans `README.md`.
2. Chaque item mentionne les fichiers touchés avant d'être commencé.
3. Les critères *Accepté quand* sont les tests d'acceptation — les écrire en
   premier si possible.
4. Toujours `cargo build -p grio --example greet` et zéro warning avant de
   cocher.