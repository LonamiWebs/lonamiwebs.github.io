use crate::{Entry, conf};

const ESTIMATED_TEMPLATE_OVERHEAD: usize = 256;

fn extend_entries<'a>(result: &mut Vec<u8>, entries: impl Iterator<Item = &'a Entry>) {
    let mut entries = entries.collect::<Vec<_>>();
    entries.sort_by(|a, b| b.date.cmp(&a.date));

    result.extend_from_slice(b"<ul>");
    for entry in entries {
        result.extend_from_slice(b"<li><a href=\"");
        result.extend_from_slice(entry.permalink.as_bytes());
        result.extend_from_slice(b"\">");
        result.extend_from_slice(entry.title.as_bytes());
        result.extend_from_slice(b"</a>");
        if let Some(category) = entry.category.as_ref() {
            result.extend_from_slice(b"<span class=\"dim\"> [mod ");
            result.extend_from_slice(category.as_bytes());
            result.extend_from_slice(b"; ");
            result.extend_from_slice(
                entry
                    .tags
                    .iter()
                    .map(|tag| format!("'{tag}"))
                    .collect::<Vec<_>>()
                    .join(", ")
                    .as_bytes(),
            );
            result.extend_from_slice(b"]</span>");
        }
        result.extend_from_slice(b"</li>");
    }
    result.extend_from_slice(b"</ul>");
}

pub fn apply(template: &[u8], entries: &[Entry], entry: &Entry) -> Vec<u8> {
    if entry.path.extension().is_none_or(|ext| ext != "md") {
        return entry.processed_contents.clone();
    }

    let first_path_component = entry
        .path
        .strip_prefix(conf::INPUT_FOLDER)
        .unwrap_or(&entry.path)
        .components()
        .next()
        .expect("path to have at least one component")
        .as_os_str()
        .to_string_lossy();

    let path_stem = entry
        .path
        .file_stem()
        .expect("path to have file stem")
        .to_string_lossy();

    let mut result = Vec::<u8>::with_capacity(
        template.len() + ESTIMATED_TEMPLATE_OVERHEAD + entry.processed_contents.len(),
    );

    let mut iter = template.iter().copied().enumerate();
    while let Some((i, c)) = iter.next() {
        if c == b'$' {
            let length = &template[i + 1..]
                .iter()
                .take_while(|c| c.is_ascii_uppercase())
                .count();
            let slot_name = &template[i + 1..i + 1 + length];

            if slot_name == b"TITLE" {
                if first_path_component == "index.md" {
                    result.extend_from_slice(b"Lonami's Site");
                } else if first_path_component == "blog" && path_stem == "_index" {
                    result.extend_from_slice(b"Lonami's Blog");
                } else if first_path_component == "golb" && path_stem == "_index" {
                    result.extend_from_slice(b"Lonami's Golb");
                } else if first_path_component == "blog" || first_path_component == "golb" {
                    result.extend_from_slice(entry.title.as_bytes());
                    result.extend_from_slice(b" | Lonami's Blog");
                } else {
                    panic!(
                        "unknown path to use for replacing template title: {}",
                        String::from_utf8_lossy(slot_name)
                    );
                }
            } else if slot_name == b"CSS" {
                result.extend_from_slice(&entry.append_css_style);
            } else if slot_name == b"BLOGPOSTINTRO" {
                if first_path_component == "blog" || first_path_component == "golb" {
                    result.extend_from_slice(b"<h1 class=\"title\">");
                    result.extend_from_slice(entry.title.as_bytes());
                    result.extend_from_slice(b"</h1>");
                    if !entry.date.is_empty() {
                        result.extend_from_slice(b"<div class=\"time\"><p>");
                        result.extend_from_slice(entry.date.as_bytes());
                        result.extend_from_slice(b"</p>");
                        if let Some(updated) = entry.updated.as_ref()
                            && *updated != entry.date
                        {
                            result.extend_from_slice(b"<p>last updated ");
                            result.extend_from_slice(updated.as_bytes());
                            result.extend_from_slice(b"</p>");
                        }
                        result.extend_from_slice(b"</div>");
                    }
                }
            } else if slot_name == b"CONTENT" {
                result.extend_from_slice(&entry.processed_contents);
            } else if slot_name == b"ROOT" {
                if first_path_component == "index.md" {
                    result.extend_from_slice(b"class=selected");
                }
            } else if slot_name == b"BLOG" {
                if first_path_component == "blog.md" || first_path_component == "blog" {
                    result.extend_from_slice(b"class=selected");
                }
            } else if slot_name == b"GOLB" {
                if first_path_component == "golb.md" || first_path_component == "golb" {
                    result.extend_from_slice(b"class=selected");
                }
            } else if slot_name == b"BLOGLIST" {
                if path_stem == "_index" {
                    extend_entries(
                        &mut result,
                        entries.iter().filter(|e| {
                            e.path.extension() == entry.path.extension()
                                && e.path_parent() == entry.path_parent()
                                && e.path != entry.path
                        }),
                    );
                }
            } else {
                panic!(
                    "unknown template variable: {}",
                    String::from_utf8_lossy(slot_name)
                );
            }
            iter.nth(length - 1);
        } else {
            result.push(c)
        }
    }

    result
}
