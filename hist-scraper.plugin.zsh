0="${ZERO:-${${0:#$ZSH_ARGZERO}:-${(%):-%N}}}"
0="${${(M)0:#/*}:-$PWD/$0}"
WORKDIR=${0:h}

## This plugin creates two files at /tmp dir:
## - one for logging errors occuring during its operation;
## - the other contains records of the form 'exit_code failed_cmd'.
## The latter is used as input to an executable written in Rust.
HIST_SCRAPER_LOG=/tmp/hist-scraper-nzcmds.txt
skipn=$(head -n 1 $HIST_SCRAPER_LOG 2> /dev/null || echo 0)  # tmp var
HIST_SCRAPER_SKIP_ROWS=$(( skipn > 0 ? skipn : 0 ))
unset skipn


## One can modify to add code-cmd pairs to an array instead of dumping to
## a file. In the former case, hist-scraper should be re-implemented for
## extraction of the array content (possibly, with the use of `std::env`).
_add_broken_cmd () {
  local code=$?

  if [[ $code != 0 ]]; then
    local line="$code $(fc -ln -1 | tr -s ' ')"

    ## Don't make duplicates
    if ! grep -q "$line" $HIST_SCRAPER_LOG; then
      echo $line >> $HIST_SCRAPER_LOG
    fi
  fi

  return $?
}


_scrape_history () {
  "$WORKDIR/hist-scraper" \
    -t "$HISTFILE" -q $HIST_SCRAPER_LOG -c ' ' \
    -n ${HIST_SCRAPER_SKIP_ROWS:-0} --in-place \
    2> /tmp/hist-scraper-error.log

  HIST_SCRAPER_SKIP_ROWS=$(cat "$HISTFILE" | wc -l)
  echo $HIST_SCRAPER_SKIP_ROWS > $HIST_SCRAPER_LOG
}


precmd_functions+=( _add_broken_cmd )
zshexit_functions+=( _scrape_history )
