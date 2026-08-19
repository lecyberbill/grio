# grio — un équivalent minimal de Gradio en Rust

Serveur web déclaratif : on décrit des **composants** et un **handler** en
Rust, et grio expose automatiquement **une UI temps réel** (CSS3 + JS vanilla,
zéro dépendance front) **et une API REST** (`/api/predict`, `/api/schema`).

```
┌─────────────── Rust (votre app) ───────────────┐
│ App::new(...) .item(Text).item(Slider)         │
│           .item(Output).on_submit(fn)          │
│                    │  .launch(addr)            │
└────────────────────┼───────────────────────────┘
                     ▼
        ┌─────────────────────────────┐
        │           grio              │
        ├─────────────────────────────┤
        │  UI    GET /                │   page + composants (HTML/CSS/JS)
        │        GET /assets/*        │   styles.css + app.js (vanilla)
        │        WS  /ws              │   événements temps réel
        │  API   GET  /api/schema     │   manifeste auto-généré
        │        POST /api/predict    │   même pipeline que l'UI
        └─────────────────────────────┘
```

> 📖 **Full Component Reference (English)**: See [COMPONENTS.md](COMPONENTS.md) for detailed APIs, parameters, and examples for all widgets.

---

## Démarrage rapide

```powershell
# dans D:\Projet\UI
cargo run -p grio --example greet
# → grio → http://127.0.0.1:7860
```

La démo reproduit l'exemple Gradio (`examples/greet.rs`) ; l'exemple Blocks
(`examples/blocks.rs`) montre la Phase 1 (flux déclarés, chaînage, onglets…).

```powershell
cargo run -p grio --example greet     # temps réel (streaming, progress, alertes)
cargo run -p grio --example blocks    # Phase 1 : interaction avancée
cargo run -p grio --example grid      # Grille CSS responsive et conteneurs imbriqués
cargo run -p grio --example chatbot   # Phase 6 : Chatbot LLM conversationnel + streaming
cargo run -p grio --example media     # Phase 3 : image/audio/vidéo, micro/caméra
cargo run -p grio --example forms     # Phase 4 : dix widgets paramétrables
```

---

## Structure du projet

```
D:\Projet\UI
├─ Cargo.toml             workspace (crate + exemples)
├─ README.md              ce manuel (à maintenir)
└─ crates/grio
   ├─ Cargo.toml
   └─ src
      ├─ lib.rs           API publique + doc crate
      ├─ app.rs           builder App + distribution des événements
      ├─ components.rs    trait Component + composants de base + conteneurs
      ├─ context.rs       Context fourni aux handlers
      ├─ events.rs        modèle d'événements (WireEvent, EventName)
      ├─ server.rs        serveur axum : UI + WebSocket + API REST
      └─ assets/          styles.css (CSS3) + app.js (vanilla)
```

Documentation générée : `cargo doc -p grio --open`.

---

## La console

Pensée pour le développement, la console est **bavarde par défaut** :

1. **Bannière de démarrage** (toujours affichée) : titre, URLs de l'UI et de
   l'API, entrées/sorties déclarées, listeners enregistrés.
2. **Logs d'activité** (`[http]`, `[ws]`, `[api]` — désactivables via
   `.quiet()`) : chaque requête page/assets, chaque connexion WebSocket,
   chaque événement reçu (cible, action, donnée) et la réponse renvoyée
   (mises à jour appliquées ou erreur).

Exemple :

```
  +----------------------------------------------
  |  grio - Greet · grio demo
  |  UI   ->  http://127.0.0.1:7860
  |  API  ->  POST http://127.0.0.1:7860/api/predict
  |  Entrees [2] : name, intensity
  |  Sorties [4] : intro, greeting, rt, log
  |  Listeners  [3]
  |    - event `reset`
  |    - Run/API
  |    - click sur `generate`
  +----------------------------------------------
  [ws] client #1 connecté
  [ws] client #1 · action `run` = click
  [run] greeting · click -> 1 mise(s) à jour -> greeting=Hello, Ada !!!!
  [api] POST /api/predict · entrées {name=Ada, intensity=3}
  [api] ok - 1 mise(s) à jour -> greeting=Hello, Ada !!!!
  [ws] client #1 déconnecté
```

Pour couper le flux : `.quiet()` sur le builder `App`.

---

## L'API Rust (déclarative)

### Composants

| Type | kind | Rôle | Constructeur | Méthodes |
|------|------|------|--------------|----------|
| `Text` | `text` | Input | `Text::new(id)` | `label`, `value`, `placeholder`, `interactive` |
| `Slider` | `slider` | Input | `Slider::new(id)` | `label`, `min`, `max`, `step`, `value`, `interactive` |
| `Output` | `output` | Output | `Output::new(id)` | `label`, `value` |
| `Markdown` | `markdown` | Output | `Markdown::new(id)` | `text` |
| `Button` | `button` | — | `Button::new(id)` | `label`, `secondary`, `primary` |
| `Progress` | `progress` | — | `Progress::new(id)` | `label` |
| `Image` | `image` | In/Out | `Image::new(id)` | `label`, `value`, `interactive`, `input`, `output` |
| `Audio` | `audio` | In/Out | `Audio::new(id)` | `label`, `value`, `interactive`, `input`, `output`, `live` |
| `Video` | `video` | In/Out | `Video::new(id)` | `label`, `value`, `interactive`, `input`, `output`, `live` |
| `Checkbox` | `checkbox` | Input | `Checkbox::new(id)` | `label`, `value` |
| `Dropdown` | `dropdown` | Input | `Dropdown::new(id)` | `label`, `choices`, `choices_str`, `value`, `value_list`, `multiple`, `allow_custom` |
| `DatePicker` | `date` | Input | `DatePicker::new(id)` | `label`, `value`, `min`, `max` |
| `TimePicker` | `time` | Input | `TimePicker::new(id)` | `label`, `value` |
| `Dataframe` | `dataframe` | Input | `Dataframe::new(id)` | `label`, `headers`, `data`, `interactive`, `addable`, `sortable` |
| `Plot` | `plot` | Output | `Plot::new(id)` | `variant`, `title`, `xlabel`, `ylabel`, `colors`, `size`, `data` |
| `Gallery` | `gallery` | In/Out | `Gallery::new(id)` | `label`, `columns`, `interactive`, `upload` |
| `Metric` | `metric` | Output | `Metric::new(id)` | `label`, `value`, `delta`, `delta_color`, `unit` |
| `Tabs` | `tabs` | — | `Tabs::new(id)` | `tab`, `selected` |
| `ImageEditor` | `imageeditor` | In/Out | `ImageEditor::new(id)` | `label`, `value`, `interactive`, `layers`, `brush`, `crop`, `shapes`, `filters`, `rotflip`, `output` |
| `SortableList` | `list` | Input | `SortableList::new(id)` | `label`, `items`, `add`, `value`, `interactive` |
| `Code` | `code` | In/Out | `Code::new(id)` | `label`, `language`, `theme`, `lines`, `value`, `interactive`, `input`, `output` |
| `Explorer` | `explorer` | Input | `Explorer::new(id)` | `label`, `root`, `pattern` |

`.interactive(false)` grise un champ sans couper son snapshot d'entrées.
Les médias transportent des **data URLs** (`data:<mime>;base64,…`) ; voir la
section *Média & Streaming Inputs* ci-dessous.

### Conteneurs

| Type | kind | But | Méthodes |
|------|------|-----|----------|
| `Row` | `row` | côte à côte | `gap`, `item` |
| `Column` | `column` | empilé | `gap`, `item` |
| `Panel` | `panel` | carte avec titre | `label`, `gap`, `item` |
| `Tabs` | `tabs` | onglets (pilotés client) | `tab(label, builder)` |
| `Accordion` | `accordion` | sections repliables (`<details>`) | `open`, `section(label, builder)` |

Depuis `App`, on utilise les raccourcis `App::row`, `App::column`,
`App::panel` avec un builder :

```rust
App::new("ex")
    .row(|r| { r.gap(12.0); r.item(Text::new("a")); r.item(Slider::new("b")); })
    .panel("Résultat", |p| { p.item(Output::new("out")); })
    .item(Markdown::new("doc").text("# Titre\n\n- liste\n- **gras**"))
```

### Mise en page (modulaire) : `Layout`

Chaque composant peut être **dimensionné et pondéré** : largeur, hauteur,
proportion dans la ligne (`scale`, comme Gradio) et largeur minimale.
Deux façons d'utiliser, une seule mécanique (clé `layout` fusionnée dans les
`props` par le serveur) :

1. **Enveloppe générique** `WithLayout::new(comp)` — s'applique à n'importe
   quel composant, brique **ou** conteneur, sans rien changer au type :

   ```rust
   WithLayout::new(Text::new("a")).width(240).min_width(120)
   WithLayout::new(Row::new("r").item(Text::new("b"))).scale(2)
   ```

2. **Groupes `row/column/panel`** : mêmes réglages directement sur le builder
   (`App::row`, `App::column`, `App::panel` partagent `RowBuilder`) :

   ```rust
   App::new("ex")
       .row(|r| { r.scale(1); r.min_width(360); r.item(Text::new("a")); })
       .panel("Carte", |p| { p.width(480); p.item(Output::new("o")); })
   ```

Le CSS appliqué : `width`/`height`/`min-width` en px, `scale` → `flex-grow`
(`flex-basis: 0%`). Tous les réglages sont optionnels et omis du JSON s'ils
sont absents.

### Événements

| Méthode | Déclenchement |
|---------|---------------|
| `on_submit(fn)` | clic sur **Run** (bouton auto-généré) ou appel `/api/predict` |
| `on_load(fn)` | montage de la page (connexion WebSocket du client) |
| `on_change(id, fn)` | modification d'un composant d'entrée |
| `on_click(id, fn)` | clic sur un composant (bouton…) |
| `on("click"\|"change", [ids…], fn)` | la **même** fonction sur plusieurs déclencheurs |
| `on_event("nom", fn)` | événement applicatif via `ctx.emit("nom", …)` |
| `live()` | mode live : chaque `change` redéclenche aussi les `submit` |

**Chaînage** (`.then`/`.success`/`.failure`) : s'applique au **dernier handler**
déclaré. `flow(inputs, outputs)` scope le dernier handler : il ne peut **lire
que** les `inputs` listés (`get`) et **écrire que** dans les `outputs` listés
(`set`/`set_prop`/`append`/`progress`) — tout accès hors flux est rejeté ou
ignoré.

```rust
App::new("chat")
    .on_submit(user_fn)          // affiche le message
    .then(bot_fn)                // réponse streaming après coup
    .on_click("fail", handler)   // erreur possible
    .success(ok_fn)              // seulement si handler a réussi
    .failure(fix_fn)             // seulement s'il a échoué (récupère)
    .flow(&["a"], &["out"])      // scope le dernier handler
```

**Le `Context`** donné aux handlers :

| Méthode | Description |
|---------|-------------|
| `ctx.get::<T>("id")` | lit une entrée (déserialisée, `Result`) |
| `ctx.get_f64` / `ctx.get_str` / `ctx.has` | variantes pratiques |
| `ctx.set("id", valeur)` | remplace la valeur d'un composant (client + API) |
| `ctx.set_prop("id", "visible"\|"disabled"\|"label", v)` | modifie une prop sans toucher à la valeur |
| `ctx.append("id", fragment)` | **streaming** : concatène un fragment en direct |
| `ctx.progress("id", f, "étape…")` | pilote un composant `Progress` (f ∈ 0.0..1.0) |
| `ctx.alert(AlertLevel::Success, "…")` | toast coloré (`Info`/`Success`/`Warn`/`Error`) |
| `ctx.cancelled()` | `true` si un re-déclenchement de la même action a annulé ce job |
| `ctx.event()` | événement d'origine : `c` (cible), `e` (action), `d` (données) |
| `ctx.skip("id")` | les prochaines écritures sur `id` sont ignorées (`unskip` inverse) |
| `ctx.emit("nom", valeur)` | déclenche les `on_event("nom")` (bus local) |

Les handlers renvoient `grio::Result<()>` ; en cas d'erreur, elle est affichée
comme **toast** dans l'UI. Une **garde anti-boucle** (64 événements par passe)
protège des cycles `emit` → handler → `emit`.

### Temps réel (Phase 2)

Chaque événement (clic, submit, predict) entre dans une **file ordonnée**
(`tokio::mpsc`) et est exécuté sur un **pool de threads**
(`spawn_blocking`) : les handlers peuvent rester **synchrones** (avec
`std::thread::sleep`, des boucles longues…) sans jamais bloquer le serveur.

- **Streaming** : `ctx.append` / `ctx.progress` / `ctx.set` sont **poussés
  immédiatement** via le canal WebSocket — le client se met à jour pendant
  l'exécution. Les `set` sont aussi inclus dans la réponse finale (donc dans
  `/api/predict`) ; les fragments `append`/`progress` sont **ou exclusivement**
  temps réel.
- **Annulation** : re-cliquer sur la même action (composant + événement) pose
  un drapeau `cancel` que le job en cours consulte via `ctx.cancelled()` —
  la boucle s'arrête au pas suivant. Testé : un clic pendant la génération
  coupe le job 1 (alerte `Warn`) avant que le job 2 ne parte.
- **Exemple** (`examples/greet.rs`, panneau « Temps réel ») : handler qui
  écrit *token par token* (`append`), pilote une barre (`progress`) et clôture
  par `alert(Success)`. Vérifiable en console : `[run] generate · click ->
  10 mise(s) à jour → log=…, pg=…`.

### Média & Streaming Inputs (Phase 3)

Trois composants média portant des **data URLs** (`data:<mime>;base64,…`) :

| Composant | Rôle | Particularité |
| --- | --- | --- |
| `Image::new(id)` | entrée (upload) ou sortie (affichage) | drag & drop, `.output()` |
| `Audio::new(id)` | entrée (upload) ou sortie (lecteur) | `.live(true)` → **micro live** |
| `Video::new(id)` | sortie (lecteur) par défaut | `.live(true)` → **caméra live** |

- **Analyse serveur** : `media::inspect(data_url)` -> `MediaInfo {
  mime, size_bytes, width, height }` (dimensions PNG/GIF/JPEG ; les autres
  formats : `None`). `media::decode` accepte base64 standard **et** URL-safe.
- **Streaming** : le client capture (micro/caméra via `getUserMedia` +
  `MediaRecorder`) et envoie des fragments `{t:"stream", c, p:{mime, b64}}`.
  Le serveur cumule `StreamInfo { mime, bytes, chunks }` pour le composant,
  pousse un patch temps réel `{stream: stats}` et enfile l'événement
  `"stream"`. Le handler (`App::on_stream(id, fn)`) lit le total avec
  `ctx.get::<StreamInfo>(id)`.
- **Événements transports** : `play`, `pause`, `stop` (lecteurs audio/vidéo)
  se branchent via `App::on_play/on_pause/on_stop`. L'API côté JS publique
  expose `MG.stream(id).send(blob)`.
- **Exemple** (`examples/media.rs`) : upload d'une image → dimensions
  calculées côté serveur ; micro/caméra live → stats temps réel ; lecteur
  vidéo → alertes `play`/`pause`/`stop`. Testé par WebSocket (11 scénarios).

### Widgets paramétrables (Phase 4)

Dix widgets de plus, tous construits sur le même principe déclaratif — valeurs
lisibles via `ctx.get` (`bool`, `String`, `Vec`, objet…) :

| Widget | Lecture (`ctx.get`) | Particularités |
| --- | --- | --- |
| `Checkbox` | `bool` | case à cocher |
| `Dropdown` | `String` ou `Vec` | `choices`, multi, saisie libre (`allow_custom`) |
| `DatePicker` / `TimePicker` | `String` (ISO `yyyy-mm-dd` / `HH:MM`) | pickers natifs + `min`/`max` |
| `Dataframe` | `Value` `{headers, data}` | tableau éditable : cellules, `+ ligne`, suppression, **tri par colonne** (clic entête) |
| `Plot` | — (Output) | SVG maison : `line`/`bar`/`scatter`, axes, légende |
| `Gallery` | `Vec<String>` (data URLs) | grille, upload multiple + drag-drop, clic → index |
| `SortableList` | `Vec<String>` (valeurs) | drag & drop natif, `change` = nouvel ordre |
| `Code` | `String` | éditeur colorisé (rust/python/js/json/md), lecture seule en sortie |
| `Explorer` | `String` (chemin relatif) | navigateur de fichiers **serveur** (`/api/explore`) |

- **`Code`** : tokenizer maison dans `assets/app.js` (`LANGS`, `highlight`) ;
  superposition `<pre>` (colorisé) + `<textarea>` transparent, scroll
  synchronisé, numéros de ligne, émission `change` debouncée 250 ms.
- **`Plot`** : `drawPlot(spec, p)` génère le SVG (grille, axes gradués
  `fmtV`, barres/`polyline`/scatter) — les `ctx.set("chart", spec)` du
  serveur redessinent le graphique.
- **`Explorer`** : `GET /api/explore?root=&path=&pattern=` liste dossiers puis
  fichiers (globe `*`/`?`), **racine bornée** (`canonicalize` +
  `strip_prefix` : sortie → `error`) ; le clic émet `change` avec le chemin.
- **`ImageEditor`** : retouche sur **canvas** (client) — pinceau, gomme,
  formes (rectangle/ligne/flèche), rognage, rotation 90°, filtres
  (gris/inverse/clair/sombre/flou), annuler/rétablir (pile 20), zoom/pan et
  1–4 **calques** RGBA (visibilité + opacité). Valeur de `change` :
  `{ image, layers[], mask }` où le **`mask`** (blanc sur noir) désigne les
  zones dessinées — directement exploitable pour de l'**inpainting** côté
  serveur.
- **Exemple** (`examples/forms.rs`) : checkbox + dropdown multi + date/heure +
  tableau éditable + liste triable + éditeur de code + galerie + plot + file
  explorer + **retouche photo** — boutons « Barres »/« Lignes » dessinent le
  plot, « Run » résume tous les widgets. Testé par WebSocket + HTTP (16
  scénarios + masque d'inpainting).

### Mise en page

Le titre/sous-titre : `App::new("Titre").subtitle("…")`. Libellé du bouton
Run : `.run_label("C'est parti")`.

---

## L'API REST (automatique)

Générée depuis les composants marqués **Input** / **Output** (rôle
déclaratif). Aucun code API à écrire.

### `GET /api/schema`

Manifeste auto-généré : titre, d'éventuels endpoints, liste ordonnée des
`inputs`, `outputs`, et tous les composants (id, kind, rôle, props).

```powershell
Invoke-RestMethod http://127.0.0.1:7860/api/schema
```

### `GET /api/explore`

Navigateur de fichiers serveur (composant `Explorer`) : liste dossiers puis
fichiers d'un chemin relatif à une racine, filtrés par globe (`*`/`?`).

```powershell
Invoke-RestMethod "http://127.0.0.1:7860/api/explore?root=.&path=crates/grio/src&pattern=*.rs"
```

Réponse : `{ "t": "ok", "root": ".", "path": "crates/grio/src", "dirs": […], "files": […] }`
— dossiers et fichiers triés, **racine bornée** (`canonicalize` +
`strip_prefix`) : un chemin qui en sort → `{ "t": "error", "msg": … }`.

### `POST /api/predict`

Exécute exactement le même pipeline que le clic sur Run (UI connectée =
mise à jour en direct). Deux formats acceptés :

```jsonc
// par position — une valeur par input, dans l'ordre du schéma
{ "data": ["Ada", 3] }
// par identifiant
{ "data": { "name": "Ada", "intensity": 3 } }
// alias
{ "inputs": { "name": "Ada", "intensity": 3 } }
```

Réponses :

```jsonc
{ "ok": true, "data": ["Hello, Ada !!!!"], "outputs": { "greeting": "Hello, Ada !!!!" } }
{ "ok": false, "error": {"missing_input": "whoami"} }
```

Exécution **isolée et stricte** : chaque appel n'utilise que les entrées
fournies ; si une entrée déclarée est absente, la réponse indique son
identifiant. Les identifiants inconnus sont ignorés. Chaque appel
`/api/predict` met aussi à jour les UIs ouvertes en direct (même pipeline que
le clic sur Run).

Exemple PowerShell :

```powershell
$body = @{ data = @('Ada', 3) } | ConvertTo-Json
Invoke-RestMethod http://127.0.0.1:7860/api/predict -Method Post -Body $body -ContentType 'application/json'
```

Exemple curl :

```bash
curl -s -X POST http://127.0.0.1:7860/api/predict \
  -H 'Content-Type: application/json' -d '{"data":["Ada",3]}'
```

---

## Le front (vanilla)

- **`assets/styles.css`** — CSS3 uniquement : variables de thème, mode sombre
  automatique (`prefers-color-scheme`), flexbox/grid, animations, respect de
  `prefers-reduced-motion`.
- **`assets/app.js`** — zéro dépendance. Le cœur expose un registre :

  ```js
  window.MG.register('mon_composant', {
    mount(c) { ... },      // construit le DOM depuis c.props
    apply(c, patch) { ... } // applique les mises à jour serveur
  });
  ```

  Un composant d'entrée pose `c.getValue` ; le snapshot est envoyé au serveur
  via WebSocket, qui répond par des *patches* (`apply`). `window.MG.emit`
  permet d'émettre un événement programmatiquement.

---

## Étendre le moteur : ajouter un composant

1. **Rust** — `crates/grio/src/components.rs` : définir le type, implémenter
   `Component` (`id`, `kind`, `props`, `role()`, éventuellement `layout()` et
   `children()`) et le ré-exporter dans `lib.rs`. Pour que la brique hérite
   des réglages de taille sans code dupliqué, enveloppez-la dans
   `WithLayout::new(…)` — déjà disponible pour tout composant.
2. **JS** — `assets/app.js` : `MG.register(kind, { mount, apply })`. Si
   c'est une entrée, fournir `getValue` (détection auto via `data-role`).
3. **CSS** — `assets/styles.css` : styles sous `.mg-<kind>`.
4. **API** — rien à faire : le rôle déclaratif classe automatiquement le
   nouveau composant dans `/api/schema` et `/api/predict`.
5. **Docs** — ajouter la ligne dans le tableau des composants (§ « Api
   Rust ») et mettre à jour cette section.

---

## Conventions de maintenance

- **Documentation obligatoire** : tous les items publics Rust portent des
  `///` (vérifié par `#![warn(missing_docs)]`).
- **Chaque nouvelle fonctionnalité** doit mettre à jour : le code (`///`), le
  tableau des composants, la section API REST et la feuille de route ci-dessous.
- **Vérification** : `cargo build -p grio --example greet`.

---

## Feuille de route

Le plan détaillé (phases, fichiers concernés, critères « Accepté quand ») est
dans **[`ROADMAP.md`](ROADMAP.md)** — onglet maintenu à jour.

- [x] Phase 0 — moteur, composants, événements, API REST auto, console, docs
- [x] Phase 1 — interaction avancée (Blocks) : flux déclarés `.flow()`, chaînage
      `.then()/.success()/.failure()`, `ctx.event()`, `ctx.skip()`/`set_prop`,
      `.interactive()`, `on_load`, multi-déclencheurs `.on()`, onglets & accordéon
- [x] Phase 2 — temps réel : file d'attente ordonnée + pool de threads,
      **queuing** & annulation, **streaming outputs** (`ctx.append`), **alerts**
      (`ctx.alert`), **progress bars** (`ctx.progress` + `Progress`)
- [x] Phase 3 — média & **streaming inputs** (image/audio/video, micro/caméra)
- [x] Phase 4 — dix **widgets paramétrables** : checkbox, dropdown (multi+custom),
      date/heure, dataframe éditable, plot SVG maison, galerie, liste glisser-
      déposer, éditeur de code colorisé, explorateur de fichiers serveur
      (`/api/explore`)
- [x] Phase 5 — production : sessions isolées, OpenAPI complet, Swagger UI (`/docs`), auth API Key, CLI `grio` (`crates/grio-cli`), tests d'intégration
- [x] Phase 6 — écosystème IA : Dark Mode & Thèmes natifs (`Theme`), Onglets réactifs (`Tabs`), Métriques d'inférence (`Metric`), Chatbot LLM, Studio Multimodal SDXL (`examples/prompt_to_image.rs`)