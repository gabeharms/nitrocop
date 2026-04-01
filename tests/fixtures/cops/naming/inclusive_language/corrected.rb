allowlist_users = %w(admin)

denylist_ips = []

# Remove replica nodes from cluster

msg = "connected to #{replica_host}"

# Symbol literals should be flagged (CheckSymbols: true by default)
config[:allowlist] = []

alias allowlist= allowlist=

alias blocklist= denylist=

query = "grant select to '#{env.db_replica_user}'"

script = <<~SQL
  grant select on *.* to '#{env.db_replica_user}'@'%'
SQL

# Method definitions with ?/! suffixes are tIDENTIFIER in parser gem (checked)
def allowlisted?
  true
end

def denylisted?(value)
  false
end

def _makara_denylist!
  disconnect!
end

# Bare symbols inside string interpolation are tSYMBOL in parser gem (CheckSymbols: true)
opts << "dsn=#{@opts[:replica_monitor_dsn]}"

# undef arguments are tIDENTIFIER in parser gem (CheckIdentifiers: true)
undef new_replica, new_safe_replica
