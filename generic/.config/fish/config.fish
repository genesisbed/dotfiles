alias l='eza -loa --no-permissions --smart-group --time-style=relative --git'
alias c='clear'
alias ff='fastfetch'

alias img='mpv --keep-open=always'

function fish_write
    set_color $argv[1]
    echo -en $argv[2]
    set_color normal
end

function fish_prompt
    fish_write normal "\n "
    fish_write magenta (prompt_pwd --full-length-dirs=99999)
    fish_write normal "\n := "
end

if status is-interactive
    bass source /etc/profile
end
