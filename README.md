# Application de Gestion Boutique

## Stack technique

| Couche         | Techno                                   |
|----------------|-------------------------------------------|
| Interface (front) | React + TypeScript + Vite + Tailwind CSS |
| Application desktop | Tauri 2 (wrapper natif, rendu WebView) |
| Backend / logique métier | Rust (dans `src-tauri/`)          |
| Base de données | SQLite (fichier local, via `rusqlite`)   |
| Génération PDF | `printpdf` (facture pro forma, 100 % locale, sans dépendance externe) |


## Structure du projet
gestion-boutique/
├── src/                      # Frontend React/TypeScript
│   ├── pages/                 # Une page par rubrique du menu
│   ├── components/Sidebar.tsx
│   ├── api/tauri.ts           # Appels vers le backend Rust (invoke)
│   ├── store/auth.ts          # État de connexion (zustand)
│   └── types/index.ts
├── src-tauri/                # Backend Rust + configuration Tauri
│   ├── src/
│   │   ├── main.rs            # Point d'entrée, enregistrement des commandes
│   │   ├── db.rs              # Connexion SQLite, schéma, migrations
│   │   ├── models.rs          # Structures de données partagées
│   │   ├── pdf.rs              # Génération du PDF de facture
│   │   └── commands/          # Une commande Tauri par domaine métier
│   ├── Cargo.toml
│   └── tauri.conf.json
├── package.json
└── README.md                 # Ce fichier


## Fonctionnalités couvertes dans cette version

- Authentification (utilisateur unique, mot de passe hashé avec bcrypt)
- Tableau de bord (indicateurs + filtrage Aujourd'hui / Hier / 7 jours)
- Produits : création, modification, ajout de stock, unicité insensible à la
  casse et aux espaces, recherche
- Catégories : création, modification, suppression protégée si utilisée
- Mouvements de stock : entrée / sortie / ajustement, historique consultable
- Fournisseurs et Clients : CRUD + recherche
- Ventes : panier multi-produits, vérification du stock disponible, calcul du
  total, décrément automatique du stock, numérotation automatique, annulation
  (avec restauration du stock et conservation de l'historique)
- Factures pro forma : génération PDF automatique par vente, numérotation
  `PF-AAAA-NNNNNN`, enregistrement dans `Documents/GestionBoutique/Factures/AAAA/`,
  ouverture du dossier contenant le PDF
- Statistiques : CA, coût, bénéfice brut, panier moyen, top 5 / flop 5 produits,
  sur plusieurs périodes (jour, semaine, mois, semestre, année)
- Paramètres boutique (nom, adresse, téléphones, devise, slogan...)
- Sauvegarde manuelle de la base (copie horodatée) et restauration

## Prérequis (à installer une seule fois sur le PC de développement)

1. **Node.js** ≥ 18 → https://nodejs.org
2. **Rust** (toolchain stable) → https://www.rust-lang.org/tools/install
   (`rustup-init`, puis `rustup default stable`)
3. **Tauri CLI** : installé automatiquement via `npm install` (`@tauri-apps/cli`
   est dans `devDependencies`)
4. **Dépendances système** selon l'OS de développement :
   - **Windows** : Microsoft C++ Build Tools + WebView2 (déjà présent sur
     Windows 10/11 à jour). Voir https://tauri.app/start/prerequisites/#windows
   - **macOS** : Xcode Command Line Tools (`xcode-select --install`)
   - **Linux (Ubuntu/Debian)** :
     ```bash
     sudo apt update
     sudo apt install -y libwebkit2gtk-4.1-dev build-essential curl wget \
       file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
     ```

## Lancer l'application en local (mode développement)

```bash
cd gestion-boutique
npm install
npm run tauri dev
```

La première commande installe les dépendances JS. `npm run tauri dev` compile
le backend Rust (la première fois, cela télécharge les crates et prend
quelques minutes) puis ouvre la fenêtre de l'application avec rechargement à
chaud du frontend.

**Connexion par défaut :** `admin` / `admin123` — à changer immédiatement dans
*Paramètres → Compte utilisateur*.

La base de données est créée automatiquement au premier lancement dans le
dossier de données de l'utilisateur (ex. `%APPDATA%\GestionBoutique\boutique.db`
sous Windows, `~/.local/share/GestionBoutique/boutique.db` sous Linux). Les
factures PDF et les sauvegardes vont dans `Documents/GestionBoutique/`.

## Construire l'installateur pour le remettre au gérant

Une fois l'application testée et prête :

```bash
npm run tauri build
```

Cela génère, selon l'OS sur lequel la commande est lancée :

- **Windows** : un installateur `.msi` et/ou `.exe` (NSIS) dans
  `src-tauri/target/release/bundle/msi/` et `.../nsis/`
- **macOS** : un `.dmg` et une app `.app` dans
  `src-tauri/target/release/bundle/dmg/`
- **Linux** : un `.deb` et/ou `.AppImage` dans
  `src-tauri/target/release/bundle/deb/` et `.../appimage/`

> **Important : Tauri compile pour l'OS sur lequel la commande est exécutée.**
> Pour livrer un installateur Windows, lancez `npm run tauri build` **sur un PC
> Windows** (le vôtre ou celui du gérant). Il n'est pas nécessaire d'avoir Node
> ou Rust installés en permanence sur le PC du gérant : une fois l'installateur
> généré, il vous suffit de le lui transmettre (clé USB, WeTransfer, etc.) et
> de l'exécuter — il embarque tout ce qu'il faut.

### Remettre l'application au gérant (résumé pratique)

1. Sur votre PC : `npm run tauri build` → récupérez le fichier `.msi` (ou
   `.exe`) dans `src-tauri/target/release/bundle/`.
2. Copiez ce fichier sur une clé USB ou envoyez-le par un moyen quelconque.
3. Sur le PC du gérant : double-cliquez sur l'installateur, suivez les étapes
   (l'application s'installe comme n'importe quel logiciel Windows classique).
4. Lancez l'application depuis le raccourci créé sur le bureau / menu Démarrer.
5. Connectez-vous avec `admin` / `admin123`, changez immédiatement le mot de
   passe et renseignez les informations de la boutique dans *Paramètres*.
6. Montrez au gérant où se trouve le bouton **« Créer une sauvegarde
   maintenant »** dans *Paramètres*, et encouragez-le à l'utiliser
   régulièrement (ou copiez vous-même le dossier `Documents/GestionBoutique/`
   sur une clé USB de temps en temps).

Aucune connexion Internet n'est requise pour l'utilisation quotidienne — elle
n'est nécessaire que lors du build initial (téléchargement des dépendances).

## Personnaliser le logo / l'icône de l'application

Des icônes minimales (carré vert uni) sont fournies dans `src-tauri/icons/`
pour que le build ne casse pas. Pour mettre une vraie icône avant le build
final :

```bash
npm run tauri icon chemin/vers/votre-logo.png
```

Cette commande régénère automatiquement toutes les tailles et tous les formats
nécessaires (`.ico` pour Windows, `.icns` pour macOS, `.png` pour Linux).

## À vérifier au premier lancement (`npm run tauri dev`)

Le frontend (React/TS) a été entièrement testé et compile sans erreur dans
cet environnement. Le backend Rust a été écrit avec soin mais **n'a pas pu
être compilé ici** (pas de toolchain Rust/Tauri disponible dans ce
conteneur). Deux zones à surveiller lors du tout premier `cargo`/`tauri dev` :

1. **`src-tauri/src/pdf.rs`** — génération du PDF avec la crate `printpdf`
   (version épinglée `0.6` dans `Cargo.toml`). L'API utilisée (`PdfDocument::new`,
   `add_shape`, `use_text`, `Line { ... }`) correspond à l'API classique de
   printpdf 0.5/0.6. Si le compilateur signale une erreur sur `add_shape` ou
   sur les champs de `Line`, consultez les exemples officiels sur
   https://docs.rs/printpdf/0.6 — l'ajustement est généralement mécanique
   (renommage d'une méthode ou d'un champ).
2. Versions de crates : `Cargo.toml` fixe des versions majeures larges
   (`rusqlite = "0.31"`, `tauri = "2"`, etc.) ; `cargo build` résoudra les
   dernières versions compatibles automatiquement.