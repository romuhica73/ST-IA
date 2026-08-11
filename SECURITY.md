# Politique de sécurité — ST-IA

*English speakers: please report vulnerabilities through GitHub's private
["Report a vulnerability"](https://github.com/romuhica73/ST-IA/security/advisories/new)
form. Reports in English are welcome. The rest of this document is in French.*

---

## Versions supportées

ST-IA est en pré-release. Aucune version n'a encore été publiée.

| Version | Supportée |
|---|---|
| `main` (0.1.0 en développement) | ✅ Oui — seule branche recevant des correctifs |
| Builds locaux antérieurs | ❌ Non |

Tant que 0.1.0 n'est pas publiée, seul l'état courant de `main` est maintenu.
Il n'y a pas de backport.

## Signaler une vulnérabilité

**N'ouvrez pas d'issue publique pour une vulnérabilité.** Une issue est visible
immédiatement par tout le monde, y compris avant qu'un correctif existe.

Utilisez le **Private Vulnerability Reporting** de GitHub :

> Onglet **Security** du dépôt → **Report a vulnerability**

Ce canal est privé entre vous et les mainteneurs, et permet de publier ensuite un
avis de sécurité coordonné.

Si le Private Vulnerability Reporting n'est pas disponible sur le dépôt au moment où
vous lisez ceci, ouvrez une issue **sans détail technique** demandant un canal privé,
et attendez une réponse avant de publier quoi que ce soit.

## Ce qu'un bon rapport contient

* la version de ST-IA (visible dans **Réglages → À propos**) et la version de macOS ;
* le composant concerné (frontend, commande Tauri, pipeline, gestionnaire de modèle,
  sidecar FFmpeg/whisper-cli, script de build) ;
* les étapes de reproduction, aussi précises que possible ;
* l'impact concret : que peut faire un attaquant qu'il ne pouvait pas faire avant ?
  et depuis quelle position de départ ?
* le cas échéant, un fichier de démonstration — **anonymisé**. N'envoyez jamais un
  média personnel réel.

Un rapport qui explique clairement le **modèle d'attaque** (« depuis X, un attaquant
obtient Y ») est bien plus utile qu'un scanner qui signale un motif.

## Périmètre

ST-IA est une application de bureau **locale**. Elle n'a ni serveur, ni compte, ni
API key, ni télémétrie. Le modèle de menace complet est décrit dans
[`docs/security/THREAT_MODEL.md`](docs/security/THREAT_MODEL.md) — merci de le lire
avant de signaler : il documente explicitement les risques déjà connus et acceptés.

### Dans le périmètre

* échappement de la frontière IPC (frontend → Rust) ;
* traversée de chemin, abus de lien symbolique, suppression hors des répertoires ST-IA ;
* exécution de commande ou injection d'argument via un chemin ou un nom de fichier ;
* contournement de la vérification d'intégrité du modèle ;
* toute sortie réseau autre que le téléchargement explicite du modèle ;
* toute fuite d'un média, d'un chemin ou d'une transcription hors de la machine ;
* contournement de la CSP ;
* secret ou donnée personnelle présent dans le dépôt ou son historique.

### Hors périmètre

* Un attaquant disposant déjà d'exécution de code natif sous le compte de
  l'utilisateur : il a déjà accès aux médias, ST-IA ne peut rien y changer.
* L'absence de signature et de notarisation Apple : connue, planifiée pour la
  première release publique.
* Les avis « unmaintained » sur des crates transitives non compilées sur macOS
  (bindings GTK3, notamment) : voir `docs/security/M8_SECURITY_REVIEW.md`.
* Les vulnérabilités d'upstream (FFmpeg, whisper.cpp, ggml) : signalez-les d'abord
  au projet concerné. Dites-le nous quand même si ST-IA y est exposée — nous
  mettrons à jour le sidecar pinné.
* Les rapports issus d'un scanner automatique sans analyse d'atteignabilité.

## Réponse attendue

Projet maintenu par une seule personne, sur temps disponible. Engagements
raisonnables, sans promesse contractuelle :

| Étape | Délai visé |
|---|---|
| Accusé de réception | 7 jours |
| Évaluation initiale (valide / invalide / sévérité) | 30 jours |
| Correctif pour une vulnérabilité critique ou élevée | dès que possible, avant toute nouvelle release |
| Avis de sécurité public | après le correctif, en coordination avec vous |

Vous serez crédité dans l'avis de sécurité, sauf si vous préférez rester anonyme.

Il n'existe **pas** de programme de bug bounty : ce projet n'a aucun financement.

## Divulgation

Divulgation coordonnée. Merci de nous laisser corriger avant publication. Si vous
n'obtenez aucune réponse dans les 30 jours, considérez-vous libre de publier — mais
prévenez-nous d'abord de votre intention.
