/**
 * French catalogue — the reference language. `en.ts` must have exactly the
 * same key shape (checked by scripts/check-i18n-parity.mjs and by a Vitest
 * test); values are never empty.
 */
const fr = {
  common: {
    localProcessing: "Traitement 100 % local",
  },
  drop: {
    title: "Déposez votre média ici",
    selectFile: "ou sélectionner un fichier",
  },
  media: {
    changeFile: "Changer de fichier",
    generate: "Générer les sous-titres",
    generateDisabledTitle: "Le modèle de transcription doit être installé au préalable.",
    outputs: "Sorties",
    outputsError: "Sélectionnez au moins un format de sortie.",
    srt: "SRT",
    txt: "TXT",
  },
  transcription: {
    language: "Langue de transcription",
    languageFrench: "Français",
    cancel: "Annuler",
    cancelling: "Annulation…",
  },
  progress: {
    label: "Progression",
    audioPreparation: "Préparation de l'audio",
    modelLoading: "Chargement du modèle",
    transcription: "Transcription",
    outputGeneration: "Génération des fichiers",
    stateDone: "Terminée",
    stateActive: "En cours…",
    statePending: "En attente",
    cancellingText: "Arrêt du traitement en cours…",
  },
  success: {
    title: "Sous-titres générés",
    subtitle_one: "{{count}} fichier créé avec succès.",
    subtitle_other: "{{count}} fichiers créés avec succès.",
    openFolder: "Ouvrir le dossier",
    openFolderError: "Impossible d'ouvrir le dossier.",
    copyTranscript: "Copier la transcription",
    copied: "Copié !",
    newFile: "Nouveau fichier",
    readyForDavinci: "Prêt pour DaVinci Resolve",
  },
  error: {
    chooseAnother: "Choisir un autre fichier",
    installModel: "Installer le modèle",
    retry: "Réessayer",
    technicalDetails: "Détails techniques",
    fieldError: "Erreur",
    fieldCode: "Code",
    fieldFile: "Fichier",
    codes: {
      modelMissing: {
        title: "Modèle non installé",
        message: "Le modèle de transcription n'est pas encore installé.",
      },
      noAudioTrack: {
        title: "Impossible de traiter ce fichier",
        message:
          "Aucune piste audio détectée. Vérifiez que la vidéo contient de l'audio ou choisissez un autre fichier.",
      },
      audioPreparationFailed: {
        title: "Impossible de traiter ce fichier",
        message: "L'audio n'a pas pu être préparé à partir de ce fichier.",
      },
      transcriptionFailed: {
        title: "La transcription a échoué",
        message: "La transcription a échoué.",
      },
      writeFailed: {
        title: "Impossible d'enregistrer les fichiers",
        message: "Les fichiers de sous-titres n'ont pas pu être enregistrés.",
      },
      noOutputSelected: {
        title: "Aucune sortie sélectionnée",
        message: "Sélectionnez au moins un format de sortie (SRT ou TXT).",
      },
      alreadyRunning: {
        title: "Transcription déjà en cours",
        message: "Une transcription est déjà en cours.",
      },
      insufficientDiskSpace: {
        title: "Espace disque insuffisant",
        message: "L'espace disque disponible est insuffisant pour traiter ce fichier.",
      },
    },
    mediaCodes: {
      notFound: {
        title: "Fichier introuvable",
        message: "Le fichier sélectionné est introuvable.",
      },
      unsupported: {
        title: "Format non pris en charge",
        message: "Ce format de fichier n'est pas pris en charge.",
      },
      empty: {
        title: "Fichier vide",
        message: "Le fichier sélectionné est vide.",
      },
      multipleFiles: {
        title: "Plusieurs fichiers",
        message: "Veuillez sélectionner un seul fichier.",
      },
      unknown: {
        title: "Erreur",
        message: "Une erreur inattendue est survenue.",
      },
    },
    modelCodes: {
      networkError: {
        title: "Erreur réseau",
        message: "Une erreur réseau est survenue pendant le téléchargement.",
      },
      writeError: {
        title: "Erreur d'écriture",
        message: "Le modèle n'a pas pu être enregistré sur le disque.",
      },
      integrityMismatch: {
        title: "Fichier invalide",
        message: "Le fichier téléchargé est invalide.",
      },
    },
  },
  model: {
    requiredTitle: "Le modèle local n'est pas encore installé",
    requiredSubtitle: "Télécharger le modèle pour utiliser l'application hors ligne.",
    name: "Whisper — large-v3-turbo",
    sizeApprox: "~{{size}}",
    storageInfo: "Le modèle sera stocké sur votre ordinateur.",
    later: "Plus tard",
    download: "Télécharger",
    downloading: "Téléchargement du modèle",
    downloadedOf: "{{downloaded}} / {{total}}",
    verifying: "Vérification du modèle",
    corruptedTitle: "Modèle endommagé",
    corruptedSubtitle: "Le modèle de transcription est endommagé et doit être réinstallé.",
    reinstall: "Réinstaller le modèle",
  },
  settings: {
    title: "Réglages",
    open: "Réglages",
    close: "Fermer",
    general: "Général",
    appearance: "Apparence",
    accessibility: "Accessibilité",
    language: "Langue",
    about: "À propos",
    themeLabel: "Thème",
    themeSystem: "Système",
    themeLight: "Clair",
    themeDark: "Sombre",
    motionLabel: "Réduire les animations",
    motionSystem: "Suivre le système",
    motionOn: "Activé",
    motionOff: "Désactivé",
    languageLabel: "Langue de l'application",
    languageSystem: "Système",
    languageFrench: "Français",
    languageEnglish: "English",
    themeQuickSystem: "Thème : Système. Passer en mode clair.",
    themeQuickLight: "Thème : Clair. Passer en mode sombre.",
    themeQuickDark: "Thème : Sombre. Revenir au thème système.",
  },
  about: {
    title: "À propos",
    tagline: "Transcription locale de médias en sous-titres.",
    version: "Version {{version}}",
    localProcessing: "Traitement local — vos médias ne quittent jamais votre Mac.",
    licenses: "Licences des composants tiers",
  },
} as const;

export default fr;
