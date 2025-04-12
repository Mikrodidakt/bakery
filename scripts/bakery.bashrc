# /etc/bakery/bakery.bashrc sourced in by /etc/bash.bashrc
# when running a bakery-workspace
BKRY_BIN_DIR="/usr/bin"
BKRY_VERSION=$(${BKRY_BIN_DIR}/bakery --version)
BKRY_VERSION=${BKRY_VERSION##* }
BKRY_BRANCH=$(cd ${BKRY_WORK_DIR} && git rev-parse --abbrev-ref HEAD)

# If not running interactively, don't do anything
case $- in
    *i*) ;;
      *) return;;
esac

# don't put duplicate lines or lines starting with space in the history.
# See bash(1) for more options
HISTCONTROL=ignoreboth

# append to the history file, don't overwrite it
shopt -s histappend

# for setting history length see HISTSIZE and HISTFILESIZE in bash(1)
HISTSIZE=1000
HISTFILESIZE=2000

# check the window size after each command and, if necessary,
# update the values of LINES and COLUMNS.
shopt -s checkwinsize

# If set, the pattern "**" used in a pathname expansion context will
# match all files and zero or more directories and subdirectories.
#shopt -s globstar

# make less more friendly for non-text input files, see lesspipe(1)
[ -x /usr/bin/lesspipe ] && eval "$(SHELL=/bin/sh lesspipe)"

# set variable identifying the chroot you work in (used in the prompt below)
if [ -z "${debian_chroot:-}" ] && [ -r /etc/debian_chroot ]; then
    debian_chroot=$(cat /etc/debian_chroot)
fi

# set a fancy prompt (non-color, unless we know we "want" color)
case "$TERM" in
    xterm-color|*-256color) color_prompt=yes;;
esac

# uncomment for a colored prompt, if the terminal has the capability; turned
# off by default to not distract the user: the focus in a terminal window
# should be on the output of commands, not on the prompt
force_color_prompt=yes

if [ -n "$force_color_prompt" ]; then
    if [ -x /usr/bin/tput ] && tput setaf 1 >&/dev/null; then
	# We have color support; assume it's compliant with Ecma-48
	# (ISO/IEC-6429). (Lack of such support is extremely rare, and such
	# a case would tend to support setf rather than setaf.)
	color_prompt=yes
    else
	color_prompt=
    fi
fi

if [ "$color_prompt" = yes ]; then
    PS1='\[\033[01;32m\]${BKRY_BUILD_CONFIG}@[${BKRY_BRANCH}]:\[\033[01;34m\]\w\[\033[00m\]\$ '
else
    PS1='${debian_chroot:+($debian_chroot)}\u@\h:\w\$ '
fi
unset color_prompt force_color_prompt

# If this is an xterm set the title to user@host:dir
case "$TERM" in
xterm*|rxvt*)
    PS1="\[\e]0;${debian_chroot:+($debian_chroot)}\u@\h: \w\a\]$PS1"
    ;;
*)
    ;;
esac

# enable color support of ls and also add handy aliases
if [ -x /usr/bin/dircolors ]; then
    test -r ~/.dircolors && eval "$(dircolors -b ~/.dircolors)" || eval "$(dircolors -b)"
    alias ls='ls --color=auto'
    #alias dir='dir --color=auto'
    #alias vdir='vdir --color=auto'

    alias grep='grep --color=auto'
    alias fgrep='fgrep --color=auto'
    alias egrep='egrep --color=auto'
fi

# colored GCC warnings and errors
#export GCC_COLORS='error=01;31:warning=01;35:note=01;36:caret=01;32:locus=01:quote=01'

# some more ls aliases
alias ll='ls -alF'
alias la='ls -A'
alias l='ls -CF'

# Add an "alert" alias for long running commands.  Use like so:
#   sleep 10; alert
alias alert='notify-send --urgency=low -i "$([ $? = 0 ] && echo terminal || echo error)" "$(history|tail -n1|sed -e '\''s/^\s*[0-9]\+\s*//;s/[;&|]\s*alert$//'\'')"'

# Alias definitions.
# You may want to put all your additions into a separate file like
# ~/.bash_aliases, instead of adding them here directly.
# See /usr/share/doc/bash-doc/examples in the bash-doc package.

if [ -f ~/.bash_aliases ]; then
    . ~/.bash_aliases
fi

# enable programmable completion features (you don't need to enable
# this, if it's already enabled in /etc/bash.bashrc and /etc/profile
# sources /etc/bash.bashrc).
if ! shopt -oq posix; then
  if [ -f /usr/share/bash-completion/bash_completion ]; then
    . /usr/share/bash-completion/bash_completion
  elif [ -f /etc/bash_completion ]; then
    . /etc/bash_completion
  fi
fi

update_ps1() {
  if git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    export BKRY_BRANCH=$(git branch --show-current 2>/dev/null)
  fi

  if [ -n "${BKRY_BRANCH}" ]; then
    PS1='\[\033[01;32m\]${BKRY_BUILD_CONFIG}@[${BKRY_BRANCH}]:\[\033[01;34m\]\w\[\033[00m\]\$ '
  fi
}

PROMPT_COMMAND=update_ps1

PATH=${BKRY_BIN_DIR}:${PATH}

# The BKRY_BUILD_CONFIG will be set by
# bakery when initializing a workspace shell
build() {
     (cd "${BKRY_WORK_DIR}"; "${BKRY_BIN_DIR}/bakery" build -c "${BKRY_BUILD_CONFIG}" "$@")
}

clean() {
    (cd "${BKRY_WORK_DIR}"; "${BKRY_BIN_DIR}/bakery" clean -c "${BKRY_BUILD_CONFIG}" "$@")
}

deploy() {
    (cd "${BKRY_WORK_DIR}"; "${BKRY_BIN_DIR}/bakery" deploy -c "${BKRY_BUILD_CONFIG}" "$@")
}

upload() {
    (cd "${BKRY_WORK_DIR}"; "${BKRY_BIN_DIR}/bakery" upload -c "${BKRY_BUILD_CONFIG}" "$@")
}

setup() {
    (cd "${BKRY_WORK_DIR}"; "${BKRY_BIN_DIR}/bakery" setup -c "${BKRY_BUILD_CONFIG}" "$@")
}

sync() {
    (cd "${BKRY_WORK_DIR}"; "${BKRY_BIN_DIR}/bakery" sync -c "${BKRY_BUILD_CONFIG}" "$@")
}

benv() {
    env | grep BKRY
}

help() {
    (cd "${BKRY_WORK_DIR}"; "${BKRY_BIN_DIR}/bakery" help "$@")
    echo
    echo -e "\e[1;4mBakery Shell:\e[0m"
    echo
    echo "When inside a bakery shell there are some helpers and aliases available."
    echo
    echo -e "\e[1mHelpers:\e[0m has no args but are simple wrappers that can be"
    echo "usefull and should be easily accessible."
    echo
    echo -e "\e[1;4mUsage:\e[0m <HELPER>"
    echo
    echo -e "\e[1;4mHelpers:\e[0m"
    echo -e "  \e[1mversion\e[0m  Print version of bakery"
    echo -e "  \e[1mconfig\e[0m   Print current bakery build config and build variant"
    echo -e "  \e[1mbenv\e[0m     Print all bakery env variables available starting with BKRY"
    echo -e "  \e[1mctx\e[0m      Print all ctx variables available for a '${BKRY_BUILD_CONFIG}'"
    echo
    echo -e "\e[1mAliases:\e[0m are making use of the preselected bakery build config and are"
    echo "connected to a bakery subcommand. An alias doe not require any args"
    echo "instead bakery <subcommand> -c ${BKRY_BUILD_CONFIG} is used as"
    echo "default args. Any of the supported args for a subcommand can be used."
    echo "To get a list of the supported args available for an alias run <alias> -h"
    echo
    echo -e "\e[1;4mUsage:\e[0m <ALIAS>"
    echo
    echo -e "\e[1;4mAliases:\e[0m"
    echo -e "  \e[1mlist\e[0m     Aliase for 'bakery list -c ${BKRY_BUILD_CONFIG}', list all tasks available"
    echo -e "  \e[1mbuild\e[0m    Aliase for 'bakery build -c ${BKRY_BUILD_CONFIG}', build all or a specific task"
    echo -e "  \e[1mclean\e[0m    Aliase for 'bakery clean -c ${BKRY_BUILD_CONFIG}', clean all or a specific task"
    echo -e "  \e[1msync\e[0m     Aliase for 'bakery sync -c ${BKRY_BUILD_CONFIG}', sync the workspace"
    echo -e "  \e[1msetup\e[0m    Aliase for 'bakery setup -c ${BKRY_BUILD_CONFIG}', setup the workspace"
    echo -e "  \e[1mdeploy\e[0m   Aliase for 'bakery deploy -c ${BKRY_BUILD_CONFIG}', deploy firmware to target"
    echo -e "  \e[1mupload\e[0m   Aliase for 'bakery upload -c ${BKRY_BUILD_CONFIG}', upload firmware to artifactory server"
    echo
}

list() {
    (cd "${BKRY_WORK_DIR}"; "${BKRY_BIN_DIR}/bakery" list -c "${BKRY_BUILD_CONFIG}" "$@")
}

ctx() {
    (cd "${BKRY_WORK_DIR}"; "${BKRY_BIN_DIR}/bakery" list -c "${BKRY_BUILD_CONFIG}" --ctx)
}

version() {
    "${BKRY_BIN_DIR}/bakery" --version
}

config() {
    echo "name: ${BKRY_BUILD_CONFIG}"
}
