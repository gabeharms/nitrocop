$stdout.puts('hello')

hash = { out: $stderr, key: value }

def m(out = $stdin)
  out.gets
end

# FN #4, #5, #7, #8: assignment to non-std gvar where const appears on RHS
$stderr = $stdout

$stdout = $stderr

# FN #9, #10, #11: multi-assignment with std stream constant on RHS
$stderr = @stderr =  $stderr

$stdin  = @stdin  =  $stdin

$stdout = @stdout =  $stdout
