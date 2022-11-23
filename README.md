# Impeccable History

<!--
Make your history shameless by dismissing all commands that failed.

I believe, avid shell users do understand how important is to build the history
with relevant commands for self-referencing and accurate completions.
-->

`history-scraper` is an executable file of a small program written in Rust.
Built on my system, it might not work on yours. If this is the case, you will
have to compile it from source, first installing Rust and Cargo. Later, there
will be added an easier way of compilation with Docker.

You can find more about `hist-scapper`'s CLI by calling it with `-h` option. In
case you want to amend or adjust the plugin to your needs, please, see the
source code ([hist-scraper.rs](src/hist-scraper.rs),
[hist-scraper.plugin.zsh](hist-scraper.plugin.zsh)), then contribute.

The plugin works by remembering failed commands in a
`/tmp/hist-scraper-nzcmds.txt` during a Zsh session (a hook in
`precmd_functions`) and removing them on exit from `$HISTFILE` (a hook in
`zshexit_functions`). You can find info about errors occurred during the plugin
operation at `/tmp/hist-scraper-error.log`.

_**Note:** you can not remove with_ `impeccable-history` _commands in_
`$HISTFILE` _that failed before the installation._


## Installation

For instance, with [**zcomet**](https://github.com/agkozak/zcomet):
```bash
zcomet load lukoshkin/impeccable-history

## The command below pulls from the 'develop' branch.
# zcomet load lukoshkin/impeccable-history@develop
```
I guess, most of the popular plugin managers will work as well.  
Just add the corresponding entry to your `.zshrc`.

## Known issues

Zsh history file is saved in metafied format. Currently, hist-scraper reads it
directly, without transforming it back to a standard encoding. That is, the
plugin will not work for failed commands containing non-ASCII characters. This
will be fixed in the next patch.

**Possible solutions:**

- unmetafying some lines of $HISTFILE after reading it right in the Rust source
  code (see, e.g., [this](https://www.zsh.org/mla/users/2011/msg00154.html)).
- feeding the last few commands of $HISTFILE converted with Zsh's `history` to
  <br> `hist-scraper` and obtaining the indices of the lines to delete. Then
  doing something like
  ```
  awk 'FNR==NR{a[$0];next} !(FNR in a)' indices.txt $HISTFILE
  ```
