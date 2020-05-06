# Changelog

All notable changes to this project will be documented in this file.

## [0.3.0] - 2020-05-06

- Added: Code highlighting
- Added: GFM features like tables and strikethrough (basically, whatever [pulldown-cmark provides](https://docs.rs/pulldown-cmark/0.7.0/pulldown_cmark/struct.Options.html))

## [0.2.0] - 2020-04-05

- Added: Image sizes are cached via javascript and set on page reload. That way, reloading the
  browser due to text changes should not lead to any flicker.
- Added: Reading markdown from STDIN is now possible by providing `-` as a filename.
- Changed: Custom header bar was removed. One should be added automatically by the window
  manager/desktop environment.

## [0.1.2] - 2020-03-23

- Fixed: file-watching bug

## [0.1.1] - 2020-03-22

- Initial release
