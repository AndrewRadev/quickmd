use std::collections::HashSet;
use std::io::Write;

use tempfile::NamedTempFile;
use quickmd::markdown::Renderer;

#[test]
fn test_keeps_track_of_rendered_languages() {
    let mut file = NamedTempFile::new().unwrap();

    writeln!(file, "``` vim"     ).unwrap();
    writeln!(file, "```"         ).unwrap();
    writeln!(file, ""            ).unwrap();
    writeln!(file, "    ignored" ).unwrap();
    writeln!(file, ""            ).unwrap();
    writeln!(file, "```ruby"     ).unwrap();
    writeln!(file, "```"         ).unwrap();
    writeln!(file, ""            ).unwrap();
    writeln!(file, "```"         ).unwrap();
    writeln!(file, "```"         ).unwrap();
    writeln!(file, "```     rust").unwrap();
    writeln!(file, "```"         ).unwrap();

    let renderer = Renderer::new(file.path().to_path_buf());
    let content = renderer.run().unwrap();
    let expected: HashSet<_> =
        vec!["vim", "ruby", "rust"].into_iter().map(String::from).collect();

    assert_eq!(expected, content.code_languages);
}

#[test]
fn test_renders_local_images() {
    let mut file = NamedTempFile::new().unwrap();
    let tempdir = file.path().parent().unwrap().to_path_buf();

    writeln!(file, "![demo image](./local-image.png)").unwrap();
    writeln!(file, "![demo image](http://remote-image.png)").unwrap();
    writeln!(file, "![demo image](unprefixed_image.png)").unwrap();

    let renderer = Renderer::new(file.path().to_path_buf());
    let content = renderer.run().unwrap();

    let local_src = format!("src=\"file://{}/local-image.png\"", tempdir.display());
    assert!(content.html.contains(&local_src));
    assert!(content.html.contains("src=\"http://remote-image.png\""));

    // Without a ./ prefix, it's left alone
    assert!(content.html.contains("src=\"unprefixed_image.png\""));
}
