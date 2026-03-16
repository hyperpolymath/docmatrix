// SPDX-License-Identifier: PMPL-1.0-or-later
// Formatrix Docs - Main application

%%raw(`import "../styles/main.css"`)

module Toolbar = {
  @react.component
  let make = (
    ~modified: bool,
    ~viewMode: Model.viewMode,
    ~showGraph: bool,
    ~onNew: unit => unit,
    ~onOpen: unit => unit,
    ~onSave: unit => unit,
    ~onViewMode: Model.viewMode => unit,
    ~onToggleGraph: unit => unit,
  ) => {
    <header className="toolbar">
      <div className="toolbar-group">
        <button className="toolbar-button" onClick={_ => onNew()}>
          {React.string("New")}
        </button>
        <button className="toolbar-button" onClick={_ => onOpen()}>
          {React.string("Open")}
        </button>
        <button
          className={`toolbar-button ${modified ? "modified" : ""}`}
          onClick={_ => onSave()}>
          {React.string(modified ? "Save *" : "Save")}
        </button>
      </div>
      <div className="toolbar-group">
        <button
          className={`toolbar-button ${viewMode == Model.Edit ? "active" : ""}`}
          onClick={_ => onViewMode(Model.Edit)}>
          {React.string("Edit")}
        </button>
        <button
          className={`toolbar-button ${viewMode == Model.Split ? "active" : ""}`}
          onClick={_ => onViewMode(Model.Split)}>
          {React.string("Split")}
        </button>
        <button
          className={`toolbar-button ${viewMode == Model.Preview ? "active" : ""}`}
          onClick={_ => onViewMode(Model.Preview)}>
          {React.string("Preview")}
        </button>
      </div>
      <div className="toolbar-group">
        <button
          className={`toolbar-button ${showGraph ? "active" : ""}`}
          onClick={_ => onToggleGraph()}>
          {React.string("Graph")}
        </button>
      </div>
    </header>
  }
}

module StatusBar = {
  @react.component
  let make = (
    ~path: option<string>,
    ~format: Model.documentFormat,
    ~wordCount: int,
    ~charCount: int,
  ) => {
    <footer className="status-bar">
      <span className="status-path">
        {React.string(path->Option.getOr("Untitled"))}
      </span>
      <span className="status-format">
        {React.string(Model.formatLabel(format))}
      </span>
      <span className="status-counts">
        {React.string(`${Int.toString(wordCount)} words | ${Int.toString(charCount)} chars`)}
      </span>
    </footer>
  }
}

module Editor = {
  @react.component
  let make = (
    ~content: string,
    ~format: Model.documentFormat,
    ~readOnly: bool,
    ~onChange: string => unit,
  ) => {
    <div className="editor-container">
      <textarea
        className="editor-textarea"
        value={content}
        readOnly
        onChange={e => {
          let target = e->ReactEvent.Form.target
          onChange(target["value"])
        }}
        placeholder="Start writing..."
      />
    </div>
  }
}

@react.component
let make = () => {
  let (model, setModel) = React.useState(() => Model.initial)

  let dispatch = msg => {
    setModel(prev => {
      switch msg {
      | Msg.NewDocument => {...Model.initial, editorReady: prev.editorReady}

      | Msg.ContentChanged(content) =>
        let wordCount = content->String.split(" ")->Array.filter(s => String.length(s) > 0)->Array.length
        let charCount = String.length(content)
        {
          ...prev,
          document: {
            ...prev.document,
            content,
            meta: {
              ...prev.document.meta,
              modified: true,
              wordCount,
              charCount,
            },
          },
          convertedContent: Dict.fromArray([(Model.formatToString(prev.originalFormat), content)]),
        }

      | Msg.SwitchFormat(format) =>
        if format == prev.activeFormat {
          prev
        } else {
          {...prev, activeFormat: format}
        }

      | Msg.SetViewMode(viewMode) => {...prev, viewMode}

      | Msg.ToggleGraph => {...prev, showGraph: !prev.showGraph}

      | Msg.DismissError => {...prev, error: None}

      | Msg.EditorReady => {...prev, editorReady: true}

      | _ => prev
      }
    })
  }

  let displayContent = switch Dict.get(
    model.convertedContent,
    Model.formatToString(model.activeFormat),
  ) {
  | Some(content) => content
  | None => model.document.content
  }

  let isReadOnly = model.activeFormat != model.originalFormat

  <div className="formatrix-docs">
    {switch model.error {
    | Some(err) =>
      <div className="error-banner" role="alert">
        <span> {React.string(err)} </span>
        <button onClick={_ => dispatch(Msg.DismissError)}>
          {React.string("Dismiss")}
        </button>
      </div>
    | None => React.null
    }}
    <Toolbar
      modified={model.document.meta.modified}
      viewMode={model.viewMode}
      showGraph={model.showGraph}
      onNew={() => dispatch(Msg.NewDocument)}
      onOpen={() => dispatch(Msg.OpenDocument)}
      onSave={() => dispatch(Msg.SaveDocument)}
      onViewMode={mode => dispatch(Msg.SetViewMode(mode))}
      onToggleGraph={() => dispatch(Msg.ToggleGraph)}
    />
    <FormatTabs
      activeFormat={model.activeFormat}
      originalFormat={model.originalFormat}
      conversionState={model.conversionState}
      onSwitch={format => dispatch(Msg.SwitchFormat(format))}
    />
    {model.loading
      ? <div className="loading-overlay">
          <div className="spinner" />
        </div>
      : React.null}
    {switch model.conversionState {
    | Model.ConversionFailed(err) =>
      <div className="conversion-error">
        {React.string(`Conversion failed: ${err}`)}
      </div>
    | _ => React.null
    }}
    <main className="main-content">
      <Editor
        content={displayContent}
        format={model.activeFormat}
        readOnly={isReadOnly}
        onChange={content => dispatch(Msg.ContentChanged(content))}
      />
      {model.showGraph
        ? <div className="graph-panel">
            <div className="graph-placeholder">
              {React.string("Graph View (Coming in v2)")}
            </div>
          </div>
        : React.null}
    </main>
    <StatusBar
      path={model.document.meta.path}
      format={model.originalFormat}
      wordCount={model.document.meta.wordCount}
      charCount={model.document.meta.charCount}
    />
  </div>
}

// Mount
switch ReactDOM.querySelector("#app") {
| Some(root) => ReactDOM.Client.createRoot(root)->ReactDOM.Client.Root.render(<make />)
| None => Console.error("Could not find #app element")
}
