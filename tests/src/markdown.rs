use pulldown_cmark::{html, Options, Parser};
use regex::Regex;

pub fn parse_markdown_to_html(markdown: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);

    let mut html_output = String::new();
    html::push_html(&mut html_output, Parser::new_ext(markdown, options));

    html_output
}

pub fn extract_wiki_links(markdown: &str) -> Vec<String> {
    let re = Regex::new(r"\[\[([^\]]+)\]\]").unwrap();
    re.captures_iter(markdown)
        .map(|c| c[1].to_string())
        .collect()
}

pub fn convert_wiki_links(markdown: &str) -> String {
    let re = Regex::new(r"\[\[([^\]]+)\]\]").unwrap();
    re.replace_all(markdown, |caps: &regex::Captures| {
        let link = &caps[1];
        let slug = link.to_lowercase().replace(' ', "-");
        format!("[{}]({})", link, slug)
    })
    .to_string()
}

pub fn render_markdown(content: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);

    let parser = Parser::new_ext(content, options);

    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    html_output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_markdown() {
        let markdown = "# Hello\n\n**Bold** and *italic*";
        let html = parse_markdown_to_html(markdown);
        assert!(html.contains("<h1"));
        assert!(html.contains("<strong>Bold</strong>"));
    }

    #[test]
    fn test_extract_links() {
        let markdown = "Check out [[Page One]] and [[Page Two]]";
        let links = extract_wiki_links(markdown);
        assert_eq!(links, vec!["Page One", "Page Two"]);
    }

    #[test]
    fn test_convert_wiki_links() {
        let markdown = "See [[My Note]] for more info";
        let converted = convert_wiki_links(markdown);
        assert!(converted.contains("[My Note](my-note)"));
    }
}
