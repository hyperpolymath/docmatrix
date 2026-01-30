;; SPDX-License-Identifier: MPL-2.0-or-later
;; ECOSYSTEM.scm - Project ecosystem positioning
;; Copyright (C) 2025 Jonathan D.A. Jewell

(ecosystem
  ((version . "1.0.0")
   (name . "Formatrix Docs")
   (type . "application")
   (purpose . "Cross-platform document editor with format tabs and graph visualization")

   (position-in-ecosystem . "core-application")

   (related-projects
     ((recon-silly-ation . ((relationship . "consumer")
                           (description . "Machine reconciliation of documents")))
      (docubot . ((relationship . "assistant")
                 (description . "LLM-powered doc generation")))
      (docudactyl . ((relationship . "orchestrator")
                    (description . "Workflow coordination")))
      (rhodium-standard . "sibling-standard")
      (git-hud . "infrastructure")
      (mustfile . "build-system")
      (nickel . "configuration")))

   (external-inspirations
     ((anytype . "UI/UX and graph concepts")
      (obsidian . "Graph visualization")
      (flying-logic . "Reasoning diagrams (v3)")
      (pandoc . "Format conversion bridge")))

   (integrations
     ((arangodb . "Graph and document storage")
      (tesseract . "OCR engine")
      (hunspell . "Spell checking")
      (espeak-ng . "Text-to-speech")
      (vosk . "Speech-to-text")
      (pandoc . "Extended format support")))

   (what-this-is
     ("A cross-platform document editor")
     ("Format-agnostic with live conversion")
     ("Graph-based knowledge visualization")
     ("Anytype-inspired UI with modern stack"))

   (what-this-is-not
     ("A simple text editor")
     ("A web-only application")
     ("A replacement for Obsidian/Notion")
     ("A collaborative editing platform (until v3)"))))
