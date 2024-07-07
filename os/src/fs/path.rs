/*
This code is a derivative work based on .NET Standard Library source code
(c) Microsoft Corporation, licensed under the MIT license.

All original attributions and licenses apply to this work.

Adapter: Caiyi Shyu<cai1hsu@outlook.com>
*/

use alloc::{
    fmt::format,
    string::{String, ToString},
};

pub const ROOT_STR: &str = "/";
pub const SEPARATOR_STR: &str = "/";
pub const DOT_STR: &str = ".";

pub const SEPARATOR: char = '/';
pub const DOT: char = '.';

// Combine two paths
pub fn combine(path1: &str, path2: &str) -> Option<String> {
    combine_internal(path1, path2)
}

pub fn is_separator(c: char) -> bool {
    c == SEPARATOR
}

pub fn is_path_rooted(path: &str) -> bool {
    starts_with_separator(path)
}

pub fn ends_in_separator(path: &str) -> bool {
    !path.is_empty() && path.chars().last().unwrap() == SEPARATOR
}

pub fn starts_with_separator(path: &str) -> bool {
    !path.is_empty() && path.chars().next().unwrap() == SEPARATOR
}

pub fn trim_end_separator(path: &str) -> &str {
    match ends_in_separator(path) && !is_root(path) {
        true => &path[..path.len() - 1],
        false => path,
    }
}

pub fn is_root(path: &str) -> bool {
    path.len() == get_root_length(path)
}

pub fn is_path_fully_qualified(path: &str) -> bool {
    !is_partially_qualified(path)
}

pub fn has_extension(path: &str) -> bool {
    path.chars()
        .rev()
        .take_while(|c| !is_separator(*c))
        .any(|c| c == DOT)
}

pub fn get_extension(path: &str) -> Option<&str> {
    let mut iter = path.chars().rev().take_while(|c| !is_separator(*c));
    let mut dot = iter.position(|c| c == DOT);

    match dot {
        Some(dot) => Some(&path[path.len() - dot..]),
        None => None,
    }
}

pub fn get_filename(path: &str) -> &str {
    match index_of_filename(path) {
        ..=0 => path,
        pos => &path[pos..],
    }
}

pub fn get_filename_without_extension(path: &str) -> &str {
    let filename = get_filename(path);

    match filename.rfind(DOT) {
        Some(dot) => &filename[..dot],
        None => filename,
    }
}

pub fn change_extension(path: &str, extension: &str) -> Option<String> {
    if path.is_empty() {
        return None;
    }

    let filename = get_filename_without_extension(path);

    match extension.is_empty() {
        true => Some(filename.to_string()),
        false => Some(format!("{}{}{}", filename, DOT, extension)),
    }
}

pub fn get_path_root(path: &str) -> Option<&str> {
    match path.is_empty() || !is_path_rooted(path) {
        true => None,
        false => Some(ROOT_STR),
    }
}

pub fn get_directory_name(path: &str) -> Option<String> {
    match (path.is_empty()) {
        true => None,
        false => {
            let end = get_directory_name_offset(path) as usize;

            normalize_path(&path[..end])
        }
    }
}

pub fn get_relative_path(relative_to: &str, path: &str) -> Option<String> {
    unimplemented!()
}

pub fn get_full_path(path: &str, cwd: &str) -> Option<String> {
    match path.is_empty() || cwd.is_empty() {
        true => None,
        false => get_full_path_internal(path, cwd),
    }
}

pub fn remove_relative_segments(path: &str) -> String {
    match remove_relative_segments_internal(path) {
        Some(p) => p,
        None => path.to_string(),
    }
}

fn get_full_path_internal(path: &str, cwd: &str) -> Option<String> {
    match is_path_rooted(path) {
        true => {
            let collapsed = remove_relative_segments(path);

            match collapsed.len() {
                0 => Some(ROOT_STR.to_string()),
                _ => Some(collapsed),
            }
        }
        false => combine(cwd, path),
    }
}

fn remove_relative_segments_internal(path: &str) -> Option<String> {
    let root_length = get_root_length(path);
    let mut sb = String::with_capacity(path.len());

    let mut skip = root_length;
    let path_len = path.len();

    // We treat "\.." , "\." and "\\" as a relative segment. We want to collapse the first separator past the root presuming
    // the root actually ends in a separator. Otherwise the first segment for RemoveRelativeSegments
    // in cases like "\\?\C:\.\" and "\\?\C:\..\", the first segment after the root will be ".\" and "..\" which is not considered as a relative segment and hence not be removed.
    // Since the root on unix is only one character, we only have to check the first character.
    if root_length == 1 && is_separator(path.chars().next().unwrap()) {
        skip -= 1;
    }

    // Remove "//", "/./", and "/../" from the path by copying each character to the output,
    // except the ones we're removing, such that the builder contains the normalized path
    // at the end.
    if skip > 0 {
        sb.push_str(&path[0..skip]);
    }

    let mut enu = path.chars().skip(skip).enumerate();

    while let Some((i, c)) = enu.next() {
        if is_separator(c) && i + 1 < path_len {
            let mut cloned_it = enu.clone();
            let next = cloned_it.next().unwrap().1;

            // Skip this character if it's a directory separator and if the next character is, too,
            // e.g. "parent//child" => "parent/child"
            // path[i + 1] == '/'
            if is_separator(next) {
                continue;
            }

            let next_next = cloned_it.next().unwrap().1;

            // Skip this character and the next if it's referring to the current directory,
            // e.g. "parent/./child" => "parent/child"
            if (i + 2 == path_len || is_separator(next_next)) && next == DOT {
                // skip the next dot, and `continue` skips the slash
                enu.next();
                continue;
            }

            let next3 = cloned_it.next().unwrap().1;

            // Skip this character and the next two if it's referring to the parent directory,
            // e.g. "parent/child/../grandchild" => "parent/grandchild"
            if i + 2 < path_len
                && (i + 3 == path_len || is_separator(next3))
                && next == DOT
                && next_next == DOT
            {
                let mut si = sb.len();
                for c in sb.chars().rev() {
                    si -= 1;

                    if si <= skip {
                        break;
                    }

                    if is_separator(c) {
                        let new_len = match i + 3 >= path_len && si == skip {
                            true => si + 1,
                            false => si,
                        };

                        sb.truncate(new_len);
                        break;
                    }
                }

                if (si < skip) {
                    sb.truncate(skip);
                }

                // skip the next 2 dots, and `continue` skips the slash
                enu.next();
                enu.next();
                continue;
            }
        }

        sb.push(c);
    }

    let sb_len = sb.len();

    // If we haven't changed the source path, return the original
    if sb_len == path_len {
        return None;
    }

    // We may have eaten the trailing separator from the root when we started and not replaced it
    if skip != root_length && sb_len < root_length && root_length == 1 {
        sb.push(SEPARATOR);
    }

    Some(sb)
}

// Remove alternate directory separator('//' or '\\')
pub fn normalize_path(path: &str) -> Option<String> {
    if path.is_empty() {
        return None;
    }

    let mut normalized = false;

    let mut it = path.chars();
    while let Some(c) = it.next() {
        // path[i] == '/' && path[i + 1] == '/'
        if is_separator(c) {
            if let Some(next) = it.next() {
                if is_separator(next) {
                    normalized = false;
                    break;
                }
            }
        }
    }

    if normalized {
        return Some(path.to_string());
    }

    let mut result = String::with_capacity(path.len());

    let mut it = path.chars();
    while let Some(c) = it.next() {
        if is_separator(c) {
            if let Some(next) = it.next() {
                if is_separator(next) {
                    continue;
                }
            }
        }

        result.push(c);
    }

    Some(result)
}

fn get_directory_name_offset(path: &str) -> isize {
    let len = path.len();
    let root_len = get_root_length(path);

    match len <= root_len {
        true => -1,
        false => {
            let mut it = path.chars().rev();
            let mut end = len;

            for c in it.by_ref() {
                if end > root_len && !is_separator(c) {
                    end -= 1;
                }
            }

            // Handle alternate directory separator('//' or '\\')
            it.next();
            for c in it.by_ref() {
                if end > root_len && is_separator(c) {
                    end -= 1;
                }
            }

            end as isize
        }
    }
}

fn index_of_filename(path: &str) -> usize {
    match path.is_empty() {
        true => 0,
        false => match path.chars().rev().position(is_separator) {
            Some(pos) => path.len() - pos,
            None => 0,
        },
    }
}

fn is_partially_qualified(path: &str) -> bool {
    !is_path_rooted(path)
}

fn get_root_length(path: &str) -> usize {
    match starts_with_separator(path) {
        true => 1,
        false => 0,
    }
}

fn combine_internal(first: &str, second: &str) -> Option<String> {
    if (first.is_empty()) {
        return Some(second.to_string());
    }

    if (second.is_empty()) {
        return Some(first.to_string());
    }

    match is_path_rooted(second) {
        true => Some(second.to_string()),
        false => join_internal(first, second),
    }
}

fn join_internal(first: &str, second: &str) -> Option<String> {
    assert!(!first.is_empty());
    assert!(!second.is_empty());

    let has_separator =
        is_separator(first.chars().last().unwrap()) || is_separator(second.chars().next().unwrap());

    match has_separator {
        true => Some(format!("{}{}", first, second)),
        false => Some(format!("{}{}{}", first, SEPARATOR, second)),
    }
}
