use pulldown_cmark::{Parser, Options, html};
use comrak::{markdown_to_html, ComrakOptions};
use html5ever::parse_document;
use html5ever::tendril::TendrilSink;
use html5ever::driver::ParseOpts;
use std::io::Cursor;

pub fn parse_markdown_to_html(markdown: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);
    
    let mut html_output = String::new();
    html::push_html(&mut html_output, Parser::new_ext(markdown, options));
    
    html_output
}

pub fn parse_markdown_to_html_comrak(markdown: &str) -> String {
    let mut options = ComrakOptions::default();
    options.render.hardbreaks = true;
    options.render.width = 80;
    
    markdown_to_html(markdown, &options)
}

pub fn sanitize_html(html: &str) -> String {
    let parse_opts = ParseOpts::default();
    let document = parse_document(Cursor::new(html), parse_opts);
    
    // Return sanitized HTML
    document.to_string()
}

pub fn extract_markdown_links(markdown: &str) -> Vec<String> {
    let mut links = Vec::new();
    
    // Simple regex for [[WikiLink]] pattern
    let re = regex::Regex::new(r"\[\[([^\]]+)\]\]").unwrap();
    for cap in re.captures_iter(markdown) {
        links.push(cap[1].to_string());
    }
    
    links
}

pub fn convert_wiki_links_to_markdown(markdown: &str) -> String {
    // Convert [[Link Name]] to [Link Name](#link-name)
    let re = regex::Regex::new(r"\[\[([^\]]+)\]\]").unwrap();
    re.replace_all(markdown, |caps: &regex::Captures| {
        let link = &caps[1];
        let slug = link.to_lowercase().replace(' ', "-");
        format!("[{}]({})", link, slug)
    }).to_string()
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
        let links = extract_markdown_links(markdown);
        assert_eq!(links, vec!["Page One", "Page Two"]);
    }

    #[test]
    fn test_convert_wiki_links() {
        let markdown = "See [[My Note]] for more info";
        let converted = convert_wiki_links_to_markdown(markdown);
        assert!(converted.contains("[My Note](#my-note)"));
    }
}