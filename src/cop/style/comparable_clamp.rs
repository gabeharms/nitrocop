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
        // Pattern A: [[x, low].max, high].min or [[x, high].min, low].max
        if let Some(call) = node.as_call_node() {
            if let Some((x, min, max)) = extract_array_clamp(&call, source) {
                let loc = call.location();
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
                return;
            }
        }

        // Pattern B: if x < low then low elsif x > high then high else x end
        // (or with > / reversed operand positions)
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

/// Extract clamp from array pattern: `[[x, low].max, high].min` or `[[x, high].min, low].max`.
fn extract_array_clamp(
    outer_call: &ruby_prism::CallNode<'_>,
    source: &SourceFile,
) -> Option<(String, String, String)> {
    let outer_method = outer_call.name().as_slice();
    if outer_method != b"min" && outer_method != b"max" {
        return None;
    }
    // Must have no arguments (it's `array.min`, not `array.min(something)`)
    if outer_call.arguments().is_some() {
        return None;
    }
    // Receiver must be an ArrayNode with exactly 2 elements
    let outer_recv = outer_call.receiver()?;
    let outer_array = outer_recv.as_array_node()?;
    let outer_elems: Vec<_> = outer_array.elements().iter().collect();
    if outer_elems.len() != 2 {
        return None;
    }

    // One element must be a call to the opposite method on an array
    let (inner_call_node, outer_bound_node) = if let Some(c) = outer_elems[0].as_call_node() {
        (c, &outer_elems[1])
    } else if let Some(c) = outer_elems[1].as_call_node() {
        (c, &outer_elems[0])
    } else {
        return None;
    };

    let inner_method = inner_call_node.name().as_slice();
    // Inner must be opposite of outer
    let valid_pair = (outer_method == b"min" && inner_method == b"max")
        || (outer_method == b"max" && inner_method == b"min");
    if !valid_pair {
        return None;
    }
    if inner_call_node.arguments().is_some() {
        return None;
    }

    let inner_recv = inner_call_node.receiver()?;
    let inner_array = inner_recv.as_array_node()?;
    let inner_elems: Vec<_> = inner_array.elements().iter().collect();
    if inner_elems.len() != 2 {
        return None;
    }

    let x_src = node_src(&inner_elems[0], source);
    let inner_bound_src = node_src(&inner_elems[1], source);
    let outer_bound_src = node_src(outer_bound_node, source);

    // Determine min/max based on outer method
    let (min, max) = if outer_method == b"min" {
        // [[x, low].max, high].min → low is min, high is max
        (inner_bound_src, outer_bound_src)
    } else {
        // [[x, high].min, low].max → low is min, high is max
        (outer_bound_src, inner_bound_src)
    };

    Some((x_src, min, max))
}

fn node_src(node: &ruby_prism::Node<'_>, source: &SourceFile) -> String {
    let loc = node.location();
    String::from_utf8_lossy(&source.as_bytes()[loc.start_offset()..loc.end_offset()]).to_string()
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
