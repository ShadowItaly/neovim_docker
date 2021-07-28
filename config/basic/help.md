# Introduction
Press ``q`` to close this introduction help!
This environment has a lot of tools to improve the programming experience some are listed here:

| Tool | Description |
|------|-------------|
| glow | For showing markdown files in the command line
| zsh  | To improve the terminal experience, default shell
| nvim | Neovim together with a lot of plugins and mappings
| tmux | Terminal multiplexer to better safe sessions etc.
| python,pip,nodejs,git | Some tools almost all programmers need 
| rg (ripgrep) and fzf | Fast searching and terminal mappings as well
| github-cli and many more| Many more tools a full list will be given shortly

# zsh
The installed theme is powershell10k configuring it via ``p10k configure``.
1. ``CTRL-d`` - will disconnect the current session but does not close it. The terminal history will be list on reconnect so use tmux when wanting to review output history.
2. ``CTRL-t`` - fuzzy search files in subfolders. 
3. ``CTRL-r`` - fuzzy search command history 
4. ``help`` - display this help by typing help in the terminal it is an alias to show this page!

# nvim
Coc is used for autocompletition, any extensions for coc must be installed manually.
*  ``ch`` - will open a fuzzy search window for all key mappings
*  ``jj`` - alternative way to enter normal mode when pressing in insert mode
*  ``jj`` - alternative way to enter normal mode when in terminal
*  ``cs`` - list all snippets from UltiSnips
*  ``cp`` - search all file names in the working directory
*  ``cb`` - search all buffers that are opened
*  ``cl`` - search all lines from all buffers
*  ``cd`` - search the current git status and show diffs of each change
*  ``ca`` - search the commit history and checkout commit on enter
*  ``s`` - highlight a single character in the window and jump to it on input
*  ``S`` - highlight all lines in all open windows and jump to them
* ``cj`` - commit the current changes (fugitive)
* ``ck`` - push the current branch (fugitive)
* ``cg`` - fuzzy search all files in the working directory for the searched term
* ``ce`` - add the changes in the current file to git
* ``ch`` - show all key mappings and search them
* ``cy`` - cycle through the yank history
* ``cY`` - cycle to the newer yank history

# tmux
Does not have a config yet, just configure as usual.
