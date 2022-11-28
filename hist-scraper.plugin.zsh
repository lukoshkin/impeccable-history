## Standardized `$0` Handling.
## See Zsh Plugin Standard by Sebastian Gniazdowski.
## Currently, it is the only point where we follow the standard.
0="${ZERO:-${${0:#$ZSH_ARGZERO}:-${(%):-%N}}}"
0="${${(M)0:#/*}:-$PWD/$0}"
HIST_SCRAPER_DIR=${0:h}


## If first char is whitespace -> do not add the cmd to the history.
## Remove repeating whitespace characters.
## Do not store duplicates and history cmds (`fc ..`).
setopt histignorespace
setopt histreduceblanks
setopt histignorealldups
setopt histnostore

## Zsh hook on appending lines to the history file.
## Note: # a command is added to the history before being executed.
_zsh_add_history () {
  emulate -L zsh
  ## Squeezing repeating whitespace everywhere may change drastically what is
  ## behind the original command. However, it is OK to do so since we only
  ## compare LHS with RHS and do not modify the command in-place.
  ! [[ ${$(tr -s ' ' <<< "$1")% } =~ ${HIST_SCRAPER_IGNORE:='^.{1,3}$'} ]];
  ## NOTE: Trimming and squeezing with `xargs` may error in some cases.
}
## Two types of ignore:
## instant - right before appending to history array (via HIST_SCRAPER_IGNORE);
## late - on logout from a shell (via Zsh's HISTORY_IGNORE var).


## This plugin creates two files at the /tmp dir:
## - one for logging errors occurring during its operation;
## - the other contains records of the form 'exit_code failed_cmd'.
## The latter is used as input to an executable written in Rust.
HIST_SCRAPER_LOG=/tmp/hist-scraper-nzcmds.txt
skipn=$(head -n 1 $HIST_SCRAPER_LOG 2> /dev/null)  # tmp var
[[ $skipn =~ ^[0-9]+$ ]] || skipn=0

HIST_SCRAPER_SKIP_ROWS=$(( skipn > 0 ? skipn : 0 ))
unset skipn

## One can modify to add code-cmd pairs to an array instead of dumping to
## a file. In the former case, hist-scraper should be re-implemented for
## extraction of the array content (possibly, with the use of `std::env`).
_add_broken_cmd () {
  local code=$?

  if [[ $code != 0 ]]; then
    # local line="$code $(fc -ln -1 | tr -s ' ')"
    ## Unnecessary to squeeze, as this is handled by the options.
    local line="$code $(fc -ln -1)"

    ## Don't make duplicates.
    if ! grep -q "$line" $HIST_SCRAPER_LOG; then
      echo $line >> $HIST_SCRAPER_LOG
    fi
  fi

  return $?
}

## At the time this hook is called,
## all session commands are already in the $HISTFILE.
_scrape_history () {
  ## TODO: Allow to specify where a user keeps their history:
  #  local target=${HIST_SCRAPER_TARGET:-$HISTFILE}
  ## Use target instead of HISTFILE.
  "$HIST_SCRAPER_DIR/bin/hist-scraper" \
    -t "$HISTFILE" -q $HIST_SCRAPER_LOG -c ' ' \
    -n $HIST_SCRAPER_SKIP_ROWS --in-place \
    2> /tmp/hist-scraper-error.log
    ## Unfortunately, it unmetafies $HISTFILE
    ## (will be fixed in the future).

  wc -l < "$HISTFILE" > $HIST_SCRAPER_LOG
}


# Register hooks.
precmd_functions+=( _add_broken_cmd )
zshexit_functions+=( _scrape_history )
zshaddhistory_functions+=( _zsh_add_history )
