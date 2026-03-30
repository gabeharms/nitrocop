use crate::cop::node_type::{CALL_NODE, ELSE_NODE, IF_NODE};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct ComparableClamp;

impl Cop for ComparableClamp {
    fn name(&self) -> &'static str {
        "Style/ComparableClamp"
    }

    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn interested_node_types(&self) -> &'static [u8] {
        &[CALL_NODE, ELSE_NODE, IF_NODE]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: &ruby_prism::Node<'_>,
        _parse_result: &ruby_prism::ParseResult<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        // Pattern: if x < low then low elsif x > high then high else x end
        // (or with > / reversed operand positions)
        // Must match RuboCop's exact structural pattern:
        // - The if body must equal the bound from the condition
        // - The elsif body must equal the bound from the condition
        // - The else body must equal the clamped variable
        let if_node = match node.as_if_node() {
            Some(n) => n,
            None => return,
        };

        // Skip elsif nodes — only check outermost if
        if if_node.if_keyword_loc().is_none() {
            return;
        }
        // Also skip if the keyword is not "if" (could be ternary or modifier)
        if if_node.if_keyword_loc().unwrap().as_slice() != b"if" {
            return;
        }

        // Must have exactly one elsif and an else
        let elsif = match if_node.subsequent() {
            Some(s) => s,
            None => return,
        };

        let elsif_node = match elsif.as_if_node() {
            Some(n) => n,
            None => return, // It's a plain else, not elsif
        };

        // The elsif must have an else (no more elsifs)
        let else_clause = match elsif_node.subsequent() {
            Some(s) => s,
            None => return,
        };

        // Should not have another elsif
        if else_clause.as_if_node().is_some() {
            return;
        }

        // Get the else body as source text
        let else_body = match else_clause.as_else_node() {
            Some(e) => e,
            None => return,
        };
        let else_body_src = get_single_stmt_src(else_body.statements(), source);
        let else_body_src = match else_body_src {
            Some(s) => s,
            None => return,
        };

        // Get the if body source
        let if_body_src = get_single_stmt_src(if_node.statements(), source);
        let if_body_src = match if_body_src {
            Some(s) => s,
            None => return,
        };

        // Get the elsif body source
        let elsif_body_src = get_single_stmt_src(elsif_node.statements(), source);
        let elsif_body_src = match elsif_body_src {
            Some(s) => s,
            None => return,
        };

        // Check conditions: both must be comparisons with < or >
        let first_cmp = get_comparison(&if_node.predicate());
        let second_cmp = get_comparison(&elsif_node.predicate());

        let (f_left, f_op, f_right) = match first_cmp {
            Some(c) => c,
            None => return,
        };
        let (s_left, s_op, s_right) = match second_cmp {
            Some(c) => c,
            None => return,
        };

        // Match one of the 8 patterns from RuboCop and extract clamp parts.
        if let Some((x, min, max)) = clamp_parts(
            &f_left,
            f_op,
            &f_right,
            &if_body_src,
            &s_left,
            s_op,
            &s_right,
            &elsif_body_src,
            &else_body_src,
        ) {
            let loc = if_node.location();
            let (line, column) = source.offset_to_line_col(loc.start_offset());
            let mut diag = self.diagnostic(
                source,
                line,
                column,
                "Use `clamp` instead of `if/elsif/else`.".to_string(),
            );
            if let Some(corr) = corrections.as_mut() {
                corr.push(crate::correction::Correction {
                    start: loc.start_offset(),
                    end: loc.end_offset(),
                    replacement: format!("{}.clamp({}, {})", x, min, max),
                    cop_name: self.name(),
                    cop_index: 0,
                });
                diag.corrected = true;
            }
            diagnostics.push(diag);
        }
    }
}

/// Get source text of a single statement in a StatementsNode.
fn get_single_stmt_src(
    stmts: Option<ruby_prism::StatementsNode<'_>>,
    source: &SourceFile,
) -> Option<String> {
    let stmts = stmts?;
    let body: Vec<_> = stmts.body().iter().collect();
    if body.len() != 1 {
        return None;
    }
    let loc = body[0].location();
    let src = &source.as_bytes()[loc.start_offset()..loc.end_offset()];
    Some(String::from_utf8_lossy(src).to_string())
}

/// Extract comparison operands and operator from `x < y` or `x > y`.
fn get_comparison(node: &ruby_prism::Node<'_>) -> Option<(String, u8, String)> {
    let call = node.as_call_node()?;
    let method = call.name().as_slice();
    let op = match method {
        b"<" => b'<',
        b">" => b'>',
        _ => return None,
    };
    let receiver = call.receiver()?;
    let args = call.arguments()?;
    let arg_list: Vec<_> = args.arguments().iter().collect();
    if arg_list.len() != 1 {
        return None;
    }
    let left_loc = receiver.location();
    let right_loc = arg_list[0].location();
    // Use location slice as source text
    let left = String::from_utf8_lossy(left_loc.as_slice()).to_string();
    let right = String::from_utf8_lossy(right_loc.as_slice()).to_string();
    Some((left, op, right))
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ClampKind {
    Min,
    Max,
}

fn extract_condition_part<'a>(
    left: &'a str,
    op: u8,
    right: &'a str,
    body: &'a str,
) -> Option<(&'a str, ClampKind, &'a str)> {
    if body == left {
        let x = right;
        let kind = if op == b'>' {
            ClampKind::Min
        } else {
            ClampKind::Max
        };
        return Some((x, kind, body));
    }
    if body == right {
        let x = left;
        let kind = if op == b'<' {
            ClampKind::Min
        } else {
            ClampKind::Max
        };
        return Some((x, kind, body));
    }
    None
}

/// Extract `(x, min, max)` if this if/elsif/else is a clamp pattern.
#[allow(clippy::too_many_arguments)]
fn clamp_parts<'a>(
    f_left: &'a str,
    f_op: u8,
    f_right: &'a str,
    if_body: &'a str,
    s_left: &'a str,
    s_op: u8,
    s_right: &'a str,
    elsif_body: &'a str,
    else_body: &'a str,
) -> Option<(&'a str, &'a str, &'a str)> {
    let (x1, kind1, bound1) = extract_condition_part(f_left, f_op, f_right, if_body)?;
    let (x2, kind2, bound2) = extract_condition_part(s_left, s_op, s_right, elsif_body)?;

    if x1 != x2 || else_body != x1 || kind1 == kind2 || bound1 == bound2 {
        return None;
    }

    let (min, max) = if kind1 == ClampKind::Min {
        (bound1, bound2)
    } else {
        (bound2, bound1)
    };

    Some((x1, min, max))
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(ComparableClamp, "cops/style/comparable_clamp");
    crate::cop_autocorrect_fixture_tests!(ComparableClamp, "cops/style/comparable_clamp");
}
