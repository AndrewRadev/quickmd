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
