# Autosave configuration

Autosave, added in `v0.6.0`, allows external cron-like runners to save the
current tmux client sessions regularly. The principle is explained in the
README.md.

This page explains how to configure it on various systems.

## On macOS

### 1. Create your script

First, ensure your script exists and is executable. Let's assume it's at
~/.tmux/periodic_task.sh.

```bash
  #!/bin/bash
  # Your logic here
  tmux-backup autosave --ignore-last-lines=1 --to-tmux=errors
```

Make it executable:

```bash
  chmod +x ~/.tmux/periodic_task.sh
```

### 2. Create the .plist file

Create a file at ~/Library/LaunchAgents/com.user.tmux-periodic.plist.

Note: launchd requires absolute paths for everything (including the shell and
the script).

```xml
  <?xml version="1.0" encoding="UTF-8"?>
  <!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
  <plist version="1.0">
  <dict>
      <key>Label</key>
      <string>com.user.tmux-periodic</string>
      <key>ProgramArguments</key>
      <array>
          <string>/Users/YOUR_USERNAME/.tmux/periodic_task.sh</string>
      </array>
      <key>StartInterval</key>
      <integer>900</integer> <!-- 15 minutes in seconds -->
      <key>RunAtLoad</key>
      <true/>
      <key>StandardOutPath</key>
      <string>/tmp/com.user.tmux-periodic.out</string>
      <key>StandardErrorPath</key>
      <string>/tmp/com.user.tmux-periodic.err</string>
  </dict>
  </plist>
```

### 3. Register it with Launchd

| Action        | Command                                                              |
| ------------- | -------------------------------------------------------------------- |
| Start/Load    | launchctl load ~/Library/LaunchAgents/com.user.tmux-periodic.plist   |
| Stop/Unload   | launchctl unload ~/Library/LaunchAgents/com.user.tmux-periodic.plist |
| Force Run Now | launchctl start com.user.tmux-periodic                               |
| Check Logs    | tail -f /tmp/com.user.tmux-periodic.err                              |

## On Linux

TODO
