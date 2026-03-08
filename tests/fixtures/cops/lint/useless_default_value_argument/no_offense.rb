x.fetch(key) { block_value }
Array.new(size) { block_value }
x.fetch(key, default_value)
Array.new(size, default_value)
x.fetch(key, keyword: :arg) { block_value }

# forwarded block argument (&block) is not a literal block — not flagged
x.fetch(key, default_value, &block)
@data.fetch(key.to_s, *args, &block)
@cache.fetch(key, options, &block)
cache_store.fetch(key, @ttl, &block)
