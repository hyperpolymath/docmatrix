// SPDX-License-Identifier: PMPL-1.0-or-later
// Formatrix Docs - Format tabs component

@react.component
let make = (
  ~activeFormat: Model.documentFormat,
  ~originalFormat: Model.documentFormat,
  ~conversionState: Model.conversionState,
  ~onSwitch: Model.documentFormat => unit,
) => {
  let isOriginal = format => format == originalFormat

  <nav className="format-tabs" role="tablist">
    {Model.allFormats
    ->Array.map(format => {
      let isActive = format == activeFormat
      let isConverting = switch conversionState {
      | Model.Converting => format == activeFormat
      | _ => false
      }

      let className =
        [
          "format-tab",
          isActive ? "active" : "",
          isOriginal(format) ? "original" : "",
          isConverting ? "converting" : "",
        ]
        ->Array.filter(s => String.length(s) > 0)
        ->Array.join(" ")

      <button
        key={Model.formatToString(format)}
        className
        role="tab"
        ariaSelected={isActive}
        onClick={_ => onSwitch(format)}
        disabled={isConverting}>
        {React.string(Model.formatLabel(format))}
        {isOriginal(format)
          ? <span className="original-indicator" title="Original format">
              {React.string(" *")}
            </span>
          : React.null}
      </button>
    })
    ->React.array}
  </nav>
}
