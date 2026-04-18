function greet
    echo "Hello, $argv!"
end

function fish_prompt
    echo (prompt_pwd) '> '
end

function backup_file
    set -l src $argv[1]
    set -l dst $src.bak
    cp $src $dst
    echo "Backed up $src -> $dst"
end

alias ll='ls -lah'
alias gs='git status'
alias gp='git pull'
