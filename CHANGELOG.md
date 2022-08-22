# Changelog

## [Unreleased](https://github.com/graelo/tmux-backup/compare/v0.0.0...HEAD) (2022-08-21)

### Features

* **restore:** report the actual metadata after restoration
  ([40ad1df](https://github.com/graelo/tmux-backup/commit/40ad1df6715934d1727d93ee076f09598f343dc8))
* **restore:** restore panes content 🎉
  ([f1675e4](https://github.com/graelo/tmux-backup/commit/f1675e49b494ad9a9492ca6760aaeb30030cacf9))
* **catalog:** add filesize to --details columns
  ([a9285e4](https://github.com/graelo/tmux-backup/commit/a9285e415d016e658c4deef1d3ada767ecd29181))
* **tmux:** add show_options()
  ([da43ae2](https://github.com/graelo/tmux-backup/commit/da43ae214fd409dfcbf02b44bec83ca9d119cefd))
* **restore:** unpack archive during restore
  ([d919551](https://github.com/graelo/tmux-backup/commit/d9195510d2b06fc8503bd2742bca642f27036b80))
* **save:** capture ansi escape codes
  ([3dbac0e](https://github.com/graelo/tmux-backup/commit/3dbac0eb8c3a9d1d64a07fea346ad2a67e493bbf))
* **restore:** WIP correct state restoration: sessions, windows, panes
  ([ad3b860](https://github.com/graelo/tmux-backup/commit/ad3b86007179e2552e3841aa87f71c29e74eb2a4))
* **restore:** WIP assemble all pairs
  ([5f0df8a](https://github.com/graelo/tmux-backup/commit/5f0df8af590074679d8e0004d561d4420ddbe77e))
* **restore:** WIP restore windows, panes & layout
  ([01b7755](https://github.com/graelo/tmux-backup/commit/01b7755e56b42085dde85eb7523811333e149b7f))
* **restore:** WIP prepare for creating panes
  ([117279b](https://github.com/graelo/tmux-backup/commit/117279bd9ef54154891f50f07a47309a64f0619a))
* **window_layout:** integrate into Window
  ([aa9661b](https://github.com/graelo/tmux-backup/commit/aa9661bdf78211ca688833dc188d54a8fd8d42b1))
* **window_layout:** extract pane_ids
  ([e2e684e](https://github.com/graelo/tmux-backup/commit/e2e684e0765d678aa0b90ad3136ff10add3986f2))
* **window_layout:** use nom to parse the window layout
  ([ea2e6bd](https://github.com/graelo/tmux-backup/commit/ea2e6bd92c14861cbcf4b1e7164f42efbd6d84df))
* **restore:** can restore sessions & windows
  ([b282c63](https://github.com/graelo/tmux-backup/commit/b282c6309dd75a9f3639ad3997d43a33eb3853bb))
* **restore:** can restore sessions
  ([eb30813](https://github.com/graelo/tmux-backup/commit/eb3081338dd97956736149acac122d703f2774a0))
* **actions:** stub support for restore
  ([1674548](https://github.com/graelo/tmux-backup/commit/1674548988bdcfd09e76b42ee8ecdec8fde3b200))
* **tmux:** capture session_path
  ([d5e9ec4](https://github.com/graelo/tmux-backup/commit/d5e9ec4105f76414c83647e1f290ee96ceafcf1a))
* **catalog:** better list cli API
  ([47dc489](https://github.com/graelo/tmux-backup/commit/47dc489786e71a3d03494aebcc7c9f47a1590844))
* **compaction:** implement the classic strategy
  ([b79e274](https://github.com/graelo/tmux-backup/commit/b79e2743e92d66a4e65213f26e2794541dc5f76f))
* **backup:** add impl Display for BackupStatus
  ([e9baef7](https://github.com/graelo/tmux-backup/commit/e9baef730974ed8497bd77145f3e470483f2c4df))
* **archive:** save the version inside `Metadata` & check on read
  ([f5e2fe9](https://github.com/graelo/tmux-backup/commit/f5e2fe979faaca7eb84e8b29a9e19f57607b30ac))
* **catalog:** support `catalog list --details`
  ([1b8724a](https://github.com/graelo/tmux-backup/commit/1b8724a1607365241ff3a40311b49eb459795149))
* **archive:** introduce format version
  ([8894127](https://github.com/graelo/tmux-backup/commit/88941279e65cb87463e12f6cee120fca1b51d3a8))
* **save:** add panes to archive
  ([70d0e9c](https://github.com/graelo/tmux-backup/commit/70d0e9ce7b0ef4aa19e66c04d19d330d0b9ac0fb))
* **catalog:** further support for describe
  ([1926ca2](https://github.com/graelo/tmux-backup/commit/1926ca28953ba915d6aa05e1dbd30e09783822cb))
* **catalog:** initial support for describe
  ([444b0f5](https://github.com/graelo/tmux-backup/commit/444b0f587ee06b253ba1ad5e6a3419ef2f3cdafd))
* **cli:** add dirpath completion with value_hint
  ([d754072](https://github.com/graelo/tmux-backup/commit/d754072aabb3ae2b86554a25e492268e0a89eaaa))
* **cli:** save add `--compact` option
  ([7690c99](https://github.com/graelo/tmux-backup/commit/7690c999152811dfaa2158e81a36d897f107aceb))
* **cli:** add shell completion
  ([eb424e4](https://github.com/graelo/tmux-backup/commit/eb424e4b783a210f919ad36ccace00e8fd333dd9))
* **cli:** support defining the strategy with env vars
  ([9ede704](https://github.com/graelo/tmux-backup/commit/9ede704a1ba62e71692b96c5963b10cec53e5f74))
* **catalog:** compute backup age
  ([62a3acc](https://github.com/graelo/tmux-backup/commit/62a3acc26cbf1427c63d4332ba067ce8fb3b3c39))
* **catalog:** add catalog compact
  ([cb54d00](https://github.com/graelo/tmux-backup/commit/cb54d00ba7e7e25d18db4f32d53e12eba90e4d59))
* **catalog:** list counts backups
  ([0c761a4](https://github.com/graelo/tmux-backup/commit/0c761a43140a634563fe0cfdf744039b8364766c))
* **compaction:** better planning
  ([46c1c01](https://github.com/graelo/tmux-backup/commit/46c1c01af6330362c3a3c84b1d5978a677c5b1e4))
* **catalog:** introduce the archives catalog
  ([70d602e](https://github.com/graelo/tmux-backup/commit/70d602e2e34c15e017c0bb5d5336a6bce08abe13))
* **save:** print report on save
  ([892cc41](https://github.com/graelo/tmux-backup/commit/892cc416580a8bafa7d61f51dd4305f491846ff8))
* **archive:** save archive
  ([0cad1f3](https://github.com/graelo/tmux-backup/commit/0cad1f354f5625d626ab5b16a7c5623a4cf5cce3))
* **save:** compress panes-content
  ([bdc2085](https://github.com/graelo/tmux-backup/commit/bdc208590a38ec5f25a32ca4e48ca74fc3c65e0e))
* **tmux:** concurrent async comm with tmux
  ([4381b70](https://github.com/graelo/tmux-backup/commit/4381b703acc0265b55e1ac2f4ba3ff85805d67b3))
* **tmux:** capture panes concurrently
  ([cfd2e35](https://github.com/graelo/tmux-backup/commit/cfd2e35266b12f8f2d3d5f97f74f509306c4f858))
* **tmux:** capture panes, windows & panes
  ([89ccdb6](https://github.com/graelo/tmux-backup/commit/89ccdb63f451e784f2659d1786d26471476c4e34))

### Fixes

* **license:** consistent files & statement
  ([e5ba5d7](https://github.com/graelo/tmux-backup/commit/e5ba5d7b3e30aca754bc25bceef93307c49a1766))
* **tmux:** default command if bash
  ([cad5a81](https://github.com/graelo/tmux-backup/commit/cad5a81bb2851952b19bfc545714bde314d3414c))
* **archive:** use tempdir for save & restore
  ([809214a](https://github.com/graelo/tmux-backup/commit/809214a198032d50d473d1f69b5348b677232b23))
* **restore:** restore_session similar code for 1st window and the rest
  ([1121f80](https://github.com/graelo/tmux-backup/commit/1121f8084c2ce999c5a5547b6cf9c8ecdeb0c093))
* add missing docs
  ([0bf3738](https://github.com/graelo/tmux-backup/commit/0bf37381d353ffa78ac10abde296a477ac232155))
* **restore:** WIP add time measurements
  ([89cc537](https://github.com/graelo/tmux-backup/commit/89cc53737cb5688e42faaed0da5e40da04528c05))
* **restore:** WIP report the new session id
  ([1f21090](https://github.com/graelo/tmux-backup/commit/1f2109096a6da278208fd4ee627d529822888933))
* **window_layout:** simplify by using hex_digit1
  ([56d2324](https://github.com/graelo/tmux-backup/commit/56d232416a64cb7d53c95e19cbfe8fe378c22fb8))
* **compaction:** support having no backups yet
  ([0ac1b2a](https://github.com/graelo/tmux-backup/commit/0ac1b2ac92b1eeb176a5df7f46db0d11678b090f))
* **cli:** generate -> generate-completion
  ([d5bc338](https://github.com/graelo/tmux-backup/commit/d5bc33824ba09159c87e3ccb95e152f5ae5a9bd8))
* **catalog:** minor refinements
  ([eca142d](https://github.com/graelo/tmux-backup/commit/eca142d7e262bcb110e76e8efdc5c956cba1eabb))
* **catalog:** extract full_list
  ([bf4d9e6](https://github.com/graelo/tmux-backup/commit/bf4d9e6d391a36ffa7572b99ff7adc5b0c080bcf))
* **catalog:** read metadata concurrently
  ([fb5be4b](https://github.com/graelo/tmux-backup/commit/fb5be4b16d9512eed00ab7fce0e49f6225ec5c05))
* **strategy:** set classic as unimplemented!()
  ([832f51e](https://github.com/graelo/tmux-backup/commit/832f51e1518c4c88bbdb43b74d9074d38c0b9d10))
* **save:** refresh the catalog before compacting
  ([f85458c](https://github.com/graelo/tmux-backup/commit/f85458c8ad0ff7d7f79081a1c0a3a80b703c9f53))
* **catalog:** better list layout
  ([4ff9855](https://github.com/graelo/tmux-backup/commit/4ff9855e5a968f7c1a5906934bdd96460e8b8a3e))
* **async:** switch to async-std -> 2x speedup!
  ([708f3e2](https://github.com/graelo/tmux-backup/commit/708f3e269488a1426ca2b31fa7fcd0950d27cd92))

## v0.0.0 (2022-08-18)
