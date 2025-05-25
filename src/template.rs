use crate::entry;

const ESTIMATED_TEMPLATE_OVERHEAD: usize = 256;

fn extend_entries(result: &mut Vec<u8>, mut entries: Vec<entry::Entry>) {
    entries.sort_by(|a, b| b.date.cmp(&a.date));

    result.extend_from_slice(b"<ul>");
    for entry in entries {
        result.extend_from_slice(b"<li><a href=\"");
        result.extend_from_slice(entry.permalink.as_bytes());
        result.extend_from_slice(b"\">");
        result.extend_from_slice(entry.title.as_bytes());
        result.extend_from_slice(b"</a>");
        if let Some(category) = entry.category {
            result.extend_from_slice(b"<span class=\"dim\"> [mod ");
            result.extend_from_slice(category.as_bytes());
            result.extend_from_slice(b"; ");
            result.extend_from_slice(
                entry
                    .tags
                    .into_iter()
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

pub fn apply(template: &[u8], entry: entry::Entry) -> Vec<u8> {
    let first_path_component = entry
        .path
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
        template.len() + ESTIMATED_TEMPLATE_OVERHEAD + entry.contents.len(),
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
            } else if slot_name == b"BLOGPOSTINTRO" {
                if first_path_component == "blog" || first_path_component == "golb" {
                    result.extend_from_slice(b"<h1 class=\"title\">");
                    result.extend_from_slice(entry.title.as_bytes());
                    result.extend_from_slice(b"</h1>");
                    if !entry.date.is_empty() {
                        result.extend_from_slice(b"<div class=\"time\"><p>");
                        result.extend_from_slice(entry.date.as_bytes());
                        if let Some(updated) = entry.updated.as_ref() {
                            if *updated != entry.date {
                                result.extend_from_slice(b"<p>last updated ");
                                result.extend_from_slice(updated.as_bytes());
                                result.extend_from_slice(b"</p>");
                            }
                        }
                        result.extend_from_slice(b"</div>");
                    }
                }
            } else if slot_name == b"CONTENT" {
                result.extend_from_slice(&entry.contents);
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
                        entry::from_markdown_in_path(
                            entry.path.parent().expect("path to have parent"),
                        ),
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
