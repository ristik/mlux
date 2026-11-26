// Paper: pure white page, pure black text, light gray shading for blocks.
// Latin fonts (Fira Sans); glyphs missing from them fall back to embedded fonts.
#set page(width: 660pt, height: auto, margin: 40pt, fill: rgb("#ffffff"))
#set text(font: "Fira Sans", size: 18pt * scale, fill: rgb("#000000"), lang: "en")
// Leading: 0.75em (~169% line-height). Butterick's 120-145% is print-oriented
// and reads tight on screen (KGP @ 144 PPI); Fira Sans's tall x-height also
// pushes us toward looser values. Keeps visual contrast small against the CJK
// theme (catppuccin-latte.typ) which uses 0.9em (~190%).
#set par(leading: 0.75em, justify: true, first-line-indent: 0pt)

// Headings (above: 1.4em, below: 0.9em — em is relative to font size)
#show heading.where(level: 1): it => text(28pt * scale,
  block(below: 0.9em, above: 1.4em, text(weight: "bold", fill: rgb("#000000"), it.body)))
#show heading.where(level: 2): it => text(24pt * scale,
  block(below: 0.9em, above: 1.4em, text(weight: "bold", fill: rgb("#000000"), it.body)))
#show heading.where(level: 3): it => text(20pt * scale,
  block(below: 0.9em, above: 1.4em, text(weight: "bold", fill: rgb("#000000"), it.body)))

// Code block
#show raw.where(block: true): it => block(
  fill: rgb("#f0f0f0"), inset: 12pt * scale, radius: 6pt * scale, width: 100%,
  text(font: "Fira Mono", size: 12pt * scale, it))

// Inline code (raw's ShowSet applies size: 0.8em, so divide by 0.8 to compensate)
#show raw.where(block: false): it => box(
  fill: rgb("#f0f0f0"), inset: (x: 0.3em / 0.8), outset: (y: 0.15em / 0.8), radius: 3pt * scale,
  text(font: "Fira Mono", size: 0.85em / 0.8, it))

// Lists
#set list(marker: ([•], [‣], [–]), indent: 1em, body-indent: 0.7em)
#set enum(indent: 1em, body-indent: 0.7em)

// Block quote
#show quote.where(block: true): it => block(
  inset: (left: 16pt * scale, y: 8pt * scale),
  stroke: (left: 3pt * scale + rgb("#0b4fc4")),
  text(fill: rgb("#333333"), it.body))

// Tables
#set table(stroke: 0.5pt * scale + rgb("#999999"), inset: 8pt * scale,
  fill: (_, y) => if y == 0 { rgb("#e2e2e2") } else { none })

// Links
#show link: it => text(fill: rgb("#0b4fc4"), underline(it))

// Strong (bold): same color as body, only weight differs
#show strong: set text(fill: rgb("#000000"))

// Strikethrough: Subtext 1 stroke
#show strike: set strike(stroke: 1pt * scale + rgb("#333333"))

// Headings h4-h6
#show heading.where(level: 4): it => text(17pt * scale,
  block(below: 0.9em, above: 1.4em, text(weight: "bold", fill: rgb("#000000"), it.body)))
#show heading.where(level: 5): it => text(15pt * scale,
  block(below: 0.9em, above: 1.4em, text(weight: "bold", fill: rgb("#000000"), it.body)))
#show heading.where(level: 6): it => text(14pt * scale,
  block(below: 0.9em, above: 1.4em, text(weight: "bold", fill: rgb("#333333"), it.body)))

// Math: tuned to balance with body font
#show math.equation: set text(font: "STIX Two Math", size: 20pt * scale)

// Horizontal rule: Surface 2
#show line: set line(stroke: 1pt * scale + rgb("#999999"))

// Image placeholder: Surface 2 border
#let image-placeholder(path) = block(stroke: 0.5pt * scale + rgb("#999999"), inset: 8pt * scale, radius: 4pt * scale)[Image: #path]

// Code block: Catppuccin Latte syntax highlighting
#set raw(theme: "catppuccin-latte.tmTheme")
