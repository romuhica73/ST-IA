# ADR-005 — Cycle de vie des jobs, annulation et nettoyage

## Statut

**ACCEPTED**

Qualifié manuellement depuis le `.app` empaqueté (voir Qualification manuelle) : annulation réelle pendant Whisper, retry sans redémarrage, fermeture de l'application pendant un job, média invalide. Les quatre scénarios sont passés, avec preuves au niveau processus et au niveau journal applicatif.

## Contexte

M2 et M3 ont prouvé le chemin nominal : média → FFmpeg → WAV → whisper.cpp → SRT/TXT, avec un modèle téléchargé et vérifié. Aucune de ces missions n'a traité les chemins d'interruption. L'audit du code avant M4 a relevé quatre défauts réels :

* `run_ffmpeg` utilisait `.output().await`, qui abandonne le handle du processus enfant en interne — FFmpeg était donc **impossible à tuer** ;
* `run_whisper` liait l'enfant à `_child` et ne s'en servait jamais — whisper-cli était **lancé puis oublié** ;
* `start_transcription` testait l'état « occupé » puis **relâchait le verrou avant** de lancer la tâche : deux appels rapprochés pouvaient tous deux voir `Idle` (TOCTOU) et démarrer deux pipelines ;
* aucun hook de fermeture, aucun nettoyage au démarrage : un `.app` tué pendant un job laissait un workspace temporaire orphelin.

Le bouton « Annuler » existait visuellement mais était inerte (`aria-disabled`).

## Décision

### Un seul job, propriété explicite

Un unique emplacement de job en mémoire (`JobState`), protégé par un seul mutex portant sur une structure `Job` qui contient : le statut courant, le drapeau `active`, le drapeau `cancel_requested`, le **handle du processus enfant en cours**, et le chemin du workspace temporaire.

Le mutex n'est **jamais** conservé à travers un `.await`. Il est pris pour des opérations courtes (lire/remplacer le statut, prendre le handle, basculer un drapeau) puis relâché immédiatement. Une transcription d'une heure ne détient donc aucun verrou. Pas de `static mut`, pas de drapeau non synchronisé.

### Double lancement impossible

`start_transcription` effectue un **test-and-set atomique** (`JobState::try_claim`) : la revendication de l'emplacement et le lancement de la tâche ne sont plus séparés par une fenêtre exploitable. Un second appel obtient `alreadyRunning`. Cette garantie est côté Rust et ne dépend pas de l'état du bouton React.

L'emplacement est libéré par un garde RAII (`JobSlotGuard`) sur **tous** les chemins de sortie de `run` — succès, erreur métier, retour anticipé, annulation. C'est ce qui rend le retry possible sans redémarrer l'application.

### Ownership des processus enfants

FFmpeg passe de `.output()` à `.spawn()` : le handle (`CommandChild`, exposant `pid()` et `kill()`) est enregistré dans `JobState` pendant toute la durée du processus, puis retiré dès que son flux d'événements se ferme. whisper-cli fait de même. À tout instant, au plus un handle est détenu, et Rust sait lequel des deux sidecars tourne.

Ces handles ne sont **jamais** exposés au frontend : le frontend n'a que `cancel_transcription`, sans argument.

Une course subsiste théoriquement entre `spawn()` et l'enregistrement du handle. Elle est traitée dans `register_child` : le drapeau d'annulation est vérifié avant l'enregistrement (le processus est tué immédiatement) **et** après (le handle est repris puis tué), de sorte qu'une annulation tombant dans cette fenêtre ne laisse pas de processus vivant.

### Mécanisme d'annulation

```text
clic Annuler
  → cancel_transcription (Rust)
  → cancel_requested = true, handle repris hors du verrou
  → état cancelling émis
  → kill() sur le processus actif
  → le flux d'événements du sidecar se ferme (le processus est réellement mort)
  → la boucle du pipeline observe cancel_requested
  → le garde TempJobDir supprime le workspace
  → état cancelled émis
  → JobSlotGuard libère l'emplacement
```

L'état `cancelled` n'est émis qu'**après** la terminaison effective du processus : il n'est pas une simple bascule d'interface. Des points de contrôle d'annulation existent également entre les étapes (avant FFmpeg, entre FFmpeg et Whisper, avant la publication des sorties), pour couvrir une annulation demandée alors qu'aucun processus ne tourne.

`kill()` envoie SIGKILL via `shared_child`. Aucun délai d'attente arbitraire n'est imposé à Whisper : une vidéo d'une heure est un usage légitime (cf. §33 de la mission). Le seul « délai » du système est l'attente naturelle de la fermeture du flux d'événements après le kill.

### Garanties sur les fichiers de sortie

whisper.cpp écrit **uniquement** dans le workspace temporaire. Le dossier visible par l'utilisateur (`<Média>-sous-titres/`) est créé par `write_outputs` et nulle part ailleurs, après la réussite de Whisper et après le dernier point de contrôle d'annulation.

Conséquence : une annulation ou une erreur ne peut pas publier de sortie partielle. Renforcement ajouté en M4 — si une copie échoue en cours de publication, le dossier fraîchement créé est **supprimé** plutôt que laissé partiellement rempli. `resolve_output_dir` ne renvoyant qu'un chemin inexistant, cette suppression ne peut jamais détruire des données utilisateur préexistantes.

### Fermeture de l'application

`RunEvent::ExitRequested` et `RunEvent::Exit` appellent `pipeline::shutdown`, qui tue le processus enfant éventuel et supprime le workspace temporaire du job actif. La fonction est idempotente et ne bloque jamais (SIGKILL immédiat, pas d'attente) : la fermeture n'est pas retardée.

### Récupération au lancement

Au `setup`, avant l'affichage de la fenêtre, ST-IA nettoie deux emplacements — et strictement ces deux-là :

* `{temp}/ST-IA/<pid>-<nanos>/` : workspaces de jobs. Seules les entrées **directement** dans ce répertoire sont examinées (pas de parcours récursif du répertoire temporaire du système), elles doivent être des répertoires, et leur nom doit correspondre exactement à la forme `<pid>-<nanos>` générée par l'application. Tout le reste est explicitement ignoré et journalisé.
* `{Application Support}/<bundle-id>/models/ggml-large-v3-turbo-q5_0.bin.download` : téléchargement interrompu.

Ne sont jamais candidats à la suppression : le modèle valide, les fichiers sources de l'utilisateur, les dossiers de sortie.

### Téléchargement de modèle interrompu — stratégie A

Le fichier `.download` d'un téléchargement interrompu est **supprimé au lancement suivant**, et un nouveau téléchargement repart de zéro. Pas de reprise HTTP par plage d'octets (hors périmètre M3 comme M4).

Cette stratégie est sûre par construction : M3 écrit toujours dans le fichier temporaire puis vérifie le SHA-256 avant un renommage atomique. Une interruption ne peut donc jamais produire un fichier final partiel — au pire un `.download` résiduel, que le nettoyage au démarrage élimine.

**Limite assumée** : aucun bouton d'annulation n'est ajouté à l'écran de téléchargement en M4. La priorité fixée par la mission est la fermeture sûre et l'absence de fichier final partiel, toutes deux satisfaites. Fermer l'application pendant un téléchargement reste la façon de l'interrompre.

### Concurrence modèle / transcription

`install_model` refuse de démarrer si un job de transcription est actif (le job lit précisément le fichier modèle). Un second téléchargement concurrent était déjà refusé en M3. Le pipeline, de son côté, vérifie que le modèle est `ready` avant d'invoquer whisper-cli.

### Espace disque

Garde grossier, volontairement simple, évalué **avant** de lancer FFmpeg via `statvfs` sur le volume du workspace temporaire.

Formule : `espace_requis = taille_du_fichier_source + 256 Mio`.

Justification : le WAV intermédiaire est du 16 kHz mono s16, soit 32 ko/s = **256 kbit/s**. Tout média audio/vidéo réel est encodé à un débit supérieur, donc la taille du fichier source majore celle du WAV à extraire. On exige ce majorant plus une marge fixe couvrant les sorties SRT/TXT et le surcoût du système de fichiers. Si `statvfs` échoue, la vérification est **ignorée** plutôt que de bloquer un job sur une valeur inconnue.

Ce n'est pas un gestionnaire de stockage : c'est un garde contre un disque manifestement plein.

### États du job

```text
idle
preparingAudio
transcribing      (phases: loadingModel | processing)
writingOutputs
completed
failed
cancelling
cancelled
```

Le frontend reçoit un état métier ; ni PID, ni handle, ni stderr ne franchissent la frontière. Après `cancelled`, l'interface revient à l'écran « fichier sélectionné » — l'état `cancelled` n'a pas d'écran dédié, conformément à la consigne de ne pas multiplier les écrans.

### Erreurs remontées à l'utilisateur

Le stderr de FFmpeg n'est jamais affiché. Il est inspecté par une fonction pure (`classify_ffmpeg_failure`) qui distingue deux cas métier :

| Constat réel du sidecar | Code métier |
|---|---|
| `Output file does not contain any stream` | `noAudioTrack` |
| `moov atom not found` / `Invalid data found` | `audioPreparationFailed` |

Ces chaînes ont été relevées sur le sidecar épinglé, avec les arguments de production, contre des fixtures réelles (voir Qualification).

## Qualification

### Comportement réel du sidecar FFmpeg (mesuré)

Fixtures : un `.mov` vidéo seule (sans piste audio) et un `.mov` de 4 096 octets aléatoires. Arguments de production exacts.

| Fixture | Code de sortie | WAV produit | Sentinelle « no audio » | Classification |
|---|---|---|---|---|
| vidéo sans audio | 234 | non | oui | `noAudioTrack` |
| contenu invalide | 183 | non | non | `audioPreparationFailed` |
| fichier vide | 183 | non | non | `audioPreparationFailed` (déjà rejeté en amont par la validation M1) |

La fixture « vidéo sans audio » a été fabriquée avec un FFmpeg système, **uniquement pour produire la donnée de test** : le sidecar de ST-IA est un build minimal qui ne sait muxer que du WAV. Aucune dépendance d'exécution n'est introduite — l'application n'utilise que son propre sidecar.

### Tests automatisés

Couverts par des tests unitaires : refus du double démarrage, libération de l'emplacement (retry), remise à zéro du drapeau d'annulation entre deux jobs, annulation sans effet à vide, suivi du workspace pour la fermeture, suppression du workspace par le garde RAII, non-publication d'un dossier de sortie partiel, publication des seules sorties demandées, sûreté des chemins de nettoyage (noms de workspace acceptés/refusés), classification des échecs FFmpeg, absence de fuite de stderr dans les messages utilisateur, sérialisation des états `cancelling`/`cancelled`, formule d'espace disque.

Non couvert par des tests automatisés : la mort effective d'un processus enfant réel. Le type `CommandChild` du plugin shell ne peut pas être construit hors d'un contexte Tauri (`Command::new` est `pub(crate)`), ce qui rend ce scénario intestable en unitaire. Il a donc été qualifié manuellement (ci-dessous).

### Qualification manuelle

Réalisée sur le `.app` empaqueté (jamais `pnpm tauri dev`), clics utilisateur réels, surveillance système en parallèle. Les processus enfants ont été identifiés par le **chemin de leur exécutable dans le bundle**, afin qu'un `ffmpeg` appartenant à une autre application ne puisse jamais être confondu avec un processus ST-IA.

**Test A — annulation pendant Whisper.** whisper-cli PID 89075 actif pendant au moins 8 s (transcription en cours), puis clic sur « Annuler ». Après : processus absent, workspace `27712-…`-équivalent supprimé, **aucun dossier de sortie créé** par ce cycle, aucun orphelin. La terminaison est bien prématurée : un job mené à terme publie son dossier de sortie, ce qui n'a pas eu lieu.

**Test B — retry sans redémarrage.** Dans la **même session applicative** (PID inchangé), un nouveau job a été accepté après l'annulation et mené jusqu'au bout : `IMG_8484.srt` (4959 o) et `IMG_8484.txt` (3428 o), SRT couvrant l'intégralité du média (premier segment à 00:00:00,920, dernier à 00:03:00,440). L'emplacement de job est donc bien libéré par le garde RAII.

**Test C — fermeture pendant Whisper (⌘Q).** Preuve directe au niveau du journal applicatif :

```text
[st-ia] whisper-cli pid 44374
[st-ia] shutdown: killed child pid 44374
[st-ia] shutdown: removed temp workspace …/ST-IA/27712-1786366990859354000
```

Après fermeture : aucun `st-ia`, `ffmpeg` ni `whisper-cli` résiduel ; workspace supprimé ; aucune sortie incomplète publiée. À la réouverture : démarrage normal, `0 stale job dir(s)`, modèle toujours `valid=true`, application utilisable. Le scénario s'est reproduit à l'identique lors d'une seconde fermeture (`killed child pid 52965`).

**Test D — média invalide.** Fixture de 8 Ko de données aléatoires en `.mov`. FFmpeg échoue (`exit 183`, `moov atom not found`, aucun WAV), la sentinelle « no audio » est absente donc l'erreur est classée `audioPreparationFailed` et non `noAudioTrack`. **whisper-cli n'est jamais lancé.** Le stderr de FFmpeg reste dans le journal développeur ; l'interface n'affiche que le message métier. Workspace supprimé, aucune sortie. Un nouveau fichier a ensuite pu être choisi et transcrit sans redémarrage.

### Limite de couverture assumée

L'annulation **pendant l'étape FFmpeg** n'a pas été observée en conditions réelles : sur les médias qualifiés, l'extraction audio dure moins de deux secondes, soit moins que l'intervalle d'échantillonnage. Le chemin de code est identique à celui de Whisper (même enregistrement de handle, même `kill`, même point de contrôle), et le Test D prouve que FFmpeg est bien lancé avec un PID suivi et un code de sortie exploité — mais la mort d'un FFmpeg sous annulation reste non mesurée.

## Conséquences

* Le bouton « Annuler » devient fonctionnel et tue réellement le processus actif.
* Un `.app` tué pendant un job ne laisse plus de workspace orphelin (nettoyage à la fermeture, filet de sécurité au démarrage suivant).
* Le retry après échec ou annulation ne nécessite plus de redémarrage.
* Le coût est une gestion explicite du handle enfant dans le pipeline, en échange d'un cycle de vie réellement maîtrisé.
