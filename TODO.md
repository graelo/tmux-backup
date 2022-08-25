# TODO

## Global

- [x] use `thiserror` in the library
- [ ] cleanup error types, for instance `UnexpectedOutput`
- [ ] go over functions such as windows_related_to which return copies, and make
  them return references instead
- [x] check clap config file support
- [ ] use the strategy option only in save and catalog commands

## Related to save

- [ ] add list of captured environment variables (in conf file?)

## Related to restore

- [x] if in $TMUX, replace the existing session named `0` and switch to client
  else display a message `tmux attach -t last-session-name`
- [ ] add `restore --attach` to automatically attach if running from the terminal
- [ ] add `restore --override` to replace each existing session by its version from
  the archive
- [ ] add `restore --skip-last-lines n` to not restore the last n lines of each
  buffer
- [x] in `restore()` gather the true metadata for displaying the overview, instead
  of the metadata from the archive
