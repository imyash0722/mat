# Completions for mdview and md
for cmd in mdview md
    complete -c $cmd -f
    complete -c $cmd -s p -l no-pager -d 'Do not pipe output into a pager'
    complete -c $cmd -s w -l width -x -d 'Override terminal display width'
    complete -c $cmd -s c -l completions -x -a 'bash zsh fish' -d 'Generate shell completions'
    complete -c $cmd -s v -s V -l version -d 'Print version information'
    complete -c $cmd -s h -l help -d 'Print help information'
    complete -c $cmd -k -a '(__fish_complete_suffix .md)'
    complete -c $cmd -k -a '(__fish_complete_suffix .markdown)'
    complete -c $cmd -a '(_fish_complete_file)'
end
