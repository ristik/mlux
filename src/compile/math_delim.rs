//! Normalize alternative LaTeX math delimiters to the `$`/`$$` form that
//! pulldown-cmark understands.
//!
//! Many LLMs emit `\(...\)` for inline math and `\[...\]` for display math.
//! pulldown-cmark treats these as backslash escapes, so the formula would be
//! mangled by emphasis parsing (`_`, `*`) before we ever see it. We therefore
//! rewrite the delimiters in the source before parsing.
//!
//! Rewrites (inline `$x$` must not have whitespace next to the `$`, so no
//! padding is possible and inline delimiters shrink by one byte):
//!
//! - `\(` → `$` and `\)` → `$`    (inline)
//! - `\[` → `$$` and `\]` → `$$`  (display, same length)
//!
//! Newlines are untouched. Because inline delimiters change the byte length,
//! the result carries a [`OffsetMap`] to translate offsets in the rewritten
//! text back to the original Markdown (source map, sidebar lines, yank).
//!
//! Detection is per-delimiter and by pairing: a `\(` is only rewritten when a
//! matching `\)` follows in the same paragraph, so a stray escaped paren in
//! prose is left alone. Code spans and fenced code blocks are never touched.
//! `$...$` / `$$...$$` are handled natively by pulldown-cmark and mix freely.

use std::borrow::Cow;

/// Maps byte offsets of the normalized text back to the original text.
#[derive(Default)]
pub(crate) struct OffsetMap {
    /// Offsets (in the normalized text) of each `$` that replaced a 2-byte
    /// inline delimiter, ascending. Each one shifted later text left by 1.
    shrink_at: Vec<usize>,
}

impl OffsetMap {
    pub(crate) fn to_original(&self, off: usize) -> usize {
        off + self.shrink_at.partition_point(|&p| p < off)
    }

    pub(crate) fn map_range(&self, r: std::ops::Range<usize>) -> std::ops::Range<usize> {
        if self.shrink_at.is_empty() {
            return r;
        }
        self.to_original(r.start)..self.to_original(r.end)
    }
}

pub(crate) fn normalize_math_delimiters(md: &str) -> (Cow<'_, str>, OffsetMap) {
    if !md.contains("\\(") && !md.contains("\\[") {
        return (Cow::Borrowed(md), OffsetMap::default());
    }
    let src = md.as_bytes();
    let code = code_mask(src);
    let mut out: Vec<u8> = Vec::with_capacity(src.len());
    let mut map = OffsetMap::default();
    let mut changed = false;
    let mut i = 0;
    let mut copied = 0; // src[..copied] already emitted
    while i < src.len() {
        if code[i] || src[i] != b'\\' || i + 1 >= src.len() {
            i += 1;
            continue;
        }
        let (open, close) = match src[i + 1] {
            b'(' => (b'(', b')'),
            b'[' => (b'[', b']'),
            _ => {
                i += 2; // escaped char (including `\\`)
                continue;
            }
        };
        match find_close(src, &code, i + 2, close) {
            Some(j) => {
                let rep: &[u8] = if open == b'(' { b"$" } else { b"$$" };
                for pos in [i, j] {
                    out.extend_from_slice(&src[copied..pos]);
                    if rep.len() < 2 {
                        map.shrink_at.push(out.len());
                    }
                    out.extend_from_slice(rep);
                    copied = pos + 2;
                }
                changed = true;
                i = j + 2;
            }
            None => i += 2,
        }
    }
    if !changed {
        return (Cow::Borrowed(md), OffsetMap::default());
    }
    out.extend_from_slice(&src[copied..]);
    // Only ASCII delimiters were replaced, so this stays valid UTF-8.
    let out = String::from_utf8(out).expect("ASCII-only replacement");
    (Cow::Owned(out), map)
}

/// Find `\<close>` starting at `from`, stopping at a blank line (paragraph end)
/// or at code. Returns the index of the backslash.
fn find_close(src: &[u8], code: &[bool], from: usize, close: u8) -> Option<usize> {
    let mut j = from;
    let mut newline_run = false; // saw "\n" + optional spaces since last content
    while j < src.len() {
        if code[j] {
            return None;
        }
        match src[j] {
            b'\n' => {
                if newline_run {
                    return None;
                }
                newline_run = true;
            }
            b' ' | b'\t' | b'\r' => {}
            b'\\' if j + 1 < src.len() => {
                newline_run = false;
                if src[j + 1] == close {
                    return Some(j);
                }
                j += 1; // skip escaped char
            }
            _ => newline_run = false,
        }
        j += 1;
    }
    None
}

/// Mark bytes belonging to fenced code blocks and inline code spans.
fn code_mask(src: &[u8]) -> Vec<bool> {
    let mut mask = vec![false; src.len()];
    let mut fence: Option<(u8, usize)> = None; // (char, run length)
    let mut pos = 0;
    while pos < src.len() {
        let end = src[pos..]
            .iter()
            .position(|&b| b == b'\n')
            .map_or(src.len(), |n| pos + n + 1);
        let line = &src[pos..end];
        let indent = line.iter().take_while(|&&b| b == b' ').count();
        let rest = &line[indent.min(3)..];
        let run = fence_run(rest);
        match fence {
            Some((ch, len)) => {
                mask[pos..end].fill(true);
                if let Some((c, n)) = run
                    && c == ch
                    && n >= len
                    && rest[n..].iter().all(|b| b.is_ascii_whitespace())
                {
                    fence = None;
                }
            }
            None => {
                if indent <= 3
                    && let Some((c, n)) = run
                {
                    fence = Some((c, n));
                    mask[pos..end].fill(true);
                }
            }
        }
        pos = end;
    }
    // Inline code spans: a backtick run closed by a run of equal length.
    let mut i = 0;
    while i < src.len() {
        if mask[i] || src[i] != b'`' {
            i += 1;
            continue;
        }
        let n = src[i..].iter().take_while(|&&b| b == b'`').count();
        let mut k = i + n;
        let mut closed = None;
        while k < src.len() {
            if mask[k] {
                break;
            }
            if src[k] == b'`' {
                let m = src[k..].iter().take_while(|&&b| b == b'`').count();
                if m == n {
                    closed = Some(k + m);
                    break;
                }
                k += m;
            } else {
                k += 1;
            }
        }
        match closed {
            Some(e) => {
                mask[i..e].fill(true);
                i = e;
            }
            None => i += n,
        }
    }
    mask
}

fn fence_run(line: &[u8]) -> Option<(u8, usize)> {
    let c = *line.first()?;
    if c != b'`' && c != b'~' {
        return None;
    }
    let n = line.iter().take_while(|&&b| b == c).count();
    (n >= 3).then_some((c, n))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn norm(s: &str) -> String {
        normalize_math_delimiters(s).0.into_owned()
    }

    #[test]
    fn inline_paren() {
        assert_eq!(norm(r"a \(x_1\) b"), "a $x_1$ b");
    }

    #[test]
    fn display_bracket() {
        assert_eq!(norm("\\[\nx^2\n\\]"), "$$\nx^2\n$$");
    }

    #[test]
    fn untouched_without_pair() {
        assert_eq!(norm(r"a \( b"), r"a \( b");
        assert!(matches!(
            normalize_math_delimiters("plain $x$").0,
            Cow::Borrowed(_)
        ));
    }

    #[test]
    fn no_pair_across_paragraph() {
        let s = "a \\( b\n\nc \\) d";
        assert_eq!(norm(s), s);
    }

    #[test]
    fn code_is_skipped() {
        let s = "`\\(x\\)`\n\n```\n\\[y\\]\n```\n";
        assert_eq!(norm(s), s);
    }

    #[test]
    fn escaped_backslash_not_delimiter() {
        let s = r"a \\(b\\) c";
        assert_eq!(norm(s), s);
    }

    #[test]
    fn multiple_and_mixed() {
        assert_eq!(norm(r"\(a\) and \(b\) $c$"), "$a$ and $b$ $c$");
    }

    #[test]
    fn offsets_map_back_to_original() {
        let orig = r"\(a\) b \[c\] \(dd\) end";
        let (n, map) = normalize_math_delimiters(orig);
        for word in ["b", "c", "dd", "end"] {
            let at = n.find(word).unwrap();
            let o = map.to_original(at);
            assert_eq!(&orig[o..o + word.len()], word, "{word} in {n:?}");
        }
        // Start of a rewritten delimiter maps to its backslash.
        let at = n.rfind('$').unwrap();
        assert_eq!(&orig[map.to_original(at)..][..2], r"\)");
    }

    #[test]
    fn unicode_preserved() {
        assert_eq!(norm(r"õ \(α\) ü"), "õ $α$ ü");
    }
}
