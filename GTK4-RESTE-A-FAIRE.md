# GTK4 : état d'avancement et reste à faire

Branche : `feat/gtk4`. Le plan complet, avec l'architecture, la correspondance des widgets et les jalons, est dans [`GTK4-PLAN.md`](GTK4-PLAN.md). Ce fichier dit où en est le travail et ce qu'il reste.

**Rien n'est encore livré aux utilisateurs.** Les paquets de release restent en interface classique (winit/wgpu) jusqu'au jalon M9.

## Fait

| Jalon | État | Commits |
|---|---|---|
| **M1** : fenêtre GTK4/libadwaita avec un terminal | Fait. CI verte sur les 3 OS | `f3e2cc7` |
| **M2** : coutures neutres | Fait. Relu, CI verte | `765c605` → `f1e5b09` |
| **M3** : extraction de `Desk` | 8 étapes sur 8 faites, relecture à faire, voir ci-dessous | `c48aac8` → `62f61a7` |

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
7. ✅ Sauvegarde et restauration, fermeture avec confirmation (`Closing`, `Question`), confirmation de collage (`62f61a7`).
   - ⚠️ Cette étape compile, passe clippy sur les 3 OS, passe les 1106 tests et le smoke test, mais **elle n'a pas encore été relue**.
8. ✅ Partie système de `Desk` :
   - `src/desk/system.rs` : `tick`, `sync_rest`, `set_appearance`, `rests`, `hidden`. `load_icons` est dans `src/desk/edit/look.rs`.
   - `src/desk/output.rs` : `output(id, Seen)`, avec les réponses aux mots de passe SSH, plus `Effect::Clipboard(String)` pour OSC 52 et `Effect::Notify { title, body }` pour OSC 9/777.
   - `src/desk/deadline.rs` : `next_deadline`, le minimum de save, transcripts, secrets, redial et power, évalués dans cet ordre. `Running::about_to_wait` n'ajoute que ses réveils de vue (visibilité, chargement, travail).
   - Test garde-fou `tests/desk/neutral.rs` : aucun import winit, wgpu, arboard, ui, render, text, gtk ou adw dans `src/desk`, et 150 lignes au plus par fichier dans `src/desk` et `src/app/running`.
   - ⚠️ Comme l'étape 7, cette étape n'a pas encore été relue.

**Avant de passer à M4** : relire tout M3 (`git diff f1e5b09..HEAD`). Points à vérifier :
- comportement identique sur Windows et macOS ;
- ordre des effets ;
- fichiers de 150 lignes au plus ;
- pas de logique neutre restée dans `Running`.

## Reste à faire

| Jalon | Contenu | Terminé quand |
|---|---|---|
| **M3** | Relecture de M3 (étapes 7 et 8 surtout) | UI winit inchangée sur les 3 OS, fichiers de 150 lignes max, clippy propre |
| **M4** | Onglets, splits et actions sous GTK : `TabView`/`TabBar` par pane, `Paned` depuis `Node`, registre `SessionId` → vue, `GAction` par `Command` + accélérateurs depuis `Keymap`, taille de police | Ouvrir, fermer, splitter, changer de pane et tous les raccourcis fonctionnent |
| **M5** | Barre latérale (`OverlaySplitView` + `ListBox`), `Stack` par instance, `claim`/`load`/saver sur l'horloge glib, transcripts, `door` et `fleet`, page de chargement. **C'est à partir d'ici que GTK écrit le dossier d'état** | Quitter et relancer restaure tout ; un 2e lancement ouvre une fenêtre dans le processus existant |
| **M6** | Parité du terminal : sélection et copie (Ctrl+Shift+C), souris, molette, barre de défilement, liens, boîtes et soulignés, caractères larges, dépôt de fichiers, confirmation de collage, OSC 52, `gio::Notification`, `CellPainter` partagé | tmux, nvim, mc et Powerline s'affichent correctement |
| **M7** | Menus contextuels, popovers d'édition, `FileDialog`, dialogue SSH, carte hors ligne, `AlertDialog`, DnD natif de `TabView`, toasts et bannières | Chaque menu, fermeture et dépôt de l'UI winit a son équivalent |
| **M8** | `adw::PreferencesDialog` (7 pages via `Desk`), Connexions, Raccourcis | Chaque préférence prend effet en direct et est persistée |
| **M9** | Packaging et release, voir ci-dessous | Un tag produit des `.deb`/`.rpm` GTK installables, plus un `.deb` classique pour 22.04 |
| **M10** | Type de touche neutre `input::{Key, NamedKey, Mods}`, `custom-ui` optionnel | Le binaire Linux ne lie plus winit ni wgpu |
| **M11** | Performances : `TextNode`, table ASCII, cache de lignes, échelles fractionnaires. Retrait de `TRAME_UI=classic` | Débit à moins de 10 % de la version classique, 0 % de CPU au repos |

## Points à ne pas oublier

- **`release.yml` casse si on fusionne tel quel.** Les jobs Linux tournent sur ubuntu-22.04, dont GTK 4.6 et libadwaita 1.1 sont trop vieux. Avant toute fusion dans `master`, il faut soit compiler ces jobs avec `--no-default-features` (interface classique), soit faire M9.
- **API publique élargie en M2.** Beaucoup d'éléments internes de `trame::host::update` sont devenus `pub` (marqués `#[doc(hidden)]`) pour que les tests de `tests/host/` les atteignent.
- **Fichiers encore trop longs.** `src/app/fleet.rs` et quelques autres dépassaient déjà 150 lignes avant le refactor.
- **Ctrl+Shift+C** n'existe pas encore sous GTK, faute de sélection. Il arrive en M6.
- **CI.** Elle ne se déclenche que sur `master` ou sur une PR. Pour la lancer sur la branche :

```bash
gh workflow run ci.yml --ref feat/gtk4
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
