;; SPDX-License-Identifier: MPL-2.0-or-later
;; NEUROSYM.scm - Neurosymbolic integration config
;; Copyright (C) 2025 Jonathan D.A. Jewell

(define neurosym-config
  `((version . "1.0.0")

    (symbolic-layer
      ((type . "scheme")
       (reasoning . "deductive")
       (verification . "formal")
       (representations
         ((document-ast . "Block | Inline | Document")
          (graph-model . "ArangoDB vertices + edges")
          (format-spec . "Parser + Renderer traits")))))

    (neural-layer
      ((embeddings . #f)  ; v2: semantic search
       (fine-tuning . #f)
       (ocr-model . "tesseract")
       (stt-model . "vosk")
       (tts-engine . "espeak-ng")))

    (integration
      ((format-detection . "symbolic")  ; File extension + magic bytes
       (content-search . "symbolic")    ; v2: hybrid with embeddings
       (graph-analysis . "symbolic")    ; v3: neural for clustering
       (ocr-pipeline . "neural")
       (voice-pipeline . "neural")))

    (future-v3
      ((reasoning-diagrams . "Flying Logic-style symbolic reasoning")
       (collaborative . "CRDT-based with symbolic merge")
       (semantic-search . "Embeddings + symbolic filters")))))
