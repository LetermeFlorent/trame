# Revue des 3 architectures GTK4/libadwaita pour trame, et plan recommandé

## 0. Pourquoi l'app n'est toujours pas en GTK4

J'ai vérifié directement dans le code :

- `src/main.rs` appelle toujours `app::run()`, c'est-à-dire la boucle winit + wgpu.
- `Cargo.toml` ne contient ni `gtk4`, ni `libadwaita`, ni `vte`. Aucun fichier `src/` n'utilise GTK. Le seul résultat de grep, `theme/system.rs`, lance `gsettings`.
- Les jobs Linux de `release.yml` (test, linux, update-linux) et de `ci.yml` tournent sur **ubuntu-22.04**.
- Sur la machine de dev, `gtk4`, `libadwaita` et `vte291-gtk4` sont installés, mais **`gtk4-devel` et `libadwaita-devel` ne le sont pas**. Rien de GTK ne peut donc compiler ici pour l'instant.

**Conclusion : aucune ligne GTK n'a encore été écrite.** Le paquet installé et l'auto-update livrent toujours la build winit.

Autres affirmations des propositions que j'ai vérifiées :

- **Remise à zéro du mode souris.** Elle a bien lieu dans le thread lecteur (`src/session/threads.rs:66-72` : `shell_in_foreground` puis `forget_mouse`).
- **`Sessions`.** `Sessions::new(EventLoopProxy, WindowId)` enveloppe `with_notifier(Notifier)` (`src/app/sessions/mod.rs:55-63`), donc la couture existe déjà.
- **Raccourcis Linux.** Les raccourcis principaux sont en `Ctrl+Shift` (`src/input/shortcuts/defaults.rs:28`). Les accélérateurs GTK ne voleront donc pas Ctrl+C ou Ctrl+V au shell. Le risque soulevé par A est écarté.
- **Types winit.** winit fuit dans 10 fichiers de `src/input/`.

## 1. Notes (sur 10)

| Critère | A : VTE alimenté par trame | B : TermView Pango/GSK + chrome adw | C : incrémental, TermView |
|---|---|---|---|
| Parité fonctionnelle | 6 | 9 | 9 |
| Rendu GNOME natif | 9 | 8 | 8 |
| Effort / risque | 5 | 5 | 6 |
| Maintenance à côté de l'UI winit | 4 | 8 | 8 |
| Compilable en CI de release | 6 | 8 | 8 |
| **Total** | **30** | **38** | **39** |

**A (VTE en « feed mode »).** C'est l'idée la plus séduisante visuellement, mais elle repose sur un mode non standard.

- On suppose que VTE émet `commit` pour ses propres réponses DA/DSR sans PTY. Ce n'est pas prouvé.
- Chaque octet est parsé deux fois et chaque onglet garde deux scrollbacks. Les deux moteurs peuvent diverger lors du redimensionnement et du rewrap.
- La détection de liens et le défilement fluide changent.
- Le comportement sur Linux s'éloigne de Windows et macOS. Cela fait deux moteurs à maintenir pour toujours.
- Il faut une dépendance de plus (`libvte-2.91-gtk4`), absente de la 22.04.

**B.** C'est la meilleure conception technique pour le terminal :

- `gsk::TextNode` avec glyphes forcés sur la grille ;
- table ASCII pré-shapée ;
- cache de nœuds par ligne ;
- `TermPlan` partagé entre les deux front-ends.

Le terminal n'utilise que des API GTK 4.0. En revanche, son ordre de travail ne montre un terminal GTK qu'après environ 10 jours de refactor.

**C.** Mêmes choix de fond que B, avec un meilleur découpage incrémental : une PR par sujet, `Desk`, `CellPainter` et un routage de l'updater selon la variante (GTK ou classique). Son M1 arrive pourtant après environ 7 jours de préparation (M0 et M0b), ce qui n'est pas « petit ».

**Gagnant : C sur le fond, avec le rendu de B et le M1 sans préparation repris en partie de A.**

## 2. Plan recommandé

### 2.1 Architecture

On garde un seul crate (lib + bin), pour ne pas casser les chemins `trame::` des tests.

**Features Cargo**

```toml
[features]
default    = ["custom-ui", "gnome"]
custom-ui  = ["dep:winit", "dep:wgpu", "dep:pollster", "dep:bytemuck", "dep:swash", "dep:fontdb", "dep:arboard"]
gnome      = ["dep:gtk4", "dep:libadwaita", "dep:async-channel"]

[target.'cfg(target_os = "linux")'.dependencies]
gtk4          = { version = "<train gtk-rs courant>", features = ["v4_14"], optional = true }
libadwaita    = { version = "<même train>",          features = ["v1_5"],  optional = true }
async-channel = { version = "2", optional = true }
```

- Rendre `custom-ui` optionnel n'est nécessaire qu'au jalon M10. Jusque-là, winit et wgpu restent des dépendances normales.
- `build.rs` émet `cargo:rustc-check-cfg=cfg(gnome_ui)` et active `gnome_ui` quand on cible Linux avec `CARGO_FEATURE_GNOME`. Tout le code GTK est sous `#[cfg(gnome_ui)]`.
- `main.rs` lance `gnome::run()` si `gnome_ui` est actif et que `TRAME_UI` ne vaut pas `classic`, sinon `app::run()`. Les deux passent par `fatal::report`. La porte de sortie `TRAME_UI=classic` est retirée une release après la bascule.

**Bibliothèque : code neutre partagé**

| Module | Contenu | Dépend de winit/wgpu |
|---|---|---|
| `term`, `session`, `workspace`, `preferences`, `locale`, `theme`, `files`, `geometry`, `secret` | inchangés | non |
| `src/display.rs` (nouveau) | PanelSide, Density, SidebarMode, Platform, `TERMINAL_FONT_SIZE*`, réexportés par `trame::ui` | non |
| `trame::host` (nouveau) | persist, door, power, update, pending, enums Quit/Restart, déplacés depuis `src/app` | non |
| `trame::desk` (nouveau) | `Desk`, le contrôleur par fenêtre extrait de `Running` (workspace, sessions, préférences, saver, transcripts, icônes, power, update, restarts, quit, closing). Les méthodes prennent `exiting: bool` et renvoient des `Effect` | non |
| `trame::termview` (nouveau) | `CellPainter` (fill, line, border, cell_text), plus la logique de lignes, curseur, styles, boîtes et liens extraite de `ui/terminal` | non |
| `render`, `text`, `ui` | UI custom et implémentation `CellPainter` basée sur `Scene` | oui, sous `custom-ui` |

**Binaire**

- `src/app/` reste le front-end winit : `Running` devient `Desk` plus l'état de vue.
- `src/gnome/` est le nouveau front-end. Chaque fichier fait au plus 150 lignes et a un seul rôle :
  - `app.rs`, `events.rs` (async_channel), `clock.rs` (une `timeout_add_local_once` réarmée sur `Desk::next_deadline`), `fleet.rs`
  - `window/{build,header,state,theme}.rs`
  - `sidebar/{view,row,footer,rail}.rs`
  - `panes/{tree,paned,tabs,tab_page,offline}.rs`
  - `actions/{map,accels}.rs`
  - `menus/`, `dialogs/`, `prefs/` (un fichier par page), `dnd/`
  - `term/{imp,view,metrics,paint,runs,cursor,ascii,rowcache,scroll}.rs`
  - `term/input/{keyval,keys,ime,mouse,select,clipboard}.rs`
  - `notify.rs`

  Chaque `imp.rs` ne contient que le boilerplate de la sous-classe. Pour respecter la limite de 5 arguments de clippy, on passe des structs de contexte (`PaintCtx`, `RowSpec`).

### 2.2 Widget terminal : moteur trame dans un widget GTK maison, sans VTE

Le choix : `TermView`, une sous-classe de `gtk::Widget` qui se dessine elle-même dans `snapshot()`.

**Rendu**

1. Le snapshot verrouille `Session::shared_terminal()` brièvement, le temps de copier les lignes visibles, puis relâche le verrou.
2. Les fonds, la sélection, le curseur et les caractères de dessin de boîtes passent par `append_color`, alignés sur les pixels.
3. Le curseur creux utilise `append_border`.
4. Le soulignement ondulé utilise `append_cairo`.
5. Le texte :
   - Au jalon M1 : `pango::Layout` par segment de même style, avec `append_layout`. Les caractères larges et ceux venant d'une police de repli sont placés cellule par cellule.
   - Au jalon M12, si les mesures l'exigent : on adopte la variante de B, soit `gsk::TextNode` avec positions de glyphes forcées, table ASCII et cache de nœuds par ligne.
6. Les métriques viennent de `pango::FontMetrics`, avec `hint-metrics` et `round-glyph-positions` activés. La police vient de `org.gnome.desktop.interface monospace-font-name`, à la taille `Preferences.font_size`.

**Flux d'événements**

- Le `Notifier` positionne un `AtomicBool dirty`. Seul le passage de faux à vrai envoie un message dans l'`async_channel`, que `glib::spawn_future_local` vide.
- À la réception : sortie du `Desk` (réponse aux mots de passe, attention, OSC 52), puis `queue_draw`. Le frame clock regroupe les redessins.
- `size_allocate` déclenche `session.resize`.
- Une `gtk::Scrollbar` est liée à `history_len` et `display_offset`. Le défilement fluide passe par `add_tick_callback`.

**Entrées**

- `EventControllerKey` et `IMMulticontext` (le signal `commit` est écrit tel quel dans le PTY).
- `keyval.rs` convertit `gdk::Key` vers le type de touche existant, puis appelle `input::encode`. C'est winit temporairement, puis le type neutre au jalon M10.
- `GestureClick` et `GestureDrag` passent par `encode_mouse` et la logique de sélection.
- Presse-papiers via `gdk::Clipboard`, y compris PRIMARY.

**Justification**

- C'est la seule option qui garde **tout** : transcripts SGR, réponse automatique SSH, remise à zéro de la souris, spinner OSC 133/7501, OSC 52, OSC 9/777, détection de liens, `SYNC_UPDATE`, `compact()`.
- Les trois OS gardent le même moteur.
- Le terminal n'utilise que des API GTK 4.0. Aucune dépendance VTE.
- Options rejetées :
  - VTE en mode normal : perd les transcripts, le statut des programmes et la réponse SSH.
  - VTE en feed mode : deux moteurs, comportement non prouvé.
  - Rendu wgpu hors écran recopié dans une `MemoryTexture` : environ 8 Mo de readback bloquant par frame.

### 2.3 Correspondance des widgets

| UI trame actuelle | Remplacement GTK / libadwaita |
|---|---|
| Barre de titre custom, boutons de fenêtre, `Hit::Resize/TitleBar` | `adw::ApplicationWindow` + `ToolbarView` + `HeaderBar` (CSD native) + `adw::WindowTitle` (titre = instance, sous-titre = `short_path` du dossier) |
| Bouton afficher/masquer la barre latérale, `PanelSide`, largeur | `ToggleButton` avec `sidebar-show-symbolic`, lié à `adw::OverlaySplitView` (`show-sidebar`, `sidebar-position`). Les bornes de largeur réutilisent `width.rs` |
| Liste des instances et de leurs onglets | `ListBox .navigation-sidebar` reconstruite depuis `Workspace` : emblème ou texture, nom (ellipsé), nombre d'onglets, `Spinner`, repli, fermeture au survol. En-tête avec `list-add`, pied avec Réglages et Mise à jour |
| Mode Rail | `ListBox` étroite avec icônes seules et `tooltip`. Peut être livré en dernier |
| Onglets par pane | `adw::TabBar` + `TabView` (même groupe pour tous) : `loading` = travail en cours, `needs-attention`, `indicator-icon` = hôte SSH, classes CSS pour les couleurs (Swatch) |
| Splits façon Zed | `gtk::Paned` imbriqués (un split à N enfants devient une chaîne). `notify::position` est renvoyé vers `resize_split`. Registre `SessionId` → `TermView` pour reparenter les terminaux sans les recréer |
| Bascule entre instances | `gtk::Stack` |
| Menu contextuel et menu « + » | `gio::Menu` + `SimpleAction` avec paramètre u64, `PopoverMenu` via `GestureClick` (clic droit), `TabView` `setup-menu` |
| Renommer, couleurs, emblèmes, image, dossier | `Popover` + `Entry`. `FlowBox` de couleurs et `ColorDialog` (le sélecteur HSV custom disparaît). Icônes Lucide en `-symbolic` dans une GResource. `FileDialog` |
| Connexion SSH | `adw::Dialog` : groupe « Serveurs » avec un `ActionRow` par hôte, plus `EntryRow` et `PasswordEntryRow` |
| Confirmations (fermeture, collage, mise à jour) | `adw::AlertDialog` (réponse destructive pour la fermeture) |
| Fenêtre de réglages (7 onglets) | `adw::PreferencesDialog` : `SwitchRow`, `SpinRow`, `ComboRow`, `EntryRow`. Sous-pages pour les Connexions et pour les Raccourcis (`ShortcutLabel` + capture de touche). Taille d'interface, densité et menu Explorer sont masqués |
| Carte « hors ligne » | `adw::StatusPage` dans un `gtk::Overlay` au-dessus du terminal |
| Écran de chargement, notes « non sauvé » / « échec de mise à jour » | Page de `Stack` (`StatusPage` + `Spinner` + `ProgressBar`), `adw::Banner`, `Toast` |
| Glisser-déposer | Glisser natif de `TabView` (`page-attached`/`detached`, `create-window`). `DropTarget` sur le terminal avec `DropZone::at` pour split/échange. `DropTarget` sur les lignes de la barre latérale et `FileList` pour les fichiers |
| Thème, notifications, visibilité | `adw::StyleManager` (`notify::dark`), `gio::Notification`, état `GdkToplevel` vers `Power` |
| Raccourcis clavier | `win.<id ACTIONS>` + `set_accels_for_action`, générés depuis `Keymap` par un convertisseur `Combo` → chaîne d'accélérateur. Les noms persistés ne changent pas |

### 2.4 Ce qui change pour l'utilisateur

**Ce qui s'améliore**

- Fenêtre réellement libadwaita sous Linux : décorations et animations GNOME.
- Vraies notifications de bureau, ce qui n'existait pas sous Linux.
- Sélection PRIMARY.
- Thème clair/sombre suivi via `StyleManager`.

**Ce qui change ou disparaît**

- Sélecteur HSV, renommage directement dans l'onglet, logo animé.
- Réglages de taille d'interface et de densité masqués.
- La fenêtre d'un onglet arraché est placée par le compositeur.
- Le rendu des polices passe par Pango (pas d'antialiasing LCD).

**Ce qui ne change pas**

- Les données : `session.json`, les fichiers `.vt` et les raccourcis restent compatibles dans les deux sens.
- Windows et macOS ne changent pas.

**Ubuntu 22.04 et Debian 12** gardent une build « classique » winit pendant 2 à 3 releases.

**Identifiant d'application** : `io.github.LetermeFlorent.Trame`. Le fichier `.desktop` est renommé, et les lanceurs épinglés devront peut-être être ré-épinglés.

### 2.5 Build, CI et packaging

**Machine de dev (Fedora 45), une seule fois :**

```
sudo dnf install gtk4-devel libadwaita-devel
```

Cette commande tire `glib2-devel` (qui fournit `glib-compile-resources`), `pango-devel`, `cairo-devel`, `graphene-devel` et `gdk-pixbuf2-devel`. `vte291-gtk4-devel` n'est pas nécessaire. `xorg-x11-server-Xvfb` est optionnel, pour le smoke test local.

**Problème GTK 4.6 / libadwaita 1.1 sur ubuntu-22.04.** On vise `v4_14` / `v1_5` : c'est la base qui contient `OverlaySplitView`, `ToolbarView`, `SwitchRow`, `SpinRow`, `PreferencesDialog`, `AlertDialog`, `FileDialog` et `ColorDialog`.

- Viser 1.1 obligerait à utiliser des widgets dépréciés (`AdwFlap`, `PreferencesWindow`, `MessageDialog`), puis à tout réécrire : environ 25 % d'effort en plus pour le chrome. Rejeté.
- Épingler les features `v4_14` / `v1_5` empêche aussi d'appeler par erreur des API plus récentes de Fedora 45.

**`ci.yml`**

- Matrice : windows-latest, macos-latest, **ubuntu-24.04** (features par défaut, avec `libgtk-4-dev libadwaita-1-dev xvfb`), plus ubuntu-22.04 en `--no-default-features --features custom-ui` (à partir du jalon M10).
- clippy `-D warnings` est lancé pour les deux jeux de features.

**`release.yml`**

- Les jobs `test`, `linux` et `update-linux` passent sur 24.04. `.github/actions/linux-packages` installe les paquets `-dev` GTK.
- Build : `--no-default-features --features gnome`.
- `.deb` : `depends = "$auto, fontconfig"`. On retire libvulkan, libegl, xkbcommon et les `recommends` X11/Wayland.
- `.rpm` : même binaire (glibc 2.39, compatible Fedora 40+). `requires` passe à `gtk4 >= 4.14` et `libadwaita >= 1.5`. On garde le test d'installation dans un conteneur `fedora:latest`, et on bascule vers un build en conteneur Fedora en cas de problème de soname.
- Nouveau job `linux-classic` sur 22.04 qui produit `trame-classic_*.deb` (avec `Conflicts`/`Replaces: trame`).
- Smoke test : `xvfb-run` avec `GDK_BACKEND=x11` et `GSK_RENDERER=cairo`, puis le `pgrep -P` existant.

**Updater (`host/update/route.rs`)**

- La variante (GTK ou classique) est fixée à la compilation. Une installation classique ne reçoit **jamais** le paquet GTK, sauf si `ldconfig -p` trouve `libadwaita-1.so.0` dans une version suffisante.
- Le redémarrage ré-exécute le même binaire.

**Packaging** : fichiers `packaging/linux/<app-id>.desktop` (`StartupWMClass` = app-id) et `<app-id>.metainfo.xml`, icônes hicolor sous l'app-id. Pas de Flatpak dans le plan : les shells lancés via `flatpak-spawn` casseraient la lecture du cwd par `/proc` et `tcgetpgrp`.

### 2.6 Jalons

| # | Jalon | Périmètre | Terminé quand | Jours |
|---|---|---|---|---|
| **M1** | **Fenêtre GTK avec un terminal (petit, livrable)** | `dnf install` ; features + `build.rs`, `cfg(gnome_ui)` ; dispatch dans `main.rs` avec `TRAME_UI=classic` ; `adw::Application` + `ApplicationWindow` + `HeaderBar` (titre du shell) + `StyleManager` ; `TermView` (Pango `append_layout`, curseur, couleurs du thème) ; `Session` lancée directement avec un `Notifier` vers `async_channel` ; adaptateur `gdk::Key` → winit `Key` vers `encode` + `IMMulticontext` ; redimensionnement ; Ctrl+Shift+C/V. **Aucun accès en écriture au dossier d'état** (pas de `claim`, pas de sauvegarde), pour ne pas abîmer les sessions de la build classique ; job CI ubuntu-24.04 | `cargo run` sur Fedora ouvre une vraie fenêtre Adwaita (CSD, clair/sombre en direct) avec le shell ; bash, vim, htop et less s'affichent correctement ; `TRAME_UI=classic` ouvre l'app complète ; fmt et clippy passent pour les deux jeux de features ; CI verte sur les 3 OS. Les paquets de release restent classiques à ce stade | 4 |
| M2 | Coutures neutres, sans changement de comportement | `src/display.rs` + réexports ; `WindowKey(u32)` et `UserEvent<K>` ; `Sessions` construit à partir d'un `Notifier` ; `exiting: bool` au lieu de `&ActiveEventLoop` ; `trame::host` (persist, door, power, update, pending) avec ses tests déplacés dans `tests/host/` | Comportement identique sur les 3 OS, tous les tests passent, aucun import winit/wgpu dans `host` | 4 |
| M3 | Extraction de `Desk` | Un sujet par PR (onglets, commandes, réglages store/apply, commit d'édition, saved/quit/update, système) ; `Effect` ; `Running` délègue à `Desk` ; `tests/desk/` | UI winit inchangée sur les 3 OS ; fichiers de 150 lignes max ; clippy (`too_many_lines`, `too_many_arguments`) propre | 7 |
| M4 | Onglets, splits et actions sous GTK | `TabView`/`TabBar` par pane ; `Paned` depuis `Node` (synchronisation des ratios) ; registre `SessionId` → vue ; `GAction` pour chaque `Command` + accélérateurs depuis `Keymap` ; commandes de taille de police | Ouvrir, fermer, splitter, changer de pane et tous les raccourcis par défaut fonctionnent | 6 |
| M5 | Barre latérale, restauration, multi-fenêtre | `OverlaySplitView` + `ListBox` des instances et onglets ; `Stack` par instance ; `claim`, `load`, saver sur l'horloge glib, sauvegarde à l'arrêt ; transcripts et shells différés ; `door` et `fleet` ; page de chargement | Quitter puis relancer restaure instances, splits, cwd et scrollback colorés à partir des mêmes fichiers ; un 2e lancement ouvre une fenêtre dans le processus propriétaire | 7 |
| M6 | Parité du terminal | Sélection et copie ; rapports souris ; molette et flèches en écran alternatif ; barre de défilement + défilement fluide ; liens ; boîtes et soulignés ; placement des glyphes larges ; dépôt de fichiers ; confirmation de collage (`AlertDialog`) ; OSC 52 ; `gio::Notification` ; `CellPainter` partagé avec `Scene` | tmux, nvim avec la souris, mc et les prompts Powerline s'affichent correctement ; `tests/ui/terminal` passe toujours ; `tests/termview` ajouté | 8 |
| M7 | Menus, éditeurs, dialogues, SSH, glisser-déposer | Menus contextuels ; popovers de renommage, couleur, emblème et image ; `FileDialog` ; dialogue SSH + carte hors ligne + reconnexion ; `AlertDialog` de fermeture ; DnD natif de `TabView` + `create-window` ; zones de split/échange ; DnD de la barre latérale ; toasts et bannières | Chaque `MenuEntry`, `Closing` et `DropTarget` de l'UI winit a son équivalent ; SSH se connecte et répond au mot de passe | 8 |
| M8 | Préférences | `PreferencesDialog` avec 7 pages via `Desk` (store/apply) ; Connexions ; Raccourcis avec capture et notices Taken/Unsupported | Chaque préférence applicable prend effet en direct et est persistée ; les surcharges de raccourcis restent compatibles entre les deux builds | 5 |
| M9 | Packaging, CI, release | Jobs Linux sur 24.04 ; métadonnées deb/rpm ; desktop et metainfo ; smoke test xvfb ; job `linux-classic` sur 22.04 ; routage de l'updater par variante ; README (commande dnf) | Un tag produit des `.deb`/`.rpm` GTK installables sur Ubuntu 24.04 vierge et Fedora 45, plus un `.deb` classique 22.04 ; une installation existante se met à jour vers la bonne variante. **C'est seulement à ce jalon que l'app installée devient GTK** | 4 |
| M10 | Touches neutres, sortie de winit sous Linux | `input::{Key, NamedKey, Mods}` avec tests golden des noms persistés ; adaptateurs winit et gdk ; `custom-ui` rendu optionnel ; `tests/{ui,render,text}` sous `#![cfg(feature = "custom-ui")]` | Le binaire Linux de release ne lie plus winit ni wgpu ; les noms restent identiques à l'octet près | 4 |
| M11 | Performance et polices | `TextNode` + table ASCII + cache de lignes si nécessaire ; échelles fractionnaires 1.25/1.5 ; retrait de `TRAME_UI=classic` | Débit à moins de 10 % de la build classique ; pas de décalage de colonnes ; 0 % de CPU au repos | 3 |

**Total : environ 60 jours.** La fenêtre GTK avec un vrai shell est visible dès le 4e jour, une fois les paquets `-devel` installés. Les utilisateurs recevront la version GTK à la release du jalon M9.

**Principaux risques**

- L'extraction de `Desk` (M3) peut régresser Windows et macOS. Parade : petites PR protégées par les tests.
- Alignement Pango sur la grille et échelles fractionnaires.
- Synchronisation entre `Paned`/`TabView` et le modèle. Parade : `Workspace` reste la source de vérité et les widgets en sont reconstruits.
- Période de maintenance à deux variantes Linux (GTK et classique).