# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_genlint_global_optspecs
	string join \n s/stdin i/input= e/exclude= f/format= o/output= d/disable= a/text l/max-line-length= c/max-consecutive-blank= max-errors= max-warnings= max-info= m/conflict-marker-style= h/help V/version
end

function __fish_genlint_needs_command
	# Figure out if the current invocation already has a command.
	set -l cmd (commandline -opc)
	set -e cmd[1]
	argparse -s (__fish_genlint_global_optspecs) -- $cmd 2>/dev/null
	or return
	if set -q argv[1]
		# Also print the command, so this can be used to figure out what it is.
		echo $argv[1]
		return 1
	end
	return 0
end

function __fish_genlint_using_subcommand
	set -l cmd (__fish_genlint_needs_command)
	test -z "$cmd"
	and return 1
	contains -- $cmd[1] $argv
end

complete -c genlint -n "__fish_genlint_needs_command" -s i -l input -d 'Input file(s) to lint' -r
complete -c genlint -n "__fish_genlint_needs_command" -s e -l exclude -d 'Glob patterns to exclude' -r
complete -c genlint -n "__fish_genlint_needs_command" -s f -l format -d 'Output format' -r -f -a "json\t''
jsonl\t''
plain\t''"
complete -c genlint -n "__fish_genlint_needs_command" -s o -l output -d 'Output file path' -r -F
complete -c genlint -n "__fish_genlint_needs_command" -s d -l disable -d 'Disable specific checks' -r -f -a "mix-indent\t''
trailing-space\t''
conflict-marker\t''
long-line\t''
consecutive-blank\t''
final-newline\t''"
complete -c genlint -n "__fish_genlint_needs_command" -s l -l max-line-length -d 'Maximum allowed line length' -r
complete -c genlint -n "__fish_genlint_needs_command" -s c -l max-consecutive-blank -d 'Maximum allowed consecutive blank lines' -r
complete -c genlint -n "__fish_genlint_needs_command" -l max-errors -d 'Maximum number of errors to report (set to 0 for no limit)' -r
complete -c genlint -n "__fish_genlint_needs_command" -l max-warnings -d 'Maximum number of warnings to report (set to 0 for no limit)' -r
complete -c genlint -n "__fish_genlint_needs_command" -l max-info -d 'Maximum number of information to report (set to 0 for no limit)' -r
complete -c genlint -n "__fish_genlint_needs_command" -s m -l conflict-marker-style -d 'Conflict marker style' -r -f -a "git\t''
git-diff3\t''
jj\t''
jj-diff3\t''
jj-snapshot\t''"
complete -c genlint -n "__fish_genlint_needs_command" -s s -l stdin -d 'Read input from stdin'
complete -c genlint -n "__fish_genlint_needs_command" -s a -l text -d 'Treat all input as text, bypassing binary detection'
complete -c genlint -n "__fish_genlint_needs_command" -s h -l help -d 'Print help'
complete -c genlint -n "__fish_genlint_needs_command" -s V -l version -d 'Print version'
complete -c genlint -n "__fish_genlint_needs_command" -f -a "generate-completion" -d 'Generate shell completions'
complete -c genlint -n "__fish_genlint_needs_command" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c genlint -n "__fish_genlint_using_subcommand generate-completion" -s h -l help -d 'Print help'
complete -c genlint -n "__fish_genlint_using_subcommand help; and not __fish_seen_subcommand_from generate-completion help" -f -a "generate-completion" -d 'Generate shell completions'
complete -c genlint -n "__fish_genlint_using_subcommand help; and not __fish_seen_subcommand_from generate-completion help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
