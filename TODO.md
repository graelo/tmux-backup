# TODO

## Global

- use `thiserror` in the library
- go over functions such as windows_related_to which return copies, and make
  them return references instead
- check clap config file support

## Related to save

- add list of captured environment variables (in conf file?)

## Related to restore

- add `restore --attach` to automatically attach if running from the terminal
- add `restore --override` to replace each existing session by its version from
  the archive
- add `restore --skip-last-lines n` to not restore the last n lines of each
  buffer
- in `restore()` gather the true metadata for displaying the overview, instead
  of the metadata from the archive
