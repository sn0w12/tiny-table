use super::{ANSI_RESET, Trunc};

pub(super) fn split_lines(content: &str) -> Vec<String> {
    if content.is_empty() {
        return vec![String::new()];
    }

    let mut lines = Vec::new();
    let mut current = String::new();
    let mut active = String::new();
    let mut index = 0;

    while index < content.len() {
        if let Some(end) = consume_ansi_escape(content, index) {
            let sequence = &content[index..end];
            current.push_str(sequence);

            if is_reset_sequence(sequence) {
                active.clear();
            } else {
                active.push_str(sequence);
            }

            index = end;
            continue;
        }

        let ch = content[index..].chars().next().unwrap();
        index += ch.len_utf8();

        if ch == '\n' {
            if !active.is_empty() && !current.is_empty() && !current.ends_with(ANSI_RESET) {
                current.push_str(ANSI_RESET);
            }

            lines.push(std::mem::take(&mut current));
            if !active.is_empty() {
                current.push_str(&active);
            }
        } else if ch != '\r' {
            current.push(ch);
        }
    }

    if !current.is_empty() || lines.is_empty() {
        if !active.is_empty() && !current.is_empty() && !current.ends_with(ANSI_RESET) {
            current.push_str(ANSI_RESET);
        }

        lines.push(current);
    }

    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}

pub(super) fn layout_line(
    content: &str,
    max_width: Option<usize>,
    truncation: Trunc,
) -> Vec<String> {
    match truncation {
        Trunc::NewLine => wrap_line(content, max_width),
        _ => vec![truncate_line(content, max_width, truncation)],
    }
}

pub(super) fn truncate_line(content: &str, max_width: Option<usize>, truncation: Trunc) -> String {
    const ELLIPSIS: char = '…';

    let Some(max_width) = max_width else {
        return content.to_string();
    };

    let visible = visible_len(content);
    if visible <= max_width {
        return content.to_string();
    }

    match max_width {
        0 => String::new(),
        1 => ELLIPSIS.to_string(),
        _ => {
            let keep = max_width - 1;
            match truncation {
                Trunc::End => {
                    let keep = trim_visible_end_index(content, keep);
                    styled_visible_slice_with_ellipsis_end(content, 0, keep, ELLIPSIS)
                }
                Trunc::Start => {
                    let start = trim_visible_start_index(content, visible - keep);
                    styled_visible_slice_with_ellipsis_start(content, start, visible, ELLIPSIS)
                }
                Trunc::Middle => {
                    let left = keep.div_ceil(2);
                    let right = keep / 2;
                    let left = trim_visible_end_index(content, left);
                    let right = trim_visible_start_index(content, visible - right);
                    styled_visible_slice_with_ellipsis_middle(
                        content, 0, left, right, visible, ELLIPSIS,
                    )
                }
                Trunc::NewLine => content.to_string(),
            }
        }
    }
}

pub(super) fn strip_ansi(content: &str) -> String {
    let mut plain = String::new();
    let mut index = 0;

    while index < content.len() {
        if let Some(end) = consume_ansi_escape(content, index) {
            index = end;
            continue;
        }

        let ch = content[index..].chars().next().unwrap();
        index += ch.len_utf8();

        if ch != '\r' {
            plain.push(ch);
        }
    }

    plain
}

pub(super) fn visible_len(content: &str) -> usize {
    strip_ansi(content).chars().count()
}

fn styled_visible_slice_with_ellipsis_end(
    content: &str,
    start: usize,
    end: usize,
    ellipsis: char,
) -> String {
    let mut slice = slice_visible(content, start, end);
    if slice.is_empty() {
        let prefix = ansi_prefix_at_visible(content, end);
        return if prefix.is_empty() {
            String::new()
        } else {
            format!("{}{}{}", prefix, ellipsis, ANSI_RESET)
        };
    }

    if slice.ends_with(ANSI_RESET) {
        slice.truncate(slice.len() - ANSI_RESET.len());
    }

    format!("{}{}{}", slice, ellipsis, ANSI_RESET)
}

fn styled_visible_slice_with_ellipsis_start(
    content: &str,
    start: usize,
    end: usize,
    ellipsis: char,
) -> String {
    let slice = slice_visible(content, start, end);
    if slice.is_empty() {
        let prefix = ansi_prefix_at_visible(content, start);
        return if prefix.is_empty() {
            String::new()
        } else {
            format!("{}{}{}", prefix, ellipsis, ANSI_RESET)
        };
    }

    let prefix_len = leading_ansi_prefix_len(&slice);
    let prefix = &slice[..prefix_len];
    let mut body = slice[prefix_len..].to_string();

    if body.ends_with(ANSI_RESET) {
        body.truncate(body.len() - ANSI_RESET.len());
    }

    if prefix.is_empty() {
        format!("{}{}{}", ellipsis, body, ANSI_RESET)
    } else {
        format!("{}{}{}{}", prefix, ellipsis, body, ANSI_RESET)
    }
}

fn styled_visible_slice_with_ellipsis_middle(
    content: &str,
    left_start: usize,
    left_end: usize,
    right_start: usize,
    right_end: usize,
    ellipsis: char,
) -> String {
    let mut left = slice_visible(content, left_start, left_end);
    let right = slice_visible(content, right_start, right_end);

    if left.is_empty() {
        let prefix = ansi_prefix_at_visible(content, left_end);
        return if prefix.is_empty() {
            if right.is_empty() {
                String::new()
            } else {
                let right_prefix_len = leading_ansi_prefix_len(&right);
                let mut right_body = right[right_prefix_len..].to_string();
                if right_body.ends_with(ANSI_RESET) {
                    right_body.truncate(right_body.len() - ANSI_RESET.len());
                }

                let right_prefix = &right[..right_prefix_len];
                if right_prefix.is_empty() {
                    format!("{}{}{}", ellipsis, right_body, ANSI_RESET)
                } else {
                    format!("{}{}{}{}", right_prefix, ellipsis, right_body, ANSI_RESET)
                }
            }
        } else {
            format!("{}{}{}", prefix, ellipsis, ANSI_RESET)
        };
    }

    if left.ends_with(ANSI_RESET) {
        left.truncate(left.len() - ANSI_RESET.len());
    }

    if right.is_empty() {
        format!("{}{}{}", left, ellipsis, ANSI_RESET)
    } else {
        format!("{}{}{}{}", left, ellipsis, ANSI_RESET, right)
    }
}

fn trim_visible_end_index(content: &str, end: usize) -> usize {
    let chars: Vec<char> = strip_ansi(content).chars().collect();
    let mut end = end.min(chars.len());

    while end > 0 && chars[end - 1].is_whitespace() {
        end -= 1;
    }

    end
}

fn trim_visible_start_index(content: &str, start: usize) -> usize {
    let chars: Vec<char> = strip_ansi(content).chars().collect();
    let mut start = start.min(chars.len());

    while start < chars.len() && chars[start].is_whitespace() {
        start += 1;
    }

    start
}

fn leading_ansi_prefix_len(content: &str) -> usize {
    let mut index = 0;

    while index < content.len() {
        if let Some(end) = consume_ansi_escape(content, index) {
            index = end;
            continue;
        }

        break;
    }

    index
}

fn ansi_prefix_at_visible(content: &str, target: usize) -> String {
    let mut index = 0;
    let mut visible = 0;
    let mut active = String::new();

    while index < content.len() && visible < target {
        if let Some(end_escape) = consume_ansi_escape(content, index) {
            let sequence = &content[index..end_escape];

            if is_reset_sequence(sequence) {
                active.clear();
            } else {
                active.push_str(sequence);
            }

            index = end_escape;
            continue;
        }

        let ch = content[index..].chars().next().unwrap();
        index += ch.len_utf8();

        if ch != '\r' {
            visible += 1;
        }
    }

    active
}

fn wrap_line(content: &str, max_width: Option<usize>) -> Vec<String> {
    let Some(max_width) = max_width else {
        return vec![content.to_string()];
    };

    if max_width == 0 {
        return vec![String::new()];
    }

    let plain = strip_ansi(content);
    let chars: Vec<char> = plain.chars().collect();
    if chars.is_empty() {
        return vec![String::new()];
    }

    let mut lines = Vec::new();
    let mut start = 0;

    while start < chars.len() {
        let remaining = chars.len() - start;
        if remaining <= max_width {
            lines.push(slice_visible(content, start, chars.len()));
            break;
        }

        let end = start + max_width;
        let wrap_at = chars[start..end]
            .iter()
            .enumerate()
            .rev()
            .find(|(idx, ch)| *idx > 0 && ch.is_whitespace())
            .map(|(idx, _)| start + idx);

        if let Some(split_at) = wrap_at {
            lines.push(slice_visible(content, start, split_at));
            start = split_at;
            while start < chars.len() && chars[start].is_whitespace() {
                start += 1;
            }
        } else {
            lines.push(slice_visible(content, start, end));
            start = end;
        }
    }

    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}

fn consume_ansi_escape(content: &str, index: usize) -> Option<usize> {
    let bytes = content.as_bytes();
    if bytes.get(index) != Some(&0x1b) || bytes.get(index + 1) != Some(&b'[') {
        return None;
    }

    let mut cursor = index + 2;
    while cursor < content.len() {
        let ch = content[cursor..].chars().next()?;
        cursor += ch.len_utf8();

        if ('@'..='~').contains(&ch) {
            return Some(cursor);
        }
    }

    None
}

fn is_reset_sequence(sequence: &str) -> bool {
    sequence == ANSI_RESET
}

fn slice_visible(content: &str, start: usize, end: usize) -> String {
    if start >= end {
        return String::new();
    }

    let mut index = 0;
    let mut visible = 0;
    let mut active = String::new();

    while index < content.len() && visible < start {
        if let Some(end_escape) = consume_ansi_escape(content, index) {
            let sequence = &content[index..end_escape];

            if is_reset_sequence(sequence) {
                active.clear();
            } else {
                active.push_str(sequence);
            }

            index = end_escape;
            continue;
        }

        let ch = content[index..].chars().next().unwrap();
        index += ch.len_utf8();

        if ch != '\r' {
            visible += 1;
        }
    }

    let mut result = String::new();
    if !active.is_empty() {
        result.push_str(&active);
    }

    while index < content.len() && visible < end {
        if let Some(end_escape) = consume_ansi_escape(content, index) {
            result.push_str(&content[index..end_escape]);
            index = end_escape;
            continue;
        }

        let ch = content[index..].chars().next().unwrap();
        index += ch.len_utf8();

        if ch == '\r' {
            continue;
        }

        result.push(ch);
        visible += 1;
    }

    if !result.is_empty() && !result.ends_with(ANSI_RESET) && !strip_ansi(&result).is_empty() {
        result.push_str(ANSI_RESET);
    }

    result
}
