# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

- Added: Image sizes are cached via javascript and set on page reload. That way, reloading the
  browser due to text changes should not lead to any flicker.
- Added: Reading markdown from STDIN is now possible by providing "-" as a filename.
- Changed: Custom header bar was removed. One should be added automatically by the window
  manager/desktop environment.

## [0.1.2] - 2020-03-23

- Fixed: file-watching bug

## [0.1.1] - 2020-03-22

- Initial release
