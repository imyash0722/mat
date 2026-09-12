# Completions for mat
complete -c mat -f
complete -c mat -s p -l no-pager -d 'Do not pipe output into a pager'
complete -c mat -s w -l width -x -d 'Override terminal display width'
complete -c mat -s c -l completions -x -a 'bash zsh fish' -d 'Generate shell completions'
complete -c mat -s v -s V -l version -d 'Print version information'
complete -c mat -s h -l help -d 'Print help information'
complete -c mat -k -a '(__fish_complete_suffix .md)'
complete -c mat -k -a '(__fish_complete_suffix .markdown)'
complete -c mat -a '(_fish_complete_file)'
