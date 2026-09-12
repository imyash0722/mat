_mat() {
    local cur prev words cword
    _init_completion || return

    case "$prev" in
        -w|--width)
            return 0
            ;;
        -c|--completions)
            COMPREPLY=( $(compgen -W "bash zsh fish" -- "$cur") )
            return 0
            ;;
    esac

    if [[ "$cur" == -* ]]; then
        COMPREPLY=( $(compgen -W "-p --no-pager -w --width -c --completions -v -V --version -h --help" -- "$cur") )
        return 0
    fi

    _filedir '@(md|markdown|mdown|mkd|mkdn)' || _filedir
}

complete -F _mat mat
