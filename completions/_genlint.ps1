
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'genlint' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        'genlint'
        for ($i = 1; $i -lt $commandElements.Count; $i++) {
            $element = $commandElements[$i]
            if ($element -isnot [StringConstantExpressionAst] -or
                $element.StringConstantType -ne [StringConstantType]::BareWord -or
                $element.Value.StartsWith('-') -or
                $element.Value -eq $wordToComplete) {
                break
        }
        $element.Value
    }) -join ';'

    $completions = @(switch ($command) {
        'genlint' {
            [CompletionResult]::new('-i', '-i', [CompletionResultType]::ParameterName, 'Input file(s) to lint')
            [CompletionResult]::new('--input', '--input', [CompletionResultType]::ParameterName, 'Input file(s) to lint')
            [CompletionResult]::new('-e', '-e', [CompletionResultType]::ParameterName, 'Glob patterns to exclude')
            [CompletionResult]::new('--exclude', '--exclude', [CompletionResultType]::ParameterName, 'Glob patterns to exclude')
            [CompletionResult]::new('-f', '-f', [CompletionResultType]::ParameterName, 'Output format')
            [CompletionResult]::new('--format', '--format', [CompletionResultType]::ParameterName, 'Output format')
            [CompletionResult]::new('-o', '-o', [CompletionResultType]::ParameterName, 'Output file path')
            [CompletionResult]::new('--output', '--output', [CompletionResultType]::ParameterName, 'Output file path')
            [CompletionResult]::new('-d', '-d', [CompletionResultType]::ParameterName, 'Disable specific checks')
            [CompletionResult]::new('--disable', '--disable', [CompletionResultType]::ParameterName, 'Disable specific checks')
            [CompletionResult]::new('-l', '-l', [CompletionResultType]::ParameterName, 'Maximum allowed line length')
            [CompletionResult]::new('--max-line-length', '--max-line-length', [CompletionResultType]::ParameterName, 'Maximum allowed line length')
            [CompletionResult]::new('-c', '-c', [CompletionResultType]::ParameterName, 'Maximum allowed consecutive blank lines')
            [CompletionResult]::new('--max-consecutive-blank', '--max-consecutive-blank', [CompletionResultType]::ParameterName, 'Maximum allowed consecutive blank lines')
            [CompletionResult]::new('--max-errors', '--max-errors', [CompletionResultType]::ParameterName, 'Maximum number of errors to report (set to 0 to disable)')
            [CompletionResult]::new('--max-warnings', '--max-warnings', [CompletionResultType]::ParameterName, 'Maximum number of warnings to report (set to 0 to disable)')
            [CompletionResult]::new('-s', '-s', [CompletionResultType]::ParameterName, 'Read input from stdin')
            [CompletionResult]::new('--stdin', '--stdin', [CompletionResultType]::ParameterName, 'Read input from stdin')
            [CompletionResult]::new('-a', '-a', [CompletionResultType]::ParameterName, 'Treat all input as text, bypassing binary detection')
            [CompletionResult]::new('--text', '--text', [CompletionResultType]::ParameterName, 'Treat all input as text, bypassing binary detection')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('generate-completion', 'generate-completion', [CompletionResultType]::ParameterValue, 'Generate shell completions')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'genlint;generate-completion' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'genlint;help' {
            [CompletionResult]::new('generate-completion', 'generate-completion', [CompletionResultType]::ParameterValue, 'Generate shell completions')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'genlint;help;generate-completion' {
            break
        }
        'genlint;help;help' {
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
