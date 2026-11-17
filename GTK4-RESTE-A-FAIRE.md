# GTK4 : état d'avancement et reste à faire

Branche : `feat/gtk4-next`, partie de `origin/master`. Le plan complet, avec l'architecture, la correspondance des widgets et les jalons, est dans [`GTK4-PLAN.md`](GTK4-PLAN.md). Ce fichier dit où en est le travail et ce qu'il reste.

**La fenêtre GTK n'est pas encore livrée.** Les paquets Linux restent en interface classique (winit/wgpu) jusqu'au jalon M9. En revanche, le refactor `Desk` de M1 à M3 (étapes 1 à 7) est déjà dans `master` et livré dans les versions 0.3.34 à 0.3.36 : l'interface winit de Windows, macOS et Linux tourne déjà sur `Desk`.

## Fait

| Jalon | État | Commits |
|---|---|---|
| **M1** : fenêtre GTK4/libadwaita avec un terminal | Fait. CI verte sur les 3 OS | `895ac40` (dans `master`) |
| **M2** : coutures neutres | Fait. Relu, CI verte | `895ac40` (dans `master`) |
| **M3** : extraction de `Desk` | 8 étapes sur 8 faites et relues | `895ac40` (étapes 1 à 7, dans `master`), `77cba67` (étape 8), puis les corrections de relecture |

M1, M2 et les étapes 1 à 7 de M3 ont été écrits sur `feat/gtk4`, puis fusionnés en un seul commit `895ac40` dans `master`. La règle « un sujet par PR » n'a donc pas été tenue pour ces étapes. Les commits suivants sont sur `feat/gtk4-next`, un sujet par commit.

### Détail de M1

- `src/gnome/` : `adw::Application`, `ApplicationWindow`, `HeaderBar` et `TermView`, un widget GTK maison dessiné avec Pango.
- Le thème clair ou sombre suit `StyleManager` en direct.
- Le clavier passe par `IMMulticontext`. Ctrl+Shift+V colle.
- La fenêtre se redimensionne avec le terminal.
- `TRAME_UI=classic` relance l'ancienne interface. `--no-default-features` retire GTK de la compilation.
- La version GTK n'écrit jamais dans le dossier d'état, pour ne pas abîmer les sessions de la version classique.

### Détail de M2

- `trame::display`
- `trame::host` : persist, door, power, update, pending, quit. Les tests sont dans `tests/host/`.
- `Sessions` est construit à partir d'un `Notifier`.
- `UserEvent<K>` et `WindowKey(u32)`.
- Les méthodes reçoivent `exiting: bool` au lieu de `&ActiveEventLoop`.

### Détail de M3 (à relire)

Le contrôleur par fenêtre `trame::desk::Desk` (`src/desk/`) est neutre : ni winit, ni wgpu, ni ui. Ses méthodes renvoient une liste ordonnée d'`Effect`, que `Running::apply` exécute côté winit. Les tests sont dans `tests/desk/`.

1. ✅ `Sessions` déplacé dans `trame::desk::sessions`, le `LinkCache` reste dans `Running`.
2. ✅ Squelette de `Desk`, `Effect`, `Effects`, `Outcome`. Quit, update et restarts sont dans `Desk`.
3. ✅ Préférences (store/apply) et hôtes SSH (store/forget).
4. ✅ Onglets : ouvrir, démarrer, fermer, fin du shell, shells différés, dossier, SSH.
5. ✅ Commandes clavier, écriture dans l'onglet actif, transfert d'onglets et d'instances entre fenêtres.
6. ✅ Validation des éditions : nom, couleur, emblème, icône, couleurs perso, dossier, hôte.
7. ✅ Sauvegarde et restauration, fermeture avec confirmation (`Closing`, `Question`), confirmation de collage. Relue après coup, alors qu'elle était déjà livrée sous Windows et macOS.
8. ✅ Partie système de `Desk` :
   - `src/desk/system.rs` : `tick`, `sync_rest`, `set_appearance`, `rests`, `hidden`. `load_icons` est dans `src/desk/edit/look.rs`.
   - `src/desk/output.rs` : `output(id, Seen)`, avec les réponses aux mots de passe SSH, plus `Effect::Clipboard(String)` pour OSC 52 et `Effect::Notify { title, body }` pour OSC 9/777.
   - `src/desk/deadline.rs` : `next_deadline`, le minimum de save, transcripts, secrets et redial, évalués dans cet ordre. Le front-end applique les effets rendus, ajoute ses réveils de vue, puis lit `desk.power.deadline()` en dernier, comme avant l'extraction.
   - Test garde-fou `tests/desk/neutral.rs` : aucun chemin `winit::`, `wgpu::`, `arboard::`, `gtk::`, `adw::`, `gdk::`, `glib::`, ni `crate::ui`, `crate::render` ou `crate::text` dans `src/desk`, et 150 lignes au plus par fichier dans `src/desk`, `src/app` et `src/gnome`.

### Relecture de M3 (faite)

Corrections apportées après la relecture :
- `Desk::ask_forget` décide seul s'il faut confirmer l'oubli d'un hôte SSH ; `Running` ne touche plus `desk.closing`.
- `Desk::pending_question` abandonne une fermeture qui ne peut plus poser de question.
- Démarrage : `Desk::restore` vérifie que la fenêtre possède son dossier, `Desk::open_first` ouvre la première instance, `Desk::settle_start` vérifie qu'un onglet existe et efface les transcripts si on ne les garde pas. `Running` ne garde que la mise en page, le dessin et l'affichage.
- Le chargeur appelle `update_failed` au lieu d'écrire le champ.
- `src/app/window/mod.rs` découpé, sous 150 lignes.
- Tests ajoutés : oubli d'hôte, collage sur un onglet qui a perdu le focus, second collage, sauvegarde à la sortie et au redémarrage, échéance de sauvegarde et de transcripts, repos des onglets cachés, relayout après la perte d'une connexion SSH.

## Reste à faire

| Jalon | Contenu | Terminé quand |
|---|---|---|
| **M4** | Onglets, splits et actions sous GTK : `TabView`/`TabBar` par pane, `Paned` depuis `Node`, registre `SessionId` → vue, `GAction` par `Command` + accélérateurs depuis `Keymap`, taille de police | Ouvrir, fermer, splitter, changer de pane et tous les raccourcis fonctionnent |
| **M5** | Barre latérale (`OverlaySplitView` + `ListBox`), `Stack` par instance, `claim`/`load`/saver sur l'horloge glib, transcripts, `door` et `fleet`, page de chargement. **C'est à partir d'ici que GTK écrit le dossier d'état** | Quitter et relancer restaure tout ; un 2e lancement ouvre une fenêtre dans le processus existant |
| **M6** | Parité du terminal : sélection et copie (Ctrl+Shift+C), souris, molette, barre de défilement, liens, boîtes et soulignés, caractères larges, dépôt de fichiers, confirmation de collage, OSC 52, `gio::Notification`, `CellPainter` partagé | tmux, nvim, mc et Powerline s'affichent correctement |
| **M7** | Menus contextuels, popovers d'édition, `FileDialog`, dialogue SSH, carte hors ligne, `AlertDialog`, DnD natif de `TabView`, toasts et bannières | Chaque menu, fermeture et dépôt de l'UI winit a son équivalent |
| **M8** | `adw::PreferencesDialog` (7 pages via `Desk`), Connexions, Raccourcis | Chaque préférence prend effet en direct et est persistée |
| **M9** | Packaging et release, voir ci-dessous | Un tag produit des `.deb`/`.rpm` GTK installables, plus un `.deb` classique pour 22.04 |
| **M10** | Type de touche neutre `input::{Key, NamedKey, Mods}`, `custom-ui` optionnel | Le binaire Linux ne lie plus winit ni wgpu |
| **M11** | Performances : `TextNode`, table ASCII, cache de lignes, échelles fractionnaires. Retrait de `TRAME_UI=classic` | Débit à moins de 10 % de la version classique, 0 % de CPU au repos |

## Points à ne pas oublier

- **Release Linux en interface classique.** Les jobs Linux de `release.yml` tournent sur ubuntu-22.04 (GTK 4.6 et libadwaita 1.1, trop vieux) et compilent donc avec `--no-default-features` depuis `8947294` et `d08e879`. M9 changera ça.
- **API publique élargie en M2.** Beaucoup d'éléments internes de `trame::host::update` sont devenus `pub` (marqués `#[doc(hidden)]`) pour que les tests de `tests/host/` les atteignent.
- **Fichiers encore trop longs** hors de `src/desk`, `src/app` et `src/gnome`, par exemple `src/session/mod.rs`, `src/ui/chrome/painter.rs` et `src/ui/motion/mod.rs`. Ils dépassaient déjà 150 lignes avant le refactor.
- **Ctrl+Shift+C** n'existe pas encore sous GTK, faute de sélection. Il arrive en M6.
- **CI.** Elle ne se déclenche que sur `master` ou sur une PR. Pour la lancer sur la branche :

```bash
gh workflow run ci.yml --ref feat/gtk4-next
```

## Compiler en local (Fedora)

```bash
sudo dnf install gtk4-devel libadwaita-devel
```

```bash
cargo run
```

```bash
TRAME_UI=classic cargo run
```

`cargo clippy --all-targets -- -D warnings` doit passer avec et sans `--no-default-features`.
