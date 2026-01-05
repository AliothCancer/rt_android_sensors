module completions {

  # The cross-shell prompt for astronauts. ☄🌌️
  export extern starship [
    --help(-h)                # Print help
    --version(-V)             # Print version
  ]

  # Create a pre-populated GitHub issue with information about your configuration
  export extern "starship bug-report" [
    --help(-h)                # Print help
  ]

  def "nu-complete starship completions shell" [] {
    [ "bash" "elvish" "fish" "nushell" "powershell" "zsh" ]
  }

  # Generate starship shell completions for your shell to stdout
  export extern "starship completions" [
    --help(-h)                # Print help
    shell: string@"nu-complete starship completions shell"
  ]

  # Edit the starship configuration
  export extern "starship config" [
    --help(-h)                # Print help
    name?: string             # Configuration key to edit
    value?: string            # Value to place into that key
  ]

  # Explains the currently showing modules
  export extern "starship explain" [
    --status(-s): string      # The status code of the previously run command as an unsigned or signed 32bit integer
    --pipestatus: string      # Bash, Fish and Zsh support returning codes for each process in a pipeline
    --terminal-width(-w): string # The width of the current interactive terminal
    --path(-p): path          # The path that the prompt should render for
    --logical-path(-P): path  # The logical path that the prompt should render for. This path should be a virtual/logical representation of the PATH argument
    --cmd-duration(-d): string # The execution duration of the last command, in milliseconds
    --keymap(-k): string      # The keymap of fish/zsh/cmd
    --jobs(-j): string        # The number of currently running jobs
    --shlvl: string           # The current value of SHLVL, for shells that mis-handle it in $()
    --help(-h)                # Print help
  ]

  # Prints the shell function used to execute starship
  export extern "starship init" [
    --print-full-init
    --help(-h)                # Print help
    shell: string
  ]

  # Prints a specific prompt module
  export extern "starship module" [
    --list(-l)                # List out all supported modules
    --status(-s): string      # The status code of the previously run command as an unsigned or signed 32bit integer
    --pipestatus: string      # Bash, Fish and Zsh support returning codes for each process in a pipeline
    --terminal-width(-w): string # The width of the current interactive terminal
    --path(-p): path          # The path that the prompt should render for
    --logical-path(-P): path  # The logical path that the prompt should render for. This path should be a virtual/logical representation of the PATH argument
    --cmd-duration(-d): string # The execution duration of the last command, in milliseconds
    --keymap(-k): string      # The keymap of fish/zsh/cmd
    --jobs(-j): string        # The number of currently running jobs
    --shlvl: string           # The current value of SHLVL, for shells that mis-handle it in $()
    --help(-h)                # Print help
    name?: string             # The name of the module to be printed
  ]

  def "nu-complete starship preset name" [] {
    [ "bracketed-segments" "catppuccin-powerline" "gruvbox-rainbow" "jetpack" "nerd-font-symbols" "no-empty-icons" "no-nerd-font" "no-runtime-versions" "pastel-powerline" "plain-text-symbols" "pure-preset" "tokyo-night" ]
  }

  # Prints a preset config
  export extern "starship preset" [
    --output(-o): path        # Output the preset to a file instead of stdout
    --list(-l)                # List out all preset names
    --help(-h)                # Print help
    name?: string@"nu-complete starship preset name" # The name of preset to be printed
  ]

  # Prints the computed starship configuration
  export extern "starship print-config" [
    --default(-d)             # Print the default instead of the computed config
    --help(-h)                # Print help
    ...name: string           # Configuration keys to print
  ]

  # Prints the full starship prompt
  export extern "starship prompt" [
    --right                   # Print the right prompt (instead of the standard left prompt)
    --profile: string         # Print the prompt with the specified profile name (instead of the standard left prompt)
    --continuation            # Print the continuation prompt (instead of the standard left prompt)
    --status(-s): string      # The status code of the previously run command as an unsigned or signed 32bit integer
    --pipestatus: string      # Bash, Fish and Zsh support returning codes for each process in a pipeline
    --terminal-width(-w): string # The width of the current interactive terminal
    --path(-p): path          # The path that the prompt should render for
    --logical-path(-P): path  # The logical path that the prompt should render for. This path should be a virtual/logical representation of the PATH argument
    --cmd-duration(-d): string # The execution duration of the last command, in milliseconds
    --keymap(-k): string      # The keymap of fish/zsh/cmd
    --jobs(-j): string        # The number of currently running jobs
    --shlvl: string           # The current value of SHLVL, for shells that mis-handle it in $()
    --help(-h)                # Print help
  ]

  # Generate random session key
  export extern "starship session" [
    --help(-h)                # Print help
  ]

  # Prints time in milliseconds
  export extern "starship time" [
    --help(-h)                # Print help
  ]

  # Prints timings of all active modules
  export extern "starship timings" [
    --status(-s): string      # The status code of the previously run command as an unsigned or signed 32bit integer
    --pipestatus: string      # Bash, Fish and Zsh support returning codes for each process in a pipeline
    --terminal-width(-w): string # The width of the current interactive terminal
    --path(-p): path          # The path that the prompt should render for
    --logical-path(-P): path  # The logical path that the prompt should render for. This path should be a virtual/logical representation of the PATH argument
    --cmd-duration(-d): string # The execution duration of the last command, in milliseconds
    --keymap(-k): string      # The keymap of fish/zsh/cmd
    --jobs(-j): string        # The number of currently running jobs
    --shlvl: string           # The current value of SHLVL, for shells that mis-handle it in $()
    --help(-h)                # Print help
  ]

  # Toggle a given starship module
  export extern "starship toggle" [
    --help(-h)                # Print help
    name: string              # The name of the module to be toggled
    value?: string            # The key of the config to be toggled
  ]

  # Print this message or the help of the given subcommand(s)
  export extern "starship help" [
  ]

  # Create a pre-populated GitHub issue with information about your configuration
  export extern "starship help bug-report" [
  ]

  # Generate starship shell completions for your shell to stdout
  export extern "starship help completions" [
  ]

  # Edit the starship configuration
  export extern "starship help config" [
  ]

  # Explains the currently showing modules
  export extern "starship help explain" [
  ]

  # Prints the shell function used to execute starship
  export extern "starship help init" [
  ]

  # Prints a specific prompt module
  export extern "starship help module" [
  ]

  # Prints a preset config
  export extern "starship help preset" [
  ]

  # Prints the computed starship configuration
  export extern "starship help print-config" [
  ]

  # Prints the full starship prompt
  export extern "starship help prompt" [
  ]

  # Generate random session key
  export extern "starship help session" [
  ]

  # Prints time in milliseconds
  export extern "starship help time" [
  ]

  # Prints timings of all active modules
  export extern "starship help timings" [
  ]

  # Toggle a given starship module
  export extern "starship help toggle" [
  ]

  # Print this message or the help of the given subcommand(s)
  export extern "starship help help" [
  ]

}

export use completions *
