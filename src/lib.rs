use std::collections::HashMap;

use mdbook::book::Book;
use mdbook::errors::Error;
use mdbook::preprocess::{Preprocessor, PreprocessorContext};
use mdbook::BookItem;
use regex::Regex;

pub struct Multicode {
    multicode_regex: Regex,
    end_multicode: Regex,
    code_start: Regex,
    code_end: Regex,
}

pub enum ParseState {
    Nothing,
    Multicode,
    Code(String),
}

impl Multicode {
    pub fn new() -> Multicode {
        Multicode {
            multicode_regex: Regex::new(r"^```multicode$").unwrap(),
            end_multicode: Regex::new(r"^```$").unwrap(),
            code_start: Regex::new(r"^>>>>> ([a-zA-Z0-9]+)$").unwrap(),
            code_end: Regex::new(r"^<<<<<$").unwrap(),
        }
    }
}

impl Preprocessor for Multicode {
    fn name(&self) -> &str {
        "http-api"
    }

    fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book, Error> {
        // In testing we want to tell the preprocessor to blow up by setting a
        // particular config value
        if let Some(our_cfg) = ctx.config.get_preprocessor(self.name()) {
            if our_cfg.contains_key("blow-up") {
                anyhow::bail!("Blowing up!");
            }
        }

        book.for_each_mut(|book_item| {
            match book_item {
                BookItem::Separator => {}
                BookItem::PartTitle(_) => {}
                BookItem::Chapter(chapter) => {
                    let lines = chapter.content.lines();
                    let mut lang_example_no = 0usize;
                    let mut langs = Vec::new();
                    let mut lang_texts: HashMap<String, String> = HashMap::default();
                    let mut new_content = String::new();

                    new_content.push_str(include_str!("script_template.html"));
                    new_content.push('\n');

                    let mut parse_state = ParseState::Nothing;
                    for line in lines {
                        match &parse_state {
                            ParseState::Nothing => {
                                if self.multicode_regex.is_match(line) {
                                    parse_state = ParseState::Multicode;
                                } else {
                                    new_content.push_str(line);
                                    new_content.push('\n');
                                }
                            }
                            ParseState::Multicode => {
                                if self.end_multicode.is_match(line) {
                                    parse_state = ParseState::Nothing;

                                    if !langs.is_empty() {
                                        let example_class_name = format!("code-example-tab-{lang_example_no}");

                                        let lang_select_options = langs
                                            .iter()
                                            .map(|lang| format!(
                                                r#"<option value="{example_class_name}-{lang}">{lang}</option>"#
                                            ))
                                            .fold(String::new(), |mut acc, s| {
                                                acc.push_str(&s);
                                                acc
                                            });
                                        let first_lang = langs.first().unwrap();

                                        new_content.push_str(&format!(
                                            r#"<div><select onchange="changeCodeExample('{example_class_name}', event.target.value)" value="{first_lang}" class="code-example" autocomplete="off">"#
                                        ));
                                        new_content.push_str(&lang_select_options);
                                        new_content.push_str(r#"</select></div>"#);
                                        new_content.push('\n');

                                        for lang in &langs {
                                            let lang_text = lang_texts.get(lang).unwrap();
                                            new_content.push_str(&format!(
                                                r#"<div id="{example_class_name}-{lang}" class="{example_class_name}"><pre><code class="language-{lang}">"#
                                            ));
                                            new_content.push_str(&html_escape(lang_text));
                                            new_content.push_str(r#"</code></pre></div>"#);
                                        }

                                        new_content.push_str(&format!(
                                            r#"<script>(()=>{{changeCodeExample("{example_class_name}", "{example_class_name}-{first_lang}")}})()</script>"#
                                        ));
                                    }
                                    lang_example_no += 1;
                                } else if let Some(captures) = self.code_start.captures(line) {
                                    let lang_name = captures.get(1).unwrap().as_str().to_owned();
                                    langs.push(lang_name.clone());
                                    lang_texts.insert(lang_name.clone(), String::new());
                                    parse_state = ParseState::Code(lang_name);
                                }
                            }
                            ParseState::Code(language) => {
                                if self.code_end.is_match(line) {
                                    parse_state = ParseState::Multicode;
                                } else {
                                    let lang_text = lang_texts.get_mut(language).unwrap();
                                    lang_text.push_str(line);
                                    lang_text.push('\n');
                                }
                            }
                        }
                    } // End of parsing

                    chapter.content = new_content;
                }
            }
        });

        // we *are* a no-op preprocessor after all
        Ok(book)
    }

    fn supports_renderer(&self, renderer: &str) -> bool {
        renderer == "html"
    }
}

fn html_escape(text_to_escape: impl AsRef<str>) -> String {
    let mut text = text_to_escape.as_ref().to_string();
    text = text.replace('&', "&amp;");
    text = text.replace('<', "&lt;");
    text = text.replace('>', "&gt;");
    text = text.replace('"', "&quot;");
    text = text.replace('\'', "&#39;");
    text
}

// #[cfg(test)]
// mod test {
//     use super::*;

//     #[test]
//     fn preprocessor_run() {
//         let input_json = r##"[
//             {
//                 "root": "/path/to/book",
//                 "config": {
//                     "book": {
//                         "authors": ["AUTHOR"],
//                         "language": "en",
//                         "multilingual": false,
//                         "src": "src",
//                         "title": "TITLE"
//                     },
//                     "preprocessor": {
//                         "http-api": {
//                         }
//                     }
//                 },
//                 "renderer": "html",
//                 "mdbook_version": "0.4.21"
//             },
//             {
//                 "sections": [
//                     {
//                         "Chapter": {
//                             "name": "Chapter 1",
//                             "content": CONTENT_PLACEHOLDER_THINGIE_HERE,
//                             "number": [1],
//                             "sub_items": [],
//                             "path": "chapter_1.md",
//                             "source_path": "chapter_1.md",
//                             "parent_names": []
//                         }
//                     }
//                 ],
//                 "__non_exhaustive": null
//             }
//         ]"##;
//         let input_json = input_json.replace(
//             "CONTENT_PLACEHOLDER_THINGIE_HERE",
//             &format!("{:?}", include_str!("content_test_example.md")),
//         );
//         let input_json = input_json.as_bytes();

//         let (ctx, book) = mdbook::preprocess::CmdPreprocessor::parse_input(input_json).unwrap();
//         let mut expected_book = book.clone();
//         let result = Multicode::new().run(&ctx, book);
//         assert!(result.is_ok());

//         if let BookItem::Chapter(c) = expected_book.sections.first_mut().as_mut().unwrap() {
//             c.content.clear();
//             c.content.push_str(&include_str!("script_template.html"));
//             c.content.push('\n');
//             c.content.push_str(&include_str!("content_test_example.md"));
//         }

//         // <div><select onchange=\"changeCodeExample('code-example-tab-0', event.target.value)\" value=\"rust\" class=\"code-example\"><option value=\"code-example-tab-0-rust\">rust</option><option value=\"code-example-tab-0-cpp\">cpp</option></select></div>\n<div id=\"code-example-tab-0-rust\" class=\"code-example-tab-0\"><pre><code class=\"language-rust\">fn id&lt;X&gt;(x: X) -&gt; {\n    x\n}\n</code></pre></div><div id=\"code-example-tab-0-cpp\" class=\"code-example-tab-0\"><pre><code class=\"language-cpp\">X id&lt;X&gt;(X x) {\n    return x;\n}\n</code></pre></div><script>(()=>{changeCodeExample(\"code-example-tab-0\", \"code-example-tab-0-rust\")})()</script>

//         // The nop-preprocessor should not have made any changes to the book content.
//         let actual_book = result.unwrap();
//         assert_eq!(actual_book, expected_book);
//     }
// }
