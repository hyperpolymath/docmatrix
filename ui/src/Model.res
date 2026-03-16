// SPDX-License-Identifier: PMPL-1.0-or-later
// Formatrix Docs - Application state types

type documentFormat =
  | Txt
  | Md
  | Adoc
  | Djot
  | Org
  | Rst
  | Typ

let formatToString = format =>
  switch format {
  | Txt => "txt"
  | Md => "md"
  | Adoc => "adoc"
  | Djot => "djot"
  | Org => "org"
  | Rst => "rst"
  | Typ => "typ"
  }

let formatFromString = str =>
  switch str {
  | "txt" => Some(Txt)
  | "md" => Some(Md)
  | "adoc" => Some(Adoc)
  | "djot" => Some(Djot)
  | "org" => Some(Org)
  | "rst" => Some(Rst)
  | "typ" => Some(Typ)
  | _ => None
  }

let formatLabel = format =>
  switch format {
  | Txt => "TXT"
  | Md => "MD"
  | Adoc => "ADOC"
  | Djot => "DJOT"
  | Org => "ORG"
  | Rst => "RST"
  | Typ => "TYP"
  }

let allFormats = [Txt, Md, Adoc, Djot, Org, Rst, Typ]

type documentMeta = {
  path: option<string>,
  format: documentFormat,
  modified: bool,
  wordCount: int,
  charCount: int,
}

type document = {
  content: string,
  meta: documentMeta,
}

type conversionState =
  | Idle
  | Converting
  | ConversionFailed(string)

type viewMode =
  | Edit
  | Preview
  | Split

type t = {
  document: document,
  activeFormat: documentFormat,
  originalFormat: documentFormat,
  convertedContent: Dict.t<string>,
  conversionState: conversionState,
  viewMode: viewMode,
  loading: bool,
  error: option<string>,
  editorReady: bool,
  showGraph: bool,
}

let emptyDocument: document = {
  content: "",
  meta: {
    path: None,
    format: Md,
    modified: false,
    wordCount: 0,
    charCount: 0,
  },
}

let initial: t = {
  document: emptyDocument,
  activeFormat: Md,
  originalFormat: Md,
  convertedContent: Dict.make(),
  conversionState: Idle,
  viewMode: Edit,
  loading: false,
  error: None,
  editorReady: false,
  showGraph: false,
}
