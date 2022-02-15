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

    writeln!(file, "![demo image](local-image-01.png)").unwrap();
    writeln!(file, "![demo image](.local-image-02.png)").unwrap();
    writeln!(file, "![demo image](/local-image-03.png)").unwrap();
    writeln!(file, "![demo image](./local-image-04.png)").unwrap();
    writeln!(file, "![demo image](../local-image-05.png)").unwrap();
    writeln!(file, "![demo image](http://remote-image-01.png)").unwrap();
    writeln!(file, "![demo image](https://remote-image-02.png)").unwrap();

    let renderer = Renderer::new(file.path().to_path_buf());
    let content = renderer.run().unwrap();

    assert!(content.html.contains(&format!("src=\"file://{}/local-image-01.png\"", tempdir.display())));
    assert!(content.html.contains(&format!("src=\"file://{}/.local-image-02.png\"", tempdir.display())));
    assert!(content.html.contains(&format!("src=\"file://{}/local-image-03.png\"", tempdir.display())));
    assert!(content.html.contains(&format!("src=\"file://{}/local-image-04.png\"", tempdir.display())));
    assert!(content.html.contains(&format!("src=\"file://{}/../local-image-05.png\"", tempdir.display())));
    assert!(content.html.contains("src=\"http://remote-image-01.png\""));
    assert!(content.html.contains("src=\"https://remote-image-02.png\""));
}
