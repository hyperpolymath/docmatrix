// SPDX-License-Identifier: PMPL-1.0-or-later
// Formatrix Docs - Message types

type t =
  // Document operations
  | NewDocument
  | OpenDocument
  | DocumentLoaded(result<Model.document, string>)
  | SaveDocument
  | SaveDocumentAs
  | DocumentSaved(result<Model.documentMeta, string>)
  // Format switching
  | SwitchFormat(Model.documentFormat)
  | ConversionComplete(Model.documentFormat, result<string, string>)
  // Editor
  | EditorReady
  | ContentChanged(string)
  // View mode
  | SetViewMode(Model.viewMode)
  | ToggleGraph
  // UI
  | DismissError
